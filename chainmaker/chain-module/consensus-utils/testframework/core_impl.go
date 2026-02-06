/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package testframework

import (
	"sync"

	"chainmaker.org/chainmaker/consensus-utils/v2/consistent_service"

	"chainmaker.org/chainmaker/common/v2/msgbus"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	consensusPb "chainmaker.org/chainmaker/pb-go/v2/consensus"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/utils/v2"
)

// ####################################################################################################################
//                                               impls Core Engine
// ####################################################################################################################

// CoreEngineForTest struct
type CoreEngineForTest struct {
	sync.Mutex
	chainId string
	nodeId  string

	consensusType consensusPb.ConsensusType
	msgBus        msgbus.MessageBus
	ledgerCache   protocol.LedgerCache
	// only for hotStuff
	store protocol.BlockchainStore
	log   protocol.Logger

	idle  bool
	quit  chan bool
	times map[uint64]*consensusTime
	//idleMu  sync.Mutex
	isProposer    chan bool
	proposalCache protocol.ProposalCache
	// channel to receive signal to yield propose block
	finishProposeC      chan bool
	proposedBlockHeight uint64

	consistentEngine consistent_service.ConsistentEngine
}

// GetBlockProposer func
func (ce *CoreEngineForTest) GetBlockProposer() protocol.BlockProposer {
	panic("implement me")
}

type consensusTime struct {
	start int64
	end   int64
}

// NewCoreEngineForTest new core engine
func NewCoreEngineForTest(tnc *TestNodeConfig, log protocol.Logger) *CoreEngineForTest {
	return &CoreEngineForTest{
		msgBus:        tnc.MsgBus,
		nodeId:        tnc.NodeId,
		chainId:       tnc.ChainID,
		ledgerCache:   tnc.LedgerCache,
		consensusType: tnc.ConsensusType,
		store:         tnc.BlockchainStore,
		proposalCache: tnc.ProposalCache,

		log:                 log,
		idle:                true,
		quit:                make(chan bool),
		times:               make(map[uint64]*consensusTime),
		isProposer:          make(chan bool),
		finishProposeC:      make(chan bool),
		proposedBlockHeight: 0,
		consistentEngine:    nil,
	}
}

// Start core engine
func (ce *CoreEngineForTest) Start() {
	ce.log.Infof("[nodeId:%d,%s] Core Engine Start", map_nodeId_num[ce.nodeId], ce.nodeId)
	ce.msgBus.Register(msgbus.ProposeState, ce)
	ce.msgBus.Register(msgbus.VerifyBlock, ce)
	ce.msgBus.Register(msgbus.CommitBlock, ce)
	ce.msgBus.Register(msgbus.TxPoolSignal, ce)
	// only for hotStuff
	ce.msgBus.Register(msgbus.BuildProposal, ce)

	if ce.consensusType == consensusPb.ConsensusType_TBFT ||
		ce.consensusType == consensusPb.ConsensusType_RAFT {
		ce.startProposingLoop()
	}
}

// Stop core engine
func (ce *CoreEngineForTest) Stop() {
	// 输出统计数据
	ce.CalcTime()

	ce.log.Infof("[nodeId:%d,%s] Core Engine Stop", map_nodeId_num[ce.nodeId], ce.nodeId)
}

// CalcTime for engine
func (ce *CoreEngineForTest) CalcTime() {
	blockNum := int64(0)
	totalTime := int64(0)
	for _, time := range ce.times {
		if time.start > 0 && time.end > 0 {
			blockNum++
			totalTime += time.end - time.start
		}
	}
	// 在raft下，从节点不会产生区块
	var (
		tps     = int64(0)
		avgTime = int64(0)
	)
	if blockNum > 0 {
		tps = blockNum * int64(txNum) * 1000 / totalTime
		avgTime = totalTime / blockNum
	}
	ce.log.Infof("### [nodeId:%d,%s] RESULT, blockNum:%d, totalTime:%dms, blockAvgTime:%dms,"+
		" TPS:%d\n ###", map_nodeId_num[ce.nodeId], ce.nodeId, blockNum, totalTime, avgTime, tps)
}

// GetBlockCommitter get block committer
func (ce *CoreEngineForTest) GetBlockCommitter() protocol.BlockCommitter {
	return newBlockCommitterForTest(ce.msgBus, ce.ledgerCache)
}

// GetBlockVerifier get block verifier
func (ce *CoreEngineForTest) GetBlockVerifier() protocol.BlockVerifier {
	return newBlockVerifierForTest(ce.proposalCache, ce.ledgerCache)
}

// OnMessage message buss
func (ce *CoreEngineForTest) OnMessage(message *msgbus.Message) {
	switch message.Topic {
	case msgbus.ProposeState:
		// 共识 raft, tbft, dpos 通过该方式生成区块
		if proposeStatus, ok := message.Payload.(bool); ok {
			// only leader can propose block
			if proposeStatus {
				currentHeight, err := ce.ledgerCache.CurrentHeight()
				if err != nil {
					ce.log.Errorf("[nodeId:%d,%s] get current height failed, err:%s",
						map_nodeId_num[ce.nodeId], ce.nodeId, err.Error())
				}
				ce.log.Infof("[nodeId:%d,%s] is proposer in height:%d",
					map_nodeId_num[ce.nodeId], ce.nodeId, currentHeight+1)
				ce.isProposer <- proposeStatus
			}
		}
	case msgbus.VerifyBlock:
		// 共识 tbft, dpos 通过该方式验证区块
		if block, ok := message.Payload.(*commonPb.Block); ok {
			ce.log.Infof("[nodeId:%d,%s] verify block height:%d", map_nodeId_num[ce.nodeId], ce.nodeId, block.Header.BlockHeight)
			ce.msgBus.Publish(msgbus.VerifyResult, &consensusPb.VerifyResult{
				VerifiedBlock: block, TxsRwSet: nil, Code: consensusPb.VerifyResult_SUCCESS, Msg: "OK"})
		}
	case msgbus.CommitBlock:
		// 共识 maxbft, tbft, dpos 通过该方式提交区块
		if block, ok := message.Payload.(*commonPb.Block); ok {
			ce.commitBlock(block)
		}

	}
}

// commitBlock
func (ce *CoreEngineForTest) commitBlock(block *commonPb.Block) {
	ce.log.Infof("[nodeId:%d,%s] commit block height:%d, block hash:%x",
		map_nodeId_num[ce.nodeId], ce.nodeId, block.Header.BlockHeight, block.Header.BlockHash)
	// 添加该块共识时间
	if v, ok := ce.times[block.Header.BlockHeight]; ok {
		v.end = utils.CurrentTimeMillisSeconds()
		ce.times[block.Header.BlockHeight].end = utils.CurrentTimeMillisSeconds()
	}

	// 缓存和提交该区块
	_ = ce.store.PutBlock(block, nil)
	ce.ledgerCache.SetLastBlock(block)
	ce.proposalCache.ClearProposedBlockAt(block.Header.BlockHeight)
	ce.msgBus.PublishSafe(msgbus.BlockInfo, &commonPb.BlockInfo{Block: block})
}

// OnQuit on quit
func (ce *CoreEngineForTest) OnQuit() {
	ce.log.Infof("[nodeId:%d,%s] Core Engine quit", map_nodeId_num[ce.nodeId], ce.nodeId)
}

// MaxbftHelper struct
type MaxbftHelper struct{}

// DiscardBlocks DiscardAboveHeight
func (mh *MaxbftHelper) DiscardBlocks(baseHeight uint64) {
}
