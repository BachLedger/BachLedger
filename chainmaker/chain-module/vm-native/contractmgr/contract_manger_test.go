/*
 * Copyright (C) BABEC. All rights reserved.
 * Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

package contractmgr

import (
	"errors"
	"testing"

	"chainmaker.org/chainmaker/common/v2/crypto"

	configPb "chainmaker.org/chainmaker/pb-go/v2/config"

	pbac "chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/store"
	"chainmaker.org/chainmaker/pb-go/v2/syscontract"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/protocol/v2/mock"
	"chainmaker.org/chainmaker/protocol/v2/test"
	"chainmaker.org/chainmaker/utils/v2"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/assert"
)

func TestContractManagerRuntime_SaveContract(t *testing.T) {
	ctrl := gomock.NewController(t)
	txSimContext := mock.NewMockTxSimContext(ctrl)
	defer ctrl.Finish()
	acTest := mock.NewMockAccessControlProvider(ctrl)
	acTest.EXPECT().GetHashAlg().Return("SHA256").AnyTimes()
	acTest.EXPECT().NewMember(gomock.Any()).Return(&Mb{}, nil).AnyTimes()
	txSimContext.EXPECT().GetAccessControl().Return(acTest, nil).AnyTimes()

	bcCtrl := gomock.NewController(t)
	bcTest := mock.NewMockBlockchainStore(bcCtrl)
	//cfgTest := &configPb.ChainConfig{Vm: &configPb.Vm{AddrType: configPb.AddrType_CHAINMAKER}}
	cfgTest := &configPb.ChainConfig{Vm: &configPb.Vm{AddrType: configPb.AddrType_ZXL}}
	bcTest.EXPECT().GetLastChainConfig().Return(cfgTest, nil).AnyTimes()
	txSimContext.EXPECT().GetBlockchainStore().Return(bcTest).AnyTimes()
	txSimContext.EXPECT().GetBlockVersion().Return(uint32(2220)).AnyTimes()
	txSimContext.EXPECT().Get(gomock.Any(), gomock.Any()).Return([]byte{}, nil).AnyTimes()
	txSimContext.EXPECT().GetSender().Return(&pbac.Member{MemberInfo: []byte("user1")}).AnyTimes()
	txSimContext.EXPECT().Put(gomock.Any(), gomock.Any(), gomock.Any()).AnyTimes()
	//txSimContext.EXPECT().CallContract(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any(),
	//gomock.Any()).Return(&commonPb.ContractResult{Code: 0}, protocol.ExecOrderTxTypeNormal, commonPb.TxStatusCode_SUCCESS)
	runtime := &ContractManagerRuntime{log: &test.GoLogger{}}
	result, byteCodeKey, _, err := runtime.saveContract(txSimContext, "testContractName", "v1",
		[]byte("bytes"), commonPb.RuntimeType_WASMER)
	assert.Nil(t, err)
	t.Log(result)
	t.Log(byteCodeKey)
}

func TestContractManagerRuntime_InstallContract(t *testing.T) {
	ctrl := gomock.NewController(t)
	txSimContext := mock.NewMockTxSimContext(ctrl)
	defer ctrl.Finish()
	acTest := mock.NewMockAccessControlProvider(ctrl)
	acTest.EXPECT().GetHashAlg().Return("SHA256").AnyTimes()
	acTest.EXPECT().NewMember(gomock.Any()).Return(&Mb{}, nil).AnyTimes()
	txSimContext.EXPECT().GetAccessControl().Return(acTest, nil).AnyTimes()
	txSimContext.EXPECT().GetContractByName(
		syscontract.SystemContract_CONTRACT_MANAGE.String(),
	).Return(&commonPb.Contract{Name: ContractName}, nil).AnyTimes()

	bcCtrl := gomock.NewController(t)
	bcTest := mock.NewMockBlockchainStore(bcCtrl)
	cfgTest := &configPb.ChainConfig{Vm: &configPb.Vm{AddrType: configPb.AddrType_CHAINMAKER}}
	bcTest.EXPECT().GetLastChainConfig().Return(cfgTest, nil).AnyTimes()
	txSimContext.EXPECT().GetBlockchainStore().Return(bcTest).AnyTimes()
	txSimContext.EXPECT().GetBlockVersion().Return(uint32(2220)).AnyTimes()
	txSimContext.EXPECT().Get(gomock.Any(), gomock.Any()).Return([]byte{}, nil).AnyTimes()
	txSimContext.EXPECT().GetSender().Return(&pbac.Member{MemberInfo: []byte("user1")}).AnyTimes()
	txSimContext.EXPECT().Put(gomock.Any(), gomock.Any(), gomock.Any()).AnyTimes()
	txSimContext.EXPECT().CallContract(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any(),
		gomock.Any()).Return(&commonPb.ContractResult{Code: 0}, protocol.ExecOrderTxTypeNormal, commonPb.TxStatusCode_SUCCESS)
	runtime := &ContractManagerRuntime{log: &test.GoLogger{}}
	result, gas, err := runtime.installLT2300(txSimContext, "testContractName", "v1", []byte("bytes"),
		commonPb.RuntimeType_WASMER, map[string][]byte{})
	assert.Nil(t, err)
	t.Log(result)
	t.Log(gas)
}
func TestContractManagerRuntime_UpgradeContract(t *testing.T) {
	ctrl := gomock.NewController(t)
	txSimContext := mock.NewMockTxSimContext(ctrl)
	defer ctrl.Finish()
	contract := &commonPb.Contract{Name: "testContractName", Version: "v1", Status: commonPb.ContractStatus_NORMAL}
	cdata, _ := contract.Marshal()
	bcCtrl := gomock.NewController(t)
	bcTest := mock.NewMockBlockchainStore(bcCtrl)
	cfgTest := &configPb.ChainConfig{Vm: &configPb.Vm{AddrType: configPb.AddrType_CHAINMAKER}}
	bcTest.EXPECT().GetLastChainConfig().Return(cfgTest, nil).AnyTimes()
	txSimContext.EXPECT().GetBlockchainStore().Return(bcTest).AnyTimes()
	txSimContext.EXPECT().Get(gomock.Any(), gomock.Any()).Return(cdata, nil).AnyTimes()
	txSimContext.EXPECT().GetSender().Return(&pbac.Member{MemberInfo: []byte("user1")}).AnyTimes()
	txSimContext.EXPECT().GetBlockVersion().Return(uint32(2220)).AnyTimes()
	txSimContext.EXPECT().Put(gomock.Any(), gomock.Any(), gomock.Any()).AnyTimes()
	txSimContext.EXPECT().GetContractByName(
		syscontract.SystemContract_CONTRACT_MANAGE.String(),
	).Return(&commonPb.Contract{Name: ContractName}, nil).AnyTimes()
	txSimContext.EXPECT().CallContract(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any(),
		gomock.Any()).Return(&commonPb.ContractResult{Code: 0}, protocol.ExecOrderTxTypeNormal, commonPb.TxStatusCode_SUCCESS)
	runtime := &ContractManagerRuntime{log: &test.GoLogger{}}

	contract.RuntimeType = commonPb.RuntimeType_WASMER
	result, gas, err := runtime.upgradeLT2300(txSimContext, contract, []byte("bytes"),
		map[string][]byte{})
	assert.Nil(t, err) //version重复
	assert.NotNil(t, result)
	assert.Equal(t, gas, uint64(0))
}

func TestContractManagerRuntime_FreezeContract(t *testing.T) {
	ctrl := gomock.NewController(t)
	txSimContext := mock.NewMockTxSimContext(ctrl)
	defer ctrl.Finish()
	contract := &commonPb.Contract{Name: "testContractName", Version: "v1", Status: commonPb.ContractStatus_NORMAL}
	cdata, _ := contract.Marshal()
	txSimContext.EXPECT().Get(gomock.Any(), gomock.Any()).Return(cdata, nil).AnyTimes()
	txSimContext.EXPECT().GetBlockVersion().Return(uint32(2220)).AnyTimes()
	txSimContext.EXPECT().GetSender().Return(&pbac.Member{MemberInfo: []byte("user1")}).AnyTimes()
	txSimContext.EXPECT().Put(gomock.Any(), gomock.Any(), gomock.Any()).AnyTimes()
	runtime := &ContractManagerRuntime{log: &test.GoLogger{}}

	result, err := runtime.FreezeContract(txSimContext, "")
	assert.NotNil(t, err)
	t.Log(result)
	result, err = runtime.FreezeContract(txSimContext, "testContractName")
	assert.Nil(t, err)
	assert.True(t, result.Status == commonPb.ContractStatus_FROZEN)
	t.Log(result)
}

func TestContractManagerRuntime_FreezeContractFail(t *testing.T) {
	ctrl := gomock.NewController(t)
	txSimContext := mock.NewMockTxSimContext(ctrl)
	defer ctrl.Finish()
	contract := &commonPb.Contract{Name: "testContractName", Version: "v1", Status: commonPb.ContractStatus_FROZEN}
	cdata, _ := contract.Marshal()
	txSimContext.EXPECT().Get(gomock.Any(), gomock.Any()).Return(cdata, nil).AnyTimes()
	txSimContext.EXPECT().GetBlockVersion().Return(uint32(2220)).AnyTimes()
	txSimContext.EXPECT().GetSender().Return(&pbac.Member{MemberInfo: []byte("user1")}).AnyTimes()
	txSimContext.EXPECT().Put(gomock.Any(), gomock.Any(), gomock.Any()).AnyTimes()
	runtime := &ContractManagerRuntime{log: &test.GoLogger{}}

	_, err := runtime.FreezeContract(txSimContext, "testContractName")
	assert.NotNil(t, err)
	//t.Log(result)
	_, err = runtime.FreezeContract(txSimContext, "testContractName")
	assert.NotNil(t, err)
	//t.Log(result)

}

func TestContractManagerRuntime_UnfreezeContract(t *testing.T) {
	ctrl := gomock.NewController(t)
	txSimContext := mock.NewMockTxSimContext(ctrl)
	defer ctrl.Finish()
	contract := &commonPb.Contract{Name: "testContractName", Version: "v1", Status: commonPb.ContractStatus_FROZEN}
	cdata, _ := contract.Marshal()
	txSimContext.EXPECT().Get(gomock.Any(), gomock.Any()).Return(cdata, nil).AnyTimes()
	txSimContext.EXPECT().GetBlockVersion().Return(uint32(2220)).AnyTimes()
	txSimContext.EXPECT().GetSender().Return(&pbac.Member{MemberInfo: []byte("user1")}).AnyTimes()
	txSimContext.EXPECT().Put(gomock.Any(), gomock.Any(), gomock.Any()).AnyTimes()
	runtime := &ContractManagerRuntime{log: &test.GoLogger{}}
	result, err := runtime.UnfreezeContract(txSimContext, "testContractName")
	assert.Nil(t, err)
	assert.True(t, result.Status == commonPb.ContractStatus_NORMAL)
	t.Log(result)
}
func TestContractManagerRuntime_RevokeContract(t *testing.T) {
	ctrl := gomock.NewController(t)
	txSimContext := mock.NewMockTxSimContext(ctrl)
	defer ctrl.Finish()
	contract := &commonPb.Contract{Name: "testContractName", Version: "v1", Status: commonPb.ContractStatus_FROZEN}
	cdata, _ := contract.Marshal()
	txSimContext.EXPECT().Get(gomock.Any(), gomock.Any()).Return(cdata, nil).AnyTimes()
	txSimContext.EXPECT().GetBlockVersion().Return(uint32(2220)).AnyTimes()
	txSimContext.EXPECT().GetSender().Return(&pbac.Member{MemberInfo: []byte("user1")}).AnyTimes()
	txSimContext.EXPECT().Put(gomock.Any(), gomock.Any(), gomock.Any()).AnyTimes()
	runtime := &ContractManagerRuntime{log: &test.GoLogger{}}
	result, err := runtime.RevokeContract(txSimContext, "testContractName")
	assert.Nil(t, err)
	assert.True(t, result.Status == commonPb.ContractStatus_REVOKED)
	t.Log(result)
}
func TestContractManagerRuntime_GetContractInfo(t *testing.T) {
	ctrl := gomock.NewController(t)
	txSimContext := mock.NewMockTxSimContext(ctrl)
	defer ctrl.Finish()
	contract := &commonPb.Contract{Name: "testContractName", Version: "v1", Status: commonPb.ContractStatus_FROZEN}
	cdata, _ := contract.Marshal()
	txSimContext.EXPECT().Get(gomock.Any(), gomock.Any()).Return(cdata, nil).AnyTimes()
	txSimContext.EXPECT().GetSender().Return(&pbac.Member{MemberInfo: []byte("user1")}).AnyTimes()
	runtime := &ContractManagerRuntime{log: &test.GoLogger{}}
	c, err := runtime.GetContractInfo(txSimContext, "testContractName")
	assert.Nil(t, err)
	t.Log(c)
}
func TestContractManagerRuntime_GetAllContracts(t *testing.T) {
	ctrl := gomock.NewController(t)
	txSimContext := mock.NewMockTxSimContext(ctrl)
	defer ctrl.Finish()
	contract1 := &commonPb.Contract{Name: "testContractName1", Version: "v1", Status: commonPb.ContractStatus_FROZEN}
	contract2 := &commonPb.Contract{Name: "testContractName2", Version: "v1", Status: commonPb.ContractStatus_NORMAL}
	contract3 := &commonPb.Contract{Name: "testContractName3", Version: "v1", Status: commonPb.ContractStatus_FROZEN}
	list := &ListIter{list: []*commonPb.Contract{contract1, contract2, contract3}}
	txSimContext.EXPECT().Select(gomock.Any(), gomock.Any(), gomock.Any()).Return(list, nil).AnyTimes()
	txSimContext.EXPECT().GetSender().Return(&pbac.Member{MemberInfo: []byte("user1")}).AnyTimes()
	txSimContext.EXPECT().GetBlockVersion().Return(uint32(2300)).AnyTimes()
	runtime := &ContractManagerRuntime{log: &test.GoLogger{}}
	clist, err := runtime.GetAllContracts(txSimContext)
	assert.Nil(t, err)
	t.Log(clist)
	assert.EqualValues(t, 3, len(clist))
}

type ListIter struct {
	list []*commonPb.Contract
	idx  int
}

func (i *ListIter) Next() bool {
	i.idx++
	return i.idx < len(i.list)+1
}
func (i *ListIter) Value() (*store.KV, error) {
	c := i.list[i.idx-1]
	cdata, _ := c.Marshal()
	return &store.KV{
		ContractName: "",
		Key:          utils.GetContractDbKey(c.Name),
		Value:        cdata,
	}, nil
}
func (i *ListIter) Release() {
	i.list = make([]*commonPb.Contract, 0)
	i.idx = 0
}
func initParameters() map[string][]byte {
	parameters := make(map[string][]byte)
	parameters[syscontract.InitContract_CONTRACT_NAME.String()] = []byte("testContractName")
	parameters[syscontract.InitContract_CONTRACT_VERSION.String()] = []byte("v1")
	parameters[syscontract.InitContract_CONTRACT_BYTECODE.String()] = []byte("byte code!!!")
	parameters[syscontract.InitContract_CONTRACT_RUNTIME_TYPE.String()] = []byte("WASMER")
	return parameters
}

func TestContractManagerRuntime_InstallContract2(t *testing.T) {

	//-----------------------------------------------------------//
	ctrl := gomock.NewController(t)
	txSimContext := mock.NewMockTxSimContext(ctrl)
	defer ctrl.Finish()
	acTest := mock.NewMockAccessControlProvider(ctrl)
	acTest.EXPECT().GetHashAlg().Return("SHA256").AnyTimes()
	acTest.EXPECT().NewMember(gomock.Any()).Return(&Mb{}, nil).AnyTimes()
	txSimContext.EXPECT().GetAccessControl().Return(acTest, nil).AnyTimes()

	tx := &commonPb.Transaction{
		Payload: &commonPb.Payload{
			TxId: "transaction-id-12345",
		},
	}

	bcCtrl := gomock.NewController(t)
	bcTest := mock.NewMockBlockchainStore(bcCtrl)
	cfgTest := &configPb.ChainConfig{Vm: &configPb.Vm{AddrType: configPb.AddrType_CHAINMAKER}}
	bcTest.EXPECT().GetLastChainConfig().Return(cfgTest, nil).AnyTimes()
	txSimContext.EXPECT().GetBlockchainStore().Return(bcTest).AnyTimes()
	txSimContext.EXPECT().GetBlockVersion().Return(uint32(2300)).AnyTimes()
	txSimContext.EXPECT().Get(gomock.Any(), gomock.Any()).Return([]byte{}, nil).AnyTimes()
	txSimContext.EXPECT().GetTx().Return(tx).AnyTimes()
	txSimContext.EXPECT().GetSender().Return(&pbac.Member{MemberInfo: []byte("user1")}).AnyTimes()
	txSimContext.EXPECT().Put(gomock.Any(), gomock.Any(), gomock.Any()).AnyTimes()
	txSimContext.EXPECT().GetContractByName(
		syscontract.SystemContract_CONTRACT_MANAGE.String(),
	).Return(&commonPb.Contract{Name: ContractName}, nil).AnyTimes()
	txSimContext.EXPECT().CallContract(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any(),
		gomock.Any()).Return(&commonPb.ContractResult{Code: 0}, protocol.ExecOrderTxTypeNormal, commonPb.TxStatusCode_SUCCESS)
	runtime := &ContractManagerRuntime{log: &test.GoLogger{}}
	contract, result, err := runtime.install(txSimContext, "testContractName2", "v1", []byte("bytes"),
		commonPb.RuntimeType_WASMER, map[string][]byte{})
	assert.Nil(t, err)
	t.Log(contract)
	t.Log(result.GasUsed)
}

func TestContractManagerRuntime_UpgradeContract2(t *testing.T) {
	ctrl := gomock.NewController(t)
	txSimContext := mock.NewMockTxSimContext(ctrl)
	defer ctrl.Finish()
	contract := &commonPb.Contract{Name: "testContractName", Version: "v0", Status: commonPb.ContractStatus_NORMAL}
	cdata, _ := contract.Marshal()
	bcCtrl := gomock.NewController(t)
	bcTest := mock.NewMockBlockchainStore(bcCtrl)
	cfgTest := &configPb.ChainConfig{Vm: &configPb.Vm{AddrType: configPb.AddrType_CHAINMAKER}}
	bcTest.EXPECT().GetLastChainConfig().Return(cfgTest, nil).AnyTimes()
	txSimContext.EXPECT().GetBlockchainStore().Return(bcTest).AnyTimes()
	txSimContext.EXPECT().Get(gomock.Any(), gomock.Any()).Return(cdata, nil).AnyTimes()
	txSimContext.EXPECT().GetBlockVersion().Return(uint32(2220)).AnyTimes()
	txSimContext.EXPECT().GetSender().Return(&pbac.Member{MemberInfo: []byte("user1")}).AnyTimes()
	txSimContext.EXPECT().Put(gomock.Any(), gomock.Any(), gomock.Any()).AnyTimes()
	txSimContext.EXPECT().GetContractByName(
		syscontract.SystemContract_CONTRACT_MANAGE.String(),
	).Return(&commonPb.Contract{Name: ContractName}, nil).AnyTimes()
	txSimContext.EXPECT().CallContract(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any(),
		gomock.Any()).Return(&commonPb.ContractResult{Code: 0}, protocol.ExecOrderTxTypeNormal, commonPb.TxStatusCode_SUCCESS)
	runtime := &ContractManagerRuntime{log: &test.GoLogger{}}
	result := runtime.upgradeContract(txSimContext, initParameters())
	assert.EqualValues(t, 0, result.Code)
}

type Mb struct {
}

func (m Mb) GetPk() crypto.PublicKey {
	//TODO implement me
	panic("implement me")
}

func (m Mb) GetMemberId() string {
	return "memberId"
}

func (m Mb) GetOrgId() string {
	return "orgId"
}

func (m Mb) GetRole() protocol.Role {
	return "role"
}

func (m Mb) GetUid() string {
	return "uid"
}

func (m Mb) Verify(hashType string, msg []byte, sig []byte) error {
	panic("implement me")
}

func (m Mb) GetMember() (*pbac.Member, error) {
	panic("implement me")
}

func TestContractManagerRuntime_InitNewNativeContract(t *testing.T) {
	ctrl := gomock.NewController(t)
	txSimContext := mock.NewMockTxSimContext(ctrl)
	defer ctrl.Finish()

	bcCtrl := gomock.NewController(t)
	bcTest := mock.NewMockBlockchainStore(bcCtrl)
	cfgTest := &configPb.ChainConfig{Vm: &configPb.Vm{AddrType: configPb.AddrType_CHAINMAKER}}
	bcTest.EXPECT().GetLastChainConfig().Return(cfgTest, nil).AnyTimes()
	txSimContext.EXPECT().GetBlockchainStore().Return(bcTest).AnyTimes()
	txSimContext.EXPECT().GetBlockVersion().Return(uint32(2220)).AnyTimes()
	txSimContext.EXPECT().Get(gomock.Any(), gomock.Any()).AnyTimes().Return([]byte{}, errors.New("not found")).AnyTimes()

	contract := &commonPb.Contract{
		Name:        "CHAIN_CONFIG",
		Version:     "v1",
		RuntimeType: commonPb.RuntimeType_NATIVE,
		Status:      commonPb.ContractStatus_NORMAL,
	}
	list := &ListIter{list: []*commonPb.Contract{contract}}
	txSimContext.EXPECT().Select(gomock.Any(), gomock.Any(), gomock.Any()).Return(list, nil).AnyTimes()
	txSimContext.EXPECT().Get(gomock.Any(), gomock.Any()).AnyTimes().Return([]byte{}, errors.New("not found")).AnyTimes()
	txSimContext.EXPECT().Put(gomock.Any(), gomock.Any(), gomock.Any()).AnyTimes()
	runtime := &ContractManagerRuntime{log: &test.GoLogger{}}
	result, _ := runtime.InitNewNativeContract(txSimContext, initParameters())
	//assert.Nil(t, err)
	//t.Log(string(result))
	assert.NotContains(t, string(result), "CHAIN_CONFIG")
}
