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

func TestAssemblerAssembleBytes(t *testing.T) {
	pktList := make([]*Pkt, 100)
	res := make([]byte, 100)
	for i := 0; i < 100; i++ {
		pktList[i] = &Pkt{
			seq:      1,
			pktTotal: 100,
			pktSeq:   uint8(i),
			protocol: protocol,
			bytes:    []byte{byte(i)},
		}
		if i == 99 {
			pktList[i].protocolLen = uint8(len(protocol))
			pktList[i].protocol = protocol
		}
		res[i] = byte(i)
	}

	a := &Assembler{}
	res2, protocol2, err := a.AssembleBytes(pktList)
	require.Nil(t, err)
	require.True(t, bytes.Equal(res, res2))
	require.True(t, bytes.Equal(protocol, protocol2))
}

var (
	ps       []*Pkt
	adl      = &Assembler{}
	protocol = []byte("TEST")
)

func init() {
	d = make([]byte, l)
	ps, _ = dbl.DisassembleBytes(d, protocol)
}

func BenchmarkAssemblerAssembleBytes(b *testing.B) {
	for i := 0; i < b.N; i++ {
		b.ReportAllocs()
		ps2 := ps
		_, _, err := adl.AssembleBytes(ps2)
		if err != nil {
			b.Fatal(err)
		}
	}
}
