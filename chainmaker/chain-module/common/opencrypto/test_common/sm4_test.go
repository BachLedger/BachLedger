package test_common

import (
	"crypto/cipher"
	"crypto/rand"
	"testing"

	"chainmaker.org/chainmaker/common/v2/crypto/sym/util"
	"github.com/tjfoc/gmsm/sm4"

	gsm4 "chainmaker.org/chainmaker/common/v2/opencrypto/gmssl/sm4"
	"github.com/stretchr/testify/assert"
)

func TestSM4Standard(t *testing.T) {
	var key [16]byte
	_, err := rand.Read(key[:])
	assert.NoError(t, err)

	gSM4Key := gsm4.SM4Key{Key: key[:]}

	//gmssl encrypt, tencentsm decrypto
	ciphertext, err := gSM4Key.Encrypt(msg)
	assert.NoError(t, err)
	assert.Equal(t, msg, ciphertext)

	plaintext, err := gSM4Key.Decrypt(ciphertext)
	assert.NoError(t, err)

	assert.Equal(t, msg, plaintext)
}

func BenchmarkTjfocSM4Encrypt(b *testing.B) {
	var key [16]byte
	rand.Read(key[:])

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		block, _ := sm4.NewCipher(key[:])

		msgWithPad := util.PKCS5Padding(msg, block.BlockSize())
		iv := make([]byte, block.BlockSize())
		rand.Read(iv)

		blockMode := cipher.NewCBCEncrypter(block, iv)
		crypted := make([]byte, len(msgWithPad))

		blockMode.CryptBlocks(crypted[:], msgWithPad)
	}
}

func BenchmarkTjfocSM4Decrypt(b *testing.B) {
	var key [16]byte
	_, err := rand.Read(key[:])
	assert.NoError(b, err)

	block, _ := sm4.NewCipher(key[:])
	msgWithPad := util.PKCS5Padding(msg, block.BlockSize())
	iv := make([]byte, block.BlockSize())
	rand.Read(iv)

	blockMode := cipher.NewCBCEncrypter(block, iv)
	crypted := make([]byte, len(msgWithPad)+len(iv))
	blockMode.CryptBlocks(crypted[block.BlockSize():], msgWithPad)
	copy(crypted[0:block.BlockSize()], iv)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		block, _ = sm4.NewCipher(key[:])
		blockMode = cipher.NewCBCDecrypter(block, iv)
		orig := make([]byte, len(crypted)-block.BlockSize())
		blockMode.CryptBlocks(orig, crypted[block.BlockSize():])
		util.PKCS5UnPadding(orig)
	}
}

func BenchmarkGmsslSM4Encrypt(b *testing.B) {
	var key [16]byte
	_, err := rand.Read(key[:])
	assert.NoError(b, err)

	s := gsm4.SM4Key{Key: key[:]}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		s.Encrypt(msg)
	}
}

func BenchmarkGmsslSM4Decrypt(b *testing.B) {
	var key [16]byte
	_, err := rand.Read(key[:])
	assert.NoError(b, err)
	s := gsm4.SM4Key{Key: key[:]}

	cipherText, err := s.Encrypt(msg)
	assert.NoError(b, err)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		s.Decrypt(cipherText)
	}
}
