/*
 * Copyright (C) BABEC. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 */
package blockcontract

import (
	"fmt"
	"testing"

	"github.com/gogo/protobuf/proto"

	"chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	acPb "chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	storePb "chainmaker.org/chainmaker/pb-go/v2/store"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/protocol/v2/mock"
	"chainmaker.org/chainmaker/protocol/v2/test"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"
)

var (
	chainId = "chain1"
	height  = uint64(0)
	txId    = "56789"
	log     = &test.GoLogger{}
	b       = &BlockRuntime{log: log}
)

func Test_handleError(t *testing.T) {
	br := BlockRuntime{
		log: &test.GoLogger{},
	}
	//err := br.handleError(BlockHeight(0), fmt.Errorf("not found"), "chain1")
	//require.NotNil(t, err)
	err := br.handleError(getBlockHeightByTxId(0), nil, "chain1")
	require.Nil(t, err)
}

func TestNewBlockContract(t *testing.T) {

	blockContract := NewBlockContract(log)

	fmt.Println(blockContract)
}

func TestGetNodeChainList(t *testing.T) {

	txSimContext := getTxSimContext(t)

	//parameters := map[string][]byte{}
	//parameters["test"] = []byte("123456")
	//parameters[paramNameBlockHeight] = []byte("123456")
	//parameters[paramNameWithRWSet] = []byte("123456")
	//parameters[paramNameBlockHash] = []byte("123456")
	//parameters[paramNameTxId] = []byte("123456")

	res, err := b.GetNodeChainList(txSimContext, nil)

	require.Nil(t, err)

	// TODO fix validateParams blockcontract/block_contract.go:940
	fmt.Println(string(res))
}

func TestGetChainInfo(t *testing.T) {

	txSimContext := getTxSimContext(t)

	parameters := map[string][]byte{}
	parameters[paramNameBlockHeight] = []byte("123456")
	parameters[paramNameWithRWSet] = []byte("123456")
	parameters[paramNameBlockHash] = []byte("123456")
	parameters[paramNameTxId] = []byte("123456")

	res, err := b.GetChainInfo(txSimContext, nil)

	require.Nil(t, err)
	fmt.Println(string(res))
}

func TestGetBlockByHeight(t *testing.T) {

	block := createNewBlock(chainId, height)
	store := getStoreByBlock(t, block, chainId, txId)
	res, err := b.getBlockByHeight(store, chainId, height)

	require.Nil(t, err)
	fmt.Println(res)
}

func TestGetFullBlockByHeight(t *testing.T) {

	block := createNewBlock(chainId, height)
	store := getStoreByBlock(t, block, chainId, txId)
	res, err := b.getFullBlockByHeight(store, chainId, 1)

	require.NotNil(t, err)
	fmt.Println(res)
}

func TestGetBlockHeaderByHeight(t *testing.T) {

	block := createNewBlock(chainId, height)
	store := getStoreByBlock(t, block, chainId, txId)
	res, err := b.getBlockHeaderByHeight(store, chainId, height)

	require.Nil(t, err)
	fmt.Println(res)
}

func TestGetBlockByTxId(t *testing.T) {

	block := createNewBlock(chainId, height)
	store := getStoreByBlock(t, block, chainId, txId)

	res, err := b.getBlockByTxId(store, chainId, txId)

	require.Nil(t, err)
	fmt.Println(res)
}

func TestGetLastConfigBlock(t *testing.T) {
	txSimContext := getTxSimContext(t)
	res, err := b.GetLastConfigBlock(txSimContext, map[string][]byte{})

	require.Nil(t, err)
	fmt.Println(res)
}

//func TestGetTxByTxId(t *testing.T) {
//
//	block := createNewBlock(chainId, height)
//
//	store := getStoreByBlock(t, block, chainId, txId)
//
//	res, err := b.getTxByTxId(store, chainId, txId)
//
//	require.Nil(t, err)
//	fmt.Println(res)
//}

func TestGetTxRWSetsByBlock(t *testing.T) {

	block := createNewBlock(chainId, height)

	store := getStoreByBlock(t, block, chainId, txId)

	res, err := b.getTxRWSetsByBlock(store, chainId, block)

	require.Nil(t, err)
	fmt.Println(res)
}

func TestGetArchiveBlockHeight(t *testing.T) {

	txSimContext := getTxSimContext(t)

	parameters := map[string][]byte{}
	//parameters["test"] = []byte("123456")
	parameters[paramNameBlockHeight] = []byte("123456")
	parameters[paramNameWithRWSet] = []byte("123456")
	parameters[paramNameBlockHash] = []byte("123456")
	parameters[paramNameTxId] = []byte("123456")

	res, err := b.GetArchiveBlockHeight(txSimContext, parameters)

	require.Nil(t, err)
	fmt.Println(res)
}

func TestGetArchiveStatus(t *testing.T) {
	res, err := b.GetArchiveStatus(getTxSimContext(t), nil)

	require.Nil(t, err)
	fmt.Println(res)
}

func TestGetChainNodeInfo(t *testing.T) {

	provider := getChainNodesInfoProvider(t)
	res, err := b.getChainNodeInfo(provider, chainId)

	require.Nil(t, err)
	fmt.Println(res)
}

func TestGetBlockHeightByTxId(t *testing.T) {

	block := createNewBlock(chainId, height)

	store := getStoreByBlock(t, block, chainId, txId)
	res, err := b.getBlockHeightByTxId(store, chainId, txId)

	require.Nil(t, err)
	fmt.Println(res)
}

func TestGetBlockHeightByHash(t *testing.T) {

	txSimContext := getTxSimContext(t)

	parameters := map[string][]byte{}
	parameters[paramNameBlockHash] = []byte("123456")

	res, err := b.GetBlockHeightByHash(txSimContext, parameters)

	require.Nil(t, err)
	fmt.Println(res)
}

func TestGetBlockByHash(t *testing.T) {
	txSimContext := getTxSimContext(t)

	parameters := map[string][]byte{}
	parameters[paramNameBlockHash] = []byte("123456")

	res, err := b.GetBlockByHash(txSimContext, parameters)

	require.Nil(t, err)
	fmt.Println(res)
}

func TestCheckRoleAndFilterBlockTxs(t *testing.T) {
	block := createNewBlock(chainId, height)

	txSimContext := getTxSimContext(t)

	txRWSets := make([]*commonPb.TxRWSet, 0)
	txRWSets = append(txRWSets, &commonPb.TxRWSet{TxId: "1"})

	block, txRWSets, _, err := checkRoleAndFilterBlockTxs(block, txSimContext, nil, nil)
	blockInfo := &commonPb.BlockInfo{
		Block:     block,
		RwsetList: txRWSets,
	}
	blockInfoBytes, err := proto.Marshal(blockInfo)
	require.Nil(t, err)
	fmt.Println(block)
	fmt.Println(blockInfoBytes)
}

//func TestCheckRoleAndGenerateTransactionInfo(t *testing.T) {
//
//	block := createNewBlock(chainId, height)
//
//	txSimContext := getTxSimContext(t)
//	store := getStoreByBlock(t, block, chainId, txId)
//
//	tx, err := b.getTxByTxId(store, chainId, txId)
//
//	transactionInfo := &commonPb.TransactionInfo{
//		Transaction: tx,
//		BlockHeight: block.Header.BlockHeight,
//	}
//
//	txRes, err := checkRoleAndGenerateTransactionInfo(txSimContext, transactionInfo)
//
//	require.Nil(t, err)
//	fmt.Println(txRes)
//}

func createNewBlock(chainId string, height uint64) *commonPb.Block {

	block := &commonPb.Block{
		Header: &commonPb.BlockHeader{
			BlockHeight:    height,
			PreBlockHash:   nil,
			BlockHash:      nil,
			BlockVersion:   0,
			DagHash:        nil,
			RwSetRoot:      nil,
			BlockTimestamp: 1,
			Proposer:       &acPb.Member{MemberInfo: []byte{1, 2, 3}},
			ConsensusArgs:  nil,
			TxCount:        0,
			Signature:      nil,
			ChainId:        chainId,
		},
		Dag: &commonPb.DAG{
			Vertexes: nil,
		},
	}
	block.Header.PreBlockHash = nil

	return block
}

func getStoreByBlock(t *testing.T, block *commonPb.Block, chainId string, txId string) protocol.BlockchainStore {

	ctrl := gomock.NewController(t)
	store := mock.NewMockBlockchainStore(ctrl)

	store.EXPECT().GetBlock(gomock.Any()).Return(block, nil).AnyTimes()
	store.EXPECT().GetBlockWithRWSets(gomock.Any()).Return(nil, nil).AnyTimes()
	store.EXPECT().GetBlockHeaderByHeight(gomock.Any()).Return(&commonPb.BlockHeader{
		ChainId: chainId,
	}, nil).AnyTimes()

	store.EXPECT().GetLastConfigBlock().Return(block, nil).AnyTimes()
	store.EXPECT().GetLastBlock().Return(block, nil).AnyTimes()
	store.EXPECT().GetHeightByHash(gomock.Any()).Return(height, nil).AnyTimes()

	store.EXPECT().GetBlockByTx(gomock.Any()).Return(block, nil).AnyTimes()
	store.EXPECT().GetTxHeight(gomock.Any()).Return(height, nil).AnyTimes()
	store.EXPECT().GetBlockByHash(gomock.Any()).Return(block, nil).AnyTimes()

	store.EXPECT().GetTx(txId).Return(&commonPb.Transaction{
		Payload: &commonPb.Payload{
			TxId: txId,
		},
	}, nil).AnyTimes()

	store.EXPECT().GetArchivedPivot().Return(uint64(10)).AnyTimes()
	store.EXPECT().GetArchiveStatus().Return(&storePb.ArchiveStatus{
		Type:                  storePb.StoreType_BFDB,
		MaxAllowArchiveHeight: 10,
		ArchivePivot:          20,
		FileRanges:            make([]*storePb.FileRange, 0),
	}, nil).AnyTimes()

	return store
}

func getTxSimContext(t *testing.T) protocol.TxSimContext {
	ctrl := gomock.NewController(t)
	txSimContext := mock.NewMockTxSimContext(ctrl)

	txSimContext.EXPECT().GetTx().Return(&commonPb.Transaction{
		Payload: &commonPb.Payload{
			ChainId: chainId,
		},
		Sender: &commonPb.EndorsementEntry{
			Signer: &accesscontrol.Member{
				OrgId:      "org1",
				MemberInfo: []byte("org1"),
			},
		},
	}).AnyTimes()

	block := createNewBlock(chainId, height)
	store := getStoreByBlock(t, block, chainId, txId)

	txSimContext.EXPECT().GetBlockchainStore().Return(store).AnyTimes()

	acProvider := getAccessControlProvider(t)
	txSimContext.EXPECT().GetAccessControl().Return(acProvider, nil).AnyTimes()

	provider := getChainNodesInfoProvider(t)
	txSimContext.EXPECT().GetChainNodesInfoProvider().Return(provider, nil).AnyTimes()

	return txSimContext
}

func getChainNodesInfoProvider(t *testing.T) protocol.ChainNodesInfoProvider {

	ctrl := gomock.NewController(t)

	provider := mock.NewMockChainNodesInfoProvider(ctrl)

	provider.EXPECT().GetChainNodesInfo().Return([]*protocol.ChainNodeInfo{}, nil).AnyTimes()

	return provider
}

func getAccessControlProvider(t *testing.T) protocol.AccessControlProvider {
	ctrl := gomock.NewController(t)

	provider := mock.NewMockAccessControlProvider(ctrl)

	member := mock.NewMockMember(ctrl)

	member.EXPECT().GetRole().Return(protocol.RoleAdmin).AnyTimes()

	provider.EXPECT().NewMember(gomock.Any()).Return(member, nil).AnyTimes()

	return provider
}
