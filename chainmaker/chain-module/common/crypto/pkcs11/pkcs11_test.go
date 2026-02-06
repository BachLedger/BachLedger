/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package pkcs11

import (
	"fmt"
	"os"
	"runtime"
	"testing"

	bccrypto "chainmaker.org/chainmaker/common/v2/crypto"

	"github.com/stretchr/testify/assert"
)

var (
	isSoftHSM        = true
	plain            = []byte("chainmaker")
	internalSM2KeyId = []byte("1")
	internalRSAKeyId = []byte("1")
	internalAESKeyId = []byte("1")
	internalSM4KeyId = []byte("1")
	internalECCKeyId = []byte("1")
)

var (
	//zayk pkcs11 settings
	lib      = "./libupkcs11.so"
	label    = "test"
	password = "12345678"

	sessionCacheSize = 10
	hashStr          = "SHA1"
)

var (
	p11 *P11Handle
)

func TestMain(m *testing.M) {
	//set lib path
	if isSoftHSM {
		if runtime.GOOS == "darwin" {
			lib = "/usr/local/Cellar/softhsm/2.6.1/lib/softhsm/libsofthsm2.so"
			if runtime.GOARCH == "arm64" {
				lib = "/opt/homebrew/Cellar/softhsm/2.6.1/lib/softhsm/libsofthsm2.so"
			}
		} else if runtime.GOOS == "linux" {
			lib = "/usr/lib64/libsofthsm2.so"
		}
		label = "test"
		password = "1234"
		os.Unsetenv("HSM_ADAPTER_LIB")
	}
	var err error
	p11, err = New(lib, label, password, sessionCacheSize, hashStr)
	if err != nil || p11 == nil {
		fmt.Printf("Init pkcs11 handle fail, err = %s\n", err)
		os.Exit(1)
	}
	if err := genTestKeys(); err != nil {
		fmt.Printf("Init test keys fail, err = %s\n", err)
		os.Exit(1)
	}
	os.Exit(m.Run())
}

func TestFindSlotLabel(t *testing.T) {
	labels, err := listSlot(p11.ctx)
	assert.NoError(t, err)
	fmt.Printf("%v\n", labels)
}

func genTestKeys() error {
	if isSoftHSM {
		if _, err := GenKeyPair(p11, string(internalRSAKeyId), bccrypto.RSA1024, &GenOpts{KeyBits: 1024}); err != nil {
			return err
		}
		if _, err := GenSecretKey(p11, string(internalAESKeyId), bccrypto.AES, 16); err != nil {
			return err
		}
		//oidNamedCurveP256, _ := asn1.Marshal(asn1.ObjectIdentifier{1, 2, 840, 10045, 3, 1, 7})
		//if _, err := GenKeyPair(p11, string(internalECCKeyId), bccrypto.ECC_NISTP256, &GenOpts{CurveParams: string(oidNamedCurveP256)}); err != nil {
		//	return err
		//}
		_ = internalECCKeyId
	}
	return nil
}
