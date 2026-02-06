/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package hash

import (
	"encoding/hex"
	"fmt"
	"testing"
	"time"

	"chainmaker.org/chainmaker/common/v2/random/uuid"
	"github.com/stretchr/testify/require"
)

const SHA256 = "SHA256"

func TestGetNextPowerOfTwo(t *testing.T) {
	require.Equal(t, 0, getNextPowerOfTwo(0))
	require.Equal(t, 1, getNextPowerOfTwo(1))
	require.Equal(t, 2, getNextPowerOfTwo(2))
	require.Equal(t, 4, getNextPowerOfTwo(3))
	require.Equal(t, 8, getNextPowerOfTwo(5))
}

//TODO
//func TestGetMerkleRoot(t *testing.T) {
//	ret, err := GetMerkleRoot(SHA256, nil)
//	require.Equal(t, []byte(nil), ret)
//	require.Nil(t, err)
//}

func TestTenMerkleTree(_ *testing.T) {
	count := 20000
	hashes := make([][]byte, count)
	for i := 0; i < count; i++ {
		hashes[i] = []byte(uuid.GetUUID())
	}
	tick0 := CurrentTimeMillisSeconds()
	_, _ = BuildMerkleTree(SHA256, hashes)
	tick1 := CurrentTimeMillisSeconds()
	_, _ = BuildMerkleTree(SHA256, hashes)
	tick2 := CurrentTimeMillisSeconds()
	fmt.Println(tick1 - tick0)
	fmt.Println(tick2 - tick1)
}

func TestPower(_ *testing.T) {
	fmt.Println(getNextPowerOfTen(20000))
	fmt.Println(getNextPowerOfTen(10000))
	fmt.Println(getNextPowerOfTen(10))
	fmt.Println(getNextPowerOfTen(99))
	fmt.Println(getNextPowerOfTen(9))
	fmt.Println(getNextPowerOfTen(2))
}

func TestGetMerklePath(t *testing.T) {
	count := 20000
	hashes := make([][]byte, count)
	for i := 0; i < count; i++ {
		hashes[i] = []byte(uuid.GetUUID())
	}

	merkleTree, err := BuildMerkleTree(SHA256, hashes)
	if err != nil {
		t.Error(err)
		return
	}

	paths := make([][]byte, 0)
	GetMerklePath(SHA256, hashes[100], merkleTree, &paths, false)

	for i, bytes := range paths {
		fmt.Println("i is:", i, "path is:", hex.EncodeToString(bytes))
	}

}

func CurrentTimeMillisSeconds() int64 {
	return time.Now().UnixNano() / 1e6
}
