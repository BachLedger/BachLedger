/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package concurrentlru

import (
	"sync"

	"github.com/golang/groupcache/lru"
)

type Cache struct {
	mu    sync.Mutex
	cache *lru.Cache
}

func New(maxEntries int) *Cache {
	var cache Cache
	cache.cache = lru.New(maxEntries)
	return &cache
}

// Add adds a value to the cache.
func (c *Cache) Add(key lru.Key, value interface{}) {
	c.mu.Lock()
	defer c.mu.Unlock()

	c.cache.Add(key, value)
}

// Get looks up a key's value from the cache.
func (c *Cache) Get(key lru.Key) (value interface{}, ok bool) {
	c.mu.Lock()
	defer c.mu.Unlock()

	return c.cache.Get(key)
}

// Remove removes the provided key from the cache.
func (c *Cache) Remove(key lru.Key) {
	c.mu.Lock()
	defer c.mu.Unlock()

	c.cache.Remove(key)
}

// RemoveOldest removes the oldest item from the cache.
func (c *Cache) RemoveOldest() {
	c.mu.Lock()
	defer c.mu.Unlock()

	c.cache.RemoveOldest()
}

// Len returns the number of items in the cache.
func (c *Cache) Len() int {
	return c.cache.Len()
}

// Clear purges all stored items from the cache.
func (c *Cache) Clear() {
	c.mu.Lock()
	defer c.mu.Unlock()

	c.cache.Clear()
}
