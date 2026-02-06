/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package hsm

import (
	"fmt"
	"os"
	"runtime"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestLoad(t *testing.T) {
	skipTest(t)

	adapter, err := Load("./swxa/hsm_adapter.so")
	assert.NoError(t, err)

	ckm := adapter.PKCS11_GetSM3SM2CKM()
	fmt.Printf("ckm_sm3_sm2 = %0x\n", ckm)
	pubId, _ := adapter.PKCS11_GetSM2KeyId(1, true)
	priId, _ := adapter.PKCS11_GetSM2KeyId(1, false)
	fmt.Printf("pubId = %s, priId = %s\n", pubId, priId)

	keyIdx, need := adapter.SDF_GetSM2KeyAccessRight(1)
	fmt.Printf("access right, keyIdx = %d, need = %t\n", keyIdx, need)
}

func TestGetHSMAdapter(t *testing.T) {
	skipTest(t)

	os.Setenv("HSM_ADAPTER_LIB", "./swxa/hsm_adapter.so")
	adapter := GetHSMAdapter("")
	assert.NotNil(t, adapter)
	fmt.Printf("ckm_sm3_sm2 = %0x\n", adapter.PKCS11_GetSM3SM2CKM())
	os.Unsetenv("HSM_ADAPTER_LIB")

	adapter = GetHSMAdapter("./demo/demo.so")
	assert.NotNil(t, adapter)
	fmt.Printf("ckm_sm3_sm2 = %0x\n", adapter.PKCS11_GetSM3SM2CKM())
}

func skipTest(t *testing.T) {
	if runtime.GOOS != "darwin" || runtime.GOARCH != "amd64" {
		t.Skip("hsm_adapter.so is compiled in darwin platform, skip this test")
	}
}
