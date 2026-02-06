package test_common

import (
	"crypto"
	"crypto/rand"
	"encoding/hex"
	"log"
	"sync"
	"testing"
	"time"

	tjsm2 "github.com/tjfoc/gmsm/sm2"

	bccrypto "chainmaker.org/chainmaker/common/v2/crypto"

	gsm2 "chainmaker.org/chainmaker/common/v2/opencrypto/gmssl/sm2"
	tsm2 "chainmaker.org/chainmaker/common/v2/opencrypto/tencentsm/sm2"
	tsm3 "chainmaker.org/chainmaker/common/v2/opencrypto/tencentsm/sm3"
	"github.com/stretchr/testify/assert"
	tjx509 "github.com/tjfoc/gmsm/x509"
)

func TestSM2Standard(t *testing.T) {
	h := tsm3.New()
	_, err := h.Write(msg)
	assert.NoError(t, err)
	digest := h.Sum(nil)

	//tencentsm sign， tjfoc verify
	tsm2key, err := tsm2.GenerateKeyPair()
	assert.NoError(t, err)
	sig, err := tsm2key.ToStandardKey().(crypto.Signer).Sign(nil, digest, nil)
	assert.NoError(t, err)

	keyBytes, _ := tsm2.MarshalPrivateKey(tsm2key)
	tjsm2key, _ := tjx509.ParsePKCS8UnecryptedPrivateKey(keyBytes)
	ok := tjsm2key.Verify(digest, sig)
	assert.True(t, ok)

	//tjfoc sign, tencentsm verify
	sig, err = tjsm2key.Sign(rand.Reader, digest, nil)
	assert.NoError(t, err)

	ok, err = tsm2key.PublicKey().Verify(digest, sig)
	assert.NoError(t, err)
	assert.True(t, ok)

	//tjfoc sign, gmssl verify
	sig, err = tjsm2key.Sign(rand.Reader, digest, nil)
	assert.NoError(t, err)

	gsm2key, err := gsm2.UnmarshalPrivateKey(keyBytes)
	assert.NoError(t, err)
	ok, err = gsm2key.PublicKey().VerifyWithOpts(digest, sig, &bccrypto.SignOpts{Hash: bccrypto.HASH_TYPE_SM3})
	assert.NoError(t, err)
	assert.True(t, ok)

	//gmssl sign, tencetsm2 verify
	sig, err = gsm2key.SignWithOpts(digest, &bccrypto.SignOpts{Hash: bccrypto.HASH_TYPE_SM3})
	assert.NoError(t, err)
	ok, err = tsm2key.PublicKey().Verify(digest, sig)
	assert.NoError(t, err)
	assert.True(t, ok)
}

func TestSM2EncDecStandard(t *testing.T) {
	tsm2key, err := tsm2.GenerateKeyPair()
	assert.NoError(t, err)
	keyBytes, err := tsm2.MarshalPrivateKey(tsm2key)
	assert.NoError(t, err)
	tjsm2key, err := tjx509.ParsePKCS8UnecryptedPrivateKey(keyBytes)
	assert.NoError(t, err)
	gsm2key, err := gsm2.UnmarshalPrivateKey(keyBytes)
	assert.NoError(t, err)

	//tencentsm, gmssl, tjfoc encryption and decryption
	c1, err := tsm2key.EncryptKey().Encrypt(msg)
	assert.NoError(t, err)
	p1, err := tsm2key.Decrypt(c1)
	assert.NoError(t, err)
	assert.Equal(t, msg, p1)
	log.Printf("tencentSM ciphertext[%d]: %s\n", len(hex.EncodeToString(c1)), hex.EncodeToString(c1))

	c2, err := gsm2key.EncryptKey().Encrypt(msg)
	assert.NoError(t, err)
	p2, err := gsm2key.Decrypt(c2)
	assert.NoError(t, err)
	assert.Equal(t, msg, p2)
	log.Printf("gmssl ciphertext[%d]: %s\n", len(hex.EncodeToString(c2)), hex.EncodeToString(c2))

	c3, err := tjsm2key.PublicKey.EncryptAsn1(msg, rand.Reader)
	assert.NoError(t, err)
	p3, err := tjsm2key.DecryptAsn1(c3)
	assert.NoError(t, err)
	assert.Equal(t, msg, p3)
	log.Printf("tjfoc ciphertext[%d]: %s\n", len(hex.EncodeToString(c3)), hex.EncodeToString(c3))

	p4, err := gsm2key.Decrypt(c1)
	assert.NoError(t, err)
	assert.Equal(t, msg, p4)
	log.Println("p4 = ", string(p4))

	p5, err := tjsm2key.DecryptAsn1(c1)
	assert.NoError(t, err)
	assert.Equal(t, msg, p5)
	log.Println("p5 = ", string(p5))

	p6, err := tsm2key.Decrypt(c2)
	assert.NoError(t, err)
	assert.Equal(t, msg, p6)
	log.Println("p6 = ", string(p6))
}

func BenchmarkTjfocSign(b *testing.B) {
	prv, err := tjsm2.GenerateKey(rand.Reader)
	assert.NoError(b, err)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		prv.Sign(rand.Reader, msg, nil)
	}
}

func BenchmarkTjfocVerify(b *testing.B) {
	prv, err := tjsm2.GenerateKey(rand.Reader)
	assert.NoError(b, err)
	sig, err := prv.Sign(rand.Reader, msg, nil)
	assert.NoError(b, err)

	pub := prv.PublicKey

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		pub.Verify(msg, sig)
	}
}

func BenchmarkTencentSMSign(b *testing.B) {
	prv, err := tsm2.GenerateKeyPair()
	assert.NoError(b, err)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		prv.Sign(msg)
	}
}

func BenchmarkTencentSMVerify(b *testing.B) {
	prv, err := tsm2.GenerateKeyPair()
	assert.NoError(b, err)
	sig, err := prv.Sign(msg)
	assert.NoError(b, err)

	pub := prv.PublicKey()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		pub.Verify(msg, sig)
	}
}

func BenchmarkGmsslSign(b *testing.B) {
	prv, err := gsm2.GenerateKeyPair()
	assert.NoError(b, err)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		prv.Sign(msg)
	}
}

func BenchmarkGmsslVerify(b *testing.B) {
	prv, err := gsm2.GenerateKeyPair()
	assert.NoError(b, err)
	sig, err := prv.Sign(msg)
	assert.NoError(b, err)

	pub := prv.PublicKey()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		pub.Verify(msg, sig)
	}
}

func BenchmarkTjfocEncrypt(b *testing.B) {
	prv, err := tjsm2.GenerateKey(rand.Reader)
	assert.NoError(b, err)
	pub := prv.PublicKey

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		pub.EncryptAsn1(msg, rand.Reader)
	}
}
func BenchmarkTjfocDecrypt(b *testing.B) {
	prv, err := tjsm2.GenerateKey(rand.Reader)
	assert.NoError(b, err)
	pub := prv.PublicKey
	c, err := pub.EncryptAsn1(msg, rand.Reader)
	assert.NoError(b, err)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		prv.DecryptAsn1(c)
	}
}

func BenchmarkTencentSMEncrypt(b *testing.B) {
	prv, err := tsm2.GenerateKeyPair()
	assert.NoError(b, err)
	pub := prv.EncryptKey()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		pub.Encrypt(msg)
	}
}
func BenchmarkTencentSMDecrypt(b *testing.B) {
	prv, err := tsm2.GenerateKeyPair()
	assert.NoError(b, err)
	c, err := prv.EncryptKey().Encrypt(msg)
	assert.NoError(b, err)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		prv.Decrypt(c)
	}
}

func BenchmarkGmsslEncrypt(b *testing.B) {
	prv, err := gsm2.GenerateKeyPair()
	assert.NoError(b, err)
	pub := prv.EncryptKey()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		pub.Encrypt(msg)
	}
}
func BenchmarkGmsslDecrypt(b *testing.B) {
	prv, err := gsm2.GenerateKeyPair()
	assert.NoError(b, err)
	c, err := prv.EncryptKey().Encrypt(msg)
	assert.NoError(b, err)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		prv.Decrypt(c)
	}
}

func BenchmarkTencentSMVerifyParallel(b *testing.B) {
	prv, err := tsm2.GenerateKeyPair()
	assert.NoError(b, err)
	sig, err := prv.Sign(msg)
	assert.NoError(b, err)

	pub := prv.PublicKey()

	b.ResetTimer()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			ok, err := pub.Verify(msg, sig)
			assert.NoError(b, err)
			assert.True(b, ok)
		}
	})
}

func BenchmarkGmsslVerifyParallel(b *testing.B) {
	prv, err := gsm2.GenerateKeyPair()
	assert.NoError(b, err)
	sig, err := prv.Sign(msg)
	assert.NoError(b, err)

	pub := prv.PublicKey()

	b.ResetTimer()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			ok, err := pub.Verify(msg, sig)
			assert.NoError(b, err)
			assert.True(b, ok)
		}
	})
}

func TestSM2Verify(t *testing.T) {
	t.Skip("skip this test in CI")
	//viper.Set("common.tencentsm.ctx_pool_size", 1)
	keys := genSM2Key("tencentsm", 10000, t)

	start := time.Now()
	wg := &sync.WaitGroup{}
	count := 1000
	wg.Add(count)
	for p := 0; p < count; p++ {
		go func(p int) {
			for i := 0; i < 10; i++ {
				prv := keys[p*10+i]
				pub := prv.PublicKey()

				sign, err := prv.Sign(msg)
				assert.NoError(t, err)

				ok, err := pub.Verify(msg, sign)
				assert.NoError(t, err)
				assert.True(t, ok)
			}
			wg.Done()
		}(p)
	}
	wg.Wait()
	log.Println(time.Since(start))
}

func genSM2Key(engine string, n int, t *testing.T) []bccrypto.PrivateKey {
	keys := make([]bccrypto.PrivateKey, n)
	for i := 0; i < n; i++ {
		if engine == "gmssl" {
			keys[i], _ = gsm2.GenerateKeyPair()
		} else {
			keys[i], _ = tsm2.GenerateKeyPair()
		}
	}
	return keys
}

var (
	workers       = 10000
	taskPerWorker = 1000
)

func TestSigner_Sign1(t *testing.T) {
	t.Skip("SKIP: this test takes too much time")
	priv, _ := gsm2.GenerateKeyPair()
	sig, err := priv.ToStandardKey().(crypto.Signer).Sign(rand.Reader, msg, nil)
	assert.NoError(t, err)
	pub := priv.PublicKey()

	var wg sync.WaitGroup
	wg.Add(workers)
	for i := 0; i < workers; i++ {
		//index := i
		go func() {
			defer wg.Done()
			for j := 0; j < taskPerWorker; j++ {
				pass, err1 := pub.Verify(msg, sig)
				assert.NoError(t, err1)
				assert.True(t, pass)
				//time.Sleep(time.Millisecond * 10)
			}
		}()
	}
	wg.Wait()
}

func TestSigner_Sign2(t *testing.T) {
	t.Skip()
	for i := 0; i < 1000; i++ {
		for j := 0; j < 1000; j++ {
			priv, _ := gsm2.GenerateKeyPair()
			sig, err := priv.ToStandardKey().(crypto.Signer).Sign(rand.Reader, msg, nil)
			assert.NoError(t, err)

			pass, err := priv.PublicKey().Verify(msg, sig)
			assert.NoError(t, err)
			assert.True(t, pass)
		}
	}
}
