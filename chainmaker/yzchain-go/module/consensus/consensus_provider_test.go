/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package consensus

import (
	"fmt"
	"os"
	"path/filepath"
	"reflect"
	"testing"
	"time"

	"chainmaker.org/chainmaker/logger/v2"

	"chainmaker.org/chainmaker/common/v2/msgbus"
	//maxbft "chainmaker.org/chainmaker/consensus-maxbft/v2"
	raft "chainmaker.org/chainmaker/consensus-raft/v2"
	solo "chainmaker.org/chainmaker/consensus-solo/v2"
	tbft "chainmaker.org/chainmaker/consensus-tbft/v2"
	utils "chainmaker.org/chainmaker/consensus-utils/v2"
	"chainmaker.org/chainmaker/localconf/v2"
	consensuspb "chainmaker.org/chainmaker/pb-go/v2/consensus"
	"chainmaker.org/chainmaker/protocol/v2"
	"github.com/golang/mock/gomock"
)

const (
	id     = "QmQZn3pZCcuEf34FSvucqkvVJEvfzpNjQTk17HS6CYMR35"
	org1Id = "yz-org1"
)

type TestBlockchain struct {
	chainId       string
	msgBus        msgbus.MessageBus
	store         protocol.BlockchainStore
	coreEngine    protocol.CoreEngine
	identity      protocol.SigningMember
	ac            protocol.AccessControlProvider
	ledgerCache   protocol.LedgerCache
	proposalCache protocol.ProposalCache
	chainConf     protocol.ChainConf
	logger        protocol.Logger
}

func TestNewConsensusEngine(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	prePath := localconf.ChainMakerConfig.GetStorePath()
	defer func() {
		localconf.ChainMakerConfig.StorageConfig["store_path"] = prePath
	}()
	localconf.ChainMakerConfig.StorageConfig["store_path"] = filepath.Join(os.TempDir(), fmt.Sprintf("%d", time.Now().Nanosecond()))

	tests := []struct {
		name    string
		csType  consensuspb.ConsensusType
		want    protocol.ConsensusEngine
		wantErr bool
	}{
		{"new TBFT consensus engine",
			consensuspb.ConsensusType_TBFT,
			&tbft.ConsensusTBFTImpl{},
			false,
		},
		{"new SOLO consensus engine",
			consensuspb.ConsensusType_SOLO,
			&solo.ConsensusSoloImpl{},
			false,
		},
		{"new RAFT consensus engine",
			consensuspb.ConsensusType_RAFT,
			&raft.ConsensusRaftImpl{},
			false,
		},
		//{"new MAXBFT consensus engine",
		//	consensuspb.ConsensusType_MAXBFT,
		//	&maxbft.Maxbft{},
		//	false,
		//},
	}
	registerConsensuses()
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			bc := &TestBlockchain{}
			provider := GetConsensusProvider(bc.chainConf.ChainConfig().Consensus.Type)
			config := &utils.ConsensusImplConfig{
				ChainId:       bc.chainId,
				NodeId:        id,
				Ac:            bc.ac,
				Core:          bc.coreEngine,
				ChainConf:     bc.chainConf,
				Signer:        bc.identity,
				Store:         bc.store,
				LedgerCache:   bc.ledgerCache,
				ProposalCache: bc.proposalCache,
				MsgBus:        bc.msgBus,
				Logger:        bc.logger,
			}
			got, err := provider(config)
			if (err != nil) != tt.wantErr {
				t.Errorf("NewCoreEngine() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if reflect.TypeOf(got) != reflect.TypeOf(tt.want) {
				t.Errorf("NewCoreEngine() = %v, want %v", got, tt.want)
			}
		})
	}
}

func registerConsensuses() {
	// consensus
	RegisterConsensusProvider(
		consensuspb.ConsensusType_SOLO,
		func(config *utils.ConsensusImplConfig) (protocol.ConsensusEngine, error) {
			return solo.New(config)
		},
	)

	RegisterConsensusProvider(
		consensuspb.ConsensusType_RAFT,
		func(config *utils.ConsensusImplConfig) (protocol.ConsensusEngine, error) {
			return raft.New(config)
		},
	)

	RegisterConsensusProvider(
		consensuspb.ConsensusType_TBFT,
		func(config *utils.ConsensusImplConfig) (protocol.ConsensusEngine, error) {
			return tbft.New(config)
		},
	)

	//RegisterConsensusProvider(
	//	consensuspb.ConsensusType_MAXBFT,
	//	func(config *utils.ConsensusImplConfig) (protocol.ConsensusEngine, error) {
	//		return maxbft.New(config)
	//	},
	//)
}

func newMockLogger() protocol.Logger {
	return logger.GetLoggerByChain(logger.MODULE_CONSENSUS, "test_chain_id")
}
