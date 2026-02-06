/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package testframework

import (
	"testing"

	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"

	"chainmaker.org/chainmaker/logger/v2"

	"github.com/stretchr/testify/require"

	"github.com/golang/mock/gomock"

	configPb "chainmaker.org/chainmaker/pb-go/v2/config"
)

func TestCoreImpl(t *testing.T) {
	ctrl := gomock.NewController(t)
	testNodeConfigs, err := CreateTestNodeConfig(ctrl, blockchainId, consensusType, 1,
		ListenAddrs, Seeds, func(cfg *configPb.ChainConfig) []byte { return nil })
	require.Nil(t, err)
	cmLogger := logger.GetLogger(blockchainId)
	coreEngine := NewCoreEngineForTest(testNodeConfigs[0], cmLogger)
	coreEngine.GetBlockCommitter()
	coreEngine.GetBlockVerifier()
	block := &commonPb.Block{
		Header: &commonPb.BlockHeader{
			ChainId:      "chain1",
			BlockHeight:  100,
			PreBlockHash: nil,
		},
		Txs: fetchTxBatch(txNum),
	}
	coreEngine.commitBlock(block)
	coreEngine.Start()
	defer coreEngine.Stop()

}
