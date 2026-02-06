/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package common

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestNewStringMapList(t *testing.T) {
	smList := NewStringMapList()
	require.NotNil(t, smList)
}

func TestFunctions(t *testing.T) {
	smList := NewStringMapList()
	key := "peerId"

	ok := smList.Add(key)
	require.True(t, ok)

	ok = smList.Contains(key)
	require.True(t, ok)

	size := smList.Size()
	require.Equal(t, 1, size)

	lists := smList.List()
	require.NotEmpty(t, lists)

	ok = smList.Remove(key)
	require.True(t, ok)
	require.Empty(t, smList.mapList)
}
