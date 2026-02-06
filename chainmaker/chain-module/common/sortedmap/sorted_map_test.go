/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package sortedmap

import (
	"fmt"
	"sync"
	"testing"
	"time"
)

func TestIntKeySortedMap(_ *testing.T) {
	m := NewIntKeySortedMap()
	m.Put(1, "a")
	fmt.Println("put 2")
	m.Put(2, "b")
	fmt.Println("put 1")
	m.Put(3, "c")
	fmt.Println("put 3")

	fmt.Printf("contains 1: %v", m.Contains(1))
	fmt.Printf("contains 2: %v", m.Contains(2))
	fmt.Printf("contains 3: %v", m.Contains(3))
	fmt.Printf("contains 4: %v", m.Contains(4))

	m.Range(func(val interface{}) (isContinue bool) {
		if val.(string) != "c" {
			isContinue = true
		}
		fmt.Printf("range v: %s, continue: %v", val, isContinue)
		return
	})

	v, ok := m.Remove(2)
	fmt.Printf("remove 2: v: %s, ok: %v", v, ok)
	v, ok = m.Remove(4)
	fmt.Printf("remove 4: v: %s, ok: %v", v, ok)

	fmt.Printf("contains 1: %v", m.Contains(1))
	fmt.Printf("contains 2: %v", m.Contains(2))
	fmt.Printf("contains 3: %v", m.Contains(3))
	fmt.Printf("contains 4: %v", m.Contains(4))

}

type lockMap struct {
	l sync.RWMutex
	m map[int]struct{}
}

func (m *lockMap) Put(i int) {
	m.l.Lock()
	m.m[i] = struct{}{}
	m.l.Unlock()
}

func (m *lockMap) Remove(i int) {
	m.l.Lock()
	delete(m.m, i)
	m.l.Unlock()
}

func (m *lockMap) Get(i int) struct{} {
	m.l.RLock()
	defer m.l.RUnlock()
	return m.m[i]
}

func TestMapPerformance(_ *testing.T) {
	m := &lockMap{
		l: sync.RWMutex{},
		m: make(map[int]struct{}),
	}
	var num = 200000
	var goroutine = 2000
	fmt.Println("times:", num)
	fmt.Println("goroutine num:", goroutine)

	useTime := lockMapPut(num, goroutine, m)
	fmt.Println("lock map put , use time:", useTime, "ns")

	useTime = lockMapGet(num, goroutine, m)
	fmt.Println("lock map get , use time:", useTime, "ns")

	useTime = lockMapDel(num, goroutine, m)
	fmt.Println("lock map del , use time:", useTime, "ns")

	m2 := sync.Map{}
	useTime = syncMapPut(num, goroutine, &m2)
	fmt.Println("sync map put , use time:", useTime, "ns")

	useTime = syncMapGet(num, goroutine, &m2)
	fmt.Println("sync map get , use time:", useTime, "ns")

	useTime = syncMapDel(num, goroutine, &m2)
	fmt.Println("sync map del , use time:", useTime, "ns")

	m3 := NewIntKeySortedMap()
	wg := sync.WaitGroup{}
	startTime := time.Now().UnixNano()
	for i := 0; i < 2000; i++ {
		wg.Add(1)
		go func(start int) {
			for j := start; j < start+goroutine; j++ {
				m3.Put(j, struct {
				}{})
			}
			wg.Done()
		}(num / goroutine * i)
	}

	wg.Wait()
	useTime = time.Now().UnixNano() - startTime
	fmt.Println("sort map put , use time:", useTime, "ns")

	wg = sync.WaitGroup{}
	startTime = time.Now().UnixNano()
	for i := 0; i < 2000; i++ {
		wg.Add(1)
		go func(start int) {
			for j := start; j < start+goroutine; j++ {
				m3.Get(j)
			}
			wg.Done()
		}(num / goroutine * i)
	}

	wg.Wait()
	useTime = time.Now().UnixNano() - startTime
	fmt.Println("sort map get , use time:", useTime, "ns")

	wg = sync.WaitGroup{}
	startTime = time.Now().UnixNano()
	for i := 0; i < 2000; i++ {
		wg.Add(1)
		go func(start int) {
			for j := start; j < start+goroutine; j++ {
				m3.Remove(j)
			}
			wg.Done()
		}(num / goroutine * i)
	}

	wg.Wait()
	useTime = time.Now().UnixNano() - startTime
	fmt.Println("sort map del , use time:", useTime, "ns")
}

func lockMapPut(num, goroutine int, m *lockMap) int64 {
	wg := sync.WaitGroup{}
	startTime := time.Now().UnixNano()
	for i := 0; i < 2000; i++ {
		wg.Add(1)
		go func(start int) {
			for j := start; j < start+goroutine; j++ {
				m.Put(j)
			}
			wg.Done()
		}(num / goroutine * i)
	}

	wg.Wait()
	return time.Now().UnixNano() - startTime
}

func lockMapGet(num, goroutine int, m *lockMap) int64 {
	wg := sync.WaitGroup{}
	startTime := time.Now().UnixNano()
	for i := 0; i < 2000; i++ {
		wg.Add(1)
		go func(start int) {
			for j := start; j < start+goroutine; j++ {
				m.Get(j)
			}
			wg.Done()
		}(num / goroutine * i)
	}

	wg.Wait()
	return time.Now().UnixNano() - startTime
}

func lockMapDel(num, goroutine int, m *lockMap) int64 {
	wg := sync.WaitGroup{}
	startTime := time.Now().UnixNano()
	for i := 0; i < 2000; i++ {
		wg.Add(1)
		go func(start int) {
			for j := start; j < start+goroutine; j++ {
				m.Remove(j)
			}
			wg.Done()
		}(num / goroutine * i)
	}

	wg.Wait()
	return time.Now().UnixNano() - startTime
}

func syncMapPut(num, goroutine int, m *sync.Map) int64 {
	wg := sync.WaitGroup{}
	startTime := time.Now().UnixNano()
	for i := 0; i < 2000; i++ {
		wg.Add(1)
		go func(start int) {
			for j := start; j < start+goroutine; j++ {
				m.Store(j, struct {
				}{})
			}
			wg.Done()
		}(num / goroutine * i)
	}

	wg.Wait()
	return time.Now().UnixNano() - startTime
}

func syncMapGet(num, goroutine int, m *sync.Map) int64 {
	wg := sync.WaitGroup{}
	startTime := time.Now().UnixNano()
	for i := 0; i < 2000; i++ {
		wg.Add(1)
		go func(start int) {
			for j := start; j < start+goroutine; j++ {
				m.Load(j)
			}
			wg.Done()
		}(num / goroutine * i)
	}

	wg.Wait()
	return time.Now().UnixNano() - startTime
}

func syncMapDel(num, goroutine int, m *sync.Map) int64 {
	wg := sync.WaitGroup{}
	startTime := time.Now().UnixNano()
	for i := 0; i < 2000; i++ {
		wg.Add(1)
		go func(start int) {
			for j := start; j < start+goroutine; j++ {
				m.Delete(j)
			}
			wg.Done()
		}(num / goroutine * i)
	}

	wg.Wait()
	return time.Now().UnixNano() - startTime
}
