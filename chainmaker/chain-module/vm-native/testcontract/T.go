/*
 * Copyright (C) BABEC. All rights reserved.
 * Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

package testcontract

import (
	"fmt"
	"strconv"
	"time"

	"chainmaker.org/chainmaker/pb-go/v2/syscontract"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/vm-native/v2/common"
)

var (
	//ContractName current contract name
	ContractName = syscontract.SystemContract_T.String()
)

// Manager contract manager
type Manager struct {
	methods map[string]common.ContractFunc
	log     protocol.Logger
}

// NewManager constructor of Manager
// @param log
// @return *Manager
func NewManager(log protocol.Logger) *Manager {
	return &Manager{
		log:     log,
		methods: registerTestContractMethods(log),
	}
}

// GetMethod get register method by name
func (c *Manager) GetMethod(methodName string) common.ContractFunc {
	return c.methods[methodName]
}

func registerTestContractMethods(log protocol.Logger) map[string]common.ContractFunc {
	methodMap := make(map[string]common.ContractFunc, 64)
	runtime := &ManagerRuntime{log: log}
	methodMap[syscontract.TestContractFunction_P.String()] = common.WrapResultFunc(runtime.put)
	methodMap[syscontract.TestContractFunction_G.String()] = common.WrapResultFunc(runtime.get)
	methodMap[syscontract.TestContractFunction_D.String()] = common.WrapResultFunc(runtime.del)
	methodMap[syscontract.TestContractFunction_N.String()] = common.WrapResultFunc(runtime.nothing)
	//以下是各种性能测试用的查询
	methodMap["Q"] = common.WrapResultFunc(runtime.queryTest)
	methodMap["R"] = common.WrapResultFunc(runtime.rangeTest)
	methodMap["H"] = common.WrapResultFunc(runtime.historyTest)

	return methodMap
}

// ManagerRuntime runtime instance
type ManagerRuntime struct {
	log protocol.Logger
}

// get get state by key["k"]
func (r *ManagerRuntime) get(context protocol.TxSimContext, parameters map[string][]byte) ([]byte, error) {
	return context.Get(ContractName, parameters["k"])
}

// put put state by key="k" value="v"
func (r *ManagerRuntime) put(context protocol.TxSimContext, parameters map[string][]byte) ([]byte, error) {
	k := parameters["k"]
	v := parameters["v"]
	return nil, context.Put(ContractName, k, v)
}

// del delete state by key="k"
func (r *ManagerRuntime) del(context protocol.TxSimContext, parameters map[string][]byte) ([]byte, error) {
	return nil, context.Del(ContractName, parameters["k"])
}

// nothing  do nothing
func (r *ManagerRuntime) nothing(txSimContext protocol.TxSimContext, parameters map[string][]byte) (
	[]byte, error) {
	return nil, nil
}

// qtest Query Test,这是一个大量GetState的性能测试方法，主要是为了测试在一个合约方法中如果进行了大量的GetState，会耗费多少的时间
// @param txSimContext
// @param parameters
// @return []byte
// @return error
func (r *ManagerRuntime) queryTest(txSimContext protocol.TxSimContext, parameters map[string][]byte) (
	[]byte, error) {
	cname := string(parameters["contract"])
	start, _ := strconv.Atoi(string(parameters["start"]))
	end, _ := strconv.Atoi(string(parameters["end"]))
	key := string(parameters["key"])
	count := 0
	startTime := time.Now()
	for i := start; i < end; i++ {
		stateKey := fmt.Sprintf(key, i)
		v, err := txSimContext.Get(cname, []byte(stateKey))
		if err != nil {
			return nil, fmt.Errorf("query contract[%s] by key[%s] get error: %s", cname, stateKey, err)
		}
		if len(v) > 0 {
			count++
		}
	}
	return []byte(fmt.Sprintf("query count=%d, spend time:%v", count, time.Since(startTime))), nil
}

// rangeTest 范围查询（前缀查询）性能测试用
// @param txSimContext
// @param parameters
// @return []byte
// @return error
func (r *ManagerRuntime) rangeTest(txSimContext protocol.TxSimContext, parameters map[string][]byte) (
	[]byte, error) {
	cname := string(parameters["contract"])
	count, _ := strconv.Atoi(string(parameters["count"]))
	prefix := parameters["prefix"]
	resultType := string(parameters["result"])
	start, limit := bytesPrefix(prefix)
	startTime := time.Now()
	iter, err := txSimContext.Select(cname, start, limit)
	if err != nil {
		return nil, fmt.Errorf("query contract[%s] by prefix[%s] get error: %s", cname, string(prefix), err)
	}
	defer iter.Release()
	i := 0
	result := ""
	for iter.Next() {
		kv, _ := iter.Value()
		if resultType == "key" {
			result += string(kv.Key) + ";"
		} else if resultType == "kv" {
			result += kv.String() + ";"
		}
		i++
		if i >= count {
			break
		}
	}
	return []byte(fmt.Sprintf("range query keys=%s, spend time:%v", result, time.Since(startTime))), nil
}
func bytesPrefix(prefix []byte) ([]byte, []byte) {
	var limit []byte
	for i := len(prefix) - 1; i >= 0; i-- {
		c := prefix[i]
		if c < 0xff {
			limit = make([]byte, i+1)
			copy(limit, prefix)
			limit[i] = c + 1
			break
		}
	}
	return prefix, limit
}

// historyTest Key的历史记录查询
// @param txSimContext
// @param parameters
// @return []byte
// @return error
func (r *ManagerRuntime) historyTest(txSimContext protocol.TxSimContext, parameters map[string][]byte) (
	[]byte, error) {
	cname := string(parameters["contract"])
	count, _ := strconv.Atoi(string(parameters["count"]))
	key := parameters["key"]
	resultType := string(parameters["result"])

	startTime := time.Now()
	iter, err := txSimContext.GetHistoryIterForKey(cname, key)
	if err != nil {
		return nil, fmt.Errorf("query contract[%s] by key[%s] get error: %s", cname, string(key), err)
	}
	defer iter.Release()
	i := 0
	result := ""
	for iter.Next() {
		kv, _ := iter.Value()
		if resultType == "txid" {
			result += string(kv.TxId) + ";"
		} else if resultType == "all" {
			result += kv.String() + ";"
		}
		i++
		if i >= count {
			break
		}
	}
	return []byte(fmt.Sprintf("history query result=%s, spend time:%v", result, time.Since(startTime))), nil
}
