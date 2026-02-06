/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package lockfreequeue

import (
	"fmt"
	"runtime"
	"sync/atomic"
)

type valuePit struct {
	val  interface{}
	rIdx uint32
	wIdx uint32
}

type Queue struct {
	capacity  uint32
	capMod    uint32
	readerIdx uint32
	writerIdx uint32
	pits      []*valuePit
}

func NewQueue(cap uint32) *Queue {
	q := new(Queue)
	q.capacity = minQuantity(cap)
	q.capMod = q.capacity - 1
	q.readerIdx = 0
	q.writerIdx = 0
	q.pits = make([]*valuePit, q.capacity)
	for i := range q.pits {
		q.pits[i] = new(valuePit)
	}
	return q
}

func (q *Queue) String() string {
	rIdx := atomic.LoadUint32(&q.readerIdx)
	wIdx := atomic.LoadUint32(&q.writerIdx)
	return fmt.Sprintf("Queue{capacity: %d, capMod: %d, readerIdx: %d, writerIdx: %d}", q.capacity, q.capMod, rIdx, wIdx)
}

func (q *Queue) Capacity() uint32 {
	return q.capacity
}

func (q *Queue) Quantity() uint32 {
	var wIdx, rIdx, currentQuantity uint32
	wIdx = atomic.LoadUint32(&q.writerIdx)
	rIdx = atomic.LoadUint32(&q.readerIdx)
	if rIdx > wIdx {
		currentQuantity = 0
	} else {
		currentQuantity = wIdx - rIdx
	}
	return currentQuantity
}

func (q *Queue) Push(val interface{}) (ok bool, quantity uint32) {
	var wIdxNew, currentQuantity uint32
	capMod := q.capMod
	currentQuantity = q.Quantity()
	if currentQuantity >= q.capacity {
		return false, currentQuantity
	}
	wIdxNew = atomic.AddUint32(&q.writerIdx, 1)
	idx := (wIdxNew - 1) & capMod
	pit := q.pits[idx]
	for {
		pwIdx := atomic.LoadUint32(&pit.wIdx)
		prIdx := atomic.LoadUint32(&pit.rIdx)
		if pwIdx == prIdx && (wIdxNew == pwIdx+q.capacity || pwIdx == 0) {
			pit.val = val
			atomic.StoreUint32(&pit.wIdx, wIdxNew)
			return true, currentQuantity + 1
		}
		runtime.Gosched()
	}
}

func (q *Queue) Pull() (val interface{}, ok bool, quantity uint32) {
	var rIdxNew, currentQuantity uint32
	capMod := q.capMod
	currentQuantity = q.Quantity()
	if currentQuantity < 1 {
		return nil, false, currentQuantity
	}
	rIdxNew = atomic.AddUint32(&q.readerIdx, 1)
	idx := (rIdxNew - 1) & capMod
	pit := q.pits[idx]
	for {
		pwIdx := atomic.LoadUint32(&pit.wIdx)
		prIdx := atomic.LoadUint32(&pit.rIdx)
		if rIdxNew == pwIdx && (rIdxNew == prIdx+q.capacity || prIdx == 0) {
			val = pit.val
			pit.val = nil
			atomic.StoreUint32(&pit.rIdx, rIdxNew)
			return val, true, currentQuantity - 1
		}
		runtime.Gosched()
	}
}

// round 到最近的2的幂值
func minQuantity(v uint32) uint32 {
	v--
	v |= v >> 1
	v |= v >> 2
	v |= v >> 4
	v |= v >> 8
	v |= v >> 16
	v++
	return v
}
