package mock

import (
	"chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/config"
	vmPb "chainmaker.org/chainmaker/pb-go/v2/vm"
	"chainmaker.org/chainmaker/protocol/v2"
	"fmt"
)

func MockTxSimContext(blockVersion uint32, gasRemaining uint64, defaultGas uint64) protocol.TxSimContext {
	return &txSimContextMock{
		gasRemaining: gasRemaining,
		blockVersion: blockVersion,
		chainConfig: &config.ChainConfig{
			AccountConfig: &config.GasAccountConfig{
				DefaultGas: defaultGas,
				EnableGas:  true,
			},
		},
	}
}

type txSimContextMock struct {
	blockVersion uint32
	gasRemaining uint64
	chainConfig  *config.ChainConfig
}

func (t *txSimContextMock) Get(contractName string, key []byte) ([]byte, error) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetKeys(keys []*vmPb.BatchKey) ([]*vmPb.BatchKey, error) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetNoRecord(contractName string, key []byte) ([]byte, error) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetSnapshot() protocol.Snapshot {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) Put(name string, key []byte, value []byte) error {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) PutRecord(contractName string, value []byte, sqlType protocol.SqlType) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) PutIntoReadSet(contractName string, key []byte, value []byte) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) Del(name string, key []byte) error {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) Select(name string, startKey []byte, limit []byte) (protocol.StateIterator, error) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetHistoryIterForKey(contractName string, key []byte) (protocol.KeyHistoryIterator, error) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) CallContract(caller, contract *commonPb.Contract,
	method string, byteCode []byte, parameter map[string][]byte, gasUsed uint64,
	refTxType commonPb.TxType) (*commonPb.ContractResult, protocol.ExecOrderTxType, commonPb.TxStatusCode) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetCurrentResult() []byte {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetTx() *commonPb.Transaction {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetBlockHeight() uint64 {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetBlockFingerprint() string {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetBlockTimestamp() int64 {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetBlockProposer() *accesscontrol.Member {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetTxResult() *commonPb.Result {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) SetTxResult(result *commonPb.Result) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetTxRWSet(runVmSuccess bool) *commonPb.TxRWSet {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetCreator(namespace string) *accesscontrol.Member {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetSender() *accesscontrol.Member {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetBlockchainStore() protocol.BlockchainStore {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetLastChainConfig() *config.ChainConfig {
	return t.chainConfig
}

func (t *txSimContextMock) GetAccessControl() (protocol.AccessControlProvider, error) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetChainNodesInfoProvider() (protocol.ChainNodesInfoProvider, error) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetTxExecSeq() int {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) SetTxExecSeq(i int) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetDepth() int {
	//TODO implement me
	panic("implement me")
}

func (t txSimContextMock) SetIterHandle(index int32, iter interface{}) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetIterHandle(index int32) (interface{}, bool) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetBlockVersion() uint32 {
	return t.blockVersion
}

func (t *txSimContextMock) GetContractByName(name string) (*commonPb.Contract, error) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetContractBytecode(name string) ([]byte, error) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetTxRWMapByContractName(contractName string) (
	map[string]*commonPb.TxRead, map[string]*commonPb.TxWrite) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetCrossInfo() uint64 {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) HasUsed(runtimeType commonPb.RuntimeType) bool {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) RecordRuntimeTypeIntoCrossInfo(runtimeType commonPb.RuntimeType) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) RemoveRuntimeTypeFromCrossInfo() {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) GetStrAddrFromPbMember(pbMember *accesscontrol.Member) (string, error) {
	//TODO implement me
	panic("implement me")
}

func (t *txSimContextMock) SubtractGas(gasUsed uint64) error {
	oldGas := t.gasRemaining
	t.gasRemaining -= gasUsed
	if t.gasRemaining > oldGas {
		return fmt.Errorf("gas is not enough")
	}
	return nil
}

func (t *txSimContextMock) GetGasRemaining() uint64 {
	return t.gasRemaining
}
