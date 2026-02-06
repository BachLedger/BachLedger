/*
 * Copyright (C) BABEC. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

package cache

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func Test_Cache(t *testing.T) {
	key1 := "key1"
	bl := NewCacheList("chain1")

	b := bl.Exists(key1)
	assert.Equal(t, false, b)

	bl.Put(key1)
	b = bl.Exists(key1)
	assert.Equal(t, true, b)

	bl.Delete(key1)
	b = bl.Exists(key1)
	assert.Equal(t, false, b)

	bl.Put(key1)

	bl = NewCacheList("chain1")
	b = bl.Exists(key1)
	assert.Equal(t, true, b)

	bl = NewCacheList("chain2")
	b = bl.Exists(key1)
	assert.Equal(t, false, b)

	bl.Put(key1)
	b = bl.Exists(key1)
	assert.Equal(t, true, b)
}
