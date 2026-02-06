/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package utils

import (
	"testing"

	"chainmaker.org/chainmaker/pb-go/v2/config"
	"chainmaker.org/chainmaker/pb-go/v2/consensus"
	"github.com/stretchr/testify/assert"
)

func TestCreateGenesis(t *testing.T) {

	chainConfig := &config.ChainConfig{ChainId: "chain1", Version: "v2.1.0", Crypto: &config.CryptoConfig{Hash: "SM3"}, Consensus: &config.ConsensusConfig{Type: consensus.ConsensusType_SOLO}}
	genesis, rwset, err := CreateGenesis(chainConfig)
	t.Log(genesis)
	for i, rw := range rwset {
		t.Logf("%d", i)
		for _, w := range rw.TxWrites {
			t.Logf("key:%s,value:%s", w.Key, w.Value)
		}
	}
	assert.Nil(t, err)
	assert.True(t, IsConfBlock(genesis))

}
func TestGetBlockHeaderVersion(t *testing.T) {
	tt := map[string]uint32{
		"v2.2.0":       2201,
		"v2.3.0_alpha": 2300,
		"v2.3.0":       2301,
		"v2.2.2":       2220,
		"v2.0.0":       20,
		"v2.2.0_alpha": 220,
		"v2.3.1":       2030100,
		"v2.3.1.7":     2030107,
		"v2.4.0_alpha": 2040000,
		"v2.4.0":       2040001,
		"2030100":      2030100,
	}
	for v, result := range tt {
		intV := GetBlockVersion(v)
		assert.EqualValues(t, result, intV, v)
	}
}
