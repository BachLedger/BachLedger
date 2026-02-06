/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package sm2

import (
	"crypto"
	"crypto/rand"
	"testing"

	"chainmaker.org/chainmaker/common/v2/opencrypto/utils"

	tjsm2 "github.com/tjfoc/gmsm/sm2"
	tjx509 "github.com/tjfoc/gmsm/x509"

	"github.com/stretchr/testify/assert"
)

var (
	msg = []byte("hello gmssl")
)

func TestSignVerify(t *testing.T) {
	priv, err := GenerateKeyPair()
	assert.NoError(t, err)
	assert.NotNil(t, priv)

	sig, err := priv.Sign(msg)
	assert.NoError(t, err)

	ok, err := priv.Pub.Verify(msg, sig)
	assert.NoError(t, err)
	assert.True(t, ok)
}

func TestEncDec(t *testing.T) {
	priv, err := GenerateKeyPair()
	assert.NoError(t, err)
	assert.NotNil(t, priv)

	c, err := priv.Pub.Encrypt(msg)
	assert.NoError(t, err)

	p, err := priv.Decrypt(c)
	assert.NoError(t, err)
	assert.Equal(t, msg, p)
}

func BenchmarkPrivateKey_Sign(b *testing.B) {
	priv, err := GenerateKeyPair()
	assert.NoError(b, err)
	assert.NotNil(b, priv)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		priv.Sign(msg)
	}
}

func BenchmarkPublicKey_Verify(b *testing.B) {
	priv, err := GenerateKeyPair()
	assert.NoError(b, err)
	assert.NotNil(b, priv)

	sig, err := priv.Sign(msg)
	assert.NoError(b, err)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		priv.Pub.Verify(msg, sig)
	}
}

func TestSigner_Sign(t *testing.T) {
	priv, _ := GenerateKeyPair()
	sig, err := priv.ToStandardKey().(crypto.Signer).Sign(rand.Reader, msg, nil)
	assert.NoError(t, err)

	pass, err := priv.PublicKey().Verify(msg, sig)
	assert.NoError(t, err)
	assert.True(t, pass)

	pub, ok := priv.PublicKey().ToStandardKey().(*tjsm2.PublicKey)
	assert.True(t, ok)
	assert.NotNil(t, pub)
	ok = pub.Verify(msg, sig)
	assert.True(t, ok)
}

func TestSM2Std(t *testing.T) {
	//gmssl sign, tjfoc verify
	priv, _ := GenerateKeyPair()

	sig, err := priv.signWithSM3(msg, utils.SM2_DEFAULT_USER_ID)
	assert.NoError(t, err)

	pkBytes, err := MarshalPublicKey(&priv.Pub)
	assert.NoError(t, err)
	tjpk, err := tjx509.ParseSm2PublicKey(pkBytes)
	assert.NoError(t, err)

	ok := tjpk.Verify(msg, sig)
	assert.True(t, ok)

	//tjfoc sign, gmssl verify
	tjsk, _ := tjsm2.GenerateKey(rand.Reader)
	sig, _ = tjsk.Sign(rand.Reader, msg, nil)

	pkBytes, _ = tjx509.MarshalSm2PublicKey(&tjsk.PublicKey)
	gmsslpk, err := UnmarshalPublicKey(pkBytes)
	assert.NoError(t, err)

	ok = gmsslpk.verifyWithSM3(msg, sig, utils.SM2_DEFAULT_USER_ID)
	assert.True(t, ok)
}

func TestUnmarshalPrivateKey(t *testing.T) {
	priv, _ := GenerateKeyPair()
	keyBytes, err := priv.Bytes()
	assert.NoError(t, err)

	prv, err := UnmarshalPrivateKey(keyBytes)
	assert.NoError(t, err)
	assert.NotNil(t, prv)
}

func TestUnmarshalPublicKey(t *testing.T) {
	priv, _ := GenerateKeyPair()
	keyBytes, err := priv.Pub.Bytes()
	assert.NoError(t, err)

	prv, err := UnmarshalPublicKey(keyBytes)
	assert.NoError(t, err)
	assert.NotNil(t, prv)
}
