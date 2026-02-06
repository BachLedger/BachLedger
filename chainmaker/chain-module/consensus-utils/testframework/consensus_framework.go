/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

// Package testframework is consensus test framework
package testframework

import (
	"errors"
	"fmt"
	"io/ioutil"

	"chainmaker.org/chainmaker/common/v2/crypto/asym"
	"chainmaker.org/chainmaker/common/v2/helper"

	"chainmaker.org/chainmaker/protocol/v2/mock"
	"github.com/golang/mock/gomock"

	"chainmaker.org/chainmaker/common/v2/msgbus"
	"chainmaker.org/chainmaker/localconf/v2"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	configPb "chainmaker.org/chainmaker/pb-go/v2/config"
	"chainmaker.org/chainmaker/pb-go/v2/consensus"
	consensusPb "chainmaker.org/chainmaker/pb-go/v2/consensus"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/utils/v2"

	"github.com/gogo/protobuf/proto"
)

const (
	hashType             = "SHA256"
	authType             = "identity"
	pathPrefix           = "./"
	privateKeyFilePrefix = pathPrefix + "config/%s/node/consensus1/consensus1.sign.key"
	certFilePrefix       = pathPrefix + "config/%s/node/consensus1/consensus1.sign.crt"
	rootFilePrefix       = pathPrefix + "config/%s/ca/ca.crt"
)

var (
	txSize = 2 * 1024
	txNum  = 20 * 1024

	consensusSlice = []consensus.ConsensusType{
		consensus.ConsensusType_TBFT,
		consensus.ConsensusType_RAFT,
	}

	//chain_config                      = &config.ChainConfig{}
	local_config = make(map[string]*localconf.CMConfig) // local_config[orgId] = CMConfigGetMember
	//memberIds from Cert file
	memberIds = []string{
		"42baa5cc60ae451aa9e44812c7894c29",
		"43ea0b97a82945449163348585aff19c",
		"5f294aa59e3c4bde85a8a764723f57ed",
		"fc730a0f3e7241d1b1173a4897af96e7",
		"30b02fc3ae0b4a31b51c4ae6b2dcd78d",
		"333a3bd1dda7481f9fbd356c853f06bf",
		"5321a9cdc8444275a3d8f79f53c60efd",
	}
	// mapper[certId]=nodeId
	mapper_certId_nodeId = map[string]string{
		"42baa5cc60ae451aa9e44812c7894c29": "QmV9wyvnGXtKauR2MV4bLndwfS4hnHkN6RhXMmEyLyRwqq",
		"43ea0b97a82945449163348585aff19c": "QmYjXpS5RtSiScjJVxzJNUo2XdfDbSoE1BaaSQG2BWLhej",
		"5f294aa59e3c4bde85a8a764723f57ed": "QmYhNgL59EQriiojax98a8HQnB4DPqdN44eRy3RCdgbNPn",
		"fc730a0f3e7241d1b1173a4897af96e7": "Qmd6RRKw83sQrf4oZJEhuhouz48eu9BT1nLKNGqKcpD6LL",
		"30b02fc3ae0b4a31b51c4ae6b2dcd78d": "QmNVStQFk7uPpaBGLr1eZABYfyazfMGTQ6SrXVniZ74W6i",
		"333a3bd1dda7481f9fbd356c853f06bf": "QmYQQsMAdcbkY2U1FER7ZWMhWENBNUZdhUEdSXWpp2H8AK",
		"5321a9cdc8444275a3d8f79f53c60efd": "QmcUDfpj57gJSmtapzpo2TatZSZmFfJSnbGza5cLkweESQ",
	}
	//org names for full path
	org_s = []string{
		"yz-org1.yzchain.org",
		"yz-org2.yzchain.org",
		"yz-org3.yzchain.org",
		"yz-org4.yzchain.org",
		"yz-org5.yzchain.org",
		"yz-org6.yzchain.org",
		"yz-org7.yzchain.org",
	}
	//node ids
	node_s = []string{
		"QmV9wyvnGXtKauR2MV4bLndwfS4hnHkN6RhXMmEyLyRwqq",
		"QmYjXpS5RtSiScjJVxzJNUo2XdfDbSoE1BaaSQG2BWLhej",
		"QmYhNgL59EQriiojax98a8HQnB4DPqdN44eRy3RCdgbNPn",
		"Qmd6RRKw83sQrf4oZJEhuhouz48eu9BT1nLKNGqKcpD6LL",
		"QmNVStQFk7uPpaBGLr1eZABYfyazfMGTQ6SrXVniZ74W6i",
		"QmYQQsMAdcbkY2U1FER7ZWMhWENBNUZdhUEdSXWpp2H8AK",
		"QmcUDfpj57gJSmtapzpo2TatZSZmFfJSnbGza5cLkweESQ",
	}
	//map[node_id]index
	map_nodeId_num = map[string]int{
		"QmV9wyvnGXtKauR2MV4bLndwfS4hnHkN6RhXMmEyLyRwqq": 1,
		"QmYjXpS5RtSiScjJVxzJNUo2XdfDbSoE1BaaSQG2BWLhej": 2,
		"QmYhNgL59EQriiojax98a8HQnB4DPqdN44eRy3RCdgbNPn": 3,
		"Qmd6RRKw83sQrf4oZJEhuhouz48eu9BT1nLKNGqKcpD6LL": 4,
		"QmNVStQFk7uPpaBGLr1eZABYfyazfMGTQ6SrXVniZ74W6i": 5,
		"QmYQQsMAdcbkY2U1FER7ZWMhWENBNUZdhUEdSXWpp2H8AK": 6,
		"QmcUDfpj57gJSmtapzpo2TatZSZmFfJSnbGza5cLkweESQ": 7,
	}

	//网络模块，配置本节点网络和其他节点信息
	nodeIndex = 0
	//ready channel for send read signal
	readyC = make(chan struct{})
)

// ####################################################################################################################
//                                           Test Framework use for Signle node mode
// ####################################################################################################################

// TestFramework struct
type TestFramework struct {
	chainId string
	node    *TestNode
}

// NewTestFramework new framework
func NewTestFramework(chainId string, index int, consensusType consensusPb.ConsensusType, tnc *TestNodeConfig,
	consensusEngines protocol.ConsensusEngine, coreEngine protocol.CoreEngine) (*TestFramework, error) {
	tf := &TestFramework{
		chainId: chainId,
		node:    &TestNode{},
	}
	nodeIndex = index
	testNode, err := NewTestNode(consensusType, tnc, consensusEngines, coreEngine)
	if err != nil {
		return nil, err
	}
	tf.node = testNode
	return tf, nil
}

// Start testFramework
func (tf *TestFramework) Start() {
	fmt.Println("======== Test Framework Start ========")
	tf.node.Start()
}

// Stop testFramework
func (tf *TestFramework) Stop() {
	tf.node.Stop()
	fmt.Println("======== Test Framework Stop ========")

}

// ####################################################################################################################
//                                           Cluster Framework use for multi process mode
// ####################################################################################################################

// TestClusterFramework struct
type TestClusterFramework struct {
	chainId string
	tns     []*ClusterTestNode
}

// NewTestClusterFramework new cluster framework
func NewTestClusterFramework(chainId string, consensusType consensusPb.ConsensusType,
	nodeNum int, tnc []*TestNodeConfig, consensusEngines []protocol.ConsensusEngine,
	coreEngines []protocol.CoreEngine) (*TestClusterFramework, error) {

	// create TestClusterFramework
	tf := &TestClusterFramework{
		chainId: chainId,
		tns:     make([]*ClusterTestNode, nodeNum),
	}
	// create test node
	for i := 0; i < nodeNum; i++ {
		var err error
		tf.tns[i], err = NewClusterTestNode(consensusType, tnc[i], consensusEngines[i], coreEngines[i], tf)
		if err != nil {
			return nil, err
		}
	}
	return tf, nil
}

// Start framework
func (tf *TestClusterFramework) Start() {
	fmt.Println("======== Cluster Framework Start ========")
	for _, tn := range tf.tns {
		tn.Start()
	}
}

// Stop framework
func (tf *TestClusterFramework) Stop() {
	for _, tn := range tf.tns {
		tn.Stop()
	}
	fmt.Println("======== Cluster Framework Stop ========")
}

// ####################################################################################################################
//                                                  Test Node
// ####################################################################################################################

// TestNode struct
type TestNode struct {
	chainId      string
	nodeId       string
	genesisBlock *commonPb.Block

	msgBus          msgbus.MessageBus
	coreEngine      protocol.CoreEngine
	consensusEngine protocol.ConsensusEngine
	netEngine       protocol.Net
	netService      protocol.NetService
}

// ClusterTestNode struct
type ClusterTestNode struct {
	chainId      string
	nodeId       string
	genesisBlock *commonPb.Block

	msgBus          msgbus.MessageBus
	coreEngine      protocol.CoreEngine
	consensusEngine protocol.ConsensusEngine
	netEngine       *NetEngineForTest
}

// TestNodeConfig Test Node Config
type TestNodeConfig struct {
	ChainID         string
	NodeId          string
	ConsensusType   consensusPb.ConsensusType
	GenesisBlock    *commonPb.Block
	Signer          protocol.SigningMember
	Ac              protocol.AccessControlProvider
	LedgerCache     protocol.LedgerCache
	ChainConf       protocol.ChainConf
	MsgBus          msgbus.MessageBus
	BlockchainStore protocol.BlockchainStore
	ProposalCache   protocol.ProposalCache
	ListenAddr      string
	Seeds           []string
}

// GetConsensusArg func
type GetConsensusArg func(cfg *configPb.ChainConfig) []byte

// CreateTestNodeConfig create config
func CreateTestNodeConfig(ctrl *gomock.Controller, chainId string, consensusType consensus.ConsensusType,
	nodeNums int, listenAddrs, seeds []string, fn GetConsensusArg) ([]*TestNodeConfig, error) {

	if listenAddrs != nil {
		ListenAddrs = listenAddrs
	}
	if seeds != nil {
		Seeds = seeds
	}

	testNodeConfigs := make([]*TestNodeConfig, nodeNums)

	// create test node
	for i := 0; i < nodeNums; i++ {
		chainconfig := InitChainConfig(chainId, consensusType, nodeNums)
		chainConfigBytes, err := proto.Marshal(chainconfig)
		if err != nil {
			return nil, err
		}
		// todo. the return val should be process.
		var maxbftConsensusVal []byte
		if fn != nil {
			maxbftConsensusVal = fn(chainconfig)
		}

		//mock chain store
		blockchainStore := newMockStore(ctrl, maxbftConsensusVal, chainConfigBytes)
		//mock chain config
		chainConfigForTest := newChainConfImplForTest(blockchainStore, chainconfig)

		// create genesis block
		genesisBlock, _, err := utils.CreateGenesis(chainconfig)
		if err != nil {
			return nil, fmt.Errorf("create chain [%s] genesis block failed, %s", chainId, err.Error())
		}
		// create ledgerCache
		ledgerCache := NewCache(chainId)
		ledgerCache.SetLastBlock(genesisBlock)

		//NewProposalCache
		proposalCache := NewProposalCache(chainConfigForTest, ledgerCache)

		//mock access control
		ac := newMockAccessControl(ctrl, i)
		//mock signer
		signer := newMockSigner(ctrl, i)

		config := &TestNodeConfig{
			ChainID:         chainId,
			NodeId:          node_s[i],
			ConsensusType:   consensusType,
			GenesisBlock:    genesisBlock,
			Signer:          signer,
			Ac:              ac,
			LedgerCache:     ledgerCache,
			ChainConf:       chainConfigForTest,
			MsgBus:          msgbus.NewMessageBus(),
			BlockchainStore: blockchainStore,
			ProposalCache:   proposalCache,
			Seeds:           Seeds,
			ListenAddr:      ListenAddrs[i],
		}

		testNodeConfigs[i] = config
	}
	return testNodeConfigs, nil
}

// NewTestNode new test node
func NewTestNode(consensusType consensusPb.ConsensusType, config *TestNodeConfig,
	ce protocol.ConsensusEngine, core protocol.CoreEngine) (*TestNode, error) {
	tn := &TestNode{
		chainId:      config.ChainID,
		nodeId:       config.NodeId,
		genesisBlock: config.GenesisBlock,
		msgBus:       config.MsgBus,
	}
	tn.coreEngine = core

	keyPath := pathPrefix + "config/" + org_s[nodeIndex] + "/node/consensus1/consensus1.tls.key"
	certPath := pathPrefix + "config/" + org_s[nodeIndex] + "/node/consensus1/consensus1.tls.crt"
	//1
	// read key file, then set the NodeId of local config
	file, err := ioutil.ReadFile(keyPath)
	if err != nil {
		fmt.Println("ReadFile Err: ", err)
	}
	privateKey, err := asym.PrivateKeyFromPEM(file, nil)
	if err != nil {
		fmt.Println("PrivateKeyFromPEM Err: ", err)
	}
	//get nodeId form private key
	nodeId, err := helper.CreateLibp2pPeerIdWithPrivateKey(privateKey)
	if err != nil {
		fmt.Println("CreateLibp2pPeerIdWithPrivateKey Err: ", err)
	}
	localconf.ChainMakerConfig.SetNodeId(nodeId)

	//new net
	tn.netEngine, err = NewNet(keyPath, certPath, config)
	if err != nil {
		fmt.Println("Net netEngine Err: ", err)
	}
	//start new engine
	err = tn.netEngine.Start()
	if err != nil {
		panic(err)
	}

	//new net service
	//new service need net
	tn.netService, err = NewNetService(tn.netEngine, BlockchainId, config.Ac, config.ChainConf, config.MsgBus)
	if err != nil {
		panic(err)
	}

	//start net Service. need net Engine started
	err = tn.netService.Start()
	if err != nil {
		panic(err)
	}

	if err != nil {
		msg := fmt.Sprintf("new net failed, %s", err.Error())
		return nil, errors.New(msg)
	}

	// create ConsensusEngine
	if !InConsensusSlice(consensusType, consensusSlice) {
		fmt.Println(consensusSlice)
		msg := fmt.Sprintf("only support TBFT,MAXBFT,RAFT and DPOS, Current: %d", consensusType)
		return nil, errors.New(msg)
	}
	tn.consensusEngine = ce
	return tn, nil
}

// NewClusterTestNode for cluster mode
func NewClusterTestNode(
	consensusType consensusPb.ConsensusType,
	config *TestNodeConfig,
	ce protocol.ConsensusEngine,
	core protocol.CoreEngine,
	tf *TestClusterFramework) (*ClusterTestNode, error) {

	tn := &ClusterTestNode{
		chainId:      config.ChainID,
		nodeId:       config.NodeId,
		genesisBlock: config.GenesisBlock,
		msgBus:       config.MsgBus,
	}

	tn.coreEngine = core
	// new newNetEngineForTest
	net := NewNetEngineForTest(config.ChainID, config.NodeId, config.MsgBus, tf)
	tn.netEngine = net

	if !InConsensusSlice(consensusType, consensusSlice) {
		fmt.Println(consensusSlice)
		msg := fmt.Sprintf("only support TBFT,MAXBFT,RAFT and DPOS, Current: %d", consensusType)
		return nil, errors.New(msg)
	}
	tn.consensusEngine = ce

	return tn, nil
}

// Start TestNode
func (tn *TestNode) Start() {
	fmt.Printf("-------- [nodeId:%d,%s] Node Start [listen:%s] --------\n",
		map_nodeId_num[tn.nodeId], tn.nodeId, ListenAddrs[nodeIndex])
	tn.coreEngine.Start()
	// close readyC for Start Net signal
	close(readyC)
	err := tn.consensusEngine.Start()
	if err != nil {
		panic(err)
	}
}

// Start ClusterTestNode
func (tn *ClusterTestNode) Start() {
	fmt.Printf("-------- [nodeId:%d,%s] Node Start --------\n", map_nodeId_num[tn.nodeId], tn.nodeId)
	tn.netEngine.Start()
	tn.coreEngine.Start()
	err := tn.consensusEngine.Start()
	if err != nil {
		panic(err)
	}
}

// Stop TestNode
func (tn *TestNode) Stop() {
	err := tn.consensusEngine.Stop()
	if err != nil {
		panic(err)
	}

	tn.coreEngine.Stop()
	//stop net Service before net engine
	_ = tn.netService.Stop()
	_ = tn.netEngine.Stop()
	fmt.Printf("-------- [nodeId:%d,%s] Node Stop --------\n", map_nodeId_num[tn.nodeId], tn.nodeId)
}

// Stop ClusterTestNode
func (tn *ClusterTestNode) Stop() {
	err := tn.consensusEngine.Stop()
	if err != nil {
		panic(err)
	}

	tn.coreEngine.Stop()
	tn.netEngine.Stop()
	fmt.Printf("-------- [nodeId:%d,%s] Node Stop --------\n", map_nodeId_num[tn.nodeId], tn.nodeId)
}

// ####################################################################################################################
//                                       impls NetService for TBFT         (only use GetNodeUidByCertId() method)
// ####################################################################################################################

// NewNetServiceForTest new net service
func NewNetServiceForTest() protocol.NetService {
	ctrl := gomock.NewController(nil)
	netService := mock.NewMockNetService(ctrl)

	//mock GetNodeUidByCertId only for TBFT
	netService.EXPECT().GetNodeUidByCertId(gomock.Any()).DoAndReturn(
		func(certId string) (string, error) {
			if nodeId, ok := mapper_certId_nodeId[certId]; ok {
				return nodeId, nil
			}
			return "", errors.New("certId is invalid")
		},
	).AnyTimes()

	return netService
}

// InConsensusSlice 判断共识类型是否在列表内
func InConsensusSlice(need consensus.ConsensusType, typeArr []consensus.ConsensusType) bool {
	for _, num := range typeArr {
		if num == need {
			return true
		}
	}
	return false
}

// ConsensusTypeFromInt 将配置文件int类型转化为ConsensusType类型
func ConsensusTypeFromInt(configType int32) (consensusPb.ConsensusType, error) {
	for _, consensusType := range consensusSlice {
		if int32(consensusType) == configType {
			return consensusType, nil
		}
	}
	return 0, errors.New("convert failed")
}
