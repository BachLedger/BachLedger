/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package testframework

import (
	"errors"
	"fmt"
	"io/ioutil"
	"path/filepath"

	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/syscontract"
	"chainmaker.org/chainmaker/protocol/v2"
	"github.com/gogo/protobuf/proto"

	"chainmaker.org/chainmaker/localconf/v2"
	configPb "chainmaker.org/chainmaker/pb-go/v2/config"
	consensusPb "chainmaker.org/chainmaker/pb-go/v2/consensus"
)

// ####################################################################################################################
//                                         new ChainConfig, LocalConfig
// ####################################################################################################################

// SetTxSizeAndTxNum set tx size and tx num
func SetTxSizeAndTxNum(size, num int) {
	txSize = size
	txNum = num
	fmt.Printf("block size:%dMB\n", (txSize*txNum)/1024/1024)
}

// InitChainConfig init chain config
func InitChainConfig(chainId string, consensusType consensusPb.ConsensusType, nodeNum int) *configPb.ChainConfig {
	trustMemConfigs := []*configPb.TrustMemberConfig{}

	// new ConsensusConfig
	ns := make([]*configPb.OrgConfig, nodeNum)
	for i := 0; i < nodeNum; i++ {
		//build  node org config
		ns[i] = &configPb.OrgConfig{
			OrgId:  org_s[i],
			NodeId: []string{node_s[i]},
		}
		//build trust members
		trustMemConfig := &configPb.TrustMemberConfig{
			MemberInfo: memberIds[i],
			OrgId:      org_s[i],
			Role:       "Member",
			NodeId:     node_s[i],
		}
		trustMemConfigs = append(trustMemConfigs, trustMemConfig)

	}

	cc := &configPb.ChainConfig{
		ChainId:  chainId,
		Version:  "v1.2.0",
		AuthType: authType,
		Sequence: 1,
		Crypto:   &configPb.CryptoConfig{Hash: hashType},
		Block: &configPb.BlockConfig{
			BlockSize: 100,
		},
		Contract: &configPb.ContractConfig{
			EnableSqlSupport: false,
		},
		TrustMembers: trustMemConfigs,
	}

	cc.Consensus = &configPb.ConsensusConfig{
		Type:      consensusType,
		Nodes:     ns,
		ExtConfig: nil,
	}

	//build config for DPOS
	if consensusType == consensusPb.ConsensusType_DPOS {
		cc.Consensus.DposConfig = []*configPb.ConfigKeyValue{
			{
				Key:   "erc20.total",
				Value: "10000000",
			},
			{
				Key:   "erc20.owner",
				Value: "4WUXfiUpLkx7meaNu8TNS5rNM7YtZk6fkNWXihc54PbM",
			},
			{
				Key:   "erc20.decimals",
				Value: "18",
			},
			{
				Key:   "erc20.account:DPOS_STAKE",
				Value: "10000000",
			},
			{
				Key:   "stake.minSelfDelegation",
				Value: "2500000",
			},
			{
				Key:   "stake.epochValidatorNum",
				Value: "4",
			},
			{
				Key:   "stake.epochBlockNum",
				Value: "10",
			},
			{
				Key:   "stake.completionUnbondingEpochNum",
				Value: "1",
			},
			{
				Key:   "stake.candidate:6NbgYXzHhgigS8b4215iDiKxwekjkmgb8iXUqTSjC3Cm",
				Value: "2500000",
			},
			{
				Key:   "stake.candidate:3Lg6X7me2Ln2TkQchZwsJb7BRrtqoag4wwHJ2vsbeAoU",
				Value: "2500000",
			},
			{
				Key:   "stake.candidate:5Kn7aB2LLdurbtkrp1Gxvv69FLACroUqnHA2j3Wr1gW6",
				Value: "2500000",
			},
			{
				Key:   "stake.candidate:2LjvZJWcanVankmyzMKiYeoHQeTbsXM7VqG1bUwPfAkS",
				Value: "2500000",
			},
			{
				Key:   "stake.nodeID:6NbgYXzHhgigS8b4215iDiKxwekjkmgb8iXUqTSjC3Cm",
				Value: "QmV9wyvnGXtKauR2MV4bLndwfS4hnHkN6RhXMmEyLyRwqq",
			},
			{
				Key:   "stake.nodeID:3Lg6X7me2Ln2TkQchZwsJb7BRrtqoag4wwHJ2vsbeAoU",
				Value: "QmYhNgL59EQriiojax98a8HQnB4DPqdN44eRy3RCdgbNPn",
			},
			{
				Key:   "stake.nodeID:5Kn7aB2LLdurbtkrp1Gxvv69FLACroUqnHA2j3Wr1gW6",
				Value: "QmYjXpS5RtSiScjJVxzJNUo2XdfDbSoE1BaaSQG2BWLhej",
			},
			{
				Key:   "stake.nodeID:2LjvZJWcanVankmyzMKiYeoHQeTbsXM7VqG1bUwPfAkS",
				Value: "Qmd6RRKw83sQrf4oZJEhuhouz48eu9BT1nLKNGqKcpD6LL",
			},
		}
	}

	// new TrustRootConfig
	cc.TrustRoots = make([]*configPb.TrustRootConfig, nodeNum)
	for i := 0; i < nodeNum; i++ {
		absPath, _ := filepath.Abs(fmt.Sprintf(rootFilePrefix, org_s[i]))
		root, err := ioutil.ReadFile(absPath)
		if err != nil {
			panic(fmt.Errorf("init chainConfig failed, err:%s", err.Error()))
		}
		tr := &configPb.TrustRootConfig{
			OrgId: org_s[i],
			Root:  []string{string(root)},
		}
		cc.TrustRoots[i] = tr
	}
	return cc
}

// InitLocalConfig init local config
func InitLocalConfig(nodeNum int) {
	for i := 0; i < nodeNum; i++ {
		lc := &localconf.CMConfig{}

		lc.NodeConfig.Type = "full"
		lc.NodeConfig.CertFile = fmt.Sprintf(certFilePrefix, org_s[i])
		lc.NodeConfig.PrivKeyFile = fmt.Sprintf(privateKeyFilePrefix, org_s[i])
		lc.NodeConfig.PrivKeyPassword = ""
		lc.NodeConfig.AuthType = authType
		lc.NodeConfig.NodeId = node_s[i]
		lc.NodeConfig.OrgId = org_s[i]
		lc.NodeConfig.SignerCacheSize = 1000
		lc.NodeConfig.CertCacheSize = 1000

		lc.NodeConfig.P11Config.Enabled = false
		lc.NodeConfig.P11Config.Library = ""
		lc.NodeConfig.P11Config.Label = ""
		lc.NodeConfig.P11Config.Password = ""
		lc.NodeConfig.P11Config.SessionCacheSize = 10
		lc.NodeConfig.P11Config.Hash = hashType

		//lc.SetConsensusConfig(10, true)
		localconf.ChainMakerConfig = lc
		local_config[org_s[i]] = lc
	}
}

// ####################################################################################################################
//                                       impls ChainConf interface
// ####################################################################################################################

// ChainConfImplForTest chain config impl
type ChainConfImplForTest struct {
	ChainConf       *configPb.ChainConfig
	blockchainStore protocol.BlockchainStore
}

// newChainConfImplForTest
func newChainConfImplForTest(store protocol.BlockchainStore, cfg *configPb.ChainConfig) *ChainConfImplForTest {
	return &ChainConfImplForTest{
		ChainConf:       cfg,
		blockchainStore: store,
	}
}

// Init init
func (cc *ChainConfImplForTest) Init() error {
	// load chain config from store
	bytes, err := cc.blockchainStore.ReadObject(syscontract.SystemContract_CHAIN_CONFIG.String(),
		[]byte(syscontract.SystemContract_CHAIN_CONFIG.String()))
	if err != nil {
		return err
	}
	if len(bytes) == 0 {
		return errors.New("ChainConfig is empty")
	}
	var chainConfig configPb.ChainConfig
	err = proto.Unmarshal(bytes, &chainConfig)
	if err != nil {
		return err
	}

	cc.ChainConf = &chainConfig
	// compatible with versions before v1.1.1
	if cc.ChainConf.Contract == nil {
		cc.ChainConf.Contract = &configPb.ContractConfig{EnableSqlSupport: false} //by default disable sql support
	}
	return nil
}

// ChainConfig get chain config
func (cc *ChainConfImplForTest) ChainConfig() *configPb.ChainConfig {
	return cc.ChainConf
}

// GetChainConfigFromFuture get chainconfig
func (cc *ChainConfImplForTest) GetChainConfigFromFuture(blockHeight uint64) (*configPb.ChainConfig, error) {
	return cc.ChainConf, nil
}

// GetChainConfigAt get chain config at
func (cc *ChainConfImplForTest) GetChainConfigAt(blockHeight uint64) (*configPb.ChainConfig, error) {
	return cc.ChainConf, nil
}

// SetChainConfig set new chain config
func (cc *ChainConfImplForTest) SetChainConfig(chainConf *configPb.ChainConfig) error {
	cc.ChainConf = chainConf
	return nil
}

// GetConsensusNodeIdList get consensus node id list
func (cc *ChainConfImplForTest) GetConsensusNodeIdList() ([]string, error) {
	chainNodeList := make([]string, 0)
	for _, node := range cc.ChainConf.Consensus.Nodes {
		//for _, nid := range node.NodeId {
		chainNodeList = append(chainNodeList, node.NodeId...)
		//}
	}
	return chainNodeList, nil
}

// CompleteBlock complete block
func (cc *ChainConfImplForTest) CompleteBlock(block *commonPb.Block) error {
	return nil
}

// AddWatch add watcher
func (cc *ChainConfImplForTest) AddWatch(w protocol.Watcher) {}

// AddVmWatch add vm watcher
func (cc *ChainConfImplForTest) AddVmWatch(w protocol.VmWatcher) {}
