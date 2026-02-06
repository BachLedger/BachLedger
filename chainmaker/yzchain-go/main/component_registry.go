/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package main

import (
	"chainmaker.org/chainmaker-go/module/consensus"
	"chainmaker.org/chainmaker-go/module/txpool"
	"chainmaker.org/chainmaker-go/module/vm"
	raft "chainmaker.org/chainmaker/consensus-raft/v2"
	solo "chainmaker.org/chainmaker/consensus-solo/v2"
	tbft "chainmaker.org/chainmaker/consensus-tbft/v2"
	utils "chainmaker.org/chainmaker/consensus-utils/v2"
	"chainmaker.org/chainmaker/localconf/v2"
	"chainmaker.org/chainmaker/logger/v2"
	consensusPb "chainmaker.org/chainmaker/pb-go/v2/consensus"
	"chainmaker.org/chainmaker/protocol/v2"
	batch "chainmaker.org/chainmaker/txpool-batch/v2"
	normal "chainmaker.org/chainmaker/txpool-normal/v2"
	goEngine "chainmaker.org/chainmaker/vm-engine/v2"
	evm "chainmaker.org/chainmaker/vm-evm/v2"
)

func init() {
	// txPool
	txpool.RegisterTxPoolProvider(normal.TxPoolType, normal.NewNormalPool)
	txpool.RegisterTxPoolProvider(batch.TxPoolType, batch.NewBatchTxPool)

	// vm

	vm.RegisterVmProvider(
		"EVM",
		func(chainId string, configs map[string]interface{}) (protocol.VmInstancesManager, error) {
			return &evm.InstancesManager{}, nil
		})

	// chainId string, logger protocol.Logger, vmConfig map[string]interface{}
	vm.RegisterVmProvider(
		"GO",
		func(chainId string, configs map[string]interface{}) (protocol.VmInstancesManager, error) {
			return goEngine.NewInstancesManager(
				chainId,
				logger.GetLoggerByChain(logger.MODULE_VM, chainId),
				localconf.ChainMakerConfig.VMConfig.Go,
			), nil
		})

	// consensus
	consensus.RegisterConsensusProvider(
		consensusPb.ConsensusType_SOLO,
		func(config *utils.ConsensusImplConfig) (protocol.ConsensusEngine, error) {
			return solo.New(config)
		},
	)

	consensus.RegisterConsensusProvider(
		consensusPb.ConsensusType_RAFT,
		func(config *utils.ConsensusImplConfig) (protocol.ConsensusEngine, error) {
			return raft.New(config)
		},
	)

	consensus.RegisterConsensusProvider(
		consensusPb.ConsensusType_TBFT,
		func(config *utils.ConsensusImplConfig) (protocol.ConsensusEngine, error) {
			return tbft.New(config)
		},
	)

}
