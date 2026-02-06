/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package rsa

import (
	"bytes"
	"fmt"
	"testing"

	"chainmaker.org/chainmaker/common/v2/crypto"
	"github.com/stretchr/testify/require"
)

var msg = "js"

func TestRSA(t *testing.T) {

	priv, err := New(crypto.RSA2048)
	require.Nil(t, err)

	_, err = priv.Bytes()
	require.Nil(t, err)

	buf, err := priv.String()
	require.Nil(t, err)
	fmt.Println(buf)

	sign, err := priv.Sign([]byte(msg))
	require.Nil(t, err)
	require.NotEqual(t, nil, sign)
	pub := priv.PublicKey()
	buf, err = pub.String()
	require.Nil(t, err)
	fmt.Println(buf)

	b, err := pub.Verify([]byte(msg), sign)
	require.Nil(t, err)
	require.True(t, b)

	sigPss, err := priv.SignWithOpts([]byte(msg), &crypto.SignOpts{
		Hash:         crypto.HASH_TYPE_SHA256,
		UID:          "",
		EncodingType: RSA_PSS,
	})

	b, err = pub.VerifyWithOpts([]byte(msg), sigPss, &crypto.SignOpts{
		Hash:         crypto.HASH_TYPE_SHA256,
		UID:          "",
		EncodingType: RSA_PSS,
	})
	require.Nil(t, err)
	require.True(t, b)

	decKey, err := NewDecryptionKey(crypto.RSA3072)
	require.Nil(t, err)

	encKey := decKey.EncryptKey()

	cipher, err := encKey.Encrypt([]byte(msg))
	require.Nil(t, err)
	require.NotNil(t, cipher)

	plain, err := decKey.Decrypt(cipher)
	require.Nil(t, err)
	require.NotNil(t, plain)
	require.True(t, bytes.Equal(plain, []byte(msg)))

	println("rsa test done")
}
