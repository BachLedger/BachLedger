/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

	SPDX-License-Identifier: Apache-2.0
*/
package native

import (
	"fmt"
	"strings"
	"testing"

	"chainmaker.org/chainmaker/logger/v2"
	"chainmaker.org/chainmaker/pb-go/v2/syscontract"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/vm-native/v2/common"

	"chainmaker.org/chainmaker/protocol/v2/test"
)

func TestInitContract(t *testing.T) {
	log := &test.GoLogger{}
	/*	contracts := make(map[string]common.Contract, 64)
		contracts[syscontract.SystemContract_T.String()] = testcontract.NewManager(log)*/

	contracts := initContract(log)

	contracts210 := extractVersionedContracts(contracts, contractName210Suffix)
	verifyContracts210(contracts210, t)
	contracts220 := extractVersionedContracts(contracts, contractName220Suffix)
	verifyContracts220(contracts220, t)

	verifyContracts(contracts, t)
}

func extractVersionedContracts(contracts map[string]common.Contract, version string) map[string]common.Contract {
	versionedContracts := make(map[string]common.Contract)
	for name, contract := range contracts {
		if strings.HasSuffix(name, version) {
			versionedContracts[name] = contract
		}
	}

	for name := range versionedContracts {
		delete(contracts, name)
	}

	return versionedContracts
}

func verifyContracts210(contracts map[string]common.Contract, t *testing.T) {
	if len(contracts) != 3 {
		t.Fatalf("version 210 has wrong number of contracts.")
	}
	if _, exists := contracts[syscontract.SystemContract_DPOS_ERC20.String()+contractName210Suffix]; !exists {
		t.Fatalf("DPOS_ERC20 doesn't exists in version 210 contracts.")
	}
	if _, exists := contracts[syscontract.SystemContract_DPOS_STAKE.String()+contractName210Suffix]; !exists {
		t.Fatalf("DPOS_STACK doesn't exists in version 210 contracts.")
	}

	if _, exists := contracts[syscontract.SystemContract_CONTRACT_MANAGE.String()+contractName210Suffix]; !exists {
		t.Fatalf("CONTRACT_MANAGE doesn't exists in version 210 contracts.")
	}
}

func verifyContracts220(contracts map[string]common.Contract, t *testing.T) {
	if len(contracts) != 4 {
		t.Fatalf("version 220 has wrong number of contracts.")
	}
	if _, exists := contracts[syscontract.SystemContract_CHAIN_CONFIG.String()+contractName220Suffix]; !exists {
		t.Fatalf("CHAIN_CONFIG doesn't exists in version 220 contracts.")
	}

	if _, exists := contracts[syscontract.SystemContract_CERT_MANAGE.String()+contractName220Suffix]; !exists {
		t.Fatalf("CERT_MANAGE doesn't exists in version 220 contracts.")
	}

	if _, exists := contracts[syscontract.SystemContract_CONTRACT_MANAGE.String()+contractName220Suffix]; !exists {
		t.Fatalf("CONTRACT_MANAGE doesn't exists in version 220 contracts.")
	}
}

func verifyContracts(contracts map[string]common.Contract, t *testing.T) {
	contract := contracts[syscontract.SystemContract_T.String()]
	nMethod := contract.GetMethod("2")
	if nMethod != nil {
		fmt.Println("调用合约方法 N 出错:", nMethod)
		return
	}

	// 处理返回结果
	fmt.Println("调用合约方法 N 返回结果:", nMethod)
}

func TestConcurrentAccess(t *testing.T) {
	log := logger.GetLogger("native")
	t.Run("group", func(t *testing.T) {
		t.Run("TestAccess1", func(t *testing.T) {
			accessNativeRuntime(log, t)
		})
		t.Run("TestAccess2", func(t *testing.T) {
			accessNativeRuntime(log, t)
		})
		t.Run("TestAccess3", func(t *testing.T) {
			accessNativeRuntime(log, t)
		})
		t.Run("TestAccess4", func(t *testing.T) {
			accessNativeRuntime(log, t)
		})
		t.Run("TestAccess5", func(t *testing.T) {
			accessNativeRuntime(log, t)
		})
		t.Run("TestAccess6", func(t *testing.T) {
			accessNativeRuntime(log, t)
		})
		t.Run("TestAccess7", func(t *testing.T) {
			accessNativeRuntime(log, t)
		})
		t.Run("TestAccess8", func(t *testing.T) {
			accessNativeRuntime(log, t)
		})
		t.Run("TestAccess9", func(t *testing.T) {
			accessNativeRuntime(log, t)
		})
		t.Run("TestAccess10", func(t *testing.T) {
			accessNativeRuntime(log, t)
		})
		t.Run("TestAccess11", func(t *testing.T) {
			accessNativeRuntime(log, t)
		})
		t.Run("TestAccess12", func(t *testing.T) {
			accessNativeRuntime(log, t)
		})
		t.Run("TestAccess13", func(t *testing.T) {
			accessNativeRuntime(log, t)
		})
		t.Run("TestAccess14", func(t *testing.T) {
			accessNativeRuntime(log, t)
		})
		t.Run("TestAccess15", func(t *testing.T) {
			accessNativeRuntime(log, t)
		})
		t.Run("TestAccess16", func(t *testing.T) {
			accessNativeRuntime(log, t)
		})
	})
}

func accessNativeRuntime(log protocol.Logger, t *testing.T) {
	t.Parallel()

	for i := 0; i < 10000; i++ {
		chainId := fmt.Sprintf("chain-%v", i)
		fmt.Println("wwwwwwwwwwwwwwwwwwwwqqqqqqqqqqqqqqqq")
		GetRuntimeInstance(chainId)
		//time.Sleep(time.Millisecond * 1)
	}
}
