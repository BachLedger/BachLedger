package raft

import (
	"os"
	"os/exec"
	"testing"
	"time"

	"chainmaker.org/chainmaker/logger/v2"
	"github.com/golang/mock/gomock"

	consensus_utils "chainmaker.org/chainmaker/consensus-utils/v2"
	"chainmaker.org/chainmaker/consensus-utils/v2/testframework"
	consensuspb "chainmaker.org/chainmaker/pb-go/v2/consensus"
	"chainmaker.org/chainmaker/protocol/v2"
	"github.com/stretchr/testify/require"
)

var (
	nodeNum          = 3
	chainId          = "chain1"
	consensusType    = consensuspb.ConsensusType_RAFT
	ConsensusEngines = make([]protocol.ConsensusEngine, nodeNum)
	CoreEngines      = make([]protocol.CoreEngine, nodeNum)
)

func TestOnlyConsensus_RAFT(t *testing.T) {
	cmd := exec.Command("/bin/sh", "-c", "rm -rf chain1 default.*")
	err := cmd.Run()
	require.Nil(t, err)

	err = os.Mkdir("chain1", 0750)
	require.Nil(t, err)
	defer os.RemoveAll("chain1")

	err = testframework.InitLocalConfigs()
	require.Nil(t, err)
	defer testframework.RemoveLocalConfigs()

	testframework.SetTxSizeAndTxNum(300, 10000)

	// init chainConfig and LocalConfig
	testframework.InitChainConfig(chainId, consensusType, nodeNum)
	testframework.InitLocalConfig(nodeNum)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	testNodeConfig, err := testframework.CreateTestNodeConfig(ctrl, chainId, consensusType, nodeNum, nil, nil, nil)
	require.Nil(t, err)

	cmLogger := logger.GetLogger(chainId)
	for i := 0; i < nodeNum; i++ {
		// new CoreEngine
		CoreEngines[i] = testframework.NewCoreEngineForTest(testNodeConfig[i], cmLogger)
	}

	var consensus *ConsensusRaftImpl
	for i := 0; i < nodeNum; i++ {
		rc := &consensus_utils.ConsensusImplConfig{
			ChainId:     testNodeConfig[i].ChainID,
			NodeId:      testNodeConfig[i].NodeId,
			Ac:          testNodeConfig[i].Ac,
			Core:        CoreEngines[i],
			ChainConf:   testNodeConfig[i].ChainConf,
			Signer:      testNodeConfig[i].Signer,
			LedgerCache: testNodeConfig[i].LedgerCache,
			MsgBus:      testNodeConfig[i].MsgBus,
			Logger:      newMockLogger(),
		}

		consensus, err = New(rc)
		consensus.electionTick = 2
		if err != nil {
			require.Nil(t, err)
		}
		ConsensusEngines[i] = consensus
	}

	tf, err := testframework.NewTestClusterFramework(chainId, consensusType, nodeNum, testNodeConfig, ConsensusEngines, CoreEngines)
	require.Nil(t, err)

	tf.Start()
	time.Sleep(10 * time.Second)
	tf.Stop()

}
