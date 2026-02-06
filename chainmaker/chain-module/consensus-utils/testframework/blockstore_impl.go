/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package testframework

//
//import (
//	"fmt"
//
//	"chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
//	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
//	configPb "chainmaker.org/chainmaker/pb-go/v2/config"
//	storePb "chainmaker.org/chainmaker/pb-go/v2/store"
//	"chainmaker.org/chainmaker/protocol/v2"
//)
//
// ####################################################################################################################
////                                       impls BlockchainStore for DPOS and HotStuff
// ####################################################################################################################
//type BlockchainStoreForTest struct {
//	states map[string][]byte
//}
//
////NewBlockChainStoreForTest
//func NewBlockChainStoreForTest() *BlockchainStoreForTest {
//	return &BlockchainStoreForTest{
//		states: make(map[string][]byte),
//	}
//}
//
////GetTxWithInfo
//func (b *BlockchainStoreForTest) GetTxWithInfo(txId string) (*commonPb.TransactionInfo, error) {
//	panic("implement me")
//}
//
////GetTxInfoOnly
//func (b *BlockchainStoreForTest) GetTxInfoOnly(txId string) (*commonPb.TransactionInfo, error) {
//	panic("implement me")
//}
//
////SetInitStates
//func (b *BlockchainStoreForTest) SetInitStates(contractName, key string, val []byte) {
//	dbKey := fmt.Sprintf("%s%x", contractName, key)
//	b.states[dbKey] = val
//}
//
////ReadObject
//func (b *BlockchainStoreForTest) ReadObject(contractName string, key []byte) ([]byte, error) {
//	dbKey := fmt.Sprintf("%s%x", contractName, key)
//	if val, ok := b.states[dbKey]; ok {
//		return val, nil
//	}
//	return nil, fmt.Errorf("not find the state: %s", dbKey)
//}
//
////InitGenesis
//func (b *BlockchainStoreForTest) InitGenesis(genesisBlock *storePb.BlockWithRWSet) error {
//	return nil
//}
//
////PutBlock
//func (b *BlockchainStoreForTest) PutBlock(block *commonPb.Block, txRWSets []*commonPb.TxRWSet) error {
//	return nil
//}
//
////SelectObject
//func (b *BlockchainStoreForTest) SelectObject(contractName string,
//	startKey []byte, limit []byte) (protocol.StateIterator, error) {
//	panic("implement me")
//}
//
////CreateDatabase
//func (b *BlockchainStoreForTest) CreateDatabase(contractName string) error {
//	panic("implement me")
//}
//
////DropDatabase
//func (b *BlockchainStoreForTest) DropDatabase(contractName string) error {
//	panic("implement me")
//}
//
////GetContractDbName
//func (b *BlockchainStoreForTest) GetContractDbName(contractName string) string {
//	panic("implement me")
//}
//
////GetMemberExtraData
//func (b *BlockchainStoreForTest) GetMemberExtraData(
//	member *accesscontrol.Member) (*accesscontrol.MemberExtraData, error) {
//	panic("implement me")
//}
//
////GetBlock
//func (b *BlockchainStoreForTest) GetBlock(height uint64) (*commonPb.Block, error) {
//	panic("implement me")
//}
//
////QuerySingle
//func (b *BlockchainStoreForTest) QuerySingle(contractName,
//sql string, values ...interface{}) (protocol.SqlRow, error) {
//	panic("implement me")
//}
//
////QueryMulti
//func (b *BlockchainStoreForTest) QueryMulti(contractName,
//sql string, values ...interface{}) (protocol.SqlRows, error) {
//	panic("implement me")
//}
//
////ExecDdlSql
//func (b *BlockchainStoreForTest) ExecDdlSql(contractName, sql, version string) error {
//	panic("implement me")
//}
//
////BeginDbTransaction
//func (b *BlockchainStoreForTest) BeginDbTransaction(txName string) (protocol.SqlDBTransaction, error) {
//	panic("implement me")
//}
//
////GetDbTransaction
//func (b *BlockchainStoreForTest) GetDbTransaction(txName string) (protocol.SqlDBTransaction, error) {
//	panic("implement me")
//}
//
////CommitDbTransaction
//func (b *BlockchainStoreForTest) CommitDbTransaction(txName string) error {
//	panic("implement me")
//}
//
////RollbackDbTransaction
//func (b *BlockchainStoreForTest) RollbackDbTransaction(txName string) error {
//	panic("implement me")
//}
//
////GetContractByName
//func (b *BlockchainStoreForTest) GetContractByName(name string) (*commonPb.Contract, error) {
//	panic("implement me")
//}
//
////GetContractBytecode
//func (b *BlockchainStoreForTest) GetContractBytecode(name string) ([]byte, error) {
//	panic("implement me")
//}
//
////GetBlockByHash
//func (b *BlockchainStoreForTest) GetBlockByHash(blockHash []byte) (*commonPb.Block, error) {
//	panic("implement me")
//}
//
////BlockExists
//func (b *BlockchainStoreForTest) BlockExists(blockHash []byte) (bool, error) {
//	panic("implement me")
//}
//
////GetHeightByHash
//func (b *BlockchainStoreForTest) GetHeightByHash(blockHash []byte) (uint64, error) {
//	panic("implement me")
//}
//
////GetBlockHeaderByHeight
//func (b *BlockchainStoreForTest) GetBlockHeaderByHeight(height uint64) (*commonPb.BlockHeader, error) {
//	panic("implement me")
//}
//
////GetLastConfigBlock
//func (b *BlockchainStoreForTest) GetLastConfigBlock() (*commonPb.Block, error) {
//	panic("implement me")
//}
//
////GetLastChainConfig
//func (b *BlockchainStoreForTest) GetLastChainConfig() (*configPb.ChainConfig, error) {
//	panic("implement me")
//}
//
////GetBlockByTx
//func (b *BlockchainStoreForTest) GetBlockByTx(txId string) (*commonPb.Block, error) {
//	panic("implement me")
//}
//
////GetBlockWithRWSets
//func (b *BlockchainStoreForTest) GetBlockWithRWSets(height uint64) (*storePb.BlockWithRWSet, error) {
//	panic("implement me")
//}
//
////GetTx
//func (b *BlockchainStoreForTest) GetTx(txId string) (*commonPb.Transaction, error) {
//	panic("implement me")
//}
//
////TxExists
//func (b *BlockchainStoreForTest) TxExists(txId string) (bool, error) {
//	panic("implement me")
//}
//
////GetTxHeight
//func (b *BlockchainStoreForTest) GetTxHeight(txId string) (uint64, error) {
//	panic("implement me")
//}
//
////GetTxConfirmedTime
//func (b *BlockchainStoreForTest) GetTxConfirmedTime(txId string) (int64, error) {
//	panic("implement me")
//}
//
////GetLastBlock
//func (b *BlockchainStoreForTest) GetLastBlock() (*commonPb.Block, error) {
//	panic("implement me")
//}
//
////GetTxRWSet
//func (b *BlockchainStoreForTest) GetTxRWSet(txId string) (*commonPb.TxRWSet, error) {
//	panic("implement me")
//}
//
////GetTxRWSetsByHeight
//func (b *BlockchainStoreForTest) GetTxRWSetsByHeight(height uint64) ([]*commonPb.TxRWSet, error) {
//	panic("implement me")
//}
//
////GetDBHandle
//func (b *BlockchainStoreForTest) GetDBHandle(dbName string) protocol.DBHandle {
//	panic("implement me")
//}
//
////GetArchivedPivot
//func (b *BlockchainStoreForTest) GetArchivedPivot() uint64 {
//	panic("implement me")
//}
//
////ArchiveBlock
//func (b *BlockchainStoreForTest) ArchiveBlock(archiveHeight uint64) error {
//	panic("implement me")
//}
//
////RestoreBlocks
//func (b *BlockchainStoreForTest) RestoreBlocks(serializedBlocks [][]byte) error {
//	panic("implement me")
//}
//
////Close
//func (b *BlockchainStoreForTest) Close() error {
//	panic("implement me")
//}
//
////GetHistoryForKey
//func (b *BlockchainStoreForTest) GetHistoryForKey(contractName string,
//	key []byte) (protocol.KeyHistoryIterator, error) {
//	panic("implement me")
//}
//
////GetAccountTxHistory
//func (b *BlockchainStoreForTest) GetAccountTxHistory(accountId []byte) (protocol.TxHistoryIterator, error) {
//	panic("implement me")
//}
//
////GetContractTxHistory
//func (b *BlockchainStoreForTest) GetContractTxHistory(contractName string) (protocol.TxHistoryIterator, error) {
//	panic("implement me")
//}
