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

func TestPktMarshalAndUnmarshal(t *testing.T) {
	p := &Pkt{
		seq:      1,
		pktTotal: 5,
		pktSeq:   2,
		bytes:    []byte{1, 2, 3, 4, 5},
	}
	res := p.Marshal()
	p2 := &Pkt{}
	err := p2.Unmarshal(res)
	require.Nil(t, err)
	require.True(t, p.seq == p2.seq)
	require.True(t, p.pktTotal == p2.pktTotal)
	require.True(t, p.pktSeq == p2.pktSeq)
	require.True(t, bytes.Equal(p.bytes, p2.bytes))

	p = &Pkt{
		seq:         1,
		pktTotal:    5,
		pktSeq:      2,
		protocolLen: 3,
		protocol:    []byte("ABC"),
		bytes:       []byte{1, 2, 3, 4, 5},
	}
	res = p.Marshal()
	p2 = &Pkt{}
	err = p2.Unmarshal(res)
	require.Nil(t, err)
	require.True(t, p.seq == p2.seq)
	require.True(t, p.pktTotal == p2.pktTotal)
	require.True(t, p.pktSeq == p2.pktSeq)
	require.True(t, bytes.Equal(p.bytes, p2.bytes))
}
