package test_common

import (
	"sync"
	"testing"

	gsm3 "chainmaker.org/chainmaker/common/v2/opencrypto/gmssl/sm3"
	tsm3 "chainmaker.org/chainmaker/common/v2/opencrypto/tencentsm/sm3"
	"github.com/stretchr/testify/assert"
	tjsm3 "github.com/tjfoc/gmsm/sm3"
)

func TestSM3Standard(t *testing.T) {
	//tencentsm, tjfoc and gmssl hash compare
	h := tsm3.New()
	_, err := h.Write(msg)
	assert.NoError(t, err)
	digest1 := h.Sum(nil)

	h = tjsm3.New()
	_, err = h.Write(msg)
	assert.NoError(t, err)
	digest2 := h.Sum(nil)

	h = gsm3.New()
	_, err = h.Write(msg)
	assert.NoError(t, err)
	digest3 := h.Sum(nil)

	assert.Equal(t, digest1, digest2)
	assert.Equal(t, digest2, digest3)
}

func BenchmarkTjfocSM3(b *testing.B) {
	for i := 0; i < b.N; i++ {
		h := tjsm3.New()
		h.Write(msg)
		h.Sum(nil)
	}
}

func BenchmarkTecentSMSM3(b *testing.B) {
	for i := 0; i < b.N; i++ {
		h := tsm3.New()
		h.Write(msg)
		h.Sum(nil)
	}
}

func BenchmarkGmsslSM3(b *testing.B) {
	for i := 0; i < b.N; i++ {
		h := gsm3.New()
		h.Write(msg)
		h.Sum(nil)
	}
}

func TestSM3(t *testing.T) {
	t.Skip("skip gmssl sm3 parallel test")
	var wg sync.WaitGroup
	wg.Add(workers)
	for i := 0; i < workers; i++ {
		//index := i
		go func() {
			defer wg.Done()
			for j := 0; j < taskPerWorker; j++ {
				h := gsm3.New()
				h.Write(msg)
				h.Sum(nil)
			}
		}()
	}
	wg.Wait()
}
