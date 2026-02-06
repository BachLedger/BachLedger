/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package asym

import (
	"crypto/sha256"
	"fmt"
	"testing"

	"chainmaker.org/chainmaker/common/v2/crypto/engine"

	"chainmaker.org/chainmaker/common/v2/crypto"
	"github.com/stretchr/testify/require"
)

func TestGenerateKeyPairPEM(t *testing.T) {
	sk, pk, err := GenerateKeyPairPEM(crypto.SM2)
	require.Nil(t, err)
	fmt.Println("sk: ", sk)
	fmt.Println("pk: ", pk)

	sk, pk, err = GenerateKeyPairPEM(crypto.RSA2048)
	require.Nil(t, err)
	fmt.Println("sk: ", sk)
	fmt.Println("pk: ", pk)
}

func TestSignAndVerifyPass(t *testing.T) {
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
		engine.InitCryptoEngine(testCase.engine, testCase.isTls)
		testSignAndVerify(t, crypto.ECC_NISTP256)
		testSignAndVerify(t, crypto.ECC_NISTP384)
		testSignAndVerify(t, crypto.ECC_NISTP521)
		testSignAndVerify(t, crypto.SM2)
		testSignAndVerify(t, crypto.RSA2048)
		testSignAndVerify(t, crypto.RSA1024)
		testSignAndVerify(t, crypto.RSA512)
		testSignAndVerify(t, crypto.ECC_Secp256k1)
	}
}

func testSignAndVerify(t *testing.T, opt crypto.KeyType) {
	digest := sha256.Sum256([]byte("js"))

	// 方式1：
	sk, pk, err := GenerateKeyPairPEM(opt)
	require.Nil(t, err)
	sign, err := Sign(sk, digest[:])
	require.Nil(t, err)
	ok, err := Verify(pk, digest[:], sign)
	require.Nil(t, err)
	require.Equal(t, true, ok)

	// 方式2：
	sk2, pk2, err := GenerateKeyPairBytes(opt)
	require.Nil(t, err)
	sign2, err := Sign(sk2, digest[:])
	require.Nil(t, err)
	ok, err = Verify(pk2, digest[:], sign2)
	require.Nil(t, err)
	require.Equal(t, true, ok)

	// 方式3：
	sk3, err := GenerateKeyPair(opt)
	require.Nil(t, err)
	sign3, err := sk3.Sign(digest[:])
	require.Nil(t, err)
	ok, err = sk3.PublicKey().Verify(digest[:], sign3)
	require.Nil(t, err)
	require.Equal(t, true, ok)

	// 方式4:
	sk4, err := PrivateKeyFromPEM([]byte(sk), nil)
	require.Nil(t, err)
	pk4, err := PublicKeyFromPEM([]byte(pk))
	require.Nil(t, err)

	sig4, err := sk4.Sign(digest[:])
	require.Nil(t, err)
	ok, err = pk4.Verify(digest[:], sig4)
	require.Nil(t, err)
	require.Equal(t, true, ok)

	// Cross check:
	ok, err = pk4.Verify(digest[:], sign)
	require.Nil(t, err)
	require.Equal(t, true, ok)

	ok, err = Verify(pk, digest[:], sig4)
	require.Nil(t, err)
	require.Equal(t, true, ok)

	testSignAndVerifyWithOpts(t, opt)
}

func testSignAndVerifyWithOpts(t *testing.T, opt crypto.KeyType) {
	digest := sha256.Sum256([]byte("js"))

	optSHA256 := &crypto.SignOpts{
		Hash: crypto.HASH_TYPE_SHA256,
		UID:  "",
	}
	optSM3 := &crypto.SignOpts{
		Hash: crypto.HASH_TYPE_SM3,
		UID:  crypto.CRYPTO_DEFAULT_UID,
	}

	// 方式1：
	skPEM, pkPEM, err := GenerateKeyPairPEM(opt)
	require.Nil(t, err)
	sk1, err := PrivateKeyFromPEM([]byte(skPEM), nil)
	require.Nil(t, err)
	pk1, err := PublicKeyFromPEM([]byte(pkPEM))
	require.Nil(t, err)

	sig1, err := sk1.SignWithOpts(digest[:], optSHA256)
	require.Nil(t, err)
	ok, err := pk1.VerifyWithOpts(digest[:], sig1, optSHA256)
	require.Nil(t, err)
	require.Equal(t, true, ok)

	sig2, err := sk1.SignWithOpts(digest[:], optSM3)
	require.Nil(t, err)
	ok, err = pk1.VerifyWithOpts(digest[:], sig2, optSM3)
	require.Nil(t, err)
	require.Equal(t, true, ok)

	// 方式2：
	sk3, err := GenerateKeyPair(opt)
	require.Nil(t, err)
	sign3, err := sk3.SignWithOpts(digest[:], optSHA256)
	require.Nil(t, err)
	ok, err = sk3.PublicKey().VerifyWithOpts(digest[:], sign3, optSHA256)
	require.Nil(t, err)
	require.Equal(t, true, ok)
	sign4, err := sk3.SignWithOpts(digest[:], optSM3)
	require.Nil(t, err)
	ok, err = sk3.PublicKey().VerifyWithOpts(digest[:], sign4, optSM3)
	require.Nil(t, err)
	require.Equal(t, true, ok)
}

/*
func TestWriteFile(t *testing.T) {
	err := WriteFile(crypto.RSA2048, "/home/jason/Work/ChainMaker/chainmaker-go/config/dev/certs/node1")
	require.Nil(t, err)
	err = WriteFile(crypto.RSA2048, "/home/jason/Work/ChainMaker/chainmaker-go/config/dev/certs/node2")
	require.Nil(t, err)
	err = WriteFile(crypto.RSA2048, "/home/jason/Work/ChainMaker/chainmaker-go/config/dev/certs/node3")
	require.Nil(t, err)
	err = WriteFile(crypto.RSA2048, "/home/jason/Work/ChainMaker/chainmaker-go/config/dev/certs/node4")
	require.Nil(t, err)
}
*/
