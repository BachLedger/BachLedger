/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package testframework

import (
	"fmt"
	"sync"

	"chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	"chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/syscontract"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/protocol/v2/mock"
	"github.com/golang/mock/gomock"
)

var (
	//mockBlockCache store in memory
	mockBlockCacheByHash   = map[string]*common.Block{}
	mockBlockCacheByHeight = map[uint64]*common.Block{}
	//map[height]hash
	mockBlockCacheHeightToHash = map[uint64]string{}
)

// newMockSigner
func newMockSigner(ctrl *gomock.Controller, i int) protocol.SigningMember {
	signer := mock.NewMockSigningMember(ctrl)
	signer.EXPECT().Sign(gomock.Any(), gomock.Any()).Return([]byte("123"), nil).AnyTimes()
	//mock GetMember
	signer.EXPECT().GetMember().DoAndReturn(
		func() (*accesscontrol.Member, error) {
			return &accesscontrol.Member{
				OrgId:      org_s[i],
				MemberType: accesscontrol.MemberType_CERT,
				MemberInfo: []byte(memberIds[i]),
			}, nil
		}).AnyTimes()

	return signer
}

// newMockAccessControl
func newMockAccessControl(ctrl *gomock.Controller, i int) protocol.AccessControlProvider {
	ac := mock.NewMockAccessControlProvider(ctrl)
	ac.EXPECT().CreatePrincipal(gomock.Any(), gomock.Any(), gomock.Any()).Return(nil, nil).AnyTimes()
	ac.EXPECT().VerifyPrincipal(gomock.Any()).Return(true, nil).AnyTimes()
	//mock NewMember
	ac.EXPECT().NewMember(gomock.Any()).DoAndReturn(
		func(acMember *accesscontrol.Member) (protocol.Member, error) {
			member := newMockMember(ctrl, i)
			return member, nil
		}).AnyTimes()
	ac.EXPECT().GetMemberStatus(gomock.Any()).Return(accesscontrol.MemberStatus_NORMAL, nil).AnyTimes()
	return ac
}

// newMockMember
func newMockMember(ctrl *gomock.Controller, i int) protocol.Member {
	member := mock.NewMockMember(ctrl)
	member.EXPECT().GetMemberId().Return(memberIds[i]).AnyTimes()
	//mockk GetMember with nodeIndex
	member.EXPECT().GetMember().Return(&accesscontrol.Member{
		OrgId:      org_s[i],
		MemberType: accesscontrol.MemberType_CERT,
		MemberInfo: []byte(memberIds[i]),
	}, nil).AnyTimes()
	return member
}

// newMockStore
func newMockStore(ctrl *gomock.Controller, maxbftConsensusVal, chainConfVal []byte) protocol.BlockchainStore {
	store := mock.NewMockBlockchainStore(ctrl)
	content := sync.Map{}
	//Store configDBkey
	configDbKey := fmt.Sprintf("%s%s",
		syscontract.SystemContract_CHAIN_CONFIG.String(),
		syscontract.SystemContract_CHAIN_CONFIG.String())
	content.Store(configDbKey, chainConfVal)
	if len(maxbftConsensusVal) > 0 {
		consensusDbKey := fmt.Sprintf("%s%s",
			syscontract.SystemContract_GOVERNANCE.String(),
			syscontract.SystemContract_GOVERNANCE.String())
		content.Store(consensusDbKey, maxbftConsensusVal)
	}
	//GetBlockByHash from mock cache
	//mock cache store in memory
	store.EXPECT().GetBlockByHash(gomock.Any()).DoAndReturn(
		func(blockHash []byte) (*common.Block, error) {
			return GetBlockFromMockCache(string(blockHash), 0), nil
		}).AnyTimes()
	//GetBlock from mock cache
	store.EXPECT().GetBlock(gomock.Any()).DoAndReturn(
		func(height uint64) (*common.Block, error) {
			return GetBlockFromMockCache("", height), nil
		}).AnyTimes()

	//ReadObject Read from Store
	store.EXPECT().ReadObject(gomock.Any(), gomock.Any()).DoAndReturn(
		func(contractName string, key []byte) ([]byte, error) {
			dbKey := fmt.Sprintf("%s%s", contractName, key)
			val, ok := content.Load(dbKey)
			if ok {
				return val.([]byte), nil
			}
			return nil, fmt.Errorf("not find key: %s value", dbKey)
		}).AnyTimes()

	//PutBlock mock PutBlock
	store.EXPECT().PutBlock(gomock.Any(), gomock.Any()).Return(nil).AnyTimes()
	return store
}

// GetBlockFromMockCache 从Mock的Block缓存中获取Block
func GetBlockFromMockCache(blockHash string, height uint64) *common.Block {
	if blockHash != "" {
		return mockBlockCacheByHash[blockHash]
	}
	return mockBlockCacheByHeight[height]
}

// SetBlockToMockCache 将Block缓存到Mock缓存中
func SetBlockToMockCache(block *common.Block, blockHash string, height uint64) {
	//store block as blockHash
	if blockHash != "" {
		mockBlockCacheByHash[blockHash] = block
		height = block.Header.BlockHeight
	} else if height != 0 {
		//store block as height
		mockBlockCacheByHeight[height] = block
	}
	//store block map as map[height]hash
	mockBlockCacheHeightToHash[height] = string(block.Header.BlockHash)
	//only sore current block
	if height > 1 {
		delete(mockBlockCacheByHeight, height-1)
		oldBlockHash := mockBlockCacheHeightToHash[height-1]
		delete(mockBlockCacheByHash, oldBlockHash)
	}
}
