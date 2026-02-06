/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package pkt

import (
	"bytes"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

const (
	tmp = "TEMP"
)

func TestPktCache(t *testing.T) {
	bytesLen := 1000
	// prepare data
	data := make([]byte, bytesLen)
	for i := 0; i < bytesLen; i++ {
		data[i] = byte(i)
	}

	SetMinPktLen(10)
	// disassemble
	pktList, err := BytesDisassembler.DisassembleBytes(data, protocol)
	require.Nil(t, err)
	// run cache
	cache := NewPktCache()
	cache.Run()
	resC := cache.FullPktC()
	tempSeq := tmp
	// put into cache
	for i := range pktList {
		go func(pkt *Pkt) {
			cache.PutPkt(tempSeq, pkt)
		}(pktList[i])
	}
	// wait for collection complete
	timer := time.NewTimer(5 * time.Second)
	var res *FullPktResult
	select {
	case res = <-resC:
	case <-timer.C:
		t.Fatal("timeout")
	}
	// assemble
	data2, p2, err := BytesAssembler.AssembleBytes(res.PktList)
	require.Nil(t, err, "assemble pkt list failed")
	require.True(t, bytes.Equal(data, data2), "result mismatch")
	require.True(t, bytes.Equal(protocol, p2), "result mismatch")
}

func TestPktCacheTimeout(t *testing.T) {
	bytesLen := 1000
	// prepare data
	data := make([]byte, bytesLen)
	for i := 0; i < bytesLen; i++ {
		data[i] = byte(i)
	}

	SetMinPktLen(10)
	// disassemble
	pktList, err := BytesDisassembler.DisassembleBytes(data, protocol)
	require.Nil(t, err)
	// run cache
	cache := NewPktCache()
	cache.Run()
	resC := cache.fullPktNotifyC
	tempSeq := tmp
	// put into cache
	for i := 1; i < len(pktList); i++ {
		cache.PutPkt(tempSeq, pktList[i])
	}
	go func() {
		time.Sleep(15 * time.Second)
		cache.PutPkt(tempSeq, pktList[0])
	}()
	// wait for collection complete
	timer := time.NewTimer(20 * time.Second)
	select {
	case <-resC:
		t.Fatal("timeout mechanism not working")
	case <-timer.C:
	}
}
