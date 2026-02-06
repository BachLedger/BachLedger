/*
 * Copyright (C) BABEC. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 */
/*
 * sample cache with sync.map
 */

package cache

import (
	"sync"
)

var (
	lock          = &sync.Mutex{}
	cacheInstance = make(map[string]*CacheList)
)

// CacheList sample map
type CacheList struct {
	elements sync.Map
}

// NewCacheList get CacheList instance
func NewCacheList(name string) *CacheList {
	instance, ok := cacheInstance[name]
	if !ok {
		lock.Lock()
		defer lock.Unlock()
		instance, ok = cacheInstance[name]
		if !ok {
			instance = &CacheList{elements: sync.Map{}}
			cacheInstance[name] = instance
		}
	}
	return instance
}

// Put put key
func (b *CacheList) Put(key string) {
	b.elements.Store(key, true)
}

// Delete del key
func (b *CacheList) Delete(key string) {
	b.elements.Delete(key)
}

// Exists exists key,notfound return false
func (b *CacheList) Exists(key string) bool {
	val, ok := b.elements.Load(key)
	if ok {
		return val.(bool)
	}
	return false
}
