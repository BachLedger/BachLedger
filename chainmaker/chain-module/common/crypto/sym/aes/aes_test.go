/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package aes

import (
	"encoding/asn1"
	"encoding/hex"
	"fmt"

	"chainmaker.org/chainmaker/common/v2/crypto"
	"chainmaker.org/chainmaker/common/v2/crypto/sym/modes"

	"crypto/rand"
	"testing"

	"github.com/stretchr/testify/require"
)

const msg = "js"

var optCBCPlain = &crypto.EncOpts{
	EncodingType: modes.PADDING_PKCS5,
	BlockMode:    modes.BLOCK_MODE_CBC,
	EnableMAC:    false,
	Hash:         0,
	Label:        nil,
	EnableASN1:   false,
}

var optCBCASN1 = &crypto.EncOpts{
	EncodingType: modes.PADDING_PKCS5,
	BlockMode:    modes.BLOCK_MODE_CBC,
	EnableMAC:    false,
	Hash:         0,
	Label:        nil,
	EnableASN1:   true,
}

var optGCMPlain = &crypto.EncOpts{
	EncodingType: modes.PADDING_NONE,
	BlockMode:    modes.BLOCK_MODE_GCM,
	EnableMAC:    true,
	Hash:         0,
	Label:        nil,
	EnableASN1:   false,
}

var optGCMASN1 = &crypto.EncOpts{
	EncodingType: modes.PADDING_NONE,
	BlockMode:    modes.BLOCK_MODE_GCM,
	EnableMAC:    true,
	Hash:         0,
	Label:        nil,
	EnableASN1:   true,
}

func TestAES(t *testing.T) {
	key := make([]byte, 32)
	_, err := rand.Read(key)
	require.Nil(t, err)

	aes := AESKey{Key: key}

	crypt, err := aes.Encrypt([]byte(msg))
	require.Nil(t, err)

	fmt.Println("crypt data:", crypt)

	decrypted, err := aes.Decrypt(crypt)
	require.Nil(t, err)

	require.Equal(t, string(decrypted), msg)

	var ciphertext aesCiphertext
	_, err = asn1.Unmarshal(crypt, &ciphertext)
	require.Nil(t, err)

	fmt.Printf("key: \t\t%s\n", hex.EncodeToString(key))
	fmt.Printf("iv: \t\t%s\n", hex.EncodeToString(ciphertext.IV))
	fmt.Printf("tag: \t\t%s\n", hex.EncodeToString(ciphertext.Tag))
	fmt.Printf("cipher: \t%s\n", hex.EncodeToString(ciphertext.Ciphertext))
	fmt.Printf("whole: \t\t%s\n", hex.EncodeToString(crypt))

	crypt, err = aes.EncryptWithOpts([]byte(msg), optCBCASN1)
	require.Nil(t, err)
	decrypted, err = aes.DecryptWithOpts(crypt, optCBCASN1)
	require.Nil(t, err)
	require.Equal(t, string(decrypted), msg)

	crypt, err = aes.EncryptWithOpts([]byte(msg), optCBCPlain)
	require.Nil(t, err)
	decrypted, err = aes.DecryptWithOpts(crypt, optCBCPlain)
	require.Nil(t, err)
	require.Equal(t, string(decrypted), msg)

	crypt, err = aes.EncryptWithOpts([]byte(msg), optGCMASN1)
	require.Nil(t, err)
	decrypted, err = aes.DecryptWithOpts(crypt, optGCMASN1)
	require.Nil(t, err)
	require.Equal(t, string(decrypted), msg)

	crypt, err = aes.EncryptWithOpts([]byte(msg), optGCMPlain)
	require.Nil(t, err)
	decrypted, err = aes.DecryptWithOpts(crypt, optGCMPlain)
	require.Nil(t, err)
	require.Equal(t, string(decrypted), msg)
}
