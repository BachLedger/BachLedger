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

func TestAES(t *testing.T) {
	sk, err := NewAESKey(p11, internalAESKeyId)
	assert.NoError(t, err)

	cipherText, err := sk.Encrypt(plain)
	assert.NoError(t, err)
	assert.NotNil(t, cipherText)

	plainText, err := sk.Decrypt(cipherText)
	assert.NoError(t, err)
	assert.NotNil(t, plainText)
	assert.Equal(t, plain, plainText)
}

//func TestNewSecretKey_AES(t *testing.T) {
//	sk, err := NewSecretKey(p11, string(internalAESKeyId), bccrypto.AES)
//	assert.NoError(t, err)
//	assert.NotNil(t, sk)
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
//
//func TestGenerateSecretKey_AES_16(t *testing.T) {
//	keyLabel := fmt.Sprintf("%d", incNextId())
//	sk, err := GenSecretKey(p11, keyLabel, bccrypto.AES, 16)
//	assert.NoError(t, err)
//	assert.NotNil(t, sk)
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
//
//func TestGenerateSecretKey_AES_24(t *testing.T) {
//	keyLabel := fmt.Sprintf("%d", incNextId())
//	sk, err := GenSecretKey(p11, keyLabel, bccrypto.AES, 24)
//	assert.NoError(t, err)
//	assert.NotNil(t, sk)
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
//
//func TestGenerateSecretKey_AES_32(t *testing.T) {
//	keyLabel := fmt.Sprintf("%d", incNextId())
//	sk, err := GenSecretKey(p11, keyLabel, bccrypto.AES, 32)
//	assert.NoError(t, err)
//	assert.NotNil(t, sk)
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
//
//// TestGetAESKeySize test get aes key size from pkcs11, todo
//func TestGetAESKeySize(t *testing.T) {
//	keyLen := 16
//	keyLabel := "TestAESKeySize"
//	keyTemplate := []*pkcs11.Attribute{
//		pkcs11.NewAttribute(pkcs11.CKA_TOKEN, false),
//		pkcs11.NewAttribute(pkcs11.CKA_PRIVATE, true),
//		pkcs11.NewAttribute(pkcs11.CKA_ENCRYPT, true),
//		pkcs11.NewAttribute(pkcs11.CKA_LABEL, keyLabel),
//		pkcs11.NewAttribute(pkcs11.CKA_VALUE_LEN, keyLen),
//	}
//
//	mech := pkcs11.NewMechanism(pkcs11.CKM_AES_KEY_GEN, nil)
//
//	sk, err := p11.GenerateKey(mech, keyTemplate)
//	assert.NoError(t, err)
//	assert.NotNil(t, sk)
//
//	keySize, err := p11.getSecretKeySize(*sk)
//	assert.NoError(t, err)
//	assert.Equal(t, keyLen, keySize)
//}
