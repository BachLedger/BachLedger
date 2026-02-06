/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package solo

import (
	"fmt"
	"os/exec"
	"testing"
	"time"

	"chainmaker.org/chainmaker/common/v2/crypto"
	msgbusmock "chainmaker.org/chainmaker/common/v2/msgbus/mock"
	consensusUtils "chainmaker.org/chainmaker/consensus-utils/v2"
	"chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/config"
	consensuspb "chainmaker.org/chainmaker/pb-go/v2/consensus"
	"chainmaker.org/chainmaker/protocol/v2/mock"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	"chainmaker.org/chainmaker/common/v2/msgbus"
)

var (
	chainId = "chain1"
	nodeId  = "QmV9wyvnGXtKauR2MV4bLndwfS4hnHkN6RhXMmEyLyRwqq"
)

func TestSoloConsensus(t *testing.T) {

	cmd := exec.Command("/bin/sh", "-c", "rm default.*")
	_ = cmd.Run()

	ctrl := gomock.NewController(t)

	signer := mock.NewMockSigningMember(ctrl)
	signer.EXPECT().Sign(gomock.Any(), gomock.Any()).AnyTimes().Return(nil, nil)

	mockMsgbus := msgbusmock.NewMockMessageBus(ctrl)
	mockMsgbus.EXPECT().Register(gomock.Any(), gomock.Any()).AnyTimes()
	mockMsgbus.EXPECT().Publish(gomock.Any(), gomock.Any()).AnyTimes()

	chainConfig := mock.NewMockChainConf(ctrl)
	chainConfig.EXPECT().ChainConfig().AnyTimes().Return(&config.ChainConfig{
		Crypto: &config.CryptoConfig{
			Hash: crypto.CRYPTO_ALGO_SHA256,
		},
	})

	cic := &consensusUtils.ConsensusImplConfig{
		ChainId:   chainId,
		NodeId:    nodeId,
		Signer:    signer,
		MsgBus:    mockMsgbus,
		ChainConf: chainConfig,
	}
	solo, err := New(cic)
	require.Nil(t, err)

	err = solo.Start()
	require.Nil(t, err)

	time.Sleep(4 * time.Second)

	block := createBlock(1)

	proposalBlock := &consensuspb.ProposalBlock{
		Block: block,
	}
	blockMsg := &msgbus.Message{
		Topic:   msgbus.ProposedBlock,
		Payload: proposalBlock,
	}
	solo.handleProposedBlock(blockMsg)

	result := &consensuspb.VerifyResult{
		VerifiedBlock: block,
		Code:          consensuspb.VerifyResult_SUCCESS,
	}
	verifyMsg := &msgbus.Message{
		Topic:   msgbus.VerifyResult,
		Payload: result,
	}
	solo.handleVerifyResult(verifyMsg)

	height := solo.GetLastHeight()
	require.EqualValues(t, 1, height)

	_, err = solo.GetValidators()
	require.Nil(t, err)

	_, err = solo.GetConsensusStateJSON()
	require.Nil(t, err)

	solo.CanProposeBlock()

	err = solo.Stop()
	require.Nil(t, err)

}

func createBlock(height uint64) *common.Block {

	block := &common.Block{
		Header: &common.BlockHeader{
			ChainId:     chainId,
			BlockHeight: height,
			Signature:   []byte(""),
			BlockHash:   []byte(""),
		},
		Dag: &common.DAG{},
		Txs: []*common.Transaction{
			{
				Payload: &common.Payload{
					ChainId: chainId,
				},
			},
		},
	}

	// blockHash := sha256.Sum256([]byte(fmt.Sprintf("%s-%d", chainid, height)))
	blockHash := []byte(fmt.Sprintf("%s-%d-%s", chainId, height, time.Now()))
	block.Header.BlockHash = blockHash[:]

	// txHash := sha256.Sum256([]byte(fmt.Sprintf("%s-%d", blockHash, 0)))
	// block.Txs[0].Payload.TxId = string(txHash[:])

	return block
}
