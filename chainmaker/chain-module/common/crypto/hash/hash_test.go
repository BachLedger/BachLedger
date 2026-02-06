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

	"chainmaker.org/chainmaker/common/v2/crypto"
	"github.com/stretchr/testify/require"
)

func TestHash(t *testing.T) {
	data := []byte("js")
	testHash(t, crypto.HASH_TYPE_SM3, data, "036df5686d99cd847e9a2974d7bcb287fcdc6df004f1735cdf31089c8505b6f5")
	testHash(t, crypto.HASH_TYPE_SHA256, data, "16cedf80ade01c62bdd1ae931d0492330c0b62bf294c08c095ce2fab21a9298d")
	testHash(t, crypto.HASH_TYPE_SHA3_256, data, "7b942617fa4d27ad9cab6c175035827f53570353586583b648e4fa58b7221126")
}

func testHash(t *testing.T, hashType crypto.HashType, data []byte, expect string) {
	hash := Hash{
		hashType: hashType,
	}
	bytes, _ := hash.Get(data)
	fmt.Printf("%x\n", bytes)
	require.Equal(t, expect, hex.EncodeToString(bytes))
}

func TestGetHash(t *testing.T) {
	data := []byte("js")
	testGetHash(t, crypto.HASH_TYPE_SM3, data, "036df5686d99cd847e9a2974d7bcb287fcdc6df004f1735cdf31089c8505b6f5")
	testGetHash(t, crypto.HASH_TYPE_SHA256, data, "16cedf80ade01c62bdd1ae931d0492330c0b62bf294c08c095ce2fab21a9298d")
	testGetHash(t, crypto.HASH_TYPE_SHA3_256, data, "7b942617fa4d27ad9cab6c175035827f53570353586583b648e4fa58b7221126")
}

func testGetHash(t *testing.T, hashType crypto.HashType, data []byte, expect string) {
	bytes, _ := Get(hashType, data)
	fmt.Printf("%x\n", bytes)
	require.Equal(t, expect, hex.EncodeToString(bytes))
}

func TestGetHashSpeed(t *testing.T) {
	hashTimes := 1000000

	now := time.Now()
	hash := Hash{
		hashType: crypto.HASH_TYPE_SHA256,
	}
	for i := 0; i < hashTimes; i++ {
		hash.Get([]byte("this is test data"))
	}
	t.Logf("native hash. hash times: %d, time elapses: %s", hashTimes, time.Since(now))

	time.Sleep(time.Second * 2)

	now = time.Now()
	for i := 0; i < hashTimes; i++ {
		Get(crypto.HASH_TYPE_SHA256, []byte("this is test data"))
	}
	t.Logf("wrapped hash. hash times: %d, time elapses: %s", hashTimes, time.Since(now))
}
