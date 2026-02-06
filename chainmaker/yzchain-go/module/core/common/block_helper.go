/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package common

import (
	"bytes"
	"chainmaker.org/chainmaker-go/module/core/common/scheduler"
	"chainmaker.org/chainmaker-go/module/core/provider/conf"
	snapshot2 "chainmaker.org/chainmaker-go/module/snapshot"
	"chainmaker.org/chainmaker-go/module/subscriber"
	"chainmaker.org/chainmaker/common/v2/bytehelper"
	"chainmaker.org/chainmaker/common/v2/crypto/hash"
	commonErrors "chainmaker.org/chainmaker/common/v2/errors"
	"chainmaker.org/chainmaker/common/v2/monitor"
	"chainmaker.org/chainmaker/common/v2/msgbus"
	"chainmaker.org/chainmaker/localconf/v2"
	"chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/consensus"
	netpb "chainmaker.org/chainmaker/pb-go/v2/net"
	"chainmaker.org/chainmaker/protocol/v2"
	batch "chainmaker.org/chainmaker/txpool-batch/v2"
	"chainmaker.org/chainmaker/utils/v2"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"github.com/gogo/protobuf/proto"
	"github.com/panjf2000/ants/v2"
	"github.com/prometheus/client_golang/prometheus"
	"math/rand"
	"runtime/debug"
	"sort"
	"sync"
	"sync/atomic"
	"time"
)

var (
	//proposeRepeatTimer *time.Timer //timer controls the propose repeat interval
	//ProposeRepeatTimerMap = make(map[string]*time.Timer)

	ProposeRepeatTimerMap sync.Map
)

const (
	DEFAULTDURATION = 1000 // default proposal duration, millis seconds
	//blockSig:%d,vm:%d,txVerify:%d,txRoot:%d
	BlockSig            = "blockSig"
	VM                  = "vm"
	TxVerify            = "txVerify"
	TxRoot              = "txRoot"
	QuickSyncVerifyMode = uint8(1) // quick sync verify mode
	NormalVerifyMode    = uint8(0) // normal verify mode
	DEFAULTTIMEOUT      = 5000
)

type BlockBuilderConf struct {
	ChainId         string                   // chain id, to identity this chain
	TxPool          protocol.TxPool          // tx pool provides tx batch
	TxScheduler     protocol.TxScheduler     // scheduler orders tx batch into DAG form and returns a block
	SnapshotManager protocol.SnapshotManager // snapshot manager
	Identity        protocol.SigningMember   // identity manager
	LedgerCache     protocol.LedgerCache     // ledger cache
	ProposalCache   protocol.ProposalCache
	ChainConf       protocol.ChainConf // chain config
	Log             protocol.Logger
	StoreHelper     conf.StoreHelper
}

type BlockBuilder struct {
	chainId         string                   // chain id, to identity this chain
	txPool          protocol.TxPool          // tx pool provides tx batch
	txScheduler     protocol.TxScheduler     // scheduler orders tx batch into DAG form and returns a block
	snapshotManager protocol.SnapshotManager // snapshot manager
	identity        protocol.SigningMember   // identity manager
	ledgerCache     protocol.LedgerCache     // ledger cache
	proposalCache   protocol.ProposalCache
	chainConf       protocol.ChainConf // chain config
	log             protocol.Logger
	storeHelper     conf.StoreHelper
}

func NewBlockBuilder(conf *BlockBuilderConf) *BlockBuilder {
	creatorBlock := &BlockBuilder{
		chainId:         conf.ChainId,
		txPool:          conf.TxPool,
		txScheduler:     conf.TxScheduler,
		snapshotManager: conf.SnapshotManager,
		identity:        conf.Identity,
		ledgerCache:     conf.LedgerCache,
		proposalCache:   conf.ProposalCache,
		chainConf:       conf.ChainConf,
		log:             conf.Log,
		storeHelper:     conf.StoreHelper,
	}

	return creatorBlock
}

const TempLinkingFieldKey = "HashLinking"
const PreTempLinkingFieldKey = "PreBlockHashLinking"

func (bb *BlockBuilder) GenerateNewBlock(
	proposingHeight uint64, preTempHash []byte, txBatch []*commonPb.Transaction,
	batchIds []string, fetchBatches [][]*commonPb.Transaction) (
	*commonPb.Block, []int64, error) {

	timeLasts := make([]int64, 0)
	currentHeight, _ := bb.ledgerCache.CurrentHeight()
	lastBlock := bb.findLastBlockFromCache(proposingHeight, preTempHash, currentHeight)
	if lastBlock == nil {
		return nil, nil, fmt.Errorf("no pre block found [%d] (%x)", proposingHeight-1, preTempHash)
	}
	isConfigBlock := false
	if len(txBatch) == 1 && utils.IsConfigTx(txBatch[0]) {
		isConfigBlock = true
	}
	block, err := initNewBlock(lastBlock, bb.identity, bb.chainId, bb.chainConf, isConfigBlock)
	if err != nil {
		return block, timeLasts, err
	}
	if block == nil {
		bb.log.Warnf("generate new block failed, block == nil")
		return nil, timeLasts, fmt.Errorf("generate new block failed, block == nil")
	}
	//if txBatch == nil {
	//	// For ChainedBFT consensus, generate an empty block if tx batch is empty.
	//	return block, timeLasts, nil
	//}

	// validate tx and verify ACL，split into 2 slice according to result
	// validatedTxs are txs passed validate and should be executed by contract
	var aclFailTxs = make([]*commonPb.Transaction, 0) // No need to ACL check, this slice is empty
	var validatedTxs = txBatch

	ssStartTick := utils.CurrentTimeMillisSeconds()
	//snapshot := bb.snapshotManager.NewSnapshot(lastBlock, block)
	//
	beginDbTick := utils.CurrentTimeMillisSeconds()
	//bb.storeHelper.BeginDbTransaction(snapshot.GetBlockchainStore(), block.GetTxKey())
	//
	vmStartTick := utils.CurrentTimeMillisSeconds()
	//txRWSetMap, contractEventMap, err := bb.txScheduler.GetResultMaps(block, validatedTxs, snapshot)
	//
	ssLasts := beginDbTick - ssStartTick
	dbLasts := vmStartTick - beginDbTick
	vmLasts := utils.CurrentTimeMillisSeconds() - vmStartTick
	timeLasts = append(timeLasts, dbLasts, ssLasts, vmLasts)
	block.Txs = validatedTxs

	hashType := bb.chainConf.ChainConfig().Crypto.GetHash()
	tempHash, err := GenerateTxBatchHashForTempLinking(validatedTxs, hashType)
	if err != nil {
		bb.log.Errorf("Encountering an err when generate temp linking. err: %s", err)
	}
	block.AdditionalData.ExtraData[TempLinkingFieldKey] = tempHash
	bb.log.Debugf("Generated a temporary TxBatch hash for linking in Ordering stage. "+
		"Only needed by EV arch. Generated Hash: %s", tempHash)
	block.AdditionalData.ExtraData[PreTempLinkingFieldKey] = preTempHash
	bb.log.Debugf("Linked this block to previous block by temp hash link. PreTempHash: %s", preTempHash)

	// Initialize empty RWSet and eventMap to mock actual execution
	txRWSetMap, contractEventMap := make(map[string]*commonPb.TxRWSet), make(map[string][]*commonPb.ContractEvent)
	for _, tx := range block.Txs {
		rwset := &commonPb.TxRWSet{
			TxId:     tx.Payload.TxId,
			TxReads:  nil,
			TxWrites: nil,
		}
		txRWSetMap[tx.Payload.TxId] = rwset
		rwsethash, err := utils.CalcRWSetHash(bb.chainConf.ChainConfig().Crypto.Hash, rwset)
		if err != nil {
			bb.log.Errorf("Encountering an err when calc rwsethash. err: %s", err)
		}
		tx.Result = &commonPb.Result{
			Code: 0,
			ContractResult: &commonPb.ContractResult{
				Code:          0,
				Result:        make([]byte, 0),
				ContractEvent: make([]*commonPb.ContractEvent, 0),
			},
			RwSetHash: rwsethash,
		}
		contractEventMap[tx.Payload.TxId] = tx.Result.ContractResult.ContractEvent
	}

	if err != nil {
		return nil, timeLasts, fmt.Errorf("schedule block(%d,%x) error %s",
			block.Header.BlockHeight, block.Header.BlockHash, err)
	}

	// deal with the special situation：
	// 1. only one tx and schedule time out
	// 2. package the empty block
	if !utils.CanProposeEmptyBlock(bb.chainConf.ChainConfig().Consensus.Type) && len(block.Txs) == 0 {
		return nil, timeLasts, fmt.Errorf("no txs in scheduled block, proposing block ends")
	}

	finalizeStartTick := utils.CurrentTimeMillisSeconds()
	err = FinalizeBlock(
		block,
		txRWSetMap,
		aclFailTxs,
		bb.chainConf.ChainConfig().Crypto.Hash,
		bb.log)

	finalizeLasts := utils.CurrentTimeMillisSeconds() - finalizeStartTick
	if err != nil {
		return nil, timeLasts, fmt.Errorf("finalizeBlock block(%d,%s) error %s",
			block.Header.BlockHeight, hex.EncodeToString(block.Header.BlockHash), err)
	}
	timeLasts = append(timeLasts, finalizeLasts)

	if TxPoolType == batch.TxPoolType {
		var batchIdBytes []byte
		// set batchIds into additional data
		batchIdBytes, err = SerializeTxBatchInfo(batchIds, block.Txs, fetchBatches, bb.log)
		if err != nil {
			return nil, timeLasts, fmt.Errorf("finalizeBlock block(%d,%s) error %s",
				block.Header.BlockHeight, hex.EncodeToString(block.Header.BlockHash), err)
		}
		block.AdditionalData.ExtraData[batch.BatchPoolAddtionalDataKey] = batchIdBytes
		bb.log.InfoDynamic(func() string {
			return fmt.Sprintf("[%v] proposer add batchIds:%v into addition data", block.Header.BlockHeight,
				func() []string {
					var batch0 []string
					for i := range batchIds {
						batch0 = append(batch0, hex.EncodeToString([]byte(batchIds[i])))
					}
					return batch0
				}())
		})
	}

	// cache proposed block
	bb.log.Debugf("set proposed block(%d,%x)", block.Header.BlockHeight, block.Header.BlockHash)
	if err = bb.proposalCache.SetProposedBlock(block, txRWSetMap, contractEventMap, true); err != nil {
		return block, timeLasts, err
	}
	bb.proposalCache.SetProposedAt(block.Header.BlockHeight)

	return block, timeLasts, nil
}

func GenerateTxBatchHashForTempLinking(validatedTxs []*commonPb.Transaction, hashType string) ([]byte, error) {
	hashedBytes := make([]byte, 0)
	for i := 0; i < len(validatedTxs); i++ {
		txHash, e := utils.CalcTxRequestHash(hashType, validatedTxs[i])
		if e != nil {
			return nil, e
		}
		hashedBytes = append(hashedBytes, txHash...)
	}
	sort.Slice(hashedBytes, func(i, j int) bool {
		return i >= j
	})
	tempHash, err := hash.GetByStrType(hashType, hashedBytes)
	if err != nil {
		return nil, err
	}
	return tempHash, nil
}

func (bb *BlockBuilder) findLastBlockFromCache(proposingHeight uint64, preTempHash []byte,
	currentHeight uint64) *commonPb.Block {
	var lastBlock *commonPb.Block
	if currentHeight+1 == proposingHeight {
		lastBlock = bb.ledgerCache.GetLastBlock()
	} else {
		lastBlock, _ = bb.proposalCache.GetProposedBlockByHashAndHeight(preTempHash, proposingHeight-1)
	}
	return lastBlock
}

func initNewBlock(
	lastBlock *commonPb.Block,
	identity protocol.SigningMember,
	chainId string,
	chainConf protocol.ChainConf, isConfigBlock bool) (*commonPb.Block, error) {
	// get node pk from identity
	proposer, err := identity.GetMember()
	if err != nil {
		return nil, fmt.Errorf("identity serialize failed, %s", err)
	}
	preConfHeight := lastBlock.Header.PreConfHeight
	// if last block is config block, then this block.preConfHeight is last block height
	if utils.IsConfBlock(lastBlock) {
		preConfHeight = lastBlock.Header.BlockHeight
	}
	blockVersion := chainConf.ChainConfig().GetBlockVersion()
	if blockVersion == 0 {
		blockVersion = protocol.DefaultBlockVersion
	}
	block := &commonPb.Block{
		Header: &commonPb.BlockHeader{
			ChainId:        chainId,
			BlockHeight:    lastBlock.Header.BlockHeight + 1,
			PreBlockHash:   lastBlock.Header.BlockHash,
			BlockHash:      nil,
			PreConfHeight:  preConfHeight,
			BlockVersion:   blockVersion,
			DagHash:        nil,
			RwSetRoot:      nil,
			TxRoot:         nil,
			BlockTimestamp: utils.CurrentTimeSeconds(),
			Proposer:       proposer,
			ConsensusArgs:  nil,
			TxCount:        0,
			Signature:      nil,
		},
		Dag: &commonPb.DAG{},
		Txs: nil,
		AdditionalData: &commonPb.AdditionalData{
			ExtraData: make(map[string][]byte),
		},
	}
	if isConfigBlock {
		block.Header.BlockType = commonPb.BlockType_CONFIG_BLOCK
	}
	return block, nil
}

func FinalizeBlock(
	block *commonPb.Block,
	txRWSetMap map[string]*commonPb.TxRWSet,
	aclFailTxs []*commonPb.Transaction,
	hashType string,
	logger protocol.Logger) error {

	if aclFailTxs != nil && len(aclFailTxs) > 0 { //nolint: gosimple
		// append acl check failed txs to the end of block.Txs
		block.Txs = append(block.Txs, aclFailTxs...)
	}

	// TxCount contains acl verify failed txs and invoked contract txs
	txCount := len(block.Txs)
	block.Header.TxCount = uint32(txCount)

	// TxRoot/RwSetRoot
	errsC := make(chan error, txCount+3) // txCount+3 possible errors
	txHashes := make([][]byte, txCount)
	wg := &sync.WaitGroup{}
	wg.Add(txCount)
	for i, tx := range block.Txs {
		// finalize tx, put rwsethash into tx.Result
		rwSet := txRWSetMap[tx.Payload.TxId]
		if rwSet == nil {
			rwSet = &commonPb.TxRWSet{
				TxId:     tx.Payload.TxId,
				TxReads:  nil,
				TxWrites: nil,
			}
		}
		go func(tx *commonPb.Transaction, rwSet *commonPb.TxRWSet, x int) {
			defer wg.Done()
			var err error
			txHashes[x], err = getTxHash(tx, rwSet, hashType, block.Header, logger)
			if err != nil {
				errsC <- err
			}

		}(tx, rwSet, i)
	}
	wg.Wait()
	if len(errsC) > 0 {
		err := <-errsC
		return err
	}
	wg.Add(3)
	//calc tx root
	go func() {
		defer wg.Done()
		var err error
		block.Header.TxRoot, err = hash.GetMerkleRoot(hashType, txHashes)
		if err != nil {
			logger.Warnf("get tx merkle root error %s", err)
			errsC <- err
		}
		logger.DebugDynamic(func() string {
			return fmt.Sprintf("GetMerkleRoot(%s) get %x", hashType, block.Header.TxRoot)
		})
	}()
	//calc rwset root
	go func() {
		defer wg.Done()
		var err error
		block.Header.RwSetRoot, err = utils.CalcRWSetRoot(hashType, block.Txs)
		if err != nil {
			logger.Warnf("get rwset merkle root error %s", err)
			errsC <- err
		}
	}()
	//calc dag hash
	go func() {
		defer wg.Done()
		// DagDigest
		var dagHash []byte
		var err error
		dagHash, err = utils.CalcDagHash(hashType, block.Dag)
		if err != nil {
			logger.Warnf("get dag hash error %s", err)
			errsC <- err
		}
		block.Header.DagHash = dagHash
	}()
	wg.Wait()
	// not close errsC will NOT cause memory leak
	if len(errsC) > 0 {
		err := <-errsC
		return err
	}
	return nil
}

func getTxHash(tx *commonPb.Transaction,
	rwSet *commonPb.TxRWSet,
	hashType string,
	blockHeader *commonPb.BlockHeader,
	logger protocol.Logger) (
	[]byte, error) {
	var rwSetHash []byte
	rwSetHash, err := utils.CalcRWSetHash(hashType, rwSet)
	logger.DebugDynamic(func() string {
		str := fmt.Sprintf("CalcRWSetHash rwset: %+v ,hash: %x", rwSet, rwSetHash)
		if len(str) > 1024 {
			str = str[:1024] + " ......"
		}
		return str
	})
	if err != nil {
		return nil, err
	}
	if tx.Result == nil {
		// in case tx.Result is nil, avoid panic
		e := fmt.Errorf("tx(%s) result == nil", tx.Payload.TxId)
		logger.Error(e.Error())
		return nil, e
	}
	tx.Result.RwSetHash = rwSetHash
	// calculate complete tx hash, include tx.Header, tx.Payload, tx.Result
	var txHash []byte
	txHash, err = utils.CalcTxHashWithVersion(
		hashType, tx, int(blockHeader.BlockVersion))
	if err != nil {
		return nil, err
	}
	return txHash, nil
}

// IsTxCountValid to check if txcount in block is valid
func IsTxCountValid(block *commonPb.Block) error {
	if block.Header.TxCount != uint32(len(block.Txs)) {
		return fmt.Errorf("txcount expect %d, got %d", block.Header.TxCount, len(block.Txs))
	}
	return nil
}

// IsHeightValid to check if block height is valid
func IsHeightValid(block *commonPb.Block, currentHeight uint64) error {
	if currentHeight+1 != block.Header.BlockHeight {
		return fmt.Errorf("height expect %d, got %d", currentHeight+1, block.Header.BlockHeight)
	}
	return nil
}

// IsPreHashValid to check if block.preHash equals with last block hash
func IsPreHashValid(block *commonPb.Block, preHash []byte) error {
	if !bytes.Equal(preHash, block.Header.PreBlockHash) {
		return fmt.Errorf("prehash expect %x, got %x", preHash, block.Header.PreBlockHash)
	}
	return nil
}

// IsPreHashValid to check if block.preHash equals with last block hash
func IsPreTempHashValid(block *commonPb.Block, preHash []byte) error {
	if !bytes.Equal(preHash, block.AdditionalData.ExtraData[PreTempLinkingFieldKey]) {
		return fmt.Errorf("prehash expect %x, got %x", preHash, block.Header.PreBlockHash)
	}
	return nil
}

// IsBlockHashValid to check if block hash equals with result calculated from block
func IsBlockHashValid(block *commonPb.Block, hashType string) error {
	blockHash, err := utils.CalcBlockHash(hashType, block)
	if err != nil {
		return fmt.Errorf("calc block blockHash error")
	}
	if !bytes.Equal(blockHash, block.Header.BlockHash) {
		return fmt.Errorf("block blockHash expect %x, got %x", block.Header.BlockHash, blockHash)
	}
	txHash, err := GenerateTxBatchHashForTempLinking(block.Txs, hashType)
	if err != nil {
		return fmt.Errorf("generate tx batch blockHash error")
	}
	if !bytes.Equal(txHash, block.AdditionalData.ExtraData[TempLinkingFieldKey]) {
		return fmt.Errorf("tx blockHash expect %x, got %x",
			block.AdditionalData.ExtraData[TempLinkingFieldKey], txHash)
	}
	return nil
}

// IsTxDuplicate to check if there is duplicated transactions in one block
func IsTxDuplicate(txs []*commonPb.Transaction) (duplicate bool, duplicateTxs []string) {
	txSet := make(map[string]struct{}, len(txs))
	exist := struct{}{}
	for _, tx := range txs {
		if tx == nil || tx.Payload == nil {
			return true, duplicateTxs
		}
		if _, ok := txSet[tx.Payload.TxId]; ok {
			duplicateTxs = append(duplicateTxs, tx.Payload.TxId+" duplicated")
			continue
		}
		txSet[tx.Payload.TxId] = exist
	}
	// length of set < length of txs, means txs have duplicate tx
	return len(txSet) < len(txs), duplicateTxs
}

// IsMerkleRootValid to check if block merkle root equals with simulated merkle root
func IsMerkleRootValid(block *commonPb.Block, txHashes [][]byte, hashType string) error {
	txRoot, err := hash.GetMerkleRoot(hashType, txHashes)
	if err != nil || !bytes.Equal(txRoot, block.Header.TxRoot) {
		return fmt.Errorf("GetMerkleRoot(%s,%v) get %x ,txroot expect %x, got %x, err: %s",
			hashType, txHashes, txRoot, block.Header.TxRoot, txRoot, err)
	}
	return nil
}

// IsDagHashValid to check if block dag equals with simulated block dag
func IsDagHashValid(block *commonPb.Block, hashType string) error {
	dagHash, err := utils.CalcDagHash(hashType, block.Dag)
	if err != nil || !bytes.Equal(dagHash, block.Header.DagHash) {
		return fmt.Errorf("dag expect %x, got %x", block.Header.DagHash, dagHash)
	}
	return nil
}

// IsRWSetHashValid to check if read write set is valid
func IsRWSetHashValid(block *commonPb.Block, hashType string) error {
	rwSetRoot, err := utils.CalcRWSetRoot(hashType, block.Txs)
	if err != nil {
		return fmt.Errorf("calc rwset error, %s", err)
	}
	if !bytes.Equal(rwSetRoot, block.Header.RwSetRoot) {
		return fmt.Errorf("rwset expect %x, got %x", block.Header.RwSetRoot, rwSetRoot)
	}
	return nil
}

// getChainVersion, get chain version from config.
// If not access from config, use default value.
// @Deprecated
//func getChainVersion(chainConf protocol.ChainConf) []byte {
//	if chainConf == nil || chainConf.ChainConfig() == nil {
//		return []byte(protocol.DefaultBlockVersion)
//	}
//	return []byte(chainConf.ChainConfig().Version)
//}

func VerifyHeight(height uint64, ledgerCache protocol.LedgerCache) error {
	currentHeight, err := ledgerCache.CurrentHeight()
	if err != nil {
		return err
	}
	if currentHeight+1 != height {
		return fmt.Errorf("verify height fail,expected [%d]", currentHeight+1)
	}
	return nil
}

func CheckBlockDigests(block *commonPb.Block, txHashes [][]byte, hashType string, log protocol.Logger) error {
	if err := IsMerkleRootValid(block, txHashes, hashType); err != nil {
		log.Error(err)
		return err
	}
	// verify DAG hash
	if err := IsDagHashValid(block, hashType); err != nil {
		log.Error(err)
		return err
	}
	// verify read write set, check if simulate result is equal with rwset in block header
	if err := IsRWSetHashValid(block, hashType); err != nil {
		log.Error(err)
		return err
	}
	return nil
}

func CheckVacuumBlock(block *commonPb.Block, consensusType consensus.ConsensusType) error {
	if block.Header.TxCount == 0 {
		if utils.CanProposeEmptyBlock(consensusType) {
			// for consensus that allows empty block, skip txs verify
			return nil
		}

		// for consensus that NOT allows empty block, return error
		return fmt.Errorf("tx must not empty")
	}
	return nil
}

type VerifierBlockConf struct {
	ChainConf           protocol.ChainConf
	Log                 protocol.Logger
	CommitedLedgerCache protocol.LedgerCache
	Ac                  protocol.AccessControlProvider
	SnapshotManager     protocol.SnapshotManager
	VmMgr               protocol.VmManager
	TxPool              protocol.TxPool
	BlockchainStore     protocol.BlockchainStore
	ProposalCache       protocol.ProposalCache // proposal cache
	StoreHelper         conf.StoreHelper
	TxScheduler         protocol.TxScheduler
	TxFilter            protocol.TxFilter
}

type VerifierBlock struct {
	chainConf           protocol.ChainConf
	log                 protocol.Logger
	commitedLedgerCache protocol.LedgerCache
	ac                  protocol.AccessControlProvider
	snapshotManager     protocol.SnapshotManager
	vmMgr               protocol.VmManager
	txScheduler         protocol.TxScheduler
	txPool              protocol.TxPool
	blockchainStore     protocol.BlockchainStore
	proposalCache       protocol.ProposalCache // proposal cache
	storeHelper         conf.StoreHelper
	txFilter            protocol.TxFilter
}

func NewVerifierBlock(conf *VerifierBlockConf) *VerifierBlock {
	verifyBlock := &VerifierBlock{
		chainConf:           conf.ChainConf,
		log:                 conf.Log,
		commitedLedgerCache: conf.CommitedLedgerCache,
		ac:                  conf.Ac,
		snapshotManager:     conf.SnapshotManager,
		vmMgr:               conf.VmMgr,
		txPool:              conf.TxPool,
		blockchainStore:     conf.BlockchainStore,
		proposalCache:       conf.ProposalCache,
		storeHelper:         conf.StoreHelper,
		txScheduler:         conf.TxScheduler,
		txFilter:            conf.TxFilter,
	}
	var schedulerFactory scheduler.TxSchedulerFactory
	verifyBlock.txScheduler = schedulerFactory.NewTxScheduler(
		verifyBlock.vmMgr,
		verifyBlock.chainConf,
		conf.StoreHelper,
		conf.CommitedLedgerCache,
	)
	return verifyBlock
}

// SetTxScheduler sets the txScheduler of VerifierBlock
// only used for test
func (v *VerifierBlock) SetTxScheduler(txScheduler protocol.TxScheduler) {
	v.txScheduler = txScheduler
}

func (vb *VerifierBlock) FetchLastBlock(block *commonPb.Block) (*commonPb.Block, error) { //nolint: staticcheck
	currentHeight, _ := vb.commitedLedgerCache.CurrentHeight()
	if currentHeight >= block.Header.BlockHeight {
		return nil, commonErrors.ErrBlockHadBeenCommited
	}

	var lastBlock *commonPb.Block
	if currentHeight+1 == block.Header.BlockHeight {
		lastBlock = vb.commitedLedgerCache.GetLastBlock() //nolint: staticcheck
	} else {
		lastBlock, _ = vb.proposalCache.GetProposedBlockByHashAndHeight(
			block.AdditionalData.ExtraData[PreTempLinkingFieldKey], block.Header.BlockHeight-1)
	}
	if lastBlock == nil {
		return nil, fmt.Errorf("no pre block found [%d](%x)", block.Header.BlockHeight-1, block.Header.PreBlockHash)
	}
	return lastBlock, nil
}

// validateBlock, validate block and transactions
func (vb *VerifierBlock) ValidateBlock(
	block, lastBlock *commonPb.Block, hashType string, timeLasts map[string]int64, mode protocol.VerifyMode) (
	map[string]*commonPb.TxRWSet, map[string][]*commonPb.ContractEvent, map[string]int64, *RwSetVerifyFailTx, error) {

	if err := IsBlockHashValid(block, vb.chainConf.ChainConfig().Crypto.Hash); err != nil {
		return nil, nil, timeLasts, nil, err
	}

	// verify block sig and also verify identity and auth of block proposer
	startSigTick := utils.CurrentTimeMillisSeconds()
	vb.log.DebugDynamic(func() string {
		return fmt.Sprintf("verify block \n %s", utils.FormatBlock(block))
	})
	if ok, err := utils.VerifyBlockSig(hashType, block, vb.ac); !ok || err != nil {
		return nil, nil, timeLasts, nil, fmt.Errorf("(%d,%x - %x,%x) [signature]",
			block.Header.BlockHeight, block.Header.BlockHash, block.Header.Proposer, block.Header.Signature)
	}
	sigLasts := utils.CurrentTimeMillisSeconds() - startSigTick
	timeLasts[BlockSig] = sigLasts

	err := CheckVacuumBlock(block, vb.chainConf.ChainConfig().Consensus.Type)
	if err != nil {
		return nil, nil, timeLasts, nil, err
	}
	// we must new a snapshot for the vacant block,
	// otherwise the subsequent snapshot can not link to the previous snapshot.
	snapshotTick := utils.CurrentTimeMillisSeconds()
	//snapshot := vb.snapshotManager.NewSnapshot(lastBlock, block)
	if len(block.Txs) == 0 {
		if len(block.Dag.Vertexes) != 0 {
			return nil, nil, timeLasts, nil, fmt.Errorf("no txs in block[%x] but dag has vertex",
				block.Header.BlockHash)
		}
		// verify TxRoot
		startRootsTick := utils.CurrentTimeMillisSeconds()
		err = CheckBlockDigests(block, nil, hashType, vb.log)
		if err != nil {
			return nil, nil, timeLasts, nil, err
		}
		rootsLast := utils.CurrentTimeMillisSeconds() - startRootsTick
		timeLasts[TxRoot] = rootsLast
		return nil, nil, timeLasts, nil, nil
	}
	// 1. 空交易
	// 2. recoveryBlock
	// verify if txs are duplicate in this block
	//if TxPoolType != batch.TxPoolType {

	if duplicate, errors := IsTxDuplicate(block.Txs); duplicate {
		return nil, nil, timeLasts, nil, fmt.Errorf("tx duplicate, errors: %v", errors)
	}
	//}

	// simulate with DAG, and verify read write set
	startDbTxTick := utils.CurrentTimeMillisSeconds()
	//vb.storeHelper.BeginDbTransaction(snapshot.GetBlockchainStore(), block.GetTxKey())

	startVMTick := utils.CurrentTimeMillisSeconds()
	//txRWSetMap, txResultMap, err := vb.txScheduler.SimulateWithDag(block, snapshot)
	// Initialize empty RWSet and eventMap to mock actual execution
	txRWSetMap, txResultMap := make(map[string]*commonPb.TxRWSet), make(map[string]*commonPb.Result)
	for _, tx := range block.Txs {
		rwset := &commonPb.TxRWSet{
			TxId:     tx.Payload.TxId,
			TxReads:  nil,
			TxWrites: nil,
		}
		txRWSetMap[tx.Payload.TxId] = rwset
		tx.Result = &commonPb.Result{
			Code: 0,
			ContractResult: &commonPb.ContractResult{
				Code:          0,
				Result:        make([]byte, 0),
				ContractEvent: make([]*commonPb.ContractEvent, 0),
			},
		}
		txResultMap[tx.Payload.TxId] = tx.Result
	}
	vb.log.Info("Skipping execution in ValidateBlock.")
	vmLasts := utils.CurrentTimeMillisSeconds() - startVMTick
	vb.log.Infof("Validate block[%v](txs:%v), time used(new snapshot:%v, start DB transaction:%v, vm:%v)",
		block.Header.BlockHeight, block.Header.TxCount, startDbTxTick-snapshotTick, startVMTick-startDbTxTick, vmLasts)

	timeLasts[VM] = vmLasts
	//if err != nil {
	//	return nil, nil, timeLasts, nil, fmt.Errorf("simulate %s", err)
	//}
	if block.Header.TxCount != uint32(len(txRWSetMap)) || block.Header.TxCount != uint32(len(txResultMap)) {
		return nil, nil, timeLasts, nil, fmt.Errorf("simulate txcount expect %d, got txRWSetMap %d, txResultMap %d",
			block.Header.TxCount, len(txRWSetMap), len(txResultMap))
	}

	// 2.transaction verify
	startTxTick := utils.CurrentTimeMillisSeconds()
	verifierTxConf := &VerifierTxConfig{
		Block:         block,
		TxResultMap:   txResultMap,
		TxRWSetMap:    txRWSetMap,
		ChainConf:     vb.chainConf,
		Log:           vb.log,
		Ac:            vb.ac,
		TxPool:        vb.txPool,
		ProposalCache: vb.proposalCache,
		TxFilter:      vb.txFilter,
	}
	verifiertx := NewVerifierTx(verifierTxConf)
	txHashes, _, rwSetVerifyFailTx, err := verifiertx.verifierTxs(block, mode, NormalVerifyMode)
	txLasts := utils.CurrentTimeMillisSeconds() - startTxTick
	timeLasts[TxVerify] = txLasts
	if err != nil {
		return nil, nil, timeLasts, rwSetVerifyFailTx, fmt.Errorf("verify failed [%d](%x), %s ",
			block.Header.BlockHeight, block.Header.BlockHash, err)
	}
	//if protocol.CONSENSUS_VERIFY == mode && len(newAddTx) > 0 {
	//	v.txPool.AddTrustedTx(newAddTx)
	//}

	// get contract events
	contractEventMap := make(map[string][]*commonPb.ContractEvent, len(block.Txs))
	for _, tx := range block.Txs {
		var events []*commonPb.ContractEvent
		if result, ok := txResultMap[tx.Payload.TxId]; ok {
			events = result.ContractResult.ContractEvent
		}
		contractEventMap[tx.Payload.TxId] = events
	}
	// verify TxRoot
	startRootsTick := utils.CurrentTimeMillisSeconds()
	err = CheckBlockDigests(block, txHashes, hashType, vb.log)
	if err != nil {
		return txRWSetMap, contractEventMap, timeLasts, nil, err
	}
	rootsLast := utils.CurrentTimeMillisSeconds() - startRootsTick
	timeLasts[TxRoot] = rootsLast

	return txRWSetMap, contractEventMap, timeLasts, nil, nil
}

// validateBlock, validate block and transactions
func (vb *VerifierBlock) ValidateBlockWithRWSets(
	block *commonPb.Block, hashType string, timeLasts map[string]int64,
	txRWSetMap map[string]*commonPb.TxRWSet, mode protocol.VerifyMode) (
	map[string][]*commonPb.ContractEvent, map[string]int64, error) {
	// 1.block verify
	if err := IsBlockHashValid(block, vb.chainConf.ChainConfig().Crypto.Hash); err != nil {
		return nil, timeLasts, err
	}
	txResultMap := make(map[string]*commonPb.Result, len(block.GetTxs()))
	for _, tx := range block.GetTxs() {
		if tx.Result != nil {
			txResultMap[tx.Payload.TxId] = tx.Result
		}
	}
	// verify block sig and also verify identity and auth of block proposer
	startSigTick := utils.CurrentTimeMillisSeconds()
	vb.log.DebugDynamic(func() string {
		return fmt.Sprintf("verify block \n %s", utils.FormatBlock(block))
	})
	if ok, err := utils.VerifyBlockSig(hashType, block, vb.ac); !ok || err != nil {
		vb.log.Errorf("verify block signature fail,err:%s", err.Error())
		return nil, timeLasts, fmt.Errorf("(%d,%x - %x,%x) [signature]",
			block.Header.BlockHeight, block.Header.BlockHash, block.Header.Proposer, block.Header.Signature)
	}
	sigLasts := utils.CurrentTimeMillisSeconds() - startSigTick
	timeLasts[BlockSig] = sigLasts

	// we must new a snapshot for the vacant block,
	// otherwise the subsequent snapshot can not link to the previous snapshot.
	//snapshot := vb.snapshotManager.NewSnapshot(lastBlock, block)
	if len(block.Txs) == 0 {
		// verify TxRoot
		startRootsTick := utils.CurrentTimeMillisSeconds()
		err := CheckBlockDigests(block, nil, hashType, vb.log)
		if err != nil {
			return nil, timeLasts, err
		}
		rootsLast := utils.CurrentTimeMillisSeconds() - startRootsTick
		timeLasts[TxRoot] = rootsLast
		return nil, timeLasts, nil
	}

	// simulate with DAG, and verify read write set
	startVMTick := utils.CurrentTimeMillisSeconds()
	//在快速同步模式下，不能开启数据库事务，同步节点直接基于读写集的SQL语句执行，无需开启事务进行模拟执行
	//vb.storeHelper.BeginDbTransaction(snapshot.GetBlockchainStore(), block.GetTxKey())
	//txRWSetMap, txResultMap, err := vb.txScheduler.SimulateWithDag(block, snapshot)
	//if err != nil {
	//	return nil, nil, timeLasts, fmt.Errorf("simulate %s", err)
	//}

	vmLasts := utils.CurrentTimeMillisSeconds() - startVMTick
	timeLasts[VM] = vmLasts

	// 2.transaction verify
	startTxTick := utils.CurrentTimeMillisSeconds()
	verifierTxConf := &VerifierTxConfig{
		Block:         block,
		TxResultMap:   txResultMap,
		TxRWSetMap:    txRWSetMap,
		ChainConf:     vb.chainConf,
		Log:           vb.log,
		Ac:            vb.ac,
		TxPool:        vb.txPool,
		TxFilter:      vb.txFilter,
		ProposalCache: vb.proposalCache,
	}
	verifiertx := NewVerifierTx(verifierTxConf)
	txHashes, err := verifiertx.verifierTxsWithRWSet(block, mode, QuickSyncVerifyMode)
	vb.log.Infof("verifierTxs txHashCount:%d, txCount:%d, %x", len(txHashes), len(block.Txs),
		block.Header.TxRoot)
	txLasts := utils.CurrentTimeMillisSeconds() - startTxTick
	timeLasts[TxVerify] = txLasts
	if err != nil {
		return nil, timeLasts, fmt.Errorf("verify failed [%d](%x), %s ",
			block.Header.BlockHeight, block.Header.BlockHash, err)
	}
	//if protocol.CONSENSUS_VERIFY == mode && len(newAddTx) > 0 {
	//	v.txPool.AddTrustedTx(newAddTx)
	//}

	// get contract events
	contractEventMap := make(map[string][]*commonPb.ContractEvent, len(block.Txs))
	for _, tx := range block.Txs {
		var events []*commonPb.ContractEvent
		if result, ok := txResultMap[tx.Payload.TxId]; ok {
			events = result.ContractResult.ContractEvent
		}
		contractEventMap[tx.Payload.TxId] = events
	}
	// verify TxRoot
	startRootsTick := utils.CurrentTimeMillisSeconds()
	err = CheckBlockDigests(block, txHashes, hashType, vb.log)
	if err != nil {
		return contractEventMap, nil, err
	}
	rootsLast := utils.CurrentTimeMillisSeconds() - startRootsTick
	timeLasts[TxRoot] = rootsLast

	return contractEventMap, nil, nil
}

// nolint: staticcheck
func CheckPreBlock(block *commonPb.Block, lastBlockHash []byte, proposedHeight uint64) error {

	if err := IsHeightValid(block, proposedHeight); err != nil {
		return err
	}
	// check if this block pre hash is equal with last block hash
	return IsPreHashValid(block, lastBlockHash)
}

// nolint: staticcheck
func CheckPreBlockWithTempHash(block *commonPb.Block, lastBlockHash []byte, proposedHeight uint64) error {
	if err := IsHeightValid(block, proposedHeight); err != nil {
		return err
	}
	// check if this block pre hash is equal with last block hash
	return IsPreTempHashValid(block, lastBlockHash)
}

// BlockCommitterImpl implements BlockCommitter interface.
// To commit a block after it is confirmed by consensus module.
type BlockCommitterImpl struct {
	chainId string // chain id, to identity this chain
	// Store is a block store that will only fetch data locally
	blockchainStore protocol.BlockchainStore // blockchain store
	snapshotManager protocol.SnapshotManager // snapshot manager
	txPool          protocol.TxPool          // transaction pool
	chainConf       protocol.ChainConf       // chain config

	ledgerCache             protocol.LedgerCache        // ledger cache
	commitedLedgerCache     protocol.LedgerCache        // ledger cache of commited block
	proposalCache           protocol.ProposalCache      // proposal cache
	log                     protocol.Logger             // logger
	msgBus                  msgbus.MessageBus           // message bus
	mu                      sync.Mutex                  // lock, to avoid concurrent block commit
	subscriber              *subscriber.EventSubscriber // subscriber
	verifier                protocol.BlockVerifier      // block verifier
	txScheduler             protocol.TxScheduler
	commonCommit            *CommitBlock
	metricBlockSize         *prometheus.HistogramVec // metric block size
	metricBlockHeight       *prometheus.GaugeVec     // metric block height
	metricTxCounter         *prometheus.CounterVec   // metric transaction counter
	metricBlockCommitTime   *prometheus.HistogramVec // metric block commit time
	metricBlockIntervalTime *prometheus.HistogramVec // metric block interval time
	metricTpsGauge          *prometheus.GaugeVec     // metric real-time transaction per second (TPS)
	storeHelper             conf.StoreHelper
	blockInterval           int64
	identity                protocol.SigningMember

	mTxCount      uint64 // store tx total count for persistent
	mBlockHeight  uint64 // store latest block height for persistent
	goRoutinePool *ants.Pool
	orderedMutex  *OrderedMutex
	netService    protocol.NetService
	continueChan  chan commonPb.ExeSigInfo

	reserveTable ReserveTable
	AC           protocol.AccessControlProvider
}

type BlockCommitterConfig struct {
	ChainId             string
	BlockchainStore     protocol.BlockchainStore
	SnapshotManager     protocol.SnapshotManager
	TxPool              protocol.TxPool
	LedgerCache         protocol.LedgerCache
	CommitedLedgerCache protocol.LedgerCache
	ProposedCache       protocol.ProposalCache
	ChainConf           protocol.ChainConf
	MsgBus              msgbus.MessageBus
	Subscriber          *subscriber.EventSubscriber
	Verifier            protocol.BlockVerifier
	StoreHelper         conf.StoreHelper
	TxFilter            protocol.TxFilter
	TxScheduler         protocol.TxScheduler
	Identity            protocol.SigningMember
	NetService          protocol.NetService
	ContinueChan        chan commonPb.ExeSigInfo
	AC                  protocol.AccessControlProvider
}

func NewBlockCommitter(config BlockCommitterConfig, log protocol.Logger) (protocol.BlockCommitter, error) {
	var localGoRoutinePool *ants.Pool
	var err error
	poolCapacity := config.StoreHelper.GetPoolCapacity()
	//poolCapacity := 16 * 4
	log.Debugf("GetPoolCapacity() => %v", poolCapacity)
	if localGoRoutinePool, err = ants.NewPool(poolCapacity, ants.WithPreAlloc(false)); err != nil {
		log.Errorf("New Pool error: [%v]", err)
	}

	blockchain := &BlockCommitterImpl{
		chainId:             config.ChainId,
		blockchainStore:     config.BlockchainStore,
		snapshotManager:     config.SnapshotManager,
		txPool:              config.TxPool,
		ledgerCache:         config.LedgerCache,
		commitedLedgerCache: config.CommitedLedgerCache,
		proposalCache:       config.ProposedCache,
		log:                 log,
		chainConf:           config.ChainConf,
		msgBus:              config.MsgBus,
		subscriber:          config.Subscriber,
		verifier:            config.Verifier,
		storeHelper:         config.StoreHelper,
		commonCommit: &CommitBlock{
			store:               config.BlockchainStore,
			txFilter:            config.TxFilter,
			log:                 log,
			snapshotManager:     config.SnapshotManager,
			ledgerCache:         config.LedgerCache,
			commitedLedgerCache: config.CommitedLedgerCache,
			chainConf:           config.ChainConf,
			msgBus:              config.MsgBus,
		},
		txScheduler: config.TxScheduler,
		identity:    config.Identity,
		reserveTable: ReserveTable{
			Table: make(map[string]*RTableItem),
			mutex: sync.RWMutex{},
		},
		goRoutinePool: localGoRoutinePool,
		orderedMutex:  NewOrderedMutex(log),
		netService:    config.NetService,
		continueChan:  config.ContinueChan,
		AC:            config.AC,
	}

	if localconf.ChainMakerConfig.MonitorConfig.Enabled {
		blockchain.initMetrics()
	}

	return blockchain, nil
}

func (chain *BlockCommitterImpl) isBlockLegal(blk *commonPb.Block) error {
	lastCommitedBlock := chain.commitedLedgerCache.GetLastBlock()
	if lastCommitedBlock == nil {
		// 获取上一区块
		// 首次进入，从DB获取最新区块
		return fmt.Errorf("get last block == nil ")
	}
	chain.log.Infof("core loop: lastCommitedBlock[%d]", lastCommitedBlock.Header.BlockHeight)
	chain.log.Infof("core loop: blk[%d]", blk.Header.BlockHeight)
	if lastCommitedBlock.Header.BlockHeight >= blk.Header.BlockHeight {
		return commonErrors.ErrBlockHadBeenCommited
	}
	// block height verify
	if blk.Header.BlockHeight != lastCommitedBlock.Header.BlockHeight+1 {
		return fmt.Errorf("isBlockLegal() failed: Height is less than chaintip")
	}
	// block pre hash verify
	if !bytes.Equal(blk.Header.PreBlockHash, lastCommitedBlock.Header.BlockHash) {
		return fmt.Errorf("isBlockLegal() failed: PrevHash invalid (%x != %x)",
			blk.Header.PreBlockHash, lastCommitedBlock.Header.BlockHash)
	}

	blkHash, err := utils.CalcBlockHash(chain.chainConf.ChainConfig().Crypto.Hash, blk)
	if err != nil || !bytes.Equal(blk.Header.BlockHash, blkHash) {
		return fmt.Errorf("isBlockLegal() failed: BlockHash invalid (%x != %x)",
			blkHash, blk.Header.BlockHash)
	}

	return nil
}

func (chain *BlockCommitterImpl) formalizeBlock(block *commonPb.Block,
	txRWSetMap map[string]*commonPb.TxRWSet, lastBlock *commonPb.Block,
	contractEventMap map[string][]*commonPb.ContractEvent) error {
	var aclFailTxs = make([]*commonPb.Transaction, 0) // No need to ACL check, this slice is empty
	err := FinalizeBlock(
		block,
		txRWSetMap,
		aclFailTxs,
		chain.chainConf.ChainConfig().Crypto.Hash,
		chain.log)
	if err != nil {
		return fmt.Errorf("finalizeBlock block(%d,%s) error %s",
			block.Header.BlockHeight, hex.EncodeToString(block.Header.BlockHash), err)
	}
	// get txs schedule timeout and put back to txpool
	var txsTimeout = make([]*commonPb.Transaction, 0)
	var fetchBatches = make([][]*commonPb.Transaction, 0)
	if len(txRWSetMap) < len(block.Txs) {
		// if tx not in txRWSetMap, tx should be put back to txpool
		for _, tx := range block.Txs {
			if _, ok := txRWSetMap[tx.Payload.TxId]; !ok {
				txsTimeout = append(txsTimeout, tx)
			}
		}

		if TxPoolType == batch.TxPoolType {
			// retry the timeout 's tx and get the new batchIds
			batchIds, _, err := GetBatchIds(block)
			if err != nil {
				return fmt.Errorf("finalizeBlock block(%d,%s) error %s: deserialize batchInfo",
					block.Header.BlockHeight, hex.EncodeToString(block.Header.BlockHash), err)
			}
			batchIds, fetchBatches = chain.txPool.ReGenTxBatchesWithRetryTxs(block.Header.BlockHeight, batchIds,
				block.Txs)
		} else {
			RetryAndRemoveTxs(chain.txPool, txsTimeout, nil, chain.log)
		}
		block.Header.TxCount = uint32(len(block.Txs))
	}
	if TxPoolType == batch.TxPoolType {
		var batchIdBytes []byte
		// set batchIds into additional data
		batchIds, _, err := GetBatchIds(block)
		batchIdBytes, err = SerializeTxBatchInfo(batchIds, block.Txs, fetchBatches, chain.log)
		if err != nil {
			return fmt.Errorf("finalizeBlock block(%d,%s) error %s",
				block.Header.BlockHeight, hex.EncodeToString(block.Header.BlockHash), err)
		}
		block.AdditionalData.ExtraData[batch.BatchPoolAddtionalDataKey] = batchIdBytes
		chain.log.InfoDynamic(func() string {
			return fmt.Sprintf("[%v] proposer add batchIds:%v into addition data", block.Header.BlockHeight,
				func() []string {
					var batch0 []string
					for i := range batchIds {
						batch0 = append(batch0, hex.EncodeToString([]byte(batchIds[i])))
					}
					return batch0
				}())
		})
	}

	block.Header.PreBlockHash = lastBlock.Header.BlockHash
	blkHash, err := utils.CalcBlockHash(chain.chainConf.ChainConfig().Crypto.Hash, block)
	block.Header.BlockHash = blkHash
	// cache proposed block
	chain.log.Debugf("set proposed block(%d,%x)", block.Header.BlockHeight, block.Header.BlockHash)
	if err = chain.proposalCache.SetProposedBlock(block, txRWSetMap, contractEventMap, true); err != nil {
		return err
	}
	chain.proposalCache.SetProposedAt(block.Header.BlockHeight)
	return nil
}

// VerifyBlockSig verify block proposer and signature
func VerifyBlockSig(sigInfo *commonPb.ExeSigInfo, ac protocol.AccessControlProvider) (bool, error) {
	endorsements := []*commonPb.EndorsementEntry{{
		Signer:    sigInfo.Signer,
		Signature: sigInfo.Sig,
	}}
	principal, err := ac.CreatePrincipal(protocol.ResourceNameConsensusNode, endorsements, sigInfo.BlockHash)
	if err != nil {
		return false, fmt.Errorf("fail to construct authentication principal: %v", err)
	}
	ok, err := ac.VerifyPrincipal(principal)
	if err != nil {
		return false, fmt.Errorf("authentication fail: %v", err)
	}
	if !ok {
		return false, fmt.Errorf("authentication fail")
	}
	return true, nil
}

func (chain *BlockCommitterImpl) AddBlock(block *commonPb.Block) (err error) {
	defer func() {
		if panicError := recover(); panicError != nil {

			if sqlErr := chain.storeHelper.RollBack(block, chain.blockchainStore); sqlErr != nil {
				chain.log.Errorf("block [%d] rollback sql failed: %s", block.Header.BlockHeight, sqlErr)
			}

			panic(fmt.Sprintf("cache add block fail, panic: %v %s", panicError, debug.Stack()))
		}
		if err != nil {
			if err == commonErrors.ErrBlockHadBeenCommited {
				chain.log.Warnf("cache add block fail, err: %v", err)
				return
			}
			if sqlErr := chain.storeHelper.RollBack(block, chain.blockchainStore); sqlErr != nil {
				chain.log.Errorf("block [%d] rollback sql failed: %s", block.Header.BlockHeight, sqlErr)
				panic("add block err: " + err.Error() + string(debug.Stack()))
			}
		}
	}()
	chain.ledgerCache.SetLastBlock(block)

	// synchronize new block height to consensus and sync module
	lastProposed, rwSetMap, conEventMap := chain.proposalCache.GetProposedBlock(block)
	rwSet := utils.RearrangeRWSet(block, rwSetMap)
	blockInfo := &commonPb.BlockInfo{
		Block:     block,
		RwsetList: rwSet,
	}
	chain.msgBus.PublishSafe(msgbus.BlockInfo, blockInfo)

	lastBlock := chain.commitedLedgerCache.GetLastBlock()
	snapshot := chain.snapshotManager.NewSnapshot(lastBlock, block)

	lastCommittedHeight, err := chain.commitedLedgerCache.CurrentHeight()
	if err != nil {
		chain.log.Errorf("core loop: get last committed block height err: %v", err)
	}
	if chain.chainConf.ChainConfig().Consensus.Type == consensus.ConsensusType_TBFT &&
		int64(block.Header.BlockHeight)-int64(lastCommittedHeight) < 1 {
		return fmt.Errorf("no need to schedule old block, ledger height: %d, block height: %d",
			lastCommittedHeight, block.Header.BlockHeight)
	}

	successTxChan := make(chan *TxTask, len(block.Txs))
	executedTxChan := make(chan *TxTask, len(block.Txs))
	abortedTxChan := make(chan *TxTask, len(block.Txs))

	err = chain.optimisticExecuteTxs(block, snapshot, executedTxChan)
	if err != nil {
		chain.log.Errorf("core loop: execute txs failed: %s", err)
		return err
	}
	chain.orderedMutex.Lock(int(block.Header.BlockHeight))
	defer func() {
		chain.orderedMutex.Unlock(int(block.Header.BlockHeight))
		chain.log.Infof("core loop: unlock[h:%d]", block.Header.BlockHeight)
	}()
	chain.log.Infof("core loop: lock[h:%d]", block.Header.BlockHeight)

	//chain.mu.Lock()
	//defer chain.mu.Unlock() ////////////////////////////////////////////////////////////////////////////

	chain.log.Infof("core loop: Entered critical Section blk.height[%d]", block.Header.BlockHeight)
	snapshot.Seal()
	lastBlock = chain.commitedLedgerCache.GetLastBlock()
	snapshot = chain.snapshotManager.NewSnapshot(lastBlock, block)
	chain.storeHelper.BeginDbTransaction(snapshot.GetBlockchainStore(), block.GetTxKey())

	for len(executedTxChan) > 0 {
		close(executedTxChan)
		abortedTxChan = make(chan *TxTask, len(block.Txs))
		chain.detectConflict(executedTxChan, abortedTxChan, successTxChan, &snapshot)
		close(abortedTxChan)
		executedTxChan = make(chan *TxTask, len(block.Txs))
		chain.reExecuteTxs(block, abortedTxChan, &snapshot, successTxChan, executedTxChan)
	}
	snapshot.Seal()

	txRWSetMap, contractEventMap, err := chain.txScheduler.GetResultMaps(block, block.Txs, snapshot)
	err = chain.formalizeBlock(block, txRWSetMap, lastBlock, contractEventMap)
	if err != nil {
		return fmt.Errorf("error when formalizing the block, %s", err)
	}
	startTick := utils.CurrentTimeMillisSeconds()
	chain.log.Debugf("add block(%d,%x)=(%x,%d,%d)", block.Header.BlockHeight, block.Header.BlockHash,
		block.Header.PreBlockHash, block.Header.TxCount, len(block.Txs))

	height := block.Header.BlockHeight

	if err = chain.isBlockLegal(block); err != nil {
		if err == commonErrors.ErrBlockHadBeenCommited {
			chain.log.Warnf("block illegal [%d](hash:%x), %s", height, block.Header.BlockHash, err)
			return err
		}

		chain.log.Errorf("block illegal [%d](hash:%x), %s", height, block.Header.BlockHash, err)
		return err
	}

	lastProposed, rwSetMap, conEventMap = chain.proposalCache.GetProposedBlock(block)

	if lastProposed == nil {
		if lastProposed, rwSetMap, conEventMap, err = chain.checkLastProposedBlock(block); err != nil {
			return err
		}
	} else if IfOpenConsensusMessageTurbo(chain.chainConf) {
		// recover the block for proposer when enable the conensus message turbo function.
		lastProposed.Header = block.Header
	}

	// put consensus qc into block
	lastProposed.AdditionalData = block.AdditionalData
	// shallow copy, create a new block to prevent panic during storage in marshal
	commitBlock := CopyBlock(lastProposed)

	chain.log.Infof("core loop: before nodeconfig")
	jsonData, err := json.Marshal(localconf.ChainMakerConfig.NodeConfig)
	chain.log.Infof("core loop: nodeconfig: %v", string(jsonData))
	blkHash, sig, err := utils.SignBlock(chain.chainConf.ChainConfig().Crypto.Hash, chain.identity, block)
	if err != nil {
		chain.log.Errorf("sign block failed: %s", err)
		return err
	}
	signer, err := chain.identity.GetMember()
	exeSigInfo := &commonPb.ExeSigInfo{
		Sig:       sig,
		Height:    height,
		BlockHash: blkHash,
		NodeId:    localconf.ChainMakerConfig.NodeConfig.NodeId,
		Signer:    signer,
	}
	infoBytes, err := proto.Marshal(exeSigInfo)
	err = chain.netService.BroadcastMsg(infoBytes, netpb.NetMsg_EXE_SIGNATURE)
	if err != nil {
		chain.log.Errorf("core loop: broadcast message failed: %s", err)
		return err
	}
	chain.log.Infof("core loop: after broadcastmsg, height[%d]", height)

	block.Header.ExeSignatures = make([]*commonPb.ExeSigRec, 0)
	nNodes := len(chain.chainConf.ChainConfig().Consensus.Nodes)
	for i := 1; i < nNodes && len(block.Header.ExeSignatures) < (nNodes*2+3)/3; i++ {
		select {
		case sigInfo := <-chain.continueChan:
			chain.log.Infof("core loop: receive result from continueChan, height[%d]", height)
			chain.log.Infof("core loop: ExeSig, sig:%x, height:%d, block_hash:%x, node_id:%s",
				sigInfo.Sig, sigInfo.Height, sigInfo.BlockHash, sigInfo.NodeId)
			res, err := VerifyBlockSig(&sigInfo, chain.AC)
			if err != nil {
				chain.log.Errorf("core loop: verify block signature failed: %s", err)
			}
			chain.log.Infof("core loop: sig validation result[h:%d][%v]", height, res)
			if res {
				block.Header.ExeSignatures = append(block.Header.ExeSignatures,
					&commonPb.ExeSigRec{
						Sig:    sigInfo.Sig,
						NodeId: sigInfo.NodeId,
						Signer: sigInfo.Signer,
					})
			}
		case <-time.After(time.Second * 2):
			chain.log.Errorf("core loop: exe signature[%d/%d] waiting timeout, height[%d]", len(block.Header.ExeSignatures), nNodes, height)
			//panic("core loop: exe signature waiting timeout")
			break
		}
	}
	if len(block.Header.ExeSignatures) < (nNodes*2+3)/3 {
		chain.log.Errorf("core loop: fail to reach exe signature consensus. node[%v] height[%d]",
			localconf.ChainMakerConfig.NodeConfig.NodeId, height)
	}

	checkLasts := utils.CurrentTimeMillisSeconds() - startTick
	dbLasts, snapshotLasts, confLasts, otherLasts, pubEvent, filterLasts, blockInfo, err :=
		chain.commonCommit.CommitBlock(commitBlock, rwSetMap, conEventMap) // use commitBlock
	if err != nil {
		chain.log.Errorf("block common commit failed: %s, blockHeight: (%d)",
			err.Error(), lastProposed.Header.BlockHeight)
	}

	// Remove txs from txpool. Remove will invoke proposeSignal from txpool if pool size > txcount
	startPoolTick := utils.CurrentTimeMillisSeconds()
	txRetry, batchRetry, batchIds, err := chain.syncWithTxPool(lastProposed, height)
	if err != nil {
		return err
	}

	if TxPoolType == batch.TxPoolType {
		chain.log.Infof("remove batchId[%d] and retry batchId[%d] in add block", len(batchIds), len(batchRetry))
		chain.txPool.RetryAndRemoveTxBatches(batchRetry, batchIds)
	} else {
		chain.log.Infof("remove txs[%d] and retry txs[%d] in add block", len(lastProposed.Txs), len(txRetry))
		RetryAndRemoveTxs(chain.txPool, txRetry, lastProposed.Txs, chain.log)
	}

	poolLasts := utils.CurrentTimeMillisSeconds() - startPoolTick

	chain.proposalCache.ClearProposedBlockAt(height)

	// clear propose repeat map before send
	ClearProposeRepeatTimerMap()

	curTime := utils.CurrentTimeMillisSeconds()
	elapsed := curTime - startTick
	interval := curTime - chain.blockInterval
	chain.blockInterval = curTime
	chain.log.Infof(
		"commit block [%d](count:%d,hash:%x)"+
			"time used(check:%d,db:%d,ss:%d,conf:%d,pool:%d,pubConEvent:%d,filter:%d,other:%d,total:%d,interval:%d)",
		height, lastProposed.Header.TxCount, lastProposed.Header.BlockHash,
		checkLasts, dbLasts, snapshotLasts, confLasts, poolLasts, pubEvent, filterLasts, otherLasts, elapsed, interval)
	chain.log.Infof("core loop: h[%d] tx[%v] txType[%v] contract[%v] method[%v]",
		height, block.Txs[0].Payload.TxId, block.Txs[0].Payload.TxType,
		block.Txs[0].Payload.ContractName, block.Txs[0].Payload.Method)
	for i, pair := range block.Txs[0].Payload.Parameters {
		if pair.Key == "CONTRACT_BYTECODE" {
			chain.log.Infof("core loop: h[%d]p[%d]: %s-", height, i, pair.Key)
		} else {
			chain.log.Infof("core loop: h[%d]p[%d]: %s->%v", height, i, pair.Key, string(pair.Value))
		}
	}
	chain.log.Errorf("perf: commit blk[%d](size[%d]), now[%d]", block.Header.BlockHeight, len(block.Txs), time.Now().UnixMicro())
	if localconf.ChainMakerConfig.MonitorConfig.Enabled {
		blockInfoTmp := *blockInfo
		go chain.updateMetrics(&blockInfoTmp, elapsed, interval)
	}
	return nil
}

func (chain *BlockCommitterImpl) detectConflict(executedTxChan chan *TxTask,
	abortedTxChan chan *TxTask, successTxChan chan *TxTask, snapshot *protocol.Snapshot) {
	wg := sync.WaitGroup{}
	wg.Add(len(executedTxChan))
	i := 0
	for t := range executedTxChan {
		localT := t
		err := chain.goRoutinePool.Submit(func() {
			defer wg.Done()
			handleDetectConflict(chain, i, localT, abortedTxChan, successTxChan, snapshot)
		})
		if err != nil {
			chain.log.Errorf("goRoutinePool.Submit Error: %s", err.Error())
		}
		i++
	}
	wg.Wait()
}

func handleDetectConflict(chain *BlockCommitterImpl, idx int, tt *TxTask,
	abortedTxChan chan *TxTask, successTxChan chan *TxTask, snapshot *protocol.Snapshot) {
	txRWSet := tt.exeResult.TxSimCtx.GetTxRWSet(tt.exeResult.RunVmSuccess)

	for _, rs := range txRWSet.TxReads {
		finalStoreKey := snapshot2.ConstructKey(rs.ContractName, rs.Key)
		if ok, _ := chain.reserveTable.CheckDirtyWrite(finalStoreKey, tt.ReserveId); !ok {
			abortedTxChan <- tt
			return
		}
	}
	for _, ws := range txRWSet.TxWrites {
		finalStoreKey := snapshot2.ConstructKey(ws.ContractName, ws.Key)
		if ok, _ := chain.reserveTable.CheckDirtyWrite(finalStoreKey, tt.ReserveId); !ok {
			abortedTxChan <- tt
			return
		}
	}
	for _, ws := range txRWSet.TxWrites {
		finalStoreKey := snapshot2.ConstructKey(ws.ContractName, ws.Key)
		chain.reserveTable.ReleaseOwnership(finalStoreKey, tt.ReserveId)
	}
	success, appliedNum := (*snapshot).ApplyTxSimContext(tt.exeResult.TxSimCtx, tt.exeResult.SpecialTxType,
		tt.exeResult.RunVmSuccess, false)
	if !success {
		chain.log.Infof("core loop: success?[%v], appliedNum[%v]", success, appliedNum)
	}
	successTxChan <- tt
}

func (chain *BlockCommitterImpl) reExecuteTxs(block *commonPb.Block,
	abortedTxChan chan *TxTask, snapshot *protocol.Snapshot,
	successTxChan chan *TxTask, executedTxChan chan *TxTask) {
	if len(abortedTxChan) == 0 {
		return
	}
	wg := sync.WaitGroup{}
	wg.Add(len(abortedTxChan))

	i := 0
	for t := range abortedTxChan {
		localT := t
		err := chain.goRoutinePool.Submit(func() {
			defer wg.Done()
			handleReExecuteTxs(chain, i, localT, block, snapshot, successTxChan, executedTxChan)
		})
		if err != nil {
			chain.log.Errorf("goRoutinePool.Submit Error: %s", err.Error())
		}
		i++
	}
	wg.Wait()
}

func handleReExecuteTxs(chain *BlockCommitterImpl, idx int, tt *TxTask, block *commonPb.Block,
	snapshot *protocol.Snapshot, successTxChan chan *TxTask,
	executedTxChan chan *TxTask) {
	tt.exeResult = chain.txScheduler.ExecuteTx(tt.Tx, *snapshot, block)
	tt.Tx.Result = tt.exeResult.TxSimCtx.GetTxResult()
	chain.log.DebugDynamic(func() string {
		return fmt.Sprintf("handleTx(`%v`) => ExecuteTx(...) => runVmSuccess = %v", tt.Tx.GetPayload().TxId, tt.exeResult.RunVmSuccess)
	})
	txRWSet := tt.exeResult.TxSimCtx.GetTxRWSet(tt.exeResult.RunVmSuccess)
	if txRWSet.TxWrites == nil || len(txRWSet.TxWrites) == 0 {
		successTxChan <- tt
		(*snapshot).ApplyTxSimContext(tt.exeResult.TxSimCtx, tt.exeResult.SpecialTxType, tt.exeResult.RunVmSuccess, false)
	}
	for _, ws := range txRWSet.TxWrites {
		finalStoreKey := snapshot2.ConstructKey(ws.ContractName, ws.Key)
		chain.reserveTable.NaiveTryTakeOwnership(finalStoreKey, tt.ReserveId)
	}
	executedTxChan <- tt
}

func (chain *BlockCommitterImpl) optimisticExecuteTxs(block *commonPb.Block,
	snapshot protocol.Snapshot, executedTxChan chan *TxTask) error {
	wg := sync.WaitGroup{}
	wg.Add(len(block.Txs))
	hashType := chain.chainConf.ChainConfig().Crypto.Hash
	batchHash, err := GetBatchHash(hashType, block.Txs)
	if err != nil {
		chain.log.Errorf("batch hash err: %v", err)
		return err
	}
	for _, t := range block.Txs {
		localT := t
		err = chain.goRoutinePool.Submit(func() {
			defer wg.Done()
			handleOptimisticExecuteTxs(chain, localT, block, hashType, batchHash, snapshot, executedTxChan)
		})
		if err != nil {
			chain.log.Errorf("goRoutinePool.Submit Error: %s", err.Error())
		}
	}
	wg.Wait()
	return nil
}

func handleOptimisticExecuteTxs(chain *BlockCommitterImpl,
	tx *commonPb.Transaction, block *commonPb.Block, hashType string, batchHash []byte,
	snapshot protocol.Snapshot, executedTxChan chan *TxTask) {
	exeRes := chain.txScheduler.ExecuteTx(tx, snapshot, block)
	tx.Result = exeRes.TxSimCtx.GetTxResult()
	chain.log.DebugDynamic(func() string {
		return fmt.Sprintf("handleTx(`%v`) => ExecuteTx(...) => runVmSuccess = %v", tx.GetPayload().TxId, exeRes.RunVmSuccess)
	})
	rwset := exeRes.TxSimCtx.GetTxRWSet(exeRes.RunVmSuccess)
	txReserveID, err2 := CalReserveTxID(hashType, tx, batchHash, block.Header.BlockHeight)
	if err2 != nil {
		chain.log.Errorf("batch hash err: %v", err2)
	}
	for _, wRes := range rwset.TxWrites {
		finalStoreKey := snapshot2.ConstructKey(wRes.ContractName, wRes.Key)
		chain.reserveTable.TryTakeOwnership(finalStoreKey, txReserveID)
	}
	executedTxChan <- &TxTask{
		Tx:        tx,
		ReserveId: txReserveID,
		exeResult: exeRes,
		uniqueTag: rand.Int63(),
	}
}

// RetryAndRemoveTxs filter charging gas tx out before call tx pool
func RetryAndRemoveTxs(
	txPool protocol.TxPool,
	txsRetry []*commonPb.Transaction,
	txsRem []*commonPb.Transaction,
	log protocol.Logger) {
	var txs []*commonPb.Transaction
	if len(txsRetry) > 0 {
		txs = filterTxsForTxPool(txsRetry, log)
	}
	txPool.RetryAndRemoveTxs(txs, txsRem)
}

// filterTxsForTxPool filter charging gas tx out
func filterTxsForTxPool(txs []*commonPb.Transaction, log protocol.Logger) []*commonPb.Transaction {
	filteredTxs := make([]*commonPb.Transaction, 0, len(txs))
	for _, tx := range txs {
		if !isOptimizedChargingGasTx(tx) {
			filteredTxs = append(filteredTxs, tx)
		} else {
			log.Debugf("discard charging gas tx, id = %v", tx.Payload.TxId)
		}
	}
	return filteredTxs
}

func (chain *BlockCommitterImpl) syncWithTxPool(block *commonPb.Block, height uint64) (
	[]*commonPb.Transaction, []string, []string, error) {
	proposedBlocks := chain.proposalCache.GetProposedBlocksAt(height)
	txRetry := make([]*commonPb.Transaction, 0, len(block.Txs))
	batchRetry := make([]string, 0, len(block.Txs))
	chain.log.Debugf("has %d blocks in height: %d", len(proposedBlocks), height)
	keepTxs := make(map[string]struct{}, len(block.Txs))
	keepBatchIds := make(map[string]struct{}, len(block.Txs))

	if TxPoolType == batch.TxPoolType {
		batchIds, _, err := GetBatchIds(block)
		if err != nil {
			return nil, nil, nil, err
		}
		for _, batchId := range batchIds {
			keepBatchIds[batchId] = struct{}{}
		}
		for _, b := range proposedBlocks {
			if bytes.Equal(b.Header.BlockHash, block.Header.BlockHash) {
				continue
			}

			retryBatchIds, _, err := GetBatchIds(b)
			if err != nil {
				return nil, nil, batchIds, err
			}
			for _, retryBatchId := range retryBatchIds {
				if _, ok := keepBatchIds[retryBatchId]; !ok {
					batchRetry = append(batchRetry, retryBatchId)
				}
			}
		}
		return txRetry, batchRetry, batchIds, nil
	}

	// normal tx pool
	for _, tx := range block.Txs {
		keepTxs[tx.Payload.TxId] = struct{}{}
	}
	for _, b := range proposedBlocks {
		if bytes.Equal(b.Header.BlockHash, block.Header.BlockHash) {
			continue
		}
		for _, tx := range b.Txs {
			if _, ok := keepTxs[tx.Payload.TxId]; !ok {
				txRetry = append(txRetry, tx)
			}
		}
	}

	return txRetry, batchRetry, nil, nil
}

func isOptimizedChargingGasTx(t *commonPb.Transaction) bool {

	return false
}

// nolint: ineffassign, staticcheck
func (chain *BlockCommitterImpl) checkLastProposedBlock(block *commonPb.Block) (
	*commonPb.Block, map[string]*commonPb.TxRWSet, map[string][]*commonPb.ContractEvent, error) {
	err := chain.verifier.VerifyBlock(block, protocol.SYNC_VERIFY)
	if err != nil {
		chain.log.Error("block verify failed [%d](hash:%x), %s",
			block.Header.BlockHeight, block.Header.BlockHash, err)
		return nil, nil, nil, err
	}

	lastProposed, rwSetMap, conEventMap := chain.proposalCache.GetProposedBlock(block)
	if lastProposed == nil {
		chain.log.Error("block not verified [%d](hash:%x)", block.Header.BlockHeight, block.Header.BlockHash)
		return lastProposed, rwSetMap, conEventMap,
			fmt.Errorf("block not verified [%d](hash:%x)", block.Header.BlockHeight, block.Header.BlockHash)
	}
	return lastProposed, rwSetMap, conEventMap, nil
}

func IfOpenConsensusMessageTurbo(chainConf protocol.ChainConf) bool {
	consensusTurboConfig := chainConf.ChainConfig().Core.ConsensusTurboConfig
	if consensusTurboConfig != nil &&
		consensusTurboConfig.ConsensusMessageTurbo &&
		chainConf.ChainConfig().Consensus.Type != consensus.ConsensusType_SOLO {
		return true
	}
	return false
}

func GetProposerId(
	ac protocol.AccessControlProvider,
	netService protocol.NetService,
	proposer *accesscontrol.Member) (string, error) {

	member, err := ac.NewMember(proposer)
	if err != nil {
		return "", err
	}

	certId := member.GetMemberId()
	proposerId, err := netService.GetNodeUidByCertId(certId)
	if err != nil {
		return "", err
	}

	return proposerId, nil
}

func GetTurboBlock(block, turboBlock *commonPb.Block, logger protocol.Logger) *commonPb.Block {
	turboBlock.Header = block.Header
	turboBlock.Dag = block.Dag
	turboBlock.AdditionalData = block.AdditionalData

	if TxPoolType == batch.TxPoolType {
		logger.Debugf("turn on consensus message turbo, block[%d]", turboBlock.Header.BlockHeight)
		return turboBlock
	}

	newTxs := make([]*commonPb.Transaction, len(block.Txs))
	for i := range block.Txs {
		newPayload := &commonPb.Payload{
			TxId: block.Txs[i].Payload.TxId,
		}

		newTxs[i] = &commonPb.Transaction{
			Payload:   newPayload,
			Result:    block.Txs[i].Result,
			Sender:    block.Txs[i].Sender,
			Endorsers: block.Txs[i].Endorsers,
			Payer:     block.Txs[i].Payer,
		}

	}
	turboBlock.Txs = newTxs
	logger.Debugf("turn on consensus message turbo, block[%d]", turboBlock.Header.BlockHeight)

	return turboBlock
}

func RecoverBlock(
	block *commonPb.Block,
	mode protocol.VerifyMode,
	chainConf protocol.ChainConf,
	txPool protocol.TxPool,
	ac protocol.AccessControlProvider,
	netService protocol.NetService,
	logger protocol.Logger) (*commonPb.Block, []string, error) {

	if TxPoolType == batch.TxPoolType {
		return recoverBlockByBatch(block, mode, chainConf, txPool, ac, netService, logger)
	}

	return recoverBlock(block, mode, chainConf, txPool, ac, netService, logger)
}

func recoverBlockByBatch(
	block *commonPb.Block,
	mode protocol.VerifyMode,
	chainConf protocol.ChainConf,
	txPool protocol.TxPool,
	ac protocol.AccessControlProvider,
	netService protocol.NetService,
	logger protocol.Logger) (*commonPb.Block, []string, error) {

	if len(block.Txs) == 0 && block.Header.TxCount != 0 && mode != protocol.SYNC_VERIFY {

		newBlock := &commonPb.Block{
			Header:         block.Header,
			Dag:            block.Dag,
			Txs:            make([]*commonPb.Transaction, block.Header.TxCount),
			AdditionalData: block.AdditionalData,
		}

		maxRetryTime := chainConf.ChainConfig().Core.ConsensusTurboConfig.RetryTime
		retryInterval := chainConf.ChainConfig().Core.ConsensusTurboConfig.RetryInterval
		timeOut := int(maxRetryTime * retryInterval)
		if timeOut <= 0 {
			timeOut = DEFAULTTIMEOUT
		}

		proposerId, err := GetProposerId(ac, netService, block.Header.Proposer)
		if err != nil {
			return nil, nil, err
		}

		batchIds, indexes, err := GetBatchIds(block)
		if err != nil {
			return nil, nil, err
		}
		if len(indexes) != int(block.Header.TxCount) {
			return nil, nil, fmt.Errorf("recover block by batch fail, height: %d, txs: %d, indexes: %d",
				block.Header.BlockHeight, block.Header.TxCount, len(indexes))
		}
		if len(batchIds) == 0 {
			logger.DebugDynamic(func() string {
				return fmt.Sprintf("batchIds is nil, not need to recover the block[%d], additionalData :%v",
					block.Header.BlockHeight, block.AdditionalData.ExtraData)
			})
			return &commonPb.Block{
				Header:         block.Header,
				Dag:            block.Dag,
				Txs:            block.Txs,
				AdditionalData: block.AdditionalData,
			}, batchIds, nil
		}

		txs, err := txPool.GetAllTxsByBatchIds(
			batchIds,
			proposerId,
			block.Header.BlockHeight,
			timeOut)

		if err != nil {
			return nil, nil, err
		}

		//in most cases the number is the same
		newTxs := make([]*commonPb.Transaction, 0, int(block.Header.TxCount))
		for _, tx := range txs {
			newTxs = append(newTxs, tx...)
		}

		logger.Infof(fmt.Sprintf("get add txs by batchIds,height:%d, batchIds:%v, num:%d",
			block.Header.BlockHeight, batchIds, len(newTxs)))

		if len(newTxs) != int(block.Header.TxCount) {
			return nil, nil,
				fmt.Errorf("GetAllTxsByBatchIds fail,height: %d, want count: %d, got count: %d",
					block.Header.BlockHeight, block.Header.TxCount, len(newTxs))
		}

		for i, v := range indexes {
			newBlock.Txs[i] = newTxs[int(v)]
		}

		return newBlock, batchIds, nil
	}

	batchIds, _, err := GetBatchIds(block)
	if err != nil {
		return nil, nil, err
	}

	return &commonPb.Block{
		Header:         block.Header,
		Dag:            block.Dag,
		Txs:            block.Txs,
		AdditionalData: block.AdditionalData,
	}, batchIds, nil
}

func recoverBlock(
	block *commonPb.Block,
	mode protocol.VerifyMode,
	chainConf protocol.ChainConf,
	txPool protocol.TxPool,
	ac protocol.AccessControlProvider,
	netService protocol.NetService,
	logger protocol.Logger) (*commonPb.Block, []string, error) {

	if IfOpenConsensusMessageTurbo(chainConf) && mode != protocol.SYNC_VERIFY &&
		len(block.Txs) != 0 && block.Txs[0].Payload != nil {

		newBlock := &commonPb.Block{
			Header:         block.Header,
			Dag:            block.Dag,
			Txs:            make([]*commonPb.Transaction, len(block.Txs)),
			AdditionalData: block.AdditionalData,
		}

		txIds := utils.GetTxIds(block.Txs)
		maxRetryTime := chainConf.ChainConfig().Core.ConsensusTurboConfig.RetryTime
		retryInterval := chainConf.ChainConfig().Core.ConsensusTurboConfig.RetryInterval

		proposerId, err := GetProposerId(ac, netService, block.Header.Proposer)
		if err != nil {
			return nil, nil, err
		}

		txsMap, err := txPool.GetAllTxsByTxIds(txIds, proposerId, block.Header.BlockHeight,
			int(maxRetryTime*retryInterval))
		if err != nil {
			return nil, nil, err
		}

		for i := range block.Txs {
			newBlock.Txs[i] = txsMap[block.Txs[i].Payload.TxId]
			newBlock.Txs[i].Result = block.Txs[i].Result
			logger.Debugf("recover the block[%d], TxId[%s, %s]",
				newBlock.Header.BlockHeight, newBlock.Txs[i].Payload.TxId, newBlock.Txs[i].Payload.ContractName)
		}

		return newBlock, nil, nil
	}

	// new a block to avoid use the same pointer with consensus.
	return &commonPb.Block{
		Header:         block.Header,
		Dag:            block.Dag,
		Txs:            block.Txs,
		AdditionalData: block.AdditionalData,
	}, nil, nil

}

func SerializeTxBatchInfo(batchIds []string, txs []*commonPb.Transaction,
	fetchBatches [][]*commonPb.Transaction, logger protocol.Logger) ([]byte, error) {

	fetchTxs := make([]*commonPb.Transaction, 0)
	for _, fetchBatch := range fetchBatches {
		fetchTxs = append(fetchTxs, fetchBatch...)
	}

	txIndex := make(map[string]uint32, len(fetchTxs))
	for index, tx := range fetchTxs {
		txIndex[tx.Payload.TxId] = uint32(index)
	}

	txBatchInfo := &commonPb.TxBatchInfo{
		BatchIds: batchIds,
		Index:    make([]uint32, 0),
	}

	indexes := make([]uint32, 0)
	for _, tx := range txs {
		if index, ok := txIndex[tx.Payload.TxId]; ok {
			indexes = append(indexes, index)
		}
	}

	txBatchInfo.Index = indexes
	buffer, err := proto.Marshal(txBatchInfo)
	if err != nil {
		return nil, err
	}

	return buffer, err
}

func DeserializeTxBatchInfo(data []byte) (*commonPb.TxBatchInfo, error) {

	txBatchInfo := new(commonPb.TxBatchInfo)
	err := proto.Unmarshal(data, txBatchInfo)
	if err != nil {
		return nil, err
	}

	return txBatchInfo, nil
}

// metric tx counter key in db
const dbKeyTxCounterPrefix = monitor.SUBSYSTEM_CORE_COMMITTER + "_" + monitor.MetricTxCounter

// metric block height
const dbKeyBlockHeightPrefix = monitor.SUBSYSTEM_CORE_COMMITTER + "_" + monitor.MetricBlockCounter

func (chain *BlockCommitterImpl) initMetrics() {
	// new metrics
	chain.metricBlockSize = monitor.NewHistogramVec(
		monitor.SUBSYSTEM_CORE_COMMITTER,
		monitor.MetricBlockSize,
		monitor.HelpCurrentBlockSizeMetric,
		prometheus.ExponentialBuckets(1024, 2, 16),
		monitor.ChainId,
	)
	chain.metricBlockHeight = monitor.NewGaugeVec(
		monitor.SUBSYSTEM_CORE_COMMITTER,
		monitor.MetricBlockCounter,
		monitor.HelpBlockCountsMetric,
		monitor.ChainId,
	)
	chain.metricTxCounter = monitor.NewCounterVec(
		monitor.SUBSYSTEM_CORE_COMMITTER,
		monitor.MetricTxCounter,
		monitor.HelpTxCountsMetric,
		monitor.ChainId,
	)
	chain.metricBlockCommitTime = monitor.NewHistogramVec(
		monitor.SUBSYSTEM_CORE_COMMITTER,
		monitor.MetricBlockCommitTime,
		monitor.HelpBlockCommitTimeMetric,
		[]float64{0.005, 0.01, 0.015, 0.05, 0.1, 1, 2, 5, 10},
		monitor.ChainId,
	)
	chain.metricBlockIntervalTime = monitor.NewHistogramVec(
		monitor.SUBSYSTEM_CORE_COMMITTER,
		monitor.MetricBlockIntervalTime,
		monitor.HelpBlockIntervalTimeMetric,
		[]float64{0.2, 0.5, 1, 2, 5, 10, 20},
		monitor.ChainId,
	)
	chain.metricTpsGauge = monitor.NewGaugeVec(
		monitor.SUBSYSTEM_CORE_COMMITTER,
		monitor.MetricTpsGauge,
		monitor.HelpTpsGaugeMetric,
		monitor.ChainId,
	)

	localDb := chain.blockchainStore.GetDBHandle("")
	// init tx counter metric
	txCountBz, err := localDb.Get([]byte(dbKeyTxCounterPrefix + chain.chainId))
	if err == nil {
		// ignore brand new node
		if txCountBz != nil {
			chain.mTxCount, err = bytehelper.BytesToUint64(txCountBz)
			if err == nil {
				chain.metricTxCounter.WithLabelValues(chain.chainId).Add(float64(chain.mTxCount))
			} else {
				chain.log.Errorw("bytehelper.BytesToUint64 failed", "err", err)
			}
		}
	} else {
		chain.log.Errorw("localDb.Get metric failed", "err", err)
	}

	// init block height metric
	blockHeightBz, err := localDb.Get([]byte(dbKeyBlockHeightPrefix + chain.chainId))
	if err == nil {
		// ignore brand new node
		if blockHeightBz != nil {
			chain.mBlockHeight, err = bytehelper.BytesToUint64(blockHeightBz)
			if err == nil {
				chain.metricBlockHeight.WithLabelValues(chain.chainId).Set(float64(chain.mBlockHeight))
			} else {
				chain.log.Errorw("bytehelper.BytesToUint64 failed", "err", err)
			}
		}
	} else {
		chain.log.Errorw("localDb.Get metric failed", "err", err)
	}
}

func (chain *BlockCommitterImpl) updateMetrics(bi *commonPb.BlockInfo, elapsed, interval int64) {
	defer func() {
		if panicError := recover(); panicError != nil {
			chain.log.Error("updateMetrics failed ", panicError)
		}
	}()
	chain.metricBlockSize.WithLabelValues(bi.Block.Header.ChainId).Observe(float64(bi.Size()))
	chain.metricBlockHeight.WithLabelValues(bi.Block.Header.ChainId).Set(float64(bi.Block.Header.BlockHeight))
	atomic.StoreUint64(&chain.mBlockHeight, bi.Block.Header.BlockHeight)
	chain.metricTxCounter.WithLabelValues(bi.Block.Header.ChainId).Add(float64(bi.Block.Header.TxCount))
	atomic.AddUint64(&chain.mTxCount, uint64(bi.Block.Header.TxCount))
	chain.metricBlockCommitTime.WithLabelValues(chain.chainId).Observe(float64(elapsed) / 1000)
	chain.metricBlockIntervalTime.WithLabelValues(chain.chainId).Observe(float64(interval) / 1000)
	chain.metricTpsGauge.WithLabelValues(chain.chainId).
		Set(float64(bi.Block.Header.TxCount) / (float64(interval) / 1000))

	// persist metrics to local db
	localDb := chain.blockchainStore.GetDBHandle("")

	txCountBz, err := bytehelper.Uint64ToBytes(atomic.LoadUint64(&chain.mTxCount))
	if err == nil {
		err = localDb.Put([]byte(dbKeyTxCounterPrefix+chain.chainId), txCountBz)
		if err != nil {
			chain.log.Errorw("persist metric failed", "err", err)
		}
	} else {
		chain.log.Errorw("bytehelper.Uint64ToBytes failed", "err", err)
	}

	blockHeightBz, err := bytehelper.Uint64ToBytes(atomic.LoadUint64(&chain.mBlockHeight))
	if err == nil {
		err = localDb.Put([]byte(dbKeyBlockHeightPrefix+chain.chainId), blockHeightBz)
		if err != nil {
			chain.log.Errorw("persist metric failed", "err", err)
		}
	} else {
		chain.log.Errorw("bytehelper.Uint64ToBytes failed", "err", err)
	}
}

func ClearProposeRepeatTimerMap() {
	ProposeRepeatTimerMap.Range(func(key, value interface{}) bool {
		ProposeRepeatTimerMap.Delete(key)
		return true
	})
}

// CopyBlock generates a new block with a old block, internally using the same pointer
func CopyBlock(block *commonPb.Block) *commonPb.Block {
	return &commonPb.Block{
		Header:         block.Header,
		Dag:            block.Dag,
		Txs:            block.Txs,
		AdditionalData: block.AdditionalData,
	}
}
