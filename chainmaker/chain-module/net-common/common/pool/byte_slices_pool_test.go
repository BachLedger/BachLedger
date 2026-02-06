/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package pool

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestByteSlicesPool(t *testing.T) {
	p := NewByteSlicesPool(10)
	b := p.Get()
	p.Put(b)

	msg := []byte("Hello, world!")
	b2 := p.Get()
	*b2 = append(*b2, msg...)
	c := cap(*b2)
	p.Put(b2)

	b3 := p.Get()
	require.Equal(t, c, cap(*b3))
	require.Equal(t, 0, len(*b3))
	p.Put(b3)

	b4 := p.GetWithLen(1)
	require.Equal(t, c, cap(*b4))
	require.Equal(t, 1, len(*b4))
	require.Equal(t, byte(0), (*b4)[0])

	b5 := p.GetWithLen(50)
	require.Equal(t, 50, cap(*b5))
	require.Equal(t, 50, len(*b5))
	for i := 0; i < 50; i++ {
		require.Equal(t, byte(0), (*b5)[i])
	}

}
