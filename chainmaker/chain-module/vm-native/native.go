/*
 * Copyright (C) BABEC. All rights reserved.
 * Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

package native

import (
	"chainmaker.org/chainmaker/vm-native/v2/testcontract"
	"encoding/hex"
	"fmt"
	"sync"

	"chainmaker.org/chainmaker/pb-go/v2/config"
	"chainmaker.org/chainmaker/vm-native/v2/v230/chainconfigmgr2310"
	"github.com/gogo/protobuf/proto"

	"chainmaker.org/chainmaker/common/v2/msgbus"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/syscontract"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/utils/v2"
	"chainmaker.org/chainmaker/utils/v2/cache"
	gasutils "chainmaker.org/chainmaker/utils/v2/gas"
	"chainmaker.org/chainmaker/vm-native/v2/accountmgr"
	"chainmaker.org/chainmaker/vm-native/v2/blockcontract"
	"chainmaker.org/chainmaker/vm-native/v2/certmgr"
	"chainmaker.org/chainmaker/vm-native/v2/chainconfigmgr"
	"chainmaker.org/chainmaker/vm-native/v2/common"
	"chainmaker.org/chainmaker/vm-native/v2/contractmgr"
	"chainmaker.org/chainmaker/vm-native/v2/dposmgr"
	"chainmaker.org/chainmaker/vm-native/v2/government"
	"chainmaker.org/chainmaker/vm-native/v2/transactionmgr"
	contractmgr210 "chainmaker.org/chainmaker/vm-native/v2/v210/contractmgr"
	dposmgr210 "chainmaker.org/chainmaker/vm-native/v2/v210/dposmgr"
	accountmgr220 "chainmaker.org/chainmaker/vm-native/v2/v220/accountmgr"
	certmgr220 "chainmaker.org/chainmaker/vm-native/v2/v220/certmgr"
	"chainmaker.org/chainmaker/vm-native/v2/v220/chainconfigmgr220"
	"chainmaker.org/chainmaker/vm-native/v2/v220/chainconfigmgr224"
	contractmgr220 "chainmaker.org/chainmaker/vm-native/v2/v220/contractmgr"
)

const (
	blockVersion210        = uint32(20)
	blockVersion220        = uint32(220)
	blockVersion2300       = uint32(2300)
	blockVersion2312       = uint32(2030102)
	contractName210Suffix  = "_210"
	contractName220Suffix  = "_220"
	contractName224Suffix  = "_224"
	contractName2310Suffix = "_2310"
)

var (
	nativeLock     = &sync.RWMutex{}
	nativeInstance = make(map[string]*RuntimeInstance) // singleton map[chainId]instance
)

// RuntimeInstance native contract runtime instance
type RuntimeInstance struct {
	// contracts map[contractName]Contract
	contracts  map[string]common.Contract
	log        protocol.Logger
	defaultGas uint64
	gasConfig  *gasutils.GasConfig
	msgBus     msgbus.MessageBus
}

// OnMessage receive msg bus message
func (r *RuntimeInstance) OnMessage(msg *msgbus.Message) {
	switch msg.Topic {
	case msgbus.BlacklistTxIdAdd:
		data, _ := msg.Payload.([]string)
		// prefix + chainId
		bl := cache.NewCacheList(utils.NativePrefix + data[0])
		d := data[1:]
		for _, val := range d {
			bl.Put(val)
			r.log.Infof("add blacklist %s", val)
		}
	case msgbus.BlacklistTxIdDel:
		data, _ := msg.Payload.([]string)
		bl := cache.NewCacheList(utils.NativePrefix + data[0])
		d := data[1:]
		for _, val := range d {
			bl.Delete(val)
			r.log.Infof("del blacklist %s", val)
		}
	case msgbus.ChainConfig:
		dataStr, ok := msg.Payload.([]string)
		if !ok {
			return
		}
		dataBytes, err := hex.DecodeString(dataStr[0])
		if err != nil {
			r.log.Warn(err)
			return
		}
		chainConfig := &config.ChainConfig{}
		err = proto.Unmarshal(dataBytes, chainConfig)
		if err != nil {
			r.log.Warn(err)
			return
		}
		gasConfig := gasutils.NewGasConfig(chainConfig.AccountConfig)
		r.defaultGas = gasConfig.GetBaseGasForInvoke()
		r.gasConfig = gasConfig
		r.log.Infof("[native.RuntimeInstance] receive msg, topic: %s, new GasConfig = %v",
			msg.Topic.String(), r.gasConfig)
	}
}

// OnQuit quit process
func (r *RuntimeInstance) OnQuit() {
	r.log.Infof("quit subscriber success")
}

// InitInstance get singleton RuntimeInstance
func InitInstance(chainId string,
	gasConfig *gasutils.GasConfig,
	log protocol.Logger,
	msgBus msgbus.MessageBus,
	store protocol.BlockchainStore) {

	nativeLock.Lock()
	defer nativeLock.Unlock()

	_, ok := nativeInstance[chainId]
	if ok {
		return
	}

	instance := &RuntimeInstance{
		log:       log,
		contracts: initContract(log),
	}
	nativeInstance[chainId] = instance

	// 注册事件
	instance.msgBus = msgBus

	msgBus.Register(msgbus.BlacklistTxIdDel, instance)
	msgBus.Register(msgbus.BlacklistTxIdAdd, instance)
	initBlacklistTxIdsCache(chainId, store, log)

	instance.gasConfig = gasConfig
	if gasConfig != nil {
		instance.defaultGas = gasConfig.GetBaseGasForInvoke()
	} else {
		instance.defaultGas = uint64(0)
	}
}

func initBlacklistTxIdsCache(chainId string, store protocol.BlockchainStore, log protocol.Logger) {
	log.Infof("init txIds blacklist cache from db start.")
	key := transactionmgr.KeyPrefix
	limitLast := key[len(key)-1] + 1
	limit := key[:len(key)-1] + string(limitLast)

	iter, err := store.SelectObject(syscontract.SystemContract_TRANSACTION_MANAGER.String(), []byte(key), []byte(limit))
	if err != nil {
		log.Errorf("init cache failed. %s", err.Error())
		return
	}
	defer iter.Release()

	bl := cache.NewCacheList(utils.NativePrefix + chainId)
	count := 0
	startKey := ""
	endKey := ""

	preLen := len(transactionmgr.KeyPrefix)
	for iter.Next() {
		kv, err := iter.Value()
		if err != nil {
			log.Errorf("init cache failed. %s", err.Error())
			return
		}
		if len(startKey) == 0 {
			startKey = string(kv.Key)
		}
		endKey = string(kv.Key)
		bl.Put(endKey[preLen:])
		count++
	}
	log.Infof("init txIds blacklist cache from db end, from[%s] end[%s] count[%d]", startKey, endKey, count)
}

// GetRuntimeInstance get singleton RuntimeInstance
func GetRuntimeInstance(chainId string) *RuntimeInstance {
	nativeLock.RLock()
	defer nativeLock.RUnlock()
	return nativeInstance[chainId]
}

func initContract(log protocol.Logger) map[string]common.Contract {

	contracts := make(map[string]common.Contract, 64)
	contracts[syscontract.SystemContract_CHAIN_CONFIG.String()] = chainconfigmgr.NewChainConfigContract(log)
	contracts[syscontract.SystemContract_CHAIN_QUERY.String()] = blockcontract.NewBlockContract(log)
	contracts[syscontract.SystemContract_CERT_MANAGE.String()] = certmgr.NewCertManageContract(log)
	contracts[syscontract.SystemContract_GOVERNANCE.String()] = government.NewGovernmentContract(log)
	contracts[syscontract.SystemContract_DPOS_ERC20.String()] = dposmgr.NewDPoSERC20Contract(log)
	contracts[syscontract.SystemContract_DPOS_STAKE.String()] = dposmgr.NewDPoSStakeContract(log)
	contracts[syscontract.SystemContract_CONTRACT_MANAGE.String()] = contractmgr.NewContractManager(log)
	//contracts[syscontract.SystemContract_PUBKEY_MANAGE.String()] = pubkeymgr.NewPubkeyManageContract(log)
	contracts[syscontract.SystemContract_ACCOUNT_MANAGER.String()] = accountmgr.NewAccountManager(log)
	contracts[syscontract.SystemContract_T.String()] = testcontract.NewManager(log)
	contracts[syscontract.SystemContract_TRANSACTION_MANAGER.String()] = transactionmgr.NewTransactionMgrContract(log)

	// history version v2.1.0
	var (
		erc20210          = syscontract.SystemContract_DPOS_ERC20.String() + contractName210Suffix
		stake210          = syscontract.SystemContract_DPOS_STAKE.String() + contractName210Suffix
		contractManage210 = syscontract.SystemContract_CONTRACT_MANAGE.String() + contractName210Suffix
	)
	contracts[erc20210] = dposmgr210.NewDPoSERC20Contract(log)
	contracts[stake210] = dposmgr210.NewDPoSStakeContract(log)
	contracts[contractManage210] = contractmgr210.NewContractManager(log)

	// history version v2.2.0
	var (
		chainconf220  = syscontract.SystemContract_CHAIN_CONFIG.String() + contractName220Suffix
		chainconf224  = syscontract.SystemContract_CHAIN_CONFIG.String() + contractName224Suffix
		certManage220 = syscontract.SystemContract_CERT_MANAGE.String() + contractName220Suffix

		contractManage220 = syscontract.SystemContract_CONTRACT_MANAGE.String() + contractName220Suffix
		accountManager220 = syscontract.SystemContract_ACCOUNT_MANAGER.String() + contractName220Suffix
	)

	contracts[chainconf220] = chainconfigmgr220.NewChainConfigContract(log)
	contracts[chainconf224] = chainconfigmgr224.NewChainConfigContract(log)
	contracts[certManage220] = certmgr220.NewCertManageContract(log)
	//contracts[pubkeyManage220] = pubkeymgr220.NewPubkeyManageContract(log)
	contracts[contractManage220] = contractmgr220.NewContractManager(log)
	contracts[accountManager220] = accountmgr220.NewAccountManager(log)

	// history version v2.3.1.0
	var (
		chainconf2310 = syscontract.SystemContract_CHAIN_CONFIG.String() + contractName2310Suffix
	)
	contracts[chainconf2310] = chainconfigmgr2310.NewChainConfigContract(log)

	return contracts
}

// Invoke verify and run Contract method
func (r *RuntimeInstance) Invoke(contract *commonPb.Contract, methodName string, _ []byte, parameters map[string][]byte,
	txContext protocol.TxSimContext) *commonPb.ContractResult {

	result := &commonPb.ContractResult{
		Code:    uint32(1),
		Message: "contract internal error",
		Result:  nil,
	}

	blockVersion := txContext.GetBlockVersion()
	// get native func
	f, err := r.getContractFunc(contract, methodName, blockVersion)
	if err != nil {
		r.log.Warnf("the method `%s` in contract `%s` err = %v", methodName, contract.Name, err)
		result.Message = err.Error()
		return result
	}

	// verification is only required before version 220
	if blockVersion210 <= blockVersion && blockVersion < blockVersion220 { // [210, 220)
		if err := r.verify210(contract.Version, txContext); err != nil {
			result.Code = 1
			result.Message = "Access Denied"
			result.Result = nil
			return result
		}
	}

	// invoke native func
	result = f(txContext, parameters)

	return result
}

func (r *RuntimeInstance) getContractFunc(contract *commonPb.Contract, methodName string, blockVersion uint32) (
	f common.ContractFunc, err error) {
	var (
		contractInst common.Contract
		contractName = contract.Name
	)

	if blockVersion210 <= blockVersion && blockVersion < blockVersion220 { // [210, 220)
		if useHistoryContract210(contractName) {
			contractName = contractName + contractName210Suffix
		}
	} else if blockVersion220 <= blockVersion && blockVersion < blockVersion2300 { // [220, 2300)
		suffix := contractName220Suffix
		if useHistoryContract220(contractName) {
			//对于ChainConfig有点特殊，因为224版本也做了比较大的调整，所以特殊再判断了一个224版
			if contractName == syscontract.SystemContract_CHAIN_CONFIG.String() && blockVersion >= 2240 {
				suffix = contractName224Suffix
			}
			contractName = contractName + suffix

		}
	} else if blockVersion2300 <= blockVersion && blockVersion < blockVersion2312 {
		suffix := contractName2310Suffix
		if useHistoryContract2310(contractName) {
			contractName = contractName + suffix
		}
	}

	contractInst = r.contracts[contractName]
	if contractInst == nil {
		return nil, common.ErrContractNotFound
	}

	f = contractInst.GetMethod(methodName)
	if f == nil {
		return nil, common.ErrMethodNotFound
	}

	return f, nil
}

func useHistoryContract210(contractName string) bool {
	switch contractName {
	case syscontract.SystemContract_DPOS_ERC20.String():
		return true
	case syscontract.SystemContract_DPOS_STAKE.String():
		return true
	case syscontract.SystemContract_MULTI_SIGN.String():
		return true
	case syscontract.SystemContract_CONTRACT_MANAGE.String():
		return true
	default:
		return false
	}
}
func useHistoryContract220(contractName string) bool {
	switch contractName {
	case syscontract.SystemContract_CERT_MANAGE.String():
		return true
	case syscontract.SystemContract_CHAIN_CONFIG.String():
		return true
	case syscontract.SystemContract_MULTI_SIGN.String():
		return true
	case syscontract.SystemContract_PUBKEY_MANAGE.String():
		return true
	case syscontract.SystemContract_CONTRACT_MANAGE.String():
		return true
	case syscontract.SystemContract_ACCOUNT_MANAGER.String():
		return true
	default:
		return false
	}
}

func useHistoryContract2310(contractName string) bool {
	switch contractName {
	case syscontract.SystemContract_CHAIN_CONFIG.String():
		return true
	default:
		return false
	}
}

func (r *RuntimeInstance) verify210(version string, txContext protocol.TxSimContext) error {
	// verification
	var verifyAccessFunc common.ContractFunc
	verifyAccessContract := &commonPb.Contract{
		Name:        syscontract.SystemContract_CONTRACT_MANAGE.String(),
		Version:     version,
		RuntimeType: commonPb.RuntimeType_NATIVE,
		Status:      commonPb.ContractStatus_NORMAL,
		Creator:     nil,
	}
	verifyMethodName := "VERIFY_CONTRACT_ACCESS"
	verifyAccessFunc, err := r.getContractFunc(verifyAccessContract, verifyMethodName, blockVersion210)
	if err != nil {
		return err
	}

	accessResult := verifyAccessFunc(txContext, nil)
	if string(accessResult.Result) != "true" { //无权访问
		return fmt.Errorf("access denied")
	}
	return nil
}
