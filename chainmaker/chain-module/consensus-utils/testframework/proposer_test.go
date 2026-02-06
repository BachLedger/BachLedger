/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package testframework

import (
	"testing"
	"time"

	consensusPb "chainmaker.org/chainmaker/pb-go/v2/consensus"

	"chainmaker.org/chainmaker/logger/v2"
	configPb "chainmaker.org/chainmaker/pb-go/v2/config"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"
)

// TestProposal
func TestProposal(t *testing.T) {
	ctrl := gomock.NewController(t)
	testNodeConfigs, err := CreateTestNodeConfig(ctrl, blockchainId, consensusType,
		1, nil, nil, func(cfg *configPb.ChainConfig) []byte { return nil })
	require.Nil(t, err)
	cmLogger := logger.GetLogger(blockchainId)
	coreEngine := NewCoreEngineForTest(testNodeConfigs[0], cmLogger)
	coreEngine.startProposingLoop()
	coreEngine.consensusType = consensusPb.ConsensusType_TBFT
	coreEngine.isProposer <- true
	time.Sleep(time.Microsecond * 1)
	coreEngine.consensusType = consensusPb.ConsensusType_RAFT
	coreEngine.isProposer <- true
	time.Sleep(time.Microsecond * 1)
	coreEngine.consensusType = consensusPb.ConsensusType_MAXBFT
	coreEngine.isProposer <- true
	time.Sleep(time.Microsecond * 1)
	coreEngine.consensusType = consensusPb.ConsensusType_DPOS
	coreEngine.isProposer <- true

}
