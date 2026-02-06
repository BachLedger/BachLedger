/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package raft

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"reflect"
	"testing"
	"time"

	"chainmaker.org/chainmaker/common/v2/msgbus"
	utils "chainmaker.org/chainmaker/consensus-utils/v2"
	"chainmaker.org/chainmaker/localconf/v2"
	"chainmaker.org/chainmaker/pb-go/v2/common"
	configpb "chainmaker.org/chainmaker/pb-go/v2/config"
	consensuspb "chainmaker.org/chainmaker/pb-go/v2/consensus"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/protocol/v2/mock"
	blockUtils "chainmaker.org/chainmaker/utils/v2"

	"github.com/golang/mock/gomock"
	"go.etcd.io/etcd/client/pkg/v3/fileutil"
)

const (
	id         = "QmQZn3pZCcuEf34FSvucqkvVJEvfzpNjQTk17HS6CYMR35"
	org1Id     = "yz-org1"
	blockCount = 100
)

type TestBlockchain struct {
	chainId       string
	store         protocol.BlockchainStore
	coreEngine    protocol.CoreEngine
	identity      protocol.SigningMember
	ac            protocol.AccessControlProvider
	ledgerCache   protocol.LedgerCache
	proposalCache protocol.ProposalCache
	chainConf     protocol.ChainConf
	logger        protocol.Logger
}

func (bc *TestBlockchain) MockInit(ctrl *gomock.Controller, consensusType consensuspb.ConsensusType, blocks []*common.Block) {
	bc.logger = newMockLogger()
	identity := mock.NewMockSigningMember(ctrl)
	identity.EXPECT().GetMember().AnyTimes().Return(nil, nil)
	ledgerCache := mock.NewMockLedgerCache(ctrl)
	bc.ledgerCache = ledgerCache
	ledgerCache.EXPECT().CurrentHeight().AnyTimes().Return(uint64(0), nil)
	chainConf := mock.NewMockChainConf(ctrl)
	chainConf.EXPECT().ChainConfig().AnyTimes().Return(&configpb.ChainConfig{
		ChainId: id,
		Consensus: &configpb.ConsensusConfig{
			Type: consensusType,
			Nodes: []*configpb.OrgConfig{
				{
					OrgId:  org1Id,
					NodeId: []string{id},
				},
			},
		},
		Block: &configpb.BlockConfig{
			BlockSize: 10,
		},
		Crypto: &configpb.CryptoConfig{
			Hash: "SHA256",
		},
	})
	bc.chainConf = chainConf
	coreEngine := mock.NewMockCoreEngine(ctrl)
	blockVerifier := mock.NewMockBlockVerifier(ctrl)
	blockCommitter := mock.NewMockBlockCommitter(ctrl)

	for i := 0; i < blockCount; i++ {
		block := blocks[i]
		blockHash, err := blockUtils.CalcBlockHash("SHA256", block)
		if err != nil {
			ctrl.T.Errorf("CalcBlockHash() error = %v", err)
		}
		identity.EXPECT().Sign("SHA256", blockHash).AnyTimes().Return(blockHash, nil)
		block.Header.BlockHash = blockHash
		block.Header.Signature = blockHash
		if block.AdditionalData == nil {
			block.AdditionalData = &common.AdditionalData{
				ExtraData: make(map[string][]byte),
			}
		}

		serializeMember, err := identity.GetMember()
		if err != nil {
			ctrl.T.Errorf("GetMember() error = %v", err)
			return
		}

		signature := &common.EndorsementEntry{
			Signer:    serializeMember,
			Signature: blockHash,
		}

		additionalData := AdditionalData{
			Signature: mustMarshal(signature),
		}

		data, _ := json.Marshal(additionalData)
		block.AdditionalData.ExtraData[RAFTAddtionalDataKey] = data
		blockVerifier.EXPECT().VerifyBlock(block, protocol.CONSENSUS_VERIFY).AnyTimes().Return(nil)
		blockCommitter.EXPECT().AddBlock(block).AnyTimes().Return(nil)
	}
	bc.identity = identity
	coreEngine.EXPECT().GetBlockVerifier().AnyTimes().Return(blockVerifier)
	coreEngine.EXPECT().GetBlockCommitter().AnyTimes().Return(blockCommitter)
	bc.coreEngine = coreEngine
}

func TestNewConsensusEngine(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	prePath := localconf.ChainMakerConfig.GetStorePath()
	preAsync := localconf.ChainMakerConfig.ConsensusConfig.RaftConfig.AsyncWalSave
	defer func() {
		localconf.ChainMakerConfig.StorageConfig["store_path"] = prePath
		localconf.ChainMakerConfig.ConsensusConfig.RaftConfig.AsyncWalSave = preAsync
	}()
	localconf.ChainMakerConfig.StorageConfig["store_path"] = ""
	localconf.ChainMakerConfig.ConsensusConfig.RaftConfig.AsyncWalSave = true
	t.Run("new raft consensus engine", func(t *testing.T) {
		var block *common.Block
		blocks := make([]*common.Block, blockCount)
		for i := 0; i < blockCount; i++ {
			block = &common.Block{
				Header: &common.BlockHeader{
					ChainId:        id,
					BlockHeight:    uint64(i + 1),
					PreBlockHash:   []byte("77150e"),
					BlockHash:      nil,
					PreConfHeight:  0,
					BlockVersion:   protocol.DefaultBlockVersion,
					DagHash:        nil,
					RwSetRoot:      nil,
					TxRoot:         nil,
					BlockTimestamp: time.Now().Unix(),
					ConsensusArgs:  nil,
					TxCount:        0,
					Signature:      nil,
				},
				Dag:            &common.DAG{},
				Txs:            nil,
				AdditionalData: nil,
			}
			blocks[i] = block
		}
		bc := &TestBlockchain{}
		bc.MockInit(ctrl, consensuspb.ConsensusType_RAFT, blocks)
		config := &utils.ConsensusImplConfig{
			ChainId:       bc.chainId,
			NodeId:        id,
			Ac:            bc.ac,
			Core:          bc.coreEngine,
			ChainConf:     bc.chainConf,
			Signer:        bc.identity,
			Store:         bc.store,
			LedgerCache:   bc.ledgerCache,
			ProposalCache: bc.proposalCache,
			Logger:        bc.logger,
			MsgBus:        msgbus.NewMessageBus(),
		}
		consensus, err := New(config)
		if err != nil {
			t.Errorf("NewConsensusEngine() error = %v", err)
			return
		}
		consensus.electionTick = 2
		if reflect.TypeOf(consensus) != reflect.TypeOf(&ConsensusRaftImpl{}) {
			t.Errorf("NewConsensusEngine() = %v", consensus)
		}
		defer func() {
			os.RemoveAll(consensus.waldir)
			os.RemoveAll(consensus.snapdir)
		}()
		// If the last test had exited with panic, the wal and snapshot files would not have been removed
		// and they would have interfered with the results this time
		os.RemoveAll(consensus.waldir)
		os.RemoveAll(consensus.snapdir)
		err = consensus.Start()
		if err != nil {
			t.Errorf("Start error:%+v", err)
		}
		// waiting the election of raft
		time.Sleep(5 * time.Second)
		validators, err := consensus.GetValidators()
		if err != nil {
			t.Errorf("GetValidators() error = %v", err)
			return
		}
		if len(validators) != 1 {
			t.Errorf("validators count error:%d", len(validators))
		}
		last := consensus.GetLastHeight()
		if last != 0 {
			t.Errorf("consensus.GetLastHeight() error:%d", last)
		}
		status, err := consensus.GetConsensusStateJSON()
		if err != nil {
			t.Errorf("GetConsensusStateJSON() error = %v", err)
			return
		}
		statusResult := make(map[string]interface{})
		if err = json.Unmarshal(status, &statusResult); err != nil {
			t.Errorf("GetConsensusStateJSON() result unmarshal error = %v", err)
		}
		for i := 0; i < blockCount/2; i++ {
			blk := &consensuspb.ProposalBlock{
				Block:    blocks[i],
				TxsRwSet: make(map[string]*common.TxRWSet),
			}
			consensus.msgbus.PublishSafe(msgbus.ProposedBlock, blk)
		}
		err = consensus.Stop()
		if err != nil {
			t.Errorf("Stop error:%+v", err)
		}
		err = consensus.Start()
		if err != nil {
			t.Errorf("Start error:%+v", err)
		}
		// to simulate asynchronous process delays
		consensus.appliedIndex++
		time.Sleep(5 * time.Second)
		for i := blockCount / 2; i < blockCount; i++ {
			blk := &consensuspb.ProposalBlock{
				Block:    blocks[i],
				TxsRwSet: make(map[string]*common.TxRWSet),
			}
			consensus.msgbus.PublishSafe(msgbus.ProposedBlock, blk)
		}
		time.Sleep(3 * time.Second)
		if consensus.appliedIndex != blockCount+2 {
			t.Errorf("appliedIndex wrong: %d", consensus.appliedIndex)
		}
		//t.Errorf("")
	})
}

// NewMockCoreEngine
func NewMockCoreEngine(ctrl *gomock.Controller) *mock.MockCoreEngine {
	coreEngine := mock.NewMockCoreEngine(ctrl)
	coreEngine.EXPECT().GetBlockVerifier().Return(nil).AnyTimes()
	coreEngine.EXPECT().GetBlockCommitter().Return(nil).AnyTimes()
	return coreEngine
}

// /**
// * 测试Purge File前，需要在PurgeFile方法return前添加代码      time.Sleep(time.Millisecond * 200)
// * 否则，测试完成后，协程未来得及删除测试文件。
// */
func TestPurgeFile(t *testing.T) {
	storagePath := t.TempDir()
	t.Logf("File Dir : %s", storagePath)
	//创建5个WAl文件
	for i := 10; i < 15; i++ {
		index := i*2 + 5
		p := filepath.Join(storagePath, fmt.Sprintf("%016d-%016d.wal", i, index))
		f, err := fileutil.LockFile(p, os.O_WRONLY|os.O_CREATE, fileutil.PrivateFileMode)
		if err != nil {
			t.Errorf("Lock File Failed : %s", err)
		}

		if err = fileutil.Preallocate(f.File, 64*1000, true); err != nil {
			t.Errorf("Preallocate an initial WAL file Failed : %xs", err)
		}
		_ = f.Close()
	}

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	//config := ConsensusRaftImplConfig{
	config := &utils.ConsensusImplConfig{
		ChainId: "chain1",
		NodeId:  "QmQZn3pZCcuEf34FSvucqkvVJEvfzpNjQTk17HS6CYMR35",
		Core:    NewMockCoreEngine(ctrl),
		Logger:  newMockLogger(),
	}
	consensus, err := New(config)
	if err != nil {
		t.Errorf("New Raft Failed : %s", err)
	}

	consensus.PurgeFile(storagePath)

	fileNames, err := fileutil.ReadDir(storagePath)
	if err != nil {
		t.Errorf("Read Path Failed : %s", err)
	}
	want := 1
	fileCount := len(fileNames)
	if !reflect.DeepEqual(fileCount, want) {
		t.Errorf("WAL File Purged, Path File Count = %v, want %v", fileCount, want)
	}
}

func TestVerifyBlockSignatures(t *testing.T) {
	sig := []byte("test-signature")
	blk := &common.Block{
		Header: &common.BlockHeader{
			Signature: sig,
		},
		AdditionalData: &common.AdditionalData{
			ExtraData: make(map[string][]byte),
		},
	}
	endorsement := &common.EndorsementEntry{
		Signature: sig,
	}
	addData := &AdditionalData{
		Signature: mustMarshal(endorsement),
	}
	addDataByt, _ := json.Marshal(addData)
	blk.AdditionalData.ExtraData[RAFTAddtionalDataKey] = addDataByt
	if VerifyBlockSignatures(blk) != nil {
		t.Errorf("VerifyBlockSignatures error:%+v", blk)
	}
}
