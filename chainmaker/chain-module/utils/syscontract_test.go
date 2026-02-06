/*
 * Copyright (C) BABEC. All rights reserved.
 * Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

package utils

import (
	"testing"

	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/syscontract"
	"github.com/stretchr/testify/assert"
)

const contractName = "userContract1"

func TestGetContractByName(t *testing.T) {
	db := newMockDB()
	contract := &commonPb.Contract{Name: contractName, Version: "1.0"}
	contractBytes, _ := contract.Marshal()
	err := db.setObject(syscontract.SystemContract_CONTRACT_MANAGE.String(), GetContractDbKey(contractName), contractBytes)
	assert.Nil(t, err)
	dbContract, err := GetContractByName(db.readObject, contractName)
	assert.Nil(t, err)
	assert.Equal(t, contractName, dbContract.Name)
}
func TestGetContractBytecode(t *testing.T) {
	db := newMockDB()
	byteCode := []byte("Hello")
	err := db.setObject(syscontract.SystemContract_CONTRACT_MANAGE.String(), GetContractByteCodeDbKey(contractName), byteCode)
	assert.Nil(t, err)
	dbContract, err := GetContractBytecode(db.readObject, contractName)
	assert.Nil(t, err)
	assert.EqualValues(t, byteCode, dbContract)
}

type mockDb struct {
	data map[string]map[string][]byte
}

func newMockDB() *mockDb {
	return &mockDb{data: make(map[string]map[string][]byte)}
}
func (db *mockDb) readObject(contractName string, key []byte) ([]byte, error) {
	return db.data[contractName][string(key)], nil
}
func (db *mockDb) setObject(contractName string, key, value []byte) error {
	_, ok := db.data[contractName]
	if !ok {
		db.data[contractName] = make(map[string][]byte)
	}
	db.data[contractName][string(key)] = value
	return nil
}
func TestIsNativeContract(t *testing.T) {
	is := IsNativeContract("T")
	assert.True(t, is)
}
