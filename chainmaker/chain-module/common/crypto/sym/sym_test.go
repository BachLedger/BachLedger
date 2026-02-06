/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package sym

import (
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"testing"

	"chainmaker.org/chainmaker/common/v2/crypto/engine"

	"chainmaker.org/chainmaker/common/v2/crypto"
	"github.com/stretchr/testify/require"
)

const (
	msg    = "js"
	keyHex = "43494f2804a3cf33e96077637e45d211"
)

func TestSymAES(t *testing.T) {
	key := make([]byte, 16)
	_, err := rand.Read(key)
	require.Nil(t, err)

	aes, err := GenerateSymKey(crypto.AES, key)
	require.Nil(t, err)

	crypt, err := aes.Encrypt([]byte(msg))
	require.Nil(t, err)

	printCryptData(crypt)

	decrypt, err := aes.Decrypt(crypt)
	require.Nil(t, err)

	require.Equal(t, string(decrypt), msg)
}

func TestSymSM4(t *testing.T) {
	key := make([]byte, 16)
	_, err := rand.Read(key)
	require.Nil(t, err)

	testCases := []struct {
		engine string
		isTls  bool
	}{
		{"gmssl", true},
		{"gmssl", false},
		{"tencentsm", true},
		{"tencentsm", false},
		{"tjfoc", true},
		{"tjfoc", false},
		{"", true}, //default = tjfoc
		{"", false},
	}
	for _, testCase := range testCases {
		engine.InitCryptoEngine(testCase.engine, false)
		sm4, err := GenerateSymKey(crypto.SM4, key)
		require.Nil(t, err)

		crypt, err := sm4.Encrypt([]byte(msg))
		require.Nil(t, err)

		printCryptData(crypt)

		decrypt, err := sm4.Decrypt(crypt)
		require.Nil(t, err)

		require.Equal(t, string(decrypt), msg)
	}
}

func TestSymAESStr(t *testing.T) {
	aes, err := GenerateSymKeyStr(crypto.AES, keyHex)
	require.Nil(t, err)

	crypt, err := aes.Encrypt([]byte(msg))
	require.Nil(t, err)

	printCryptData(crypt)
	// PeM2ySjw4BCgMLOFxgkSRP1biXqHyF4MYCcTR4GDoNg=

	decrypt, err := aes.Decrypt(crypt)
	require.Nil(t, err)

	require.Equal(t, string(decrypt), msg)
}

func TestSymSM4Str(t *testing.T) {
	testCases := []struct {
		engine string
		isTls  bool
	}{
		{"gmssl", true},
		{"gmssl", false},
		{"tencentsm", true},
		{"tencentsm", false},
		{"tjfoc", true},
		{"tjfoc", false},
		{"", true}, //default = tjfoc
		{"", false},
	}
	for _, testCase := range testCases {
		engine.InitCryptoEngine(testCase.engine, false)
		sm4, err := GenerateSymKeyStr(crypto.SM4, keyHex)
		require.Nil(t, err)

		crypt, err := sm4.Encrypt([]byte(msg))
		require.Nil(t, err)

		printCryptData(crypt)

		decrypt, err := sm4.Decrypt(crypt)
		require.Nil(t, err)

		require.Equal(t, string(decrypt), msg)
	}
}

func printCryptData(crypt []byte) {
	fmt.Println("crypt data:", base64.StdEncoding.EncodeToString(crypt))
}
