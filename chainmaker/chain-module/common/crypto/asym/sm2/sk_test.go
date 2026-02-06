/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package sm2

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/pem"
	"fmt"
	"os"
	"testing"

	"chainmaker.org/chainmaker/common/v2/crypto"
	"github.com/stretchr/testify/require"
	tjx509 "github.com/tjfoc/gmsm/x509"
)

var msg = "js"
var plain = `"But I'm not ready..."
"We never are. Don't be afraid. Che la morte non sia crudele. Requiescat in pace."
"Nothing is true. Everything is permitted. Work in the dark, we serve the light."
"Valar morgulis."
"Valar dohaelis."
`

var (
	cMsg      = "Valar morgulis!"
	cipherHex = "3078022100e5e9275c619c6ef7bfe8c269616c351da8f645fd7d2eac82ed38c4fedc682562022043a04d8772dfd1c24ab1dd96ee4a74e44eed7ee55fecefb519228760a656520c0420a74dab08012984bcceca88d331d5e9d6b5b01f2c92733d10a12a1d596cca4a98040fc08a9e644b21d7cb51619f8d46872e"
	sigHex    = "3046022100b490cde789a44fd9ad5d64a0aef96f7a1da8785915fd9fecf35619ca98b6c81b022100f03b22e1fe614d9ba310544f4398a3c049f8fdddcf6ded4926769a7bf3667964"

	pkPEM = `-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoEcz1UBgi0DQgAEQIgYu+cpHDd3Z9XWuJHdP3ERYEHk
FkBoJWR8oRqsqogMBiYp6uZhKjWfrrv1xdlCywTA/jeXy5NhECg7zsQRtg==
-----END PUBLIC KEY-----`
	skPEM = `-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqBHM9VAYItBG0wawIBAQQgX0BughCBX1R5hUfK
Jele6M8DzmlrpZ9kaWQ3y1nSW1ihRANCAARAiBi75ykcN3dn1da4kd0/cRFgQeQW
QGglZHyhGqyqiAwGJinq5mEqNZ+uu/XF2ULLBMD+N5fLk2EQKDvOxBG2
-----END PRIVATE KEY-----`
)

func TestSM2(t *testing.T) {
	h := sha256.Sum256([]byte(msg))
	priv, err := New(crypto.SM2)
	require.Nil(t, err)

	privDER, err := priv.Bytes()
	require.Nil(t, err)

	block := &pem.Block{
		Type:  "EC PRIVATE KEY",
		Bytes: privDER,
	}

	err = pem.Encode(os.Stdout, block)
	require.Nil(t, err)

	buf, err := priv.String()
	require.Nil(t, err)
	fmt.Println(buf)

	sign, err := priv.Sign(h[:])
	require.Nil(t, err)
	require.NotEqual(t, nil, sign)
	pub := priv.PublicKey()
	buf, err = pub.String()
	require.Nil(t, err)
	fmt.Println(buf)

	b, err := pub.Verify(h[:], sign)
	require.Nil(t, err)
	require.True(t, b)

	ek, ok := pub.(crypto.EncryptKey)
	require.True(t, ok)
	dk, ok := priv.(crypto.DecryptKey)
	require.True(t, ok)
	ciphertext, err := ek.Encrypt([]byte(plain))
	require.Nil(t, err)
	decrypted, err := dk.Decrypt(ciphertext)
	require.Nil(t, err)
	require.True(t, bytes.Equal([]byte(plain), decrypted))

	fmt.Printf("plaintext: \n%s\n", plain)
	fmt.Printf("decrypted: \n%s\n", string(decrypted))

	fmt.Printf("plaintext length: %d\nciphertext length: %d\ndecrypted length: %d\n", len(plain), len(ciphertext), len(decrypted))

	cipherNonASN1, err := ek.EncryptWithOpts([]byte(plain), &crypto.EncOpts{EnableASN1: false})
	require.Nil(t, err)
	decryptedNonASN1, err := dk.DecryptWithOpts(cipherNonASN1, &crypto.EncOpts{EnableASN1: false})
	require.Nil(t, err)
	require.True(t, bytes.Equal([]byte(plain), decryptedNonASN1))

	fmt.Printf("decrypted no ASN1: \n%s\n", string(decryptedNonASN1))

	fmt.Printf("plaintext length: %d\nciphertext length: %d\ndecrypted length: %d\n", len(plain), len(ciphertext), len(decryptedNonASN1))

	pkBlock, _ := pem.Decode([]byte(pkPEM))
	skBlock, _ := pem.Decode([]byte(skPEM))
	pkFromC, err := tjx509.ParseSm2PublicKey(pkBlock.Bytes)
	require.Nil(t, err)
	skFromC, err := tjx509.ParsePKCS8UnecryptedPrivateKey(skBlock.Bytes)
	require.Nil(t, err)

	pkWrapped := &PublicKey{K: pkFromC}
	skWrapped := &PrivateKey{K: skFromC}

	cipherPlain, err := hex.DecodeString(cipherHex)
	require.Nil(t, err)
	plainFromC, err := skWrapped.Decrypt(cipherPlain)
	require.True(t, bytes.Equal(plainFromC, []byte(cMsg)))
	println("decrypted from C: " + string(plainFromC))

	sigPlain, err := hex.DecodeString(sigHex)
	require.Nil(t, err)
	isValid, err := pkWrapped.VerifyWithOpts([]byte(cMsg), sigPlain, &crypto.SignOpts{Hash: crypto.HASH_TYPE_SM3})
	require.Nil(t, err)
	require.True(t, isValid)
}
