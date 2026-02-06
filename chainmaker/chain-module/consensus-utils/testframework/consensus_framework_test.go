/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package testframework

import (
	"fmt"
	"os/exec"
	"testing"

	"github.com/golang/mock/gomock"

	consensus_utils "chainmaker.org/chainmaker/consensus-utils/v2"
	"chainmaker.org/chainmaker/consensus-utils/v2/wal_service"
	"chainmaker.org/chainmaker/logger/v2"
	"chainmaker.org/chainmaker/pb-go/v2/config"
	configPb "chainmaker.org/chainmaker/pb-go/v2/config"
	consensuspb "chainmaker.org/chainmaker/pb-go/v2/consensus"
	"chainmaker.org/chainmaker/protocol/v2"
	"github.com/stretchr/testify/require"
)

var (
	blockchainId     = "chain1"                                   //Chain ID
	nodeNums         = 4                                          //Default Node numbers
	ConsensusEngines = make([]protocol.ConsensusEngine, nodeNums) //ConsensusEngines for TF
	consensusType    = consensuspb.ConsensusType_TBFT             // Default consensusType
)

// TestOnlyConsensus_TBFT
func TestOnlyConsensus_TBFT(t *testing.T) {
	cmd := exec.Command("/bin/sh", "-c", "rm -rf chain1 default.*")
	err := cmd.Run()
	require.Nil(t, err)

	err = InitLocalConfigs()
	require.Nil(t, err)

	//SetTxSizeAndTxNum
	SetTxSizeAndTxNum(200, 10*1024)

	// init LocalConfig
	InitLocalConfig(nodeNums)

	//new mock controller
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	//create test_node_configs
	testNodeConfigs, err := CreateTestNodeConfig(ctrl, blockchainId, consensusType,
		nodeNums, nil, nil, func(cfg *configPb.ChainConfig) []byte { return nil })
	if err != nil {
		t.Errorf("%v", err)
	}

	//new CM Logger for consensus Engine
	cmLogger := logger.GetLogger(blockchainId)
	for i := 0; i < nodeNums; i++ {
		netService := NewNetServiceForTest()
		tc := &consensus_utils.ConsensusImplConfig{
			ChainId:     testNodeConfigs[i].ChainID,
			NodeId:      testNodeConfigs[i].NodeId,
			Ac:          testNodeConfigs[i].Ac,
			ChainConf:   testNodeConfigs[i].ChainConf,
			NetService:  netService,
			Signer:      testNodeConfigs[i].Signer,
			LedgerCache: testNodeConfigs[i].LedgerCache,
			MsgBus:      testNodeConfigs[i].MsgBus,
		}

		// set wal write mode to non
		if tc.ChainConf.ChainConfig().Consensus == nil {
			tc.ChainConf.ChainConfig().Consensus = &config.ConsensusConfig{
				ExtConfig: make([]*config.ConfigKeyValue, 0),
			}
		} else if tc.ChainConf.ChainConfig().Consensus.ExtConfig == nil {
			tc.ChainConf.ChainConfig().Consensus.ExtConfig = make([]*config.ConfigKeyValue, 0)
		}
		tc.ChainConf.ChainConfig().Consensus.ExtConfig = append(tc.ChainConf.ChainConfig().Consensus.ExtConfig,
			&config.ConfigKeyValue{
				Key:   wal_service.WALWriteModeKey,
				Value: fmt.Sprintf("%v", int(wal_service.NonWalWrite)),
			},
		)

		//consensus, errs := tbft.New(tc)
		//if errs != nil {
		//	t.Errorf("%v", errs)
		//}
		//
		//ConsensusEngines[i] = consensus

		// new CoreEngine
		CoreEngines[i] = NewCoreEngineForTest(testNodeConfigs[i], cmLogger)
	}

	l := &logger.LogConfig{
		SystemLog: logger.LogNodeConfig{
			FilePath:        "./default.log",
			LogLevelDefault: "DEBUG",
			LogLevels:       map[string]string{"consensus": "DEBUG", "core": "DEBUG", "net": "DEBUG"},
			LogInConsole:    false,
			ShowColor:       true,
		},
	}
	logger.SetLogConfig(l)

	_, err = NewTestClusterFramework(blockchainId, 1, nodeNums, testNodeConfigs, ConsensusEngines, CoreEngines)
	require.Nil(t, err)

	//tf.Start()
	//time.Sleep(10 * time.Second)
	//tf.Stop()

}
