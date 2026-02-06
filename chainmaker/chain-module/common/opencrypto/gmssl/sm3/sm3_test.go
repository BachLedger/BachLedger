/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package sm3

import (
	"testing"

	"github.com/tjfoc/gmsm/sm3"

	"github.com/stretchr/testify/assert"
)

var (
	msg = []byte("hello gmssl")
)

func TestSM3Std(t *testing.T) {
	h1 := New()
	_, err := h1.Write(msg)
	assert.NoError(t, err)
	digest1 := h1.Sum(nil)

	h2 := sm3.New()
	_, err = h2.Write(msg)
	assert.NoError(t, err)
	digest2 := h2.Sum(nil)

	assert.Equal(t, digest1, digest2)
}

func BenchmarkSM3(b *testing.B) {
	h1 := New()

	for i := 0; i < b.N; i++ {
		h1.Write(msg)
		h1.Sum(nil)
	}
}
