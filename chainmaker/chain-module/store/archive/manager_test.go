/*
*  Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
*
*  SPDX-License-Identifier: Apache-2.0
 */

package archive

import (
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"chainmaker.org/chainmaker/common/v2/crypto/hash"
	acPb "chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	configPb "chainmaker.org/chainmaker/pb-go/v2/config"
	storePb "chainmaker.org/chainmaker/pb-go/v2/store"
	"chainmaker.org/chainmaker/pb-go/v2/syscontract"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/protocol/v2/test"
	leveldbprovider "chainmaker.org/chainmaker/store-leveldb/v2"
	"chainmaker.org/chainmaker/store/v2/blockdb"
	"chainmaker.org/chainmaker/store/v2/blockdb/blockfiledb"
	"chainmaker.org/chainmaker/store/v2/blockdb/blockkvdb"
	"chainmaker.org/chainmaker/store/v2/conf"
	"chainmaker.org/chainmaker/store/v2/resultdb"
	"chainmaker.org/chainmaker/store/v2/resultdb/resultfiledb"
	"chainmaker.org/chainmaker/store/v2/resultdb/resultkvdb"
	"chainmaker.org/chainmaker/store/v2/serialization"
	tbf "chainmaker.org/chainmaker/store/v2/types/blockfile"
	"chainmaker.org/chainmaker/utils/v2"
	"github.com/gogo/protobuf/proto"
	"github.com/mitchellh/mapstructure"
	"github.com/stretchr/testify/assert"
)

var chainConf = configPb.ChainConfig{
	Crypto: &configPb.CryptoConfig{
		Hash: "SHA256",
	},
	Version: "v2.3.0",
}
var (
	blockFilePath          = "bfdb"
	defaultSysContractName = syscontract.SystemContract_CHAIN_CONFIG.String()
)

func TestNewArchiveMgr_RawDB_Open(t *testing.T) {
	tLogger := &test.GoLogger{}
	//pathD := "/Users/zhouhoufa/Code/src/chainmaker.org/chainmaker/store/data"
	pathD := ""
	dbConf := archiveConf(pathD)
	dbConf.DisableBlockFileDb = true
	mgr, err := buildManager(dbConf, tLogger)
	assert.Nil(t, err)
	assert.NotNil(t, mgr)

	assert.Nil(t, mgr.Close())
	mgr.blockDB.Close()
	mgr.resultDB.Close()
}

func TestNewArchiveMgr_RawDB(t *testing.T) {
	tLogger := &test.GoLogger{}
	//pathD := "/Users/zhouhoufa/Code/src/chainmaker.org/chainmaker/store/data"
	pathD := ""
	dbConf := archiveConf(pathD)
	dbConf.DisableBlockFileDb = true
	mgr, err := buildManager(dbConf, tLogger)
	assert.Nil(t, err)
	assert.NotNil(t, mgr)

	archiveHeight1 := 27
	archiveHeight2 := 30
	archiveHeight3 := 43

	//Prepare block data
	confBlocks := []uint64{0, 10, 21, 30, 35, 40}
	blocks, txRWSetMp, errb := buildArchiveDB(60, 10, confBlocks, mgr)
	assert.Nil(t, errb)

	verifyArchive(t, 0, true, blocks, mgr.blockDB)

	//archive block height1
	err = mgr.ArchiveBlock(uint64(archiveHeight1))
	assert.Nil(t, err)
	_, err = mgr.GetArchivedPivot()
	assert.Nil(t, err)
	assert.Equal(t, uint64(archiveHeight1), mgr.archivedPivot)
	as, erra := mgr.GetArchiveStatus()
	assert.Nil(t, erra)
	assert.Equal(t, uint64(archiveHeight1), as.ArchivePivot)

	verifyArchive(t, 10, true, blocks, mgr.blockDB)

	//archive block height2 which is a config block
	err1 := mgr.ArchiveBlock(uint64(archiveHeight2))
	assert.True(t, err1 == tbf.ErrConfigBlockArchive)
	_, err = mgr.GetArchivedPivot()
	assert.Nil(t, err)
	assert.Equal(t, uint64(archiveHeight1), mgr.archivedPivot)

	verifyArchive(t, 21, true, blocks, mgr.blockDB)

	//archive block height3
	err = mgr.ArchiveBlock(uint64(archiveHeight3))
	assert.Nil(t, err)
	_, err = mgr.GetArchivedPivot()
	assert.Nil(t, err)
	assert.Equal(t, uint64(archiveHeight3), mgr.archivedPivot)

	verifyArchive(t, 30, true, blocks, mgr.blockDB)

	//Prepare restore data
	restoreHeight := 0
	for i := archiveHeight3; i >= restoreHeight; i-- {
		blockBytes, _, err5 := serialization.SerializeBlock(&storePb.BlockWithRWSet{
			Block:          blocks[i],
			TxRWSets:       txRWSetMp[blocks[i].Header.BlockHeight],
			ContractEvents: nil,
		})

		assert.Nil(t, err5)
		//restore block
		assert.Nil(t, mgr.RestoreBlocks([][]byte{blockBytes}))
	}

	_, err = mgr.GetArchivedPivot()
	assert.Nil(t, err)
	assert.Equal(t, uint64(restoreHeight), mgr.archivedPivot)
	verifyArchive(t, 0, true, blocks, mgr.blockDB)

	//wait kvdb compact range
	//time.Sleep(5 * time.Second)
	assert.Nil(t, mgr.Close())
	mgr.blockDB.Close()
	mgr.resultDB.Close()
}

func TestNewArchiveMgr_BFDB_Open(t *testing.T) {
	tLogger := &test.GoLogger{}
	//pathD := "/Users/zhouhoufa/Code/src/chainmaker.org/chainmaker/store/data"
	pathD := ""
	mgr, err := buildManager(archiveConf(pathD), tLogger)
	assert.Nil(t, err)
	assert.NotNil(t, mgr)

	//close(mgr.stopCh)
	//time.Sleep(time.Duration(mgr.storeConfig.RestoreInterval) * time.Second)
	//mgr.stopCh = make(chan struct{})

	assert.Nil(t, mgr.Close())
	mgr.blockDB.Close()
	mgr.resultDB.Close()
}

func TestNewArchiveMgr_BFDB(t *testing.T) {
	tLogger := &test.GoLogger{}
	//pathD := "/Users/zhouhoufa/Code/src/chainmaker.org/chainmaker/store/data"
	pathD := ""
	mgr, err := buildManager(archiveConf(pathD), tLogger)
	assert.Nil(t, err)
	assert.NotNil(t, mgr)

	// wait mgr.inProcessStatus Normal
	close(mgr.stopCh)
	time.Sleep(time.Duration(mgr.storeConfig.RestoreInterval) * time.Second)
	mgr.stopCh = make(chan struct{})

	confBlocks := []uint64{0, 10, 20, 30}
	// 1 6 10 15 19 ||| 24 28 33
	blocks, txRWSetMp, errb := buildArchiveDB(23, 10000, confBlocks, mgr)
	assert.Nil(t, errb)
	verifyArchive(t, confBlocks[0], false, blocks, mgr.blockDB)

	// archive block height
	archiveHeight1 := []int{11, 8}
	assert.Nil(t, mgr.ArchiveBlock(uint64(archiveHeight1[0])))
	assert.Nil(t, mgr.doArchive())

	_, err = mgr.GetArchivedPivot()
	assert.Nil(t, err)
	assert.Equal(t, uint64(archiveHeight1[1]), mgr.archivedPivot)

	verifyArchive(t, confBlocks[1], false, blocks, mgr.blockDB)

	// restore block height
	restoreHeight := uint64(0)
	for i := int(mgr.archivedPivot); i >= int(restoreHeight); i-- {
		blockBytes, _, err5 := serialization.SerializeBlock(&storePb.BlockWithRWSet{
			Block:          blocks[i],
			TxRWSets:       txRWSetMp[blocks[i].Header.BlockHeight],
			ContractEvents: nil,
		})

		assert.Nil(t, err5)
		assert.Nil(t, mgr.RestoreBlocks([][]byte{blockBytes}))
	}

	as, err := mgr.GetArchiveStatus()
	assert.Nil(t, err)
	assert.Equal(t, int(mgr.archivedPivot-restoreHeight+1), len(as.FileRanges))

	// megre and then restore block
	assert.Nil(t, mgr.doRestore())
	as, err = mgr.GetArchiveStatus()
	assert.Nil(t, err)
	assert.True(t, int(mgr.archivedPivot-restoreHeight+1) >= len(as.FileRanges))
	//assert.Equal(t, mgr.archivedPivot, as.FileRanges[len(as.FileRanges)-1].End)

	for len(as.FileRanges) > 0 {
		err = mgr.doRestore()
		assert.Nil(t, err)
		as, err = mgr.GetArchiveStatus()
		assert.Nil(t, err)
	}

	assert.Equal(t, mgr.archivedPivot, uint64(0))
}

func TestNewArchiveMgr_BFDB_Resume(t *testing.T) {
	tLogger := &test.GoLogger{}
	//pathD := "/Users/zhouhoufa/Code/src/chainmaker.org/chainmaker/store/data"
	pathD := filepath.Join(os.TempDir(), fmt.Sprintf("%d", time.Now().Nanosecond()))
	mgr, err := buildManager(archiveConf(pathD), tLogger)
	assert.Nil(t, err)
	assert.NotNil(t, mgr)

	// wait mgr.inProcessStatus Normal
	close(mgr.stopCh)
	time.Sleep(time.Duration(mgr.storeConfig.RestoreInterval) * time.Second)
	mgr.stopCh = make(chan struct{})

	confBlocks := []uint64{0, 10, 20, 30}
	// 1 6 10 15 19 ||| 24 28 33
	blocks, txRWSetMp, errb := buildArchiveDB(23, 10000, confBlocks, mgr)
	assert.Nil(t, errb)
	verifyArchive(t, confBlocks[0], false, blocks, mgr.blockDB)

	// archive block height
	archiveHeight1 := []int{11, 8}
	assert.Nil(t, mgr.ArchiveBlock(uint64(archiveHeight1[0])))
	assert.Nil(t, mgr.doArchive())

	_, err = mgr.GetArchivedPivot()
	assert.Nil(t, err)
	assert.Equal(t, uint64(archiveHeight1[1]), mgr.archivedPivot)

	verifyArchive(t, confBlocks[1], false, blocks, mgr.blockDB)

	// restore block height
	restoreHeight := uint64(0)
	for i := int(mgr.archivedPivot); i >= int(restoreHeight); i-- {
		blockBytes, _, err5 := serialization.SerializeBlock(&storePb.BlockWithRWSet{
			Block:          blocks[i],
			TxRWSets:       txRWSetMp[blocks[i].Header.BlockHeight],
			ContractEvents: nil,
		})

		assert.Nil(t, err5)
		assert.Nil(t, mgr.RestoreBlocks([][]byte{blockBytes}))
	}

	as, err := mgr.GetArchiveStatus()
	assert.Nil(t, err)
	assert.Equal(t, int(mgr.archivedPivot-restoreHeight+1), len(as.FileRanges))

	// stop manager instance
	_ = mgr.Close()
	mgr.blockDB.Close()
	mgr.resultDB.Close()

	damagePoint := []uint64{5, 6, 7}
	bfdbPath := mgr.bfdbPath
	restoringPath := filepath.Join(bfdbPath, tbf.RestoringPath)

	// damage1 file by move xxx.fdb to bfdb before doRestore
	dSegName := tbf.Uint64ToSegmentName(damagePoint[0])
	assert.Nil(t, os.Rename(tbf.SegmentPath(dSegName, restoringPath), tbf.SegmentPath(dSegName, bfdbPath)))

	// damage2 file by delete xxx.fdb to bfdb after doRestore, then doRestore this point again
	dSegName = tbf.Uint64ToSegmentName(damagePoint[1])
	assert.Nil(t, os.RemoveAll(tbf.SegmentPath(dSegName, restoringPath)))

	// damage3 file by delete xxx.restoring.json after doRestore, then doRestore this point again
	dSegName = tbf.Uint64ToSegmentName(damagePoint[2])
	assert.Nil(t, os.RemoveAll(tbf.SegmentIndexPath(dSegName, restoringPath)))

	// restart store instance with previous data
	mgr, err = buildManager(archiveConf(pathD), tLogger)
	assert.Nil(t, err)

	// wait mgr.inProcessStatus Normal
	close(mgr.stopCh)
	time.Sleep(time.Duration(mgr.storeConfig.RestoreInterval) * time.Second)
	mgr.stopCh = make(chan struct{})

	as, err = mgr.GetArchiveStatus()
	assert.Nil(t, err)
	// verify restore file ranges after fix damage
	assert.Equal(t, int(mgr.archivedPivot-restoreHeight-1), len(as.FileRanges))
	time.Sleep(time.Duration(mgr.storeConfig.RestoreInterval) * time.Second)

	// restore damage point block
	rblk, _, errr := serialization.SerializeBlock(&storePb.BlockWithRWSet{
		Block:          blocks[damagePoint[1]-1],
		TxRWSets:       txRWSetMp[blocks[damagePoint[1]-1].Header.BlockHeight],
		ContractEvents: nil,
	})
	assert.Nil(t, errr)
	assert.Nil(t, mgr.RestoreBlocks([][]byte{rblk}))

	rblk, _, errr = serialization.SerializeBlock(&storePb.BlockWithRWSet{
		Block:          blocks[damagePoint[2]-1],
		TxRWSets:       txRWSetMp[blocks[damagePoint[2]-1].Header.BlockHeight],
		ContractEvents: nil,
	})
	assert.Nil(t, errr)
	assert.Nil(t, mgr.RestoreBlocks([][]byte{rblk}))

	for len(as.FileRanges) > 0 {
		err = mgr.doRestore()
		assert.Nil(t, err)
		as, err = mgr.GetArchiveStatus()
		assert.Nil(t, err)
	}
	assert.Equal(t, mgr.archivedPivot, uint64(0))

	verifyArchive(t, confBlocks[0], false, blocks, mgr.blockDB)

	_ = mgr.Close()
	mgr.blockDB.Close()
	mgr.resultDB.Close()
}

func TestNewArchiveMgr_BFDB_Archive_Height(t *testing.T) {
	tLogger := &test.GoLogger{}
	//pathD := "/Users/zhouhoufa/Code/src/chainmaker.org/chainmaker/store/data"
	pathD := ""
	mgr, err := buildManager(archiveConf(pathD), tLogger)
	assert.Nil(t, err)
	assert.NotNil(t, mgr)

	// wait mgr.inProcessStatus Normal
	close(mgr.stopCh)
	time.Sleep(time.Duration(mgr.storeConfig.RestoreInterval) * time.Second)
	mgr.stopCh = make(chan struct{})

	archiveHeight1 := []uint64{2, 0}
	archiveHeight2 := []uint64{4, 4}
	archiveHeight3 := []uint64{5, 4}
	archiveHeight4 := []uint64{6, 4}
	archiveHeight5 := []uint64{16, 13}
	//archiveHeight6 := []uint64{23, 23}
	//archiveHeight7 := []uint64{24, 23}
	//archiveHeight8 := []uint64{43, 39}

	confBlocks := []uint64{0, 10, 20, 30}
	// 1 6 10 15 19 ||| 24 28 33
	blocks, _, errb := buildArchiveDB(35, 10000, confBlocks, mgr)
	assert.Nil(t, errb)
	verifyArchive(t, confBlocks[0], false, blocks, mgr.blockDB)

	// archive block height not change
	err = mgr.ArchiveBlock(archiveHeight1[0])
	assert.True(t, strings.Contains(err.Error(), "archive range less than"))
	_, err = mgr.GetArchivedPivot()
	assert.Nil(t, err)
	assert.Equal(t, archiveHeight1[1], mgr.archivedPivot)

	// archive block height move to archiveHeight2[1]
	assert.Nil(t, mgr.ArchiveBlock(archiveHeight2[0]))
	assert.Nil(t, mgr.doArchive())
	_, err = mgr.GetArchivedPivot()
	assert.Nil(t, err)
	assert.Equal(t, archiveHeight2[1], mgr.archivedPivot)

	// archive block height
	err = mgr.ArchiveBlock(archiveHeight3[0])
	assert.True(t, strings.Contains(err.Error(), "archive range less than"))
	_, err = mgr.GetArchivedPivot()
	assert.Nil(t, err)
	assert.Equal(t, archiveHeight3[1], mgr.archivedPivot)

	// archive block height not change
	err = mgr.ArchiveBlock(archiveHeight4[0])
	assert.True(t, strings.Contains(err.Error(), "archive range less than"))
	_, err = mgr.GetArchivedPivot()
	assert.Nil(t, err)
	assert.Equal(t, archiveHeight4[1], mgr.archivedPivot)

	// archive block height
	assert.Nil(t, mgr.ArchiveBlock(archiveHeight5[0]))
	assert.Nil(t, mgr.doArchive())
	_, err = mgr.GetArchivedPivot()
	assert.Nil(t, err)
	assert.Equal(t, archiveHeight5[1], mgr.archivedPivot)
	//
	//// archive block height move to archiveHeight6[1]
	//err = s.ArchiveBlock(archiveHeight6[0])
	//time.Sleep(archiveWait)
	//assert.Nil(t, err)
	//assert.Nil(t, waitArchiveNormal(s))
	//assert.Equal(t, archiveHeight6[1], s.GetArchivedPivot())
	//
	//// archive block height not change
	//err = s.ArchiveBlock(archiveHeight7[0])
	//assert.True(t, strings.Contains(err.Error(), "archive range less than"))
	//assert.Equal(t, archiveHeight7[1], s.GetArchivedPivot())
	//
	//// archive block height move to archiveHeight8[1]
	//err = s.ArchiveBlock(archiveHeight8[0])
	//assert.Nil(t, err)
	//time.Sleep(archiveWait)
	//assert.Nil(t, waitArchiveNormal(s))
	//assert.Equal(t, archiveHeight8[1], s.GetArchivedPivot())

	_ = mgr.Close()
	mgr.blockDB.Close()
	mgr.resultDB.Close()
}

func calcTxRoot(blk *commonPb.Block) ([]byte, error) {
	txHashes := make([][]byte, 0, len(blk.Txs))
	bv := int(blk.Header.BlockVersion)
	for _, tx := range blk.Txs {
		txHash, err := utils.CalcTxHashWithVersion(cryptoType, tx, bv)
		if err != nil {
			return nil, err
		}
		txHashes = append(txHashes, txHash)
	}

	root, err := hash.GetMerkleRoot(cryptoType, txHashes)
	if err != nil {
		return nil, err
	}

	return root, nil
}

func calcRwSetsRoot(rwSets []*commonPb.TxRWSet) ([]byte, error) {
	hashes := make([][]byte, 0, len(rwSets))
	for _, rw := range rwSets {
		rwSetH, err := utils.CalcRWSetHash(cryptoType, rw)
		if err != nil {
			return nil, err
		}
		hashes = append(hashes, rwSetH)
	}

	// verify rwSet merkle root
	root, err := hash.GetMerkleRoot(cryptoType, hashes)
	if err != nil {
		return nil, err
	}
	return root, nil
}

func calcDagHash(blk *commonPb.Block) ([]byte, error) {
	// verify dag hash
	return utils.CalcDagHash(cryptoType, blk.Dag)
}

func getlvldbConfig(path string) *conf.StorageConfig {
	conf1, _ := conf.NewStorageConfig(nil)
	if path == "" {
		path = filepath.Join(os.TempDir(), fmt.Sprintf("%d", time.Now().Nanosecond()))
	}

	conf1.StorePath = path
	conf1.WriteBlockType = 0

	lvlConfig := make(map[string]interface{})
	lvlConfig["store_path"] = path

	//path2 := filepath.Join(os.TempDir(), fmt.Sprintf("%d", time.Now().Nanosecond()))

	lvlConfig2 := make(map[string]interface{})
	//lvlConfig2["store_path"] = path2
	lvlConfig2["store_path"] = path

	dbConfig := &conf.DbConfig{
		Provider:      "leveldb",
		LevelDbConfig: lvlConfig,
	}
	dbConfig2 := &conf.DbConfig{
		Provider:      "leveldb",
		LevelDbConfig: lvlConfig2,
	}
	conf1.BlockDbConfig = dbConfig
	conf1.StateDbConfig = dbConfig
	conf1.HistoryDbConfig = conf.NewHistoryDbConfig(dbConfig)
	conf1.ResultDbConfig = dbConfig
	conf1.DisableContractEventDB = true
	conf1.TxExistDbConfig = dbConfig2
	conf1.DisableBlockFileDb = false
	//conf1.DisableBigFilter = true
	//conf1.StorePath = filepath.Join(os.TempDir(), fmt.Sprintf("%d_wal", time.Now().Nanosecond()))
	return conf1
}

func buildDB(chainId, dbFolder string, dbConfig map[string]interface{},
	logger protocol.Logger) (*leveldbprovider.LevelDBHandle, error) {
	bConfig := &leveldbprovider.LevelDbConfig{}
	if err := mapstructure.Decode(dbConfig, bConfig); err != nil {
		return nil, err
	}
	return leveldbprovider.NewLevelDBHandle(
		&leveldbprovider.NewLevelDBOptions{
			Config:    bConfig,
			Logger:    logger,
			Encryptor: nil,
			ChainId:   chainId,
			DbFolder:  dbFolder,
		}), nil
}

func buildManager(dbConf *conf.StorageConfig, logger protocol.Logger) (*ArchiveMgr, error) {
	dbConf.ArchiveCheckInterval = 1
	dbConf.RestoreInterval = 2

	// build block db
	blvl, err := buildDB(chainId, "blockdb", dbConf.BlockDbConfig.LevelDbConfig, logger)
	if err != nil {
		return nil, err
	}

	rlvl, err := buildDB(chainId, "resultdb", dbConf.ResultDbConfig.LevelDbConfig, logger)
	if err != nil {
		return nil, err
	}

	var (
		bfdb     *blockfiledb.BlockFile
		blockDB  blockdb.BlockDB
		resultDB resultdb.ResultDB
	)

	bfdbPath := filepath.Join(dbConf.StorePath, chainId, blockFilePath)
	if !dbConf.DisableBlockFileDb {
		opts := blockfiledb.DefaultOptions
		opts.SegmentSize = dbConf.LogDBSegmentSize * 1024 * 1024
		bfdb, err = blockfiledb.Open(bfdbPath, "", opts, logger)
		if err != nil {
			return nil, err
		}
		blockDB = blockfiledb.NewBlockFileDB(chainId, blvl, logger, dbConf, bfdb)
		resultDB = resultfiledb.NewResultFileDB(chainId, rlvl, logger, dbConf, bfdb)
	} else {
		blockDB = blockkvdb.NewBlockKvDB(chainId, blvl, logger, dbConf)
		resultDB = resultkvdb.NewResultKvDB(chainId, rlvl, logger, dbConf)
	}

	// build archive mgr
	return NewArchiveMgr(chainId, bfdbPath, blockDB, resultDB, bfdb, dbConf, logger)
}

func buildArchiveDB(totalHeight, txNum int, confBlocks []uint64,
	mgr *ArchiveMgr) ([]*commonPb.Block, map[uint64][]*commonPb.TxRWSet, error) {
	var err error
	ci := 0
	isConf := false
	preConfHeight := confBlocks[0]

	blocks := make([]*commonPb.Block, 0, totalHeight)
	txRWSetMp := make(map[uint64][]*commonPb.TxRWSet)
	for i := 0; i < totalHeight; i++ {
		var (
			block   *commonPb.Block
			txRWSet []*commonPb.TxRWSet
		)

		for j := ci; j < len(confBlocks); j++ {
			if confBlocks[j] == uint64(i) {
				isConf = true
				ci = j + 1
				break
			}
		}

		if ci > 1 && confBlocks[ci-1] < uint64(i) {
			preConfHeight = confBlocks[ci-1]
		}

		if isConf {
			block, txRWSet = createConfBlockAndRWSets(chainId, uint64(i))
			isConf = false
		} else {
			block, txRWSet = createBlockAndRWSets(chainId, uint64(i), txNum)
		}

		block.Header.PreConfHeight = preConfHeight
		block.Header.TxRoot, _ = calcTxRoot(block)
		block.Header.RwSetRoot, _ = calcRwSetsRoot(txRWSet)
		block.Header.DagHash, _ = calcDagHash(block)
		block.Header.BlockHash, _ = utils.CalcBlockHash(chainConf.Crypto.Hash, block)
		err = putBlock(&storePb.BlockWithRWSet{
			Block:    block,
			TxRWSets: txRWSet,
		}, mgr)
		if err != nil {
			return nil, nil, err
		}
		blocks = append(blocks, block)
		txRWSetMp[block.Header.BlockHeight] = txRWSet
	}

	return blocks, txRWSetMp, err
}

func archiveConf(pathD string) *conf.StorageConfig {
	dbConf := getlvldbConfig(pathD)
	dbConf.UnArchiveBlockHeight = 10
	dbConf.LogDBSegmentSize = 32
	dbConf.EnableRWC = false
	dbConf.ReadBFDBTimeOut = 1000000
	dbConf.ArchiveCheckInterval = 1
	dbConf.RestoreInterval = 5
	dbConf.DisableBlockFileDb = false
	return dbConf
}

func createConfBlockAndRWSets(chainId string, height uint64) (*commonPb.Block, []*commonPb.TxRWSet) {
	block := createConfBlock(chainId, height)
	txRWSets := []*commonPb.TxRWSet{
		{
			TxId: block.Txs[0].Payload.TxId,
			TxWrites: []*commonPb.TxWrite{
				{
					Key:          []byte(fmt.Sprintf("key_0_%d", height)),
					Value:        []byte("value_0"),
					ContractName: defaultSysContractName,
				},
			},
		},
	}

	return block, txRWSets
}

func createConfBlock(chainId string, height uint64) *commonPb.Block {
	ccBytes, _ := proto.Marshal(&chainConf)
	block := &commonPb.Block{
		Header: &commonPb.BlockHeader{
			ChainId:      chainId,
			BlockHeight:  height,
			BlockVersion: 3000000,
			Proposer: &acPb.Member{
				OrgId:      "org1",
				MemberInfo: []byte("User1"),
			},
		},
		Txs: []*commonPb.Transaction{
			{
				Payload: &commonPb.Payload{
					ChainId:      chainId,
					TxType:       commonPb.TxType_INVOKE_CONTRACT,
					TxId:         generateTxId(chainId, height, 0),
					ContractName: syscontract.SystemContract_CHAIN_CONFIG.String(),
				},
				Sender: &commonPb.EndorsementEntry{
					Signer: &acPb.Member{
						OrgId:      "org1",
						MemberInfo: []byte("User1"),
					},
					Signature: []byte("signature1"),
				},
				Result: &commonPb.Result{
					Code: commonPb.TxStatusCode_SUCCESS,
					ContractResult: &commonPb.ContractResult{
						Result: ccBytes,
					},
				},
			},
		},
		Dag: &commonPb.DAG{
			Vertexes: []*commonPb.DAG_Neighbor{{
				Neighbors: []uint32{0, 1, 2},
			}},
		},
	}

	block.Header.BlockHash = generateBlockHash(chainId, height)
	block.Txs[0].Payload.TxId = generateTxId(chainId, height, 0)
	return block
}

func verifyArchive(t *testing.T, confHeight uint64, verifyHeader bool, blocks []*commonPb.Block,
	blockDB blockdb.BlockDB) {
	archivedPivot, err := blockDB.GetArchivedPivot()
	assert.Nil(t, err)

	if archivedPivot == 0 {
		verifyUnarchivedHeight(t, archivedPivot, verifyHeader, blocks, blockDB)
		verifyUnarchivedHeight(t, archivedPivot+1, verifyHeader, blocks, blockDB)
		return
	}

	// verify store apis: archived height
	verifyArchivedHeight(t, archivedPivot-1, verifyHeader, blocks, blockDB)

	// verify store apis: archivedPivot height
	verifyArchivedHeight(t, archivedPivot, verifyHeader, blocks, blockDB)

	// verify store apis: conf block height
	verifyUnarchivedHeight(t, confHeight, verifyHeader, blocks, blockDB)

	// verify store apis: unarchived height
	verifyUnarchivedHeight(t, archivedPivot+1, verifyHeader, blocks, blockDB)
}

func verifyUnarchivedHeight(t *testing.T, avBlkHeight uint64, verifyHeader bool,
	blocks []*commonPb.Block, blockDB blockdb.BlockDB) {
	avBlk := blocks[avBlkHeight]
	vbHeight, err1 := blockDB.GetHeightByHash(avBlk.Header.BlockHash)
	assert.True(t, err1 == nil)
	assert.Equal(t, vbHeight, avBlkHeight)

	if verifyHeader {
		header, err2 := blockDB.GetBlockHeaderByHeight(avBlk.Header.BlockHeight)
		assert.True(t, err2 == nil)
		assert.True(t, bytes.Equal(header.BlockHash, avBlk.Header.BlockHash))
	}

	vtHeight, err4 := blockDB.GetTxInfoOnly(avBlk.Txs[0].Payload.TxId)
	assert.True(t, err4 == nil)
	assert.Equal(t, vtHeight.BlockHeight, avBlkHeight)

	vtBlk, err5 := blockDB.GetBlockByTx(avBlk.Txs[0].Payload.TxId)
	assert.True(t, err5 == nil)
	assert.Equal(t, avBlk.Header.ChainId, vtBlk.Header.ChainId)

	vttx, err6 := blockDB.GetTx(avBlk.Txs[0].Payload.TxId)
	assert.True(t, err6 == nil)
	assert.Equal(t, avBlk.Header.ChainId, vttx.Payload.ChainId)

	vtBlk2, err7 := blockDB.GetBlockByHash(avBlk.Hash())
	assert.True(t, err7 == nil)
	assert.Equal(t, avBlk.Header.ChainId, vtBlk2.Header.ChainId)
}

func verifyArchivedHeight(t *testing.T, avBlkHeight uint64, verifyHeader bool,
	blocks []*commonPb.Block, blockDB blockdb.BlockDB) {
	avBlk := blocks[avBlkHeight]
	vbHeight, err1 := blockDB.GetHeightByHash(avBlk.Header.BlockHash)
	assert.True(t, err1 == nil)
	assert.Equal(t, vbHeight, avBlkHeight)

	if verifyHeader {
		header, err2 := blockDB.GetBlockHeaderByHeight(avBlk.Header.BlockHeight)
		assert.True(t, err2 == nil)
		assert.True(t, bytes.Equal(header.BlockHash, avBlk.Header.BlockHash))
	}

	vtHeight, err4 := blockDB.GetTxHeight(avBlk.Txs[0].Payload.TxId)
	assert.True(t, err4 == nil)
	assert.Equal(t, vtHeight, avBlkHeight)

	vtBlk, err5 := blockDB.GetBlockByTx(avBlk.Txs[0].Payload.TxId)
	assert.True(t, tbf.ErrArchivedBlock == err5)
	assert.True(t, vtBlk == nil)

	vttx, err6 := blockDB.GetTx(avBlk.Txs[0].Payload.TxId)
	assert.True(t, tbf.ErrArchivedTx == err6)
	assert.True(t, vttx == nil)

	vtBlk2, err7 := blockDB.GetBlockByHash(avBlk.Hash())
	assert.True(t, tbf.ErrArchivedBlock == err7)
	assert.True(t, vtBlk2 == nil)
}

func putBlock(brw *storePb.BlockWithRWSet, mgr *ArchiveMgr) error {
	blkBytes, blockWithSerializedInfo, err := serialization.SerializeBlock(brw)
	if err != nil {
		return err
	}

	if !mgr.storeConfig.DisableBlockFileDb {
		blockHeight := brw.Block.Header.BlockHeight
		fileName, offset, bytesLen, errf := mgr.fileStore.Write(blockHeight+1, blkBytes)
		if errf != nil {
			return errf
		}

		blockWithSerializedInfo.Index = &storePb.StoreInfo{
			FileName: fileName,
			Offset:   offset,
			ByteLen:  bytesLen,
		}
	}
	if err = mgr.blockDB.CommitBlock(blockWithSerializedInfo, true); err != nil {
		return err
	}
	if err = mgr.blockDB.CommitBlock(blockWithSerializedInfo, false); err != nil {
		return err
	}

	return mgr.resultDB.CommitBlock(blockWithSerializedInfo, false)
}
