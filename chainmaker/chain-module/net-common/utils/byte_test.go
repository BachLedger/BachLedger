/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package utils

import (
	"bytes"
	"math"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestUint64ToBytes(t *testing.T) {
	var a, b, c uint64 = 0, math.MaxUint64, math.MaxUint32
	aB := Uint64ToBytes(a)
	require.True(t, bytes.Equal([]byte{0, 0, 0, 0, 0, 0, 0, 0}, aB))
	bB := Uint64ToBytes(b)
	require.True(t, bytes.Equal([]byte{255, 255, 255, 255, 255, 255, 255, 255}, bB))
	cB := Uint64ToBytes(c)
	require.True(t, bytes.Equal([]byte{0, 0, 0, 0, 255, 255, 255, 255}, cB))
}

func TestBytesToUint64(t *testing.T) {
	var a, b, c = []byte{0, 0, 0, 0, 0, 0, 0, 0}, []byte{255, 255, 255, 255, 255, 255, 255, 255}, []byte{0, 0, 0, 0, 255, 255, 255, 255}
	aB := BytesToUint64(a)
	require.True(t, 0 == aB)
	bB := BytesToUint64(b)
	require.True(t, math.MaxUint64 == bB)
	cB := BytesToUint64(c)
	require.True(t, math.MaxUint32 == cB)
}

func TestUint32ToBytes(t *testing.T) {
	var a, b, c uint32 = 0, math.MaxUint32, math.MaxUint16
	aB := Uint32ToBytes(a)
	require.True(t, bytes.Equal([]byte{0, 0, 0, 0}, aB))
	bB := Uint32ToBytes(b)
	require.True(t, bytes.Equal([]byte{255, 255, 255, 255}, bB))
	cB := Uint32ToBytes(c)
	require.True(t, bytes.Equal([]byte{0, 0, 255, 255}, cB))
}

func TestBytesToUint32(t *testing.T) {
	var a, b, c = []byte{0, 0, 0, 0}, []byte{255, 255, 255, 255}, []byte{0, 0, 255, 255}
	aB := BytesToUint32(a)
	require.True(t, 0 == aB)
	bB := BytesToUint32(b)
	require.True(t, math.MaxUint32 == bB)
	cB := BytesToUint32(c)
	require.True(t, math.MaxUint16 == cB)
}
