/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package pkt

import (
	"bytes"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestDisassembler_DisassembleBytes(t *testing.T) {
	bytesLen := 1000
	data := make([]byte, bytesLen)
	for i := 0; i < bytesLen; i++ {
		data[i] = byte(i + 1)
	}

	disassembler := &Disassembler{minPktLen: 10}
	pkts, err := disassembler.DisassembleBytes(data, protocol)
	require.Nil(t, err, "disassemble failed")
	require.True(t, len(pkts) == 100, "packet count not enough")
	res := make([]byte, bytesLen)
	idx := 0
	for i := range pkts {
		pkt := pkts[i]
		require.True(t, pkt.pktTotal == 100)
		require.True(t, pkt.pktSeq == uint8(i))
		copy(res[idx:], pkt.bytes)
		idx = idx + len(pkt.bytes)
	}
	require.True(t, bytes.Equal(data, res))

	bytesLen = 25501
	data = make([]byte, bytesLen)
	for i := 0; i < bytesLen; i++ {
		data[i] = byte(i + 1)
	}
	pkts, err = disassembler.DisassembleBytes(data, protocol)
	require.Nil(t, err, "disassemble failed")
	require.True(t, len(pkts) == 255, "packet count not enough")
	res = make([]byte, bytesLen)
	idx = 0
	for i := range pkts {
		pkt := pkts[i]
		require.True(t, pkt.pktTotal == 255)
		require.True(t, pkt.pktSeq == uint8(i))
		copy(res[idx:], pkt.bytes)
		idx = idx + len(pkt.bytes)
	}
	require.True(t, bytes.Equal(data, res))
}

var (
	l   = 20 << 20
	d   []byte
	dbl = &Disassembler{minPktLen: 100 << 10}
)

func init() {
	d = make([]byte, l)
}

func BenchmarkDisassembler_DisassembleBytes(b *testing.B) {
	for i := 0; i < b.N; i++ {
		b.ReportAllocs()
		_, _ = dbl.DisassembleBytes(d, protocol)
	}
}
