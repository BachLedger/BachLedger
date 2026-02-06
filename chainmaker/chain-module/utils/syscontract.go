/*
 * Copyright (C) BABEC. All rights reserved.
 * Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

package utils

import (
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/syscontract"
)

const (
	// PrefixContractInfo prefix of contract info
	PrefixContractInfo = "Contract:"
	// PrefixContractByteCode prefix of contract bytecode
	PrefixContractByteCode = "ContractByteCode:"
)

// GetContractDbKey get contract db key
func GetContractDbKey(contractName string) []byte {
	return []byte(PrefixContractInfo + contractName)
}

// GetContractByteCodeDbKey get contract byte code db key
func GetContractByteCodeDbKey(contractName string) []byte {
	return []byte(PrefixContractByteCode + contractName)
}

// GetContractByName get contract by name
func GetContractByName(readObject func(contractName string, key []byte) ([]byte, error), name string) (
	*commonPb.Contract, error) {
	key := GetContractDbKey(name)
	value, err := readObject(syscontract.SystemContract_CONTRACT_MANAGE.String(), key)
	if err != nil {
		return nil, err
	}
	contract := &commonPb.Contract{}
	err = contract.Unmarshal(value)
	if err != nil {
		return nil, err
	}
	return contract, nil
}

// GetContractBytecode get contract bytecode
func GetContractBytecode(readObject func(contractName string, key []byte) ([]byte, error), name string) (
	[]byte, error) {
	key := GetContractByteCodeDbKey(name)
	return readObject(syscontract.SystemContract_CONTRACT_MANAGE.String(), key)
}

// IsNativeContract return is native contract name
func IsNativeContract(contractName string) bool {
	_, ok := syscontract.SystemContract_value[contractName]
	return ok
}
