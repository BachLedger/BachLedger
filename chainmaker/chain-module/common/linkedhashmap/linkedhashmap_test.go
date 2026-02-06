/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package linkedhashmap

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestLinkedHashMap_AddAndGet(t *testing.T) {
	hashMap := NewLinkedHashMap()

	require.True(t, hashMap.Add("key1", "value1"))
	require.False(t, hashMap.Add("key1", "value11"))
	require.False(t, hashMap.Add("key1", hashMap.Get("key1")))
	require.EqualValues(t, 1, hashMap.Size())
	require.True(t, hashMap.Add("key2", "value2"))
	require.True(t, hashMap.Add("key3", "value3"))
	require.EqualValues(t, 3, hashMap.Size())

	require.EqualValues(t, "value1", hashMap.Get("key1").(string))
	require.EqualValues(t, "value2", hashMap.Get("key2").(string))
	require.EqualValues(t, "value3", hashMap.Get("key3").(string))
	require.Nil(t, hashMap.Get("key5"))
}

func TestLinkedHashMap_Remove(t *testing.T) {
	hashMap := NewLinkedHashMap()

	require.True(t, hashMap.Add("key1", "value10"))
	require.True(t, hashMap.Add("key2", "value20"))
	require.True(t, hashMap.Add("key3", "value30"))

	ok, val := hashMap.Remove("key1")
	require.True(t, ok)
	require.EqualValues(t, "value10", val.(string))
	require.Nil(t, hashMap.Get("key1"))
	require.EqualValues(t, 2, hashMap.Size())

	ok, val = hashMap.Remove("key1")
	require.False(t, ok)
	require.Nil(t, val)
	require.EqualValues(t, 2, hashMap.Size())
}
