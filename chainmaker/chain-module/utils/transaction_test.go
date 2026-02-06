/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package utils

import (
	"crypto/rand"
	"fmt"
	"testing"

	"chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/syscontract"
	"chainmaker.org/chainmaker/protocol/v2/mock"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/assert"
)

func TestGenerateInstallContractPayload(t *testing.T) {
	payload, err := GenerateInstallContractPayload("name1", "v1", commonPb.RuntimeType_WASMER,
		[]byte("code"), []*commonPb.KeyValuePair{})
	assert.Nil(t, err)
	assert.EqualValues(t, []byte("v1"), payload.GetParameter("CONTRACT_VERSION"))
}
func TestIsConfigTx(t *testing.T) {
	tx := &commonPb.Transaction{
		Payload: &commonPb.Payload{
			ChainId:        "c1",
			TxType:         commonPb.TxType_INVOKE_CONTRACT,
			TxId:           "tx1",
			Timestamp:      0,
			ExpirationTime: 0,
			ContractName:   syscontract.SystemContract_CHAIN_CONFIG.String(),
			Method:         "abc",
			Parameters:     nil,
			Sequence:       0,
			Limit:          nil,
		},
	}
	isConfig := IsConfigTx(tx)
	assert.True(t, isConfig)
	isConfig = IsConfigTx(nil)
	assert.False(t, isConfig)
}
func TestIsManagementTx(t *testing.T) {
	// is management tx contains cert manage, multi sign, pub key manage and contract manage tx.
	tx := &commonPb.Transaction{
		Payload: &commonPb.Payload{
			ContractName: syscontract.SystemContract_CERT_MANAGE.String(),
		},
	}
	assert.True(t, IsManagementTx(tx))
	assert.False(t, IsManagementTx(nil))
}

func TestIsCertManagementTx(t *testing.T) {
	tx := &commonPb.Transaction{
		Payload: &commonPb.Payload{
			ContractName: syscontract.SystemContract_CERT_MANAGE.String(),
		},
	}
	assert.True(t, IsCertManagementTx(tx))
	assert.False(t, IsCertManagementTx(nil))
}

func TestIsMultiSignManagementTx(t *testing.T) {
	tx := &commonPb.Transaction{
		Payload: &commonPb.Payload{
			ContractName: syscontract.SystemContract_MULTI_SIGN.String(),
		},
	}
	assert.True(t, IsMultiSignManagementTx(tx))
	assert.False(t, IsMultiSignManagementTx(nil))
}

func TestIsPubKeyManagementTx(t *testing.T) {
	tx := &commonPb.Transaction{
		Payload: &commonPb.Payload{
			ContractName: syscontract.SystemContract_PUBKEY_MANAGE.String(),
		},
	}
	assert.True(t, IsPubKeyManagementTx(tx))
	assert.False(t, IsPubKeyManagementTx(nil))
}

func TestIsContractManagementTx(t *testing.T) {
	tx := &commonPb.Transaction{
		Payload: &commonPb.Payload{
			ContractName: syscontract.SystemContract_CONTRACT_MANAGE.String(),
		},
	}
	assert.True(t, IsContractManagementTx(tx))
	assert.False(t, IsContractManagementTx(nil))
}

func TestIsContractMgmtTx(t *testing.T) {
	tx := &commonPb.Transaction{
		Payload: &commonPb.Payload{
			ChainId:        "c1",
			TxType:         commonPb.TxType_INVOKE_CONTRACT,
			TxId:           "tx1",
			Timestamp:      0,
			ExpirationTime: 0,
			ContractName:   syscontract.SystemContract_CHAIN_CONFIG.String(),
			Method:         "abc",
			Parameters:     nil,
			Sequence:       0,
			Limit:          nil,
		},
	}
	iscmgr := IsContractMgmtTx(tx)
	assert.False(t, iscmgr)
	tx.Payload.ContractName = syscontract.SystemContract_CONTRACT_MANAGE.String()
	tx.Payload.Method = syscontract.ContractManageFunction_INIT_CONTRACT.String()
	assert.True(t, IsContractMgmtTx(tx))
}
func TestIsManageContractAsConfigTx(t *testing.T) {
	tx := &commonPb.Transaction{
		Payload: &commonPb.Payload{
			ChainId:        "c1",
			TxType:         commonPb.TxType_INVOKE_CONTRACT,
			TxId:           "tx1",
			Timestamp:      0,
			ExpirationTime: 0,
			ContractName:   syscontract.SystemContract_CHAIN_CONFIG.String(),
			Method:         "abc",
			Parameters:     nil,
			Sequence:       0,
			Limit:          nil,
		},
	}
	iscmgr := IsManageContractAsConfigTx(tx, true)
	assert.False(t, iscmgr)
	tx.Payload.ContractName = syscontract.SystemContract_CONTRACT_MANAGE.String()
	tx.Payload.Method = syscontract.ContractManageFunction_INIT_CONTRACT.String()
	assert.True(t, IsManageContractAsConfigTx(tx, true))
	assert.False(t, IsManageContractAsConfigTx(nil, true))
}

func TestCalcResultBytes(t *testing.T) {
	result := &commonPb.Result{
		Code: 0,
		ContractResult: &commonPb.ContractResult{
			Code:          0,
			Result:        []byte("OK"),
			Message:       "OK",
			GasUsed:       1230,
			ContractEvent: nil,
		},
		RwSetHash: nil,
		Message:   "OK",
	}
	hash1, err := CalcResultBytes(result)
	assert.Nil(t, err)
	result.ContractResult.GasUsed = 9999
	hash2, err := CalcResultBytes(result)
	assert.Nil(t, err)
	assert.EqualValues(t, hash1, hash2)
	result.ContractResult.Message = "Not OK"
	hash3, err := CalcResultBytes(result)
	assert.Nil(t, err)
	assert.NotEqualValues(t, hash2, hash3)
}
func TestCalcTxResultHash(t *testing.T) {
	result := &commonPb.Result{
		Code: 0,
		ContractResult: &commonPb.ContractResult{
			Code:          0,
			Result:        []byte("OK"),
			Message:       "OK",
			GasUsed:       1230,
			ContractEvent: nil,
		},
		RwSetHash: nil,
		Message:   "OK",
	}
	hash1, err := CalcTxResultHash("SHA256", result)
	assert.Nil(t, err)
	result.ContractResult.GasUsed = 9999
	hash2, err := CalcTxResultHash("SHA256", result)
	assert.Nil(t, err)
	assert.EqualValues(t, hash1, hash2)
}

func TestCalcTxRequestHash(t *testing.T) {
	tx := &commonPb.Transaction{
		Payload: &commonPb.Payload{
			ChainId:        "c1",
			TxType:         commonPb.TxType_INVOKE_CONTRACT,
			TxId:           "tx1",
			Timestamp:      0,
			ExpirationTime: 0,
			ContractName:   syscontract.SystemContract_CHAIN_CONFIG.String(),
			Method:         "abc",
			Parameters:     nil,
			Sequence:       0,
			Limit:          nil,
		},
	}
	hash1, err := CalcTxRequestHash("SHA256", tx)
	assert.Nil(t, err)
	tx.Payload.TxType = commonPb.TxType_QUERY_CONTRACT
	hash2, err := CalcTxRequestHash("SHA256", tx)
	assert.Nil(t, err)
	assert.NotEqualValues(t, hash1, hash2)
}
func TestGetTxIds(t *testing.T) {
	txs := []*commonPb.Transaction{
		{
			Payload: &commonPb.Payload{
				TxId: "tx1",
			},
		},
		{
			Payload: &commonPb.Payload{
				TxId: "tx2",
			},
		},
	}
	txIds := GetTxIds(txs)
	assert.Equal(t, txIds[0], "tx1")
	assert.Equal(t, txIds[1], "tx2")
}
func TestVerifyTxWithoutPayload(t *testing.T) {
	tx := &commonPb.Transaction{
		Payload: &commonPb.Payload{
			ChainId:        "c1",
			TxType:         commonPb.TxType_INVOKE_CONTRACT,
			TxId:           "tx1",
			Timestamp:      0,
			ExpirationTime: 0,
			ContractName:   "usercontract1",
			Method:         "abc",
			Parameters:     nil,
			Sequence:       0,
			Limit:          nil,
		},
		Sender: &commonPb.EndorsementEntry{
			Signer: &accesscontrol.Member{
				OrgId:      "org1",
				MemberType: 0,
				MemberInfo: []byte("user1"),
			},
			Signature: []byte("sign"),
		},
		Endorsers: []*commonPb.EndorsementEntry{},
	}
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	acProvider := mock.NewMockAccessControlProvider(ctrl)
	acProvider.EXPECT().LookUpExceptionalPolicy(gomock.Any()).Return(nil, nil).AnyTimes()
	acProvider.EXPECT().LookUpPolicy(gomock.Any()).Return(&accesscontrol.Policy{Rule: "ANY"}, nil)
	acProvider.EXPECT().CreatePrincipal(gomock.Any(), gomock.Any(), gomock.Any()).Return(nil, nil).AnyTimes()
	acProvider.EXPECT().VerifyPrincipal(gomock.Any()).Return(true, nil).AnyTimes()
	err := VerifyTxWithoutPayload(tx, "c1", acProvider)
	assert.Nil(t, err)
}
func TestIsNativeTx(t *testing.T) {
	tx := &commonPb.Transaction{
		Payload: &commonPb.Payload{
			ChainId:        "",
			TxType:         0,
			TxId:           GetRandTxId(),
			Timestamp:      0,
			ExpirationTime: 0,
			ContractName:   "T",
			Method:         "P",
			Parameters:     nil,
			Sequence:       0,
			Limit:          nil,
		},
		Sender:    nil,
		Endorsers: nil,
		Result:    nil,
	}
	is, cname := IsNativeTx(tx)
	assert.True(t, is)
	assert.Equal(t, "T", cname)
}

func TestGetTimestampTxIdByNano(t *testing.T) {
	b := make([]byte, 1024)
	_, _ = rand.Read(b)
	fmt.Println(b)
}

func TestGetTxIdWithSeed(t *testing.T) {
	if GetTxIdWithSeed(defaultTimestamp) != GetTxIdWithSeed(defaultTimestamp) {
		t.Errorf("gen tx id not equal")
	}
}

func TestGetTimestampTxIdByNano1(t *testing.T) {
	t.Log(GetTimestampTxIdByNano(defaultTimestamp))
}
