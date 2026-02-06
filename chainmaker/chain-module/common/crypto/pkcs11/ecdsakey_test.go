/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package pkcs11

import (
	"crypto"
	"crypto/rand"
	"testing"

	bccrypto "chainmaker.org/chainmaker/common/v2/crypto"
	"github.com/stretchr/testify/assert"
)

func TestSM2(t *testing.T) {
	if isSoftHSM {
		t.Skip("skip: softhsm not supported sm2")
	}

	bcPriv, err := NewPrivateKey(p11, string(internalSM2KeyId), bccrypto.SM2)
	assert.NoError(t, err)

	signer := bcPriv.ToStandardKey().(crypto.Signer)

	sig, err := signer.Sign(rand.Reader, plain, nil)
	assert.NoError(t, err)
	assert.NotNil(t, sig)

	ok, err := bcPriv.PublicKey().VerifyWithOpts(plain, sig, &bccrypto.SignOpts{Hash: bccrypto.HASH_TYPE_SM3})
	assert.NoError(t, err)
	assert.True(t, ok)
}

//
//func TestECC(t *testing.T) {
//	bcPriv, err := NewPrivateKey(p11, string(internalECCKeyId), bccrypto.ECC_NISTP256)
//	assert.NoError(t, err)
//
//	signer := bcPriv.ToStandardKey().(crypto.Signer)
//
//	sig, err := signer.Sign(rand.Reader, plain, nil)
//	assert.NoError(t, err)
//	assert.NotNil(t, sig)
//
//	ok, err := bcPriv.PublicKey().VerifyWithOpts(plain, sig, &bccrypto.SignOpts{Hash: bccrypto.HASH_TYPE_SHA256})
//	assert.NoError(t, err)
//	assert.True(t, ok)
//}

//
//func TestGenerateKeyPair_SM2(t *testing.T) {
//	if !support_GM {
//		t.Skipf("skip: softhsm not supported sm2")
//	}
//	tokenLabel := "TestSM2"
//	priv, err := GenKeyPair(p11, tokenLabel, bccrypto.SM2, nil)
//	assert.NoError(t, err)
//	assert.NotNil(t, priv)
//
//	sig, err := priv.Sign(plain)
//	assert.NoError(t, err)
//	assert.NotNil(t, sig)
//
//	pub, err := p11.findPublicKey([]byte(tokenLabel))
//	assert.NoError(t, err)
//	assert.NotNil(t, pub)
//	err = p11.Verify(*pub, pkcs11.NewMechanism(CKM_SM3_SM2_APPID1_DER, nil), plain, sig)
//	assert.NoError(t, err)
//}
//
//func TestP11Handle_SignVerify(t *testing.T) {
//	if !support_GM {
//		t.Skipf("skip: softhsm not supported sm2")
//	}
//
//	priv, err := p11.findPrivateKeyByLabel(privKeyLabel)
//	assert.NoError(t, err)
//	assert.NotNil(t, priv)
//
//	sig, err := p11.Sign(*priv, pkcs11.NewMechanism(CKM_SM3_SM2, nil), plain)
//	assert.NoError(t, err)
//	assert.NotNil(t, sig)
//
//	pub, err := p11.findPublicKeyByLabel(pubKeyLabel)
//	assert.NoError(t, err)
//	assert.NotNil(t, pub)
//
//	err = p11.Verify(*pub, pkcs11.NewMechanism(CKM_SM3_SM2, nil), plain, sig)
//	assert.NoError(t, err)
//}
//
//func TestP11Handle_ExportECDSAPublicKey(t *testing.T) {
//	if !support_GM {
//		t.Skipf("skip: softhsm not supported sm2")
//	}
//	priv, err := p11.findPrivateKeyByLabel(privKeyLabel)
//	assert.NoError(t, err)
//	assert.NotNil(t, priv)
//
//	sig, err := p11.Sign(*priv, pkcs11.NewMechanism(CKM_SM3_SM2_DER, nil), plain)
//	assert.NoError(t, err)
//	assert.NotNil(t, sig)
//
//	//hsm verify
//	pubOjb, err := p11.findPublicKey(pubKeyLabel)
//	err = p11.Verify(*pubOjb, pkcs11.NewMechanism(CKM_SM3_SM2_DER, nil), plain, sig)
//	assert.NoError(t, err)
//
//	//soft verify
//	pub, err := p11.ExportECDSAPublicKey(pubKeyLabel, SM2)
//	assert.NoError(t, err)
//	assert.NotNil(t, pub)
//
//	sm2Pub := pub.(*sm2.PublicKey)
//	ecPoint := elliptic.Marshal(sm2.P256Sm2(), sm2Pub.X, sm2Pub.Y)
//	t.Logf("pub[%d]= %s\n", len(ecPoint), hex.EncodeToString(ecPoint))
//	t.Logf("sig[%d] = %s\n", len(sig), hex.EncodeToString(sig))
//
//	var rsSig signature
//	_, err = asn1.Unmarshal(sig, &rsSig)
//	assert.NoError(t, err)
//
//	ok := sm2.Verify(sm2Pub, sm3.Sm3Sum(plain), rsSig.R, rsSig.S)
//	assert.True(t, ok)
//}
//
//func TestP11Handle_ExportECDSAPublicKey2(t *testing.T) {
//	if !support_GM {
//		t.Skipf("skip: softhsm not supported sm2")
//	}
//	priv, err := p11.findPrivateKeyByLabel(privKeyLabel)
//	assert.NoError(t, err)
//	assert.NotNil(t, priv)
//
//	sig, err := p11.Sign(*priv, pkcs11.NewMechanism(CKM_SM3_SM2_APPID1_DER, nil), plain)
//	assert.NoError(t, err)
//	assert.NotNil(t, sig)
//
//	//hsm verify
//	pubOjb, err := p11.findPublicKey(pubKeyLabel)
//	err = p11.Verify(*pubOjb, pkcs11.NewMechanism(CKM_SM3_SM2_APPID1_DER, nil), plain, sig)
//	assert.NoError(t, err)
//
//	//soft verify
//	pub, err := p11.ExportECDSAPublicKey(pubKeyLabel, SM2)
//	assert.NoError(t, err)
//	assert.NotNil(t, pub)
//
//	sm2Pub := pub.(*sm2.PublicKey)
//	var rsSig signature
//	_, err = asn1.Unmarshal(sig, &rsSig)
//	assert.NoError(t, err)
//
//	ok := sm2.Sm2Verify(sm2Pub, plain, nil, rsSig.R, rsSig.S)
//	assert.True(t, ok)
//}
//
//func TestSoftVerify_SM2(t *testing.T) {
//	if !support_GM {
//		t.Skipf("skip: softhsm not supported sm2")
//	}
//	bcPriv, err := NewPrivateKey(p11, string(internalSM2KeyId), bccrypto.SM2)
//	assert.NoError(t, err)
//
//	for i := 0; i < testNum; i++ {
//		sig, err := bcPriv.SignWithOpts(plain, &bccrypto.SignOpts{Hash: bccrypto.HASH_TYPE_SM3, UID: bccrypto.CRYPTO_DEFAULT_UID})
//		assert.NoError(t, err)
//		assert.NotNil(t, sig)
//
//		ok, err := bcPriv.PublicKey().VerifyWithOpts(plain, sig, &bccrypto.SignOpts{Hash: bccrypto.HASH_TYPE_SM3, UID: bccrypto.CRYPTO_DEFAULT_UID})
//		assert.NoError(t, err)
//		assert.True(t, ok)
//		if !ok || err != nil {
//			fmt.Printf("plainHex := %s\n", hex.EncodeToString(plain))
//			pubKeyBytes, err := bcPriv.PublicKey().Bytes()
//			assert.NoError(t, err)
//			fmt.Printf("pubHex := %s\n", hex.EncodeToString(pubKeyBytes))
//			fmt.Printf("sigHex := %s\n", hex.EncodeToString(sig))
//		}
//	}
//}
//
//func TestHSMVerify_SM2(t *testing.T) {
//	if !support_GM {
//		t.Skipf("skip: softhsm not supported sm2")
//	}
//	bcPriv, err := NewPrivateKey(p11, string(internalSM2KeyId), bccrypto.SM2)
//	assert.NoError(t, err)
//
//	for i := 0; i < testNum; i++ {
//		sig, err := bcPriv.SignWithOpts(plain, &bccrypto.SignOpts{Hash: bccrypto.HASH_TYPE_SM3, UID: bccrypto.CRYPTO_DEFAULT_UID})
//		assert.NoError(t, err)
//		assert.NotNil(t, sig)
//
//		pub, err := p11.findPublicKey(pubKeyLabel)
//		assert.NoError(t, err)
//		assert.NotNil(t, pub)
//		err = p11.Verify(*pub, pkcs11.NewMechanism(CKM_SM3_SM2_APPID1_DER, nil), plain, sig)
//		assert.NoError(t, err)
//	}
//}
//
//func BenchmarkHSMSign_SM2(b *testing.B) {
//	if !support_GM {
//		b.Skipf("skip: softhsm not supported sm2")
//	}
//
//	bcPriv, err := NewPrivateKey(p11, string(internalSM2KeyId), bccrypto.SM2)
//	assert.NoError(b, err)
//
//	b.ResetTimer()
//	for i := 0; i < b.N; i++ {
//		bcPriv.SignWithOpts(plain, &bccrypto.SignOpts{Hash: bccrypto.HASH_TYPE_SM3, UID: bccrypto.CRYPTO_DEFAULT_UID})
//	}
//}
//
//func BenchmarkHSMVerify_SM2(b *testing.B) {
//	if !support_GM {
//		b.Skipf("skip: softhsm not supported sm2")
//	}
//
//	bcPriv, err := NewPrivateKey(p11, string(internalSM2KeyId), bccrypto.SM2)
//	assert.NoError(b, err)
//	sig, err := bcPriv.SignWithOpts(plain, &bccrypto.SignOpts{Hash: bccrypto.HASH_TYPE_SM3, UID: bccrypto.CRYPTO_DEFAULT_UID})
//	assert.NoError(b, err)
//	pub, err := p11.findPublicKey(pubKeyLabel)
//
//	b.ResetTimer()
//	for i := 0; i < b.N; i++ {
//		p11.Verify(*pub, pkcs11.NewMechanism(CKM_SM3_SM2_APPID1_DER, nil), plain, sig)
//	}
//}
