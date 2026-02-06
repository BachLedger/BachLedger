/*
 * Copyright 2020 The SealEVM Authors
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *  http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

package storage

import (
	"encoding/hex"
	"fmt"
	"strconv"

	"chainmaker.org/chainmaker/vm-evm/v2/evm-go/params"

	"chainmaker.org/chainmaker/utils/v2"

	"chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/config"
	"chainmaker.org/chainmaker/pb-go/v2/syscontract"

	"chainmaker.org/chainmaker/common/v2/evmutils"
	"chainmaker.org/chainmaker/logger/v2"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/vm-evm/v2/evm-go/environment"
)

var log = logger.GetLogger(logger.MODULE_VM)

type ContractStorage struct {
	OutParams *CrossVmParams
	//InParams        *CrossVmParams
	ResultCache ResultCache
	//ExternalStorage IExternalStorage
	readOnlyCache readOnlyCache
	Ctx           protocol.TxSimContext
	BlockHash     *evmutils.Int
	Contract      *common.Contract // contract info
}

func NewStorage(extStorage IExternalStorage) *ContractStorage {
	s := &ContractStorage{
		ResultCache: ResultCache{
			//OriginalData: CacheUnderAddress{},
			CachedData: CacheUnderAddress{},
			Balance:    BalanceCache{},
			Logs:       LogCache{},
			Destructs:  Cache{},
		},
		//ExternalStorage: extStorage,
		readOnlyCache: readOnlyCache{
			Code:      CodeCache{},
			CodeSize:  Cache{},
			CodeHash:  Cache{},
			BlockHash: Cache{},
		},
	}
	return s
}

//func (c *ContractStorage) GetBalance(address *evmutils.Int) (*evmutils.Int, error) {
//	return evmutils.New(0), nil
//}

func (c *ContractStorage) GetBalance(address *evmutils.Int) (*evmutils.Int, error) {
	v := c.GetCurrentBlockVersion()
	if v <= params.V2217 || v == params.V2300 || v == params.V2030100 {
		//2300 and 2310 have been released, but 2217 has found bugs, so versions before 2218, as well as 2300 and 2310
		//that have been released, use the old logic, and other versions use the new logic
		return evmutils.New(0), nil
	}

	var manager common.Contract
	method := syscontract.GasAccountFunction_GET_BALANCE.String()
	manager.Name = syscontract.SystemContract_ACCOUNT_MANAGER.String()
	manager.Status = common.ContractStatus_NORMAL

	parameters := make(map[string][]byte)
	parameters["address_key"] = []byte(hex.EncodeToString(address.Bytes()))

	b := evmutils.New(0)
	res, _, stat := c.Ctx.CallContract(c.Contract, &manager, method, nil, parameters, 0,
		common.TxType_INVOKE_CONTRACT)
	if stat != common.TxStatusCode_SUCCESS {
		return b, fmt.Errorf("failed to get balance of %s", hex.EncodeToString(address.Bytes()))
	}

	b.SetBytes(res.Result)
	return b, nil
}

func (c *ContractStorage) CanTransfer(from, to, val *evmutils.Int) bool {
	return false
}

func (c *ContractStorage) GetCode(address *evmutils.Int) (code []byte, err error) {
	//return utils.GetContractBytecode(c.Ctx.Get, address.String())
	var key string
	if c.Ctx.GetBlockVersion() < 2220 {
		//version < v2.2.2, contract.name is address, and codebyte stored by address
		key = hex.EncodeToString(address.Bytes())
	} else {
		//version >= v2.2.0 code stored by name, so get contract first
		contract, err := c.Ctx.GetContractByName(hex.EncodeToString(address.Bytes()))
		if err != nil {
			return nil, err
		}
		key = contract.Name
	}

	return c.Ctx.GetContractBytecode(key)
	//return utils.GetContractBytecode(c.Ctx.Get, hex.EncodeToString(address.Bytes()))

	//if contractName, err := c.Ctx.Get(address.String(), []byte(protocol.ContractAddress)); err == nil {
	//	versionKey := []byte(protocol.ContractVersion + address.String())
	//	if contractVersion, err := c.Ctx.Get(syscontract.SystemContract_CONTRACT_MANAGE.String(), versionKey); err == nil {
	//		versionedByteCodeKey := append([]byte(protocol.ContractByteCode), contractName...)
	//		versionedByteCodeKey = append(versionedByteCodeKey, contractVersion...)
	//		code, err = c.Ctx.Get(syscontract.SystemContract_CONTRACT_MANAGE.String(), versionedByteCodeKey)
	//		return code, err
	//	} else {
	//		log.Errorf("failed to get other contract byte code version, address [%s] , error :", address.String(), err.Error())
	//	}
	//}
	//log.Error("failed to get other contract  code :", err.Error())
	//return nil, err
}

func (c *ContractStorage) GetCodeSize(address *evmutils.Int) (size *evmutils.Int, err error) {
	code, err := c.GetCode(address)
	if err != nil {
		log.Error("failed to get other contract code size :", err.Error())
		return nil, err
	}
	return evmutils.New(int64(len(code))), err
}

func (c *ContractStorage) GetCodeHash(address *evmutils.Int) (codeHase *evmutils.Int, err error) {
	code, err := c.GetCode(address)
	if err != nil {
		log.Error("failed to get other contract code hash :", err.Error())
		return nil, err
	}
	hash := evmutils.Keccak256(code)
	i := evmutils.New(0)
	i.SetBytes(hash)
	return i, err
	return evmutils.New(int64(len(code))), err
}

func (c *ContractStorage) GetBlockHash(block *evmutils.Int) (*evmutils.Int, error) {
	currentHight := c.Ctx.GetBlockHeight() - 1
	high := evmutils.MinI(int64(currentHight), block.Int64())
	Block, err := c.Ctx.GetBlockchainStore().GetBlock(uint64(high))
	if err != nil {
		return evmutils.New(0), err
	}
	hash, err := evmutils.HashBytesToEVMInt(Block.GetHeader().GetBlockHash())
	if err != nil {
		return evmutils.New(0), err
	}
	return hash, nil
}

func (c *ContractStorage) GetCurrentBlockVersion() uint32 {
	return c.Ctx.GetBlockVersion()
}

////Create a unified address generation method within EVM to avoid duplicate wheels
//func generateAddress(data []byte, addrType int32) *evmutils.Int {
//	if addrType == int32(config.AddrType_ZXL) {
//		addr, _ := evmutils.ZXAddress(data)
//		return evmutils.FromHexString(addr[2:])
//	} else {
//		return evmutils.MakeAddress(data)
//	}
//}

//This is mostly called after 2300
//func (c *ContractStorage) CreateAddress(name *evmutils.Int, addrType int32) *evmutils.Int {
//	//in seal abc smart assets application, we always create fixed contract address.
//	data := name.Bytes()
//	//return generateAddress(data, addrType)
//	addr, _ := utils.GenerateAddrInt(data, config.AddrType(addrType))
//	return addr
//}

// Only versions < 2300 are called
func (c *ContractStorage) CreateFixedAddress(caller *evmutils.Int, salt *evmutils.Int, tx environment.Transaction, addrType int32) *evmutils.Int {
	data := append(caller.Bytes(), tx.TxHash...)
	if salt != nil {
		data = append(data, salt.Bytes()...)
	}

	//return generateAddress(data, addrType)
	addr, _ := utils.NameToAddrInt(string(data), config.AddrType(addrType), 2299)
	return addr
}

func (c *ContractStorage) Load(n string, k string) (*evmutils.Int, error) {
	var val []byte
	var err error
	if c.Ctx.GetBlockVersion() < 2300 {
		//version < 2300, cross call occurs inside the vm, so there will be multiple contrats ant it's address
		val, err = c.Ctx.Get(n, []byte(k))
	} else {
		//version >= 2300, cross call will be through the chain, so each vm has only one contract name
		val, err = c.Ctx.Get(c.Contract.Name, []byte(k))
	}

	if err != nil {
		return nil, err
	}

	r := evmutils.New(0)
	////When invoking evM contracts of other VM types, InParams is used for parameter transfer
	////If the value read by the contract is null, the parameter mode of InParams is turned on, and the key is
	////a number between 0 and 8, indicating that the contract is the parameter in the read parameter array
	//if val == nil && c.InParams.IsCrossVm {
	//	ik, _ := strconv.Atoi(k)
	//	//Since solidity method calls support a maximum of 6 parameters, a method name and a
	//	//closing tag have been added, so 8
	//	if 0 <= ik && ik < 8 {
	//		//value := c.GetCrossVmInParam(strconv.Itoa(ik))
	//		value := c.InParams.ParamsCache[strconv.Itoa(ik)]
	//		//Currently, a single parameter supports only 32 bytes of data
	//		bv := make([]byte, 32)
	//		copy(bv, value)
	//		bv[31] = byte(len(value) * 2)
	//		r.SetBytes(bv)
	//	}
	//} else {
	r.SetBytes(val)
	//}

	return r, err
}

func (c ContractStorage) Store(address string, key string, val []byte) {
	if c.Ctx.GetBlockVersion() < 2300 {
		//version < 2300, cross call occurs inside the vm, so there will be multiple contrats ant it's address
		c.Ctx.Put(address, []byte(key), val)
	} else {
		//version >= 2300, cross call will be through the chain, so each vm has only one contract name
		c.Ctx.Put(c.Contract.Name, []byte(key), val)
	}
}

func (c ContractStorage) IsCrossVmMode() bool {
	//Query whether the parameter transfer mode of cross-vm contract invocation is enabled, which is used by
	//sload directives and sstore directives to distinguish regular state read/write from read/write parameters
	return c.OutParams.IsCrossVm
}

//func (c ContractStorage) GetCrossVmInParam(key string) []byte {
//	return c.InParams.ParamsCache[key]
//}

func (c *ContractStorage) SetCrossVmOutParams(index *evmutils.Int, element *evmutils.Int) {
	var val []byte
	if !element.IsInt64() && !element.IsUint64() {
		//If the element is not a number, the element value is obtained after whitespace is removed
		val = TruncateNullTail(element.Bytes())
	}

	if string(val) == CrossVmOutParamsBeginKey {
		//Turn on the pass parameter flag if the element is invoked by an external cross-vm contract
		c.OutParams.IsCrossVm = true
		//Marks the slot at which the parameter is started
		c.OutParams.ParamsBegin = index.Int64()
		//OutParams starts writing
		c.OutParams.SetParam(CrossVmOutParamsBeginKey, []byte("start"))
		return
	}

	if (index.Int64()-c.OutParams.ParamsBegin == 1) && !element.IsInt64() {
		//The 0th element is the cross-vm contract invocation token, and the first element is the invoked method
		c.OutParams.SetParam(CrossVmCallMethodKey, val)
		return
	}

	if index.IsInt64() {
		if (index.Int64()-c.OutParams.ParamsBegin)%2 == 0 { //element is param's key
			c.OutParams.LastParamKey = string(val)
		} else { //elementis param's value
			if element.IsInt64() {
				//If element is an integer, it could be the value of a param or the length of a long string of more than 32 bytes
				value := strconv.FormatInt(element.Int64(), 10)
				c.OutParams.SetParam(c.OutParams.LastParamKey, []byte(value))
			} else {
				c.OutParams.SetParam(c.OutParams.LastParamKey, val)
			}
		}
	} else { //When index is not a numeric subscript, element is a fragment of a long string
		value := val
		if c.OutParams.LongStrLen == 0 {
			//If the string is marked 0, then the last stored element is the length of the string,
			//and the current element is the first segment of the string
			num := c.OutParams.GetParam(c.OutParams.LastParamKey)
			n, _ := strconv.ParseInt(string(num), 10, 64)
			c.OutParams.LongStrLen = n
			c.OutParams.SetParam(c.OutParams.LastParamKey, val)
		} else { //Element is a subsequent fragment of a long string
			value = append(c.OutParams.GetParam(c.OutParams.LastParamKey), val...)
			c.OutParams.SetParam(c.OutParams.LastParamKey, value)
		}

		if int64(len(value)*2+1) == c.OutParams.LongStrLen {
			//Reset long parameters that exceed 32 bytes
			c.OutParams.ResetLongStrParamStatus()
		}
	}
}

func (c ContractStorage) CallContract(name string, rtType int32, method string, byteCode []byte,
	parameters map[string][]byte, gasUsed uint64, isCreate bool) (res *common.ContractResult, stat common.TxStatusCode) {

	//Parameter storage mode is enabled only when the contract is invoked across virtual machines
	if c.OutParams.IsCrossVm {
		for k, v := range c.OutParams.ParamsCache {
			parameters[k] = v
		}
	}

	caller := &common.Contract{
		Address: string(parameters["0"]),
	}

	if isCreate {
		//If the cross-contract invocation is creating the contract, you are actually calling
		//the installContract method that manages the contract
		var contract common.Contract
		method = syscontract.ContractManageFunction_INIT_CONTRACT.String()
		contract.Name = syscontract.SystemContract_CONTRACT_MANAGE.String()
		contract.Status = common.ContractStatus_NORMAL

		parameters[syscontract.InitContract_CONTRACT_NAME.String()] = []byte(name)
		parameters[syscontract.InitContract_CONTRACT_VERSION.String()] = []byte("1.0.0")
		parameters[syscontract.InitContract_CONTRACT_RUNTIME_TYPE.String()] = []byte(common.RuntimeType(rtType).String())
		parameters[syscontract.InitContract_CONTRACT_BYTECODE.String()] = byteCode
		res, _, stat = c.Ctx.CallContract(caller, &contract, method, byteCode, parameters, gasUsed, common.TxType_INVOKE_CONTRACT)
	} else {
		contract, _ := c.Ctx.GetContractByName(name)
		if c.OutParams.IsCrossVm {
			//Method is not required to create a contract. The init_contract method is automatically called
			method = string(parameters[CrossVmCallMethodKey])
		}

		res, _, stat = c.Ctx.CallContract(caller, contract, method, byteCode, parameters, gasUsed, common.TxType_INVOKE_CONTRACT)
	}

	c.OutParams.Reset()
	return res, stat
}
