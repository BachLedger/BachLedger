/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package pkcs11

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestSM4(t *testing.T) {
	if isSoftHSM {
		t.Skipf("skip: softhsm not supported sm4")
	}

	sk, err := NewSM4Key(p11, internalSM4KeyId)
	assert.NoError(t, err)

	cipherText, err := sk.Encrypt(plain)
	assert.NoError(t, err)
	assert.NotNil(t, cipherText)

	plainText, err := sk.Decrypt(cipherText)
	assert.NoError(t, err)
	assert.NotNil(t, plainText)
	assert.Equal(t, plain, plainText)
}

//func TestGenerateSecretKey_SM4(t *testing.T) {
//	if !support_GM {
//		t.Skipf("skip: softhsm not supported sm4")
//	}
//	keyLabel := fmt.Sprintf("%d", incNextId())
//	sk, err := GenSecretKey(p11, keyLabel, bccrypto.SM4, 16)
//
//	cipherText, err := sk.Encrypt(plain)
//	assert.NoError(t, err)
//	assert.NotNil(t, cipherText)
//
//	plainText, err := sk.Decrypt(cipherText)
//	assert.NoError(t, err)
//	assert.NotNil(t, plainText)
//	assert.Equal(t, plain, plainText)
//}
