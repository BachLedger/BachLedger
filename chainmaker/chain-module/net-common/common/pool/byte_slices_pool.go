/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package pool

import "sync"

// ByteSlicesPool is a pool for byte-slices reuse.
type ByteSlicesPool struct {
	initialCap int
	p          sync.Pool
}

// NewByteSlicesPool create a new ByteSlicesPool instance.
// initialCap defines the cap value of new []byte.
func NewByteSlicesPool(initialCap int) *ByteSlicesPool {
	return &ByteSlicesPool{
		initialCap: initialCap,
		p: sync.Pool{
			New: func() interface{} {
				n := make([]byte, 0, initialCap)
				return &n
			},
		},
	}
}

// Get a []byte with 0 length and unfixed cap.
func (p *ByteSlicesPool) Get() *[]byte {
	res, _ := p.p.Get().(*[]byte)
	if cap(*res) == 0 {
		s := make([]byte, 0, p.initialCap)
		res = &s
	} else if len(*res) > 0 {
		*res = (*res)[:0]
	}
	return res
}

// GetWithLen return a []byte with the designated length and unfixed cap.
func (p *ByteSlicesPool) GetWithLen(l int) *[]byte {
	var bytes *[]byte

	res, _ := p.p.Get().(*[]byte)

	if cap(*res) < l {
		p.p.Put(res)
		s := make([]byte, l)
		bytes = &s

	} else {
		bytes = res
		for i := 0; i < l; i++ {
			*bytes = append(*bytes, byte(0))
		}
	}

	return bytes
}

// Put a []byte into pool. The slice will be reset to 0 length and its cap will not change.
func (p *ByteSlicesPool) Put(n *[]byte) {
	if cap(*n) == 0 {
		return
	}
	if len(*n) > 0 {
		*n = (*n)[:0]
	}
	p.p.Put(n)
}
