/*
 * Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 */

package test

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"testing"

	acPb "chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/protocol/v2/test"
	leveldbprovider "chainmaker.org/chainmaker/store-leveldb/v2"
	"github.com/stretchr/testify/assert"
)

var ChainId = "chain1"

func createBlock(chainId string, height uint64, txNum int) *commonPb.Block {
	block := &commonPb.Block{
		Header: &commonPb.BlockHeader{
			ChainId:     chainId,
			BlockHeight: height,
			Proposer: &acPb.Member{
				OrgId:      "org1",
				MemberInfo: []byte("User1"),
			},
		},
		Txs: []*commonPb.Transaction{},
	}
	for i := 0; i < txNum; i++ {
		tx := &commonPb.Transaction{
			Payload: &commonPb.Payload{
				ChainId: chainId,
				TxType:  commonPb.TxType_INVOKE_CONTRACT,
				TxId:    generateTxId(chainId, height, i),
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
					Result: []byte("ok"),
				},
			},
		}
		block.Txs = append(block.Txs, tx)
	}
	block.Header.BlockHash = generateBlockHash(chainId, height)
	block.Txs[0].Payload.TxId = generateTxId(chainId, height, 0)

	return block
}

// 生成测试用的blockHash
func generateBlockHash(chainId string, height uint64) []byte {
	blockHash := sha256.Sum256([]byte(fmt.Sprintf("%s-%d", chainId, height)))
	return blockHash[:]
}

// 生成测试用的txid
func generateTxId(chainId string, height uint64, index int) string {
	txIdBytes := sha256.Sum256([]byte(fmt.Sprintf("%s-%d-%d", chainId, height, index)))
	return hex.EncodeToString(txIdBytes[:32])
}

func TestDebugStore_PutBlock(t *testing.T) {
	store := initDebugStore(t)
	b, err := store.GetBlock(0)
	assert.Nil(t, err)
	t.Logf("%#v", b)
}

func TestDebugStore_GetTxWithInfo(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetTxWithInfo("")
}

func TestDebugStore_GetTxInfoOnly(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetTxInfoOnly("")
}

func TestDebugStore_GetTxWithRWSet(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetTxWithRWSet("")
}

func TestDebugStore_GetTxInfoWithRWSet(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetTxInfoWithRWSet("")
}

func TestDebugStore_QuerySingle(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.QuerySingle("", "", nil)
}

func TestDebugStore_QueryMulti(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.QueryMulti("", "", nil)
}

func TestDebugStore_ExecDdlSql(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_ = store.ExecDdlSql("", "", "")
}

func TestDebugStore_BeginDbTransaction(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.BeginDbTransaction("")
}

func TestDebugStore_GetDbTransaction(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetDbTransaction("")
}

func TestDebugStore_CommitDbTransaction(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_ = store.CommitDbTransaction("")
}

func TestDebugStore_RollbackDbTransaction(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_ = store.RollbackDbTransaction("")
}

func TestDebugStore_CreateDatabase(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_ = store.CreateDatabase("")
}

func TestDebugStore_DropDatabase(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_ = store.DropDatabase("")
}

func TestDebugStore_GetContractDbName(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_ = store.GetContractDbName("")
}

func TestDebugStore_TxExistsInFullDB(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _, _ = store.TxExistsInFullDB("")
}

func TestDebugStore_TxExistsInIncrementDB(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.TxExistsInIncrementDB("", 0)
}

func TestDebugStore_TxExistsInIncrementDBState(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _, _ = store.TxExistsInIncrementDBState("", 0)
}

func TestDebugStore_GetContractByName(t *testing.T) {
	store := initDebugStore(t)
	_, err := store.GetContractByName("")
	assert.NotNil(t, err)
}

func TestDebugStore_GetContractBytecode(t *testing.T) {
	store := initDebugStore(t)
	_, err := store.GetContractBytecode("")
	assert.NotNil(t, err)
}

func TestDebugStore_GetMemberExtraData(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetMemberExtraData(nil)
}

func TestDebugStore_BlockExists(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.BlockExists(nil)
}

func TestDebugStore_GetHeightByHash(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetHeightByHash(nil)
}

func TestDebugStore_GetBlockHeaderByHeight(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetBlockHeaderByHeight(0)
}

func TestDebugStore_GetLastChainConfig(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetLastChainConfig()
}

func TestDebugStore_GetBlockByTx(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetBlockByTx("")
}

func TestDebugStore_GetBlockWithRWSets(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetBlockWithRWSets(0)
}

func TestDebugStore_GetTx(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetTx("")
}

func TestDebugStore_TxExists(t *testing.T) {
	store := initDebugStore(t)
	v, _ := store.TxExists("")
	assert.False(t, v)
}

func TestDebugStore_GetTxHeight(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetTxHeight("")
}

func TestDebugStore_GetTxConfirmedTime(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetTxConfirmedTime("")
}

func TestDebugStore_SelectObject(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.SelectObject("", nil, nil)
}

func TestDebugStore_GetTxRWSet(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetTxRWSet("")
}

func TestDebugStore_GetTxRWSetsByHeight(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetTxRWSetsByHeight(0)
}

func TestDebugStore_GetDBHandle(t *testing.T) {
	store := initDebugStore(t)
	_ = store.GetDBHandle("")
}

func TestDebugStore_GetArchivedPivot(t *testing.T) {
	store := initDebugStore(t)
	assert.Equal(t, store.GetArchivedPivot(), uint64(0))
}

func TestDebugStore_GetArchiveStatus(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetArchiveStatus()
}

func TestDebugStore_ArchiveBlock(t *testing.T) {
	store := initDebugStore(t)
	assert.Nil(t, store.ArchiveBlock(0))
}

func TestDebugStore_RestoreBlocks(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_ = store.RestoreBlocks(nil)
}

func TestDebugStore_GetHistoryForKey(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetHistoryForKey("", nil)
}

func TestDebugStore_GetAccountTxHistory(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetAccountTxHistory(nil)
}

func TestDebugStore_GetContractTxHistory(t *testing.T) {
	defer func() {
		assert.NotNil(t, recover())
	}()
	store := initDebugStore(t)
	_, _ = store.GetContractTxHistory("")
}

func initDebugStore(t *testing.T) *DebugStore {
	store := NewDebugStore(&test.GoLogger{}, nil, leveldbprovider.NewMemdbHandle())
	genesis := createBlock(ChainId, 0, 1)
	err := store.PutBlock(genesis, nil)
	assert.Nil(t, err)
	return store
}
