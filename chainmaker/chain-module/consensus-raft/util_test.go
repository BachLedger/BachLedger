/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
SPDX-License-Identifier: Apache-2.0
*/
package raft

import (
	"fmt"
	"testing"

	"chainmaker.org/chainmaker/pb-go/v2/common"
	"github.com/stretchr/testify/require"
)

func TestEntryFormatter(t *testing.T) {
	formatter := entryFormatter(nil)
	require.Equal(t, "empty entry", formatter)
	hash := []byte("test_hash")
	block := common.Block{
		Header: &common.BlockHeader{
			ChainId:     "chain1",
			BlockHeight: 1024,
			BlockHash:   hash,
		},
	}
	bytes := mustMarshal(&block)
	formatter = entryFormatter(bytes)
	require.Equal(t, formatter, fmt.Sprintf("block(1024-%x)", string(hash)))
}

func TestDescribeNodes(t *testing.T) {
	nodes := describeNodes([]uint64{0, 1, 2, 3})
	require.Equal(t, "[0, 1, 2, 3, ]", nodes)
}
