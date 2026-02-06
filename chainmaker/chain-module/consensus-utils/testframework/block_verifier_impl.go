/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package testframework

import (
	"fmt"

	consensuspb "chainmaker.org/chainmaker/pb-go/v2/consensus"

	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/protocol/v2"
)

// ####################################################################################################################
//
//	impls BlockVerifier for RAFT and HotStuff
//
// ####################################################################################################################
type blockVerifierForTest struct {
	proposalCache protocol.ProposalCache
	ledgerCache   protocol.LedgerCache
}

// newBlockVerifierForTest
func newBlockVerifierForTest(proposalCache protocol.ProposalCache,
	ledgerCache protocol.LedgerCache) *blockVerifierForTest {
	return &blockVerifierForTest{
		proposalCache: proposalCache,
		ledgerCache:   ledgerCache,
	}
}

// VerifyBlock raft, maxbft
func (b *blockVerifierForTest) VerifyBlock(block *commonPb.Block, mode protocol.VerifyMode) error {
	currentHeight, _ := b.ledgerCache.CurrentHeight()
	if currentHeight >= block.Header.BlockHeight {
		return fmt.Errorf("ErrBlockHadBeenCommited")
	}
	SetBlockToMockCache(block, string(block.Header.BlockHash), block.Header.BlockHeight)
	return b.proposalCache.SetProposedBlock(block, nil, nil, false)
}

// VerifyBlockWithRwSets
func (b *blockVerifierForTest) VerifyBlockSync(
	block *commonPb.Block, mode protocol.VerifyMode) (*consensuspb.VerifyResult, error) {
	return nil, nil
}

// VerifyBlockWithRwSets
func (b *blockVerifierForTest) VerifyBlockWithRwSets(block *commonPb.Block,
	rwsets []*commonPb.TxRWSet, mode protocol.VerifyMode) error {
	return nil
}
