/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package main

// 安装合约时会执行此方法，必须
//export init_contract
func initContract() {
	// 此处可写安装合约的初始化逻辑

}

// 升级合约时会执行此方法，必须
//export upgrade
func upgrade() {
	// 此处可写升级合约的逻辑

}

//export crossCallEvmContract
func crossCallEvmContract() {
	ctx := NewSimContext()

	// 获取参数
	name, _ := ctx.ArgString("name")
	method, _ := ctx.ArgString("method")
	calldata, _ := ctx.Arg("calldata")
	params := make(map[string][]byte, 1)
	params["data"] = calldata

	//cross call evm contract
	if result, resultCode := ctx.CallContract(name, method, params); resultCode != SUCCESS {
		// 返回结果
		ctx.ErrorResult("failed to cross call evm contract: " + name)
	} else {
		// 返回结果
		ctx.SuccessResultByte(result)
		// 记录日志
		ctx.Log("cross call evm contract result:" + string(result))
	}
}

func main() {

}
