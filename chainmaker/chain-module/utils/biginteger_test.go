/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package utils

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestBigInteger(t *testing.T) {
	bigInteger := NewBigInteger("1024000000000000000000000000000000000000000000")
	require.NotNil(t, bigInteger)
	bigInteger.Add(NewBigInteger("1024"))
	require.Equal(t, "1024000000000000000000000000000000000000001024", bigInteger.String())
	bigInteger.Sub(NewBigInteger("1024"))
	require.Equal(t, "1024000000000000000000000000000000000000000000", bigInteger.String())
}

func TestSum(t *testing.T) {
	bigInteger := NewBigInteger("1024000000000000000000000000000000000000000000")
	bigInteger2 := NewBigInteger("1024000000000000000000000000000000000000000000")
	bigIntegerSum := Sum(bigInteger, bigInteger2)
	require.Equal(t, bigIntegerSum.Cmp(NewBigInteger("2048000000000000000000000000000000000000000000")), 0)
}

func TestSub(t *testing.T) {
	bigInteger := NewBigInteger("1024000000000000000000000000000000000000000000")
	bigInteger2 := NewBigInteger("1024000000000000000000000000000000000000000000")
	bigIntegerSum := Sub(bigInteger, bigInteger2)
	require.Equal(t, bigIntegerSum.Cmp(NewZeroBigInteger()), 0)
}
