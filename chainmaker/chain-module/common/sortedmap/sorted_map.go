/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package sortedmap

import (
	"sort"
	"sync"
	"sync/atomic"
)

type StringKeySortedMap struct {
	keysLock   sync.RWMutex
	keys       []string
	m          sync.Map
	needResort uint32
}

func NewStringKeySortedMap() *StringKeySortedMap {
	return &StringKeySortedMap{
		keys:       make([]string, 0),
		m:          sync.Map{},
		needResort: 0,
	}
}

func NewStringKeySortedMapWithData(data map[string]string) *StringKeySortedMap {
	if len(data) == 0 {
		return NewStringKeySortedMap()
	}

	sortMap := &StringKeySortedMap{
		keys:       make([]string, 0),
		m:          sync.Map{},
		needResort: 1,
	}

	for key, value := range data {
		sortMap.m.Store(key, value)
		sortMap.keys = append(sortMap.keys, key)
	}

	return sortMap
}
func NewStringKeySortedMapWithBytesData(data map[string][]byte) *StringKeySortedMap {
	if len(data) == 0 {
		return NewStringKeySortedMap()
	}

	sortMap := &StringKeySortedMap{
		keys:       make([]string, 0),
		m:          sync.Map{},
		needResort: 1,
	}

	for key, value := range data {
		sortMap.m.Store(key, value)
		sortMap.keys = append(sortMap.keys, key)
	}

	return sortMap
}
func NewStringKeySortedMapWithInterfaceData(data map[string]interface{}) *StringKeySortedMap {
	if len(data) == 0 {
		return NewStringKeySortedMap()
	}

	sortMap := &StringKeySortedMap{
		keys:       make([]string, 0),
		m:          sync.Map{},
		needResort: 1,
	}

	for key, value := range data {
		sortMap.m.Store(key, value)
		sortMap.keys = append(sortMap.keys, key)
	}

	return sortMap
}

func (m *StringKeySortedMap) Put(key string, val interface{}) {
	_, exist := m.m.LoadOrStore(key, val)
	if exist {
		m.m.Store(key, val)
	} else {
		m.keysLock.Lock()
		m.keys = append(m.keys, key)
		m.keysLock.Unlock()
		atomic.CompareAndSwapUint32(&m.needResort, 0, 1)
	}
}

func (m *StringKeySortedMap) Get(key string) (val interface{}, ok bool) {
	return m.m.Load(key)
}

func (m *StringKeySortedMap) Contains(key string) (ok bool) {
	_, ok = m.m.Load(key)
	return
}

func (m *StringKeySortedMap) sort() {
	needResort := atomic.LoadUint32(&m.needResort)
	if needResort == 1 {
		m.keysLock.Lock()
		if atomic.LoadUint32(&m.needResort) == 1 {
			sort.Strings(m.keys)
			atomic.StoreUint32(&m.needResort, 0)
		}
		m.keysLock.Unlock()
	}
}

func (m *StringKeySortedMap) removeFromKeys(key string) {
	m.keysLock.Lock()
	if atomic.LoadUint32(&m.needResort) == 1 {
		sort.Strings(m.keys)
		atomic.StoreUint32(&m.needResort, 0)
	}
	length := len(m.keys)
	idx := sort.SearchStrings(m.keys, key)
	if idx != length {
		m.keys = append(m.keys[:idx], m.keys[idx+1:]...)
	}
	m.keysLock.Unlock()
}

func (m *StringKeySortedMap) Remove(key string) (val interface{}, ok bool) {
	val, ok = m.m.LoadAndDelete(key)
	if ok {
		m.removeFromKeys(key)
	}
	return
}

func (m *StringKeySortedMap) Range(f func(key string, val interface{}) (isContinue bool)) {
	m.sort()
	m.keysLock.RLock()
	keys := m.keys
	length := len(keys)
	if length == 0 {
		m.keysLock.RUnlock()
		return
	}
	tempKeys := make([]string, length)
	copy(tempKeys, keys)
	m.keysLock.RUnlock()
	for i := 0; i < length; i++ {
		k := tempKeys[i]
		v, ok := m.m.Load(k)
		if ok && !f(k, v) {
			break
		}
	}
}

func (m *StringKeySortedMap) Length() int {
	m.keysLock.RLock()
	defer m.keysLock.RUnlock()
	return len(m.keys)
}

type IntKeySortedMap struct {
	keysLock   sync.RWMutex
	keys       []int
	m          sync.Map
	needResort uint32
}

func NewIntKeySortedMap() *IntKeySortedMap {
	return &IntKeySortedMap{
		keys:       make([]int, 0),
		m:          sync.Map{},
		needResort: 0,
	}
}

func (m *IntKeySortedMap) Put(key int, val interface{}) {
	_, exist := m.m.LoadOrStore(key, val)
	if exist {
		m.m.Store(key, val)
	} else {
		m.keysLock.Lock()
		m.keys = append(m.keys, key)
		m.keysLock.Unlock()
		atomic.CompareAndSwapUint32(&m.needResort, 0, 1)
	}
}

func (m *IntKeySortedMap) Get(key int) (val interface{}, ok bool) {
	return m.m.Load(key)
}

func (m *IntKeySortedMap) Contains(key int) (ok bool) {
	_, ok = m.m.Load(key)
	return
}

func (m *IntKeySortedMap) sort() {
	needResort := atomic.LoadUint32(&m.needResort)
	if needResort == 1 {
		m.keysLock.Lock()
		if atomic.LoadUint32(&m.needResort) == 1 {
			sort.Ints(m.keys)
			atomic.StoreUint32(&m.needResort, 0)
		}
		m.keysLock.Unlock()
	}
}

func (m *IntKeySortedMap) removeFromKeys(key int) {
	m.keysLock.Lock()
	if atomic.LoadUint32(&m.needResort) == 1 {
		sort.Ints(m.keys)
		atomic.StoreUint32(&m.needResort, 0)
	}
	length := len(m.keys)
	idx := sort.SearchInts(m.keys, key)
	if idx != length {
		m.keys = append(m.keys[:idx], m.keys[idx+1:]...)
	}
	m.keysLock.Unlock()
}

func (m *IntKeySortedMap) Remove(key int) (val interface{}, ok bool) {
	val, ok = m.m.LoadAndDelete(key)
	if ok {
		m.removeFromKeys(key)
	}
	return
}

func (m *IntKeySortedMap) Range(f func(val interface{}) (isContinue bool)) {
	m.sort()
	m.keysLock.RLock()
	keys := m.keys
	length := len(keys)
	if length == 0 {
		m.keysLock.RUnlock()
		return
	}
	tempKeys := make([]int, length)
	copy(tempKeys, keys)
	m.keysLock.RUnlock()
	for i := 0; i < length; i++ {
		k := tempKeys[i]
		v, ok := m.m.Load(k)
		if ok && !f(v) {
			break
		}
	}
}

func (m *IntKeySortedMap) Length() int {
	m.keysLock.RLock()
	defer m.keysLock.RUnlock()
	return len(m.keys)
}

type Float64KeySortedMap struct {
	keysLock   sync.RWMutex
	keys       []float64
	m          sync.Map
	needResort uint32
}

func NewFloat64KeySortedMap() *Float64KeySortedMap {
	return &Float64KeySortedMap{
		keys:       make([]float64, 0),
		m:          sync.Map{},
		needResort: 0,
	}
}

func (m *Float64KeySortedMap) Put(key float64, val interface{}) {
	_, exist := m.m.LoadOrStore(key, val)
	if exist {
		m.m.Store(key, val)
	} else {
		m.keysLock.Lock()
		m.keys = append(m.keys, key)
		m.keysLock.Unlock()
		atomic.CompareAndSwapUint32(&m.needResort, 0, 1)
	}
}

func (m *Float64KeySortedMap) Get(key float64) (val interface{}, ok bool) {
	return m.m.Load(key)
}

func (m *Float64KeySortedMap) Contains(key int) (ok bool) {
	_, ok = m.m.Load(key)
	return
}

func (m *Float64KeySortedMap) sort() {
	needResort := atomic.LoadUint32(&m.needResort)
	if needResort == 1 {
		m.keysLock.Lock()
		if atomic.LoadUint32(&m.needResort) == 1 {
			sort.Float64s(m.keys)
			atomic.StoreUint32(&m.needResort, 0)
		}
		m.keysLock.Unlock()
	}
}

func (m *Float64KeySortedMap) removeFromKeys(key float64) {
	m.keysLock.Lock()
	if atomic.LoadUint32(&m.needResort) == 1 {
		sort.Float64s(m.keys)
		atomic.StoreUint32(&m.needResort, 0)
	}
	length := len(m.keys)
	idx := sort.SearchFloat64s(m.keys, key)
	if idx != length {
		m.keys = append(m.keys[:idx], m.keys[idx+1:]...)
	}
	m.keysLock.Unlock()
}

func (m *Float64KeySortedMap) Remove(key float64) (val interface{}, ok bool) {
	val, ok = m.m.LoadAndDelete(key)
	if ok {
		m.removeFromKeys(key)
	}
	return
}

func (m *Float64KeySortedMap) Range(f func(key float64, val interface{}) (isContinue bool)) {
	m.sort()
	m.keysLock.RLock()
	keys := m.keys
	length := len(keys)
	if length == 0 {
		m.keysLock.RUnlock()
		return
	}
	tempKeys := make([]float64, length)
	copy(tempKeys, keys)
	m.keysLock.RUnlock()
	for i := 0; i < length; i++ {
		k := tempKeys[i]
		v, ok := m.m.Load(k)
		if ok && !f(k, v) {
			break
		}
	}
}

func (m *Float64KeySortedMap) Length() int {
	m.keysLock.RLock()
	defer m.keysLock.RUnlock()
	return len(m.keys)
}
