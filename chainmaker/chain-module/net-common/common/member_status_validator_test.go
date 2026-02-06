/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package common

import (
	"errors"

	"github.com/stretchr/testify/assert"

	pbac "chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	"chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/config"
	"chainmaker.org/chainmaker/protocol/v2"
	"github.com/stretchr/testify/mock"

	"sync"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestMemberStatusValidator_AddPeerId(t *testing.T) {
	ms := &MemberStatusValidator{
		blockedPeerIds: sync.Map{},
	}
	ms.AddPeerId("peerId")
	_, ok := ms.blockedPeerIds.Load("peerId")
	require.True(t, ok)
}

func TestMemberStatusValidator_RemovePeerId(t *testing.T) {
	ms := &MemberStatusValidator{
		blockedPeerIds: sync.Map{},
	}
	ms.AddPeerId("peerId")
	_, ok := ms.blockedPeerIds.Load("peerId")
	require.True(t, ok)
	ms.RemovePeerId("peerId")
	_, ok = ms.blockedPeerIds.Load("peerId")
	require.False(t, ok)
}

func TestMemberStatusValidator_ContainsPeerId(t *testing.T) {
	ms := &MemberStatusValidator{
		blockedPeerIds: sync.Map{},
	}
	require.False(t, ms.ContainsPeerId("peerId"))
	ms.AddPeerId("peerId")
	require.True(t, ms.ContainsPeerId("peerId"))

}

func TestMemberStatusValidator_AddAC(t *testing.T) {
	ms := &MemberStatusValidator{
		accessControls: sync.Map{},
	}
	_, ok := ms.accessControls.Load("chain1")
	require.False(t, ok)
	ms.AddAC("chain1", new(MockAccessControlProvider))
	_, ok = ms.accessControls.Load("chain1")
	require.True(t, ok)
}

func TestMemberStatusValidator_ValidateMemberStatus(t *testing.T) {
	ms := &MemberStatusValidator{
		accessControls: sync.Map{},
	}
	var members []*pbac.Member
	chainId := "chain1"

	status, ok, err := ms.ValidateMemberStatus(members)
	require.Nil(t, err)
	require.False(t, ok)
	require.Empty(t, status)

	ms.AddAC(chainId, nil)
	status, ok, err = ms.ValidateMemberStatus(members)
	require.Nil(t, err)
	require.False(t, ok)
	require.Empty(t, status)

	mockACProvider := new(MockAccessControlProvider)
	ms.accessControls.Delete(chainId)
	ms.AddAC(chainId, mockACProvider)

	member1 := &pbac.Member{
		OrgId:      "org1",
		MemberType: 0,
		MemberInfo: nil,
	}
	members = append(members, member1)
	mockACProvider.On("GetMemberStatus", member1).Return(0, nil)
	status, b, err := ms.ValidateMemberStatus(members)
	require.Nil(t, err)
	require.True(t, b)
	require.NotEmpty(t, status)

	member2 := &pbac.Member{
		OrgId:      "org2",
		MemberType: 0,
		MemberInfo: nil,
	}
	members = append(members, member2)
	mockACProvider.On("GetMemberStatus", member2).Return(1, nil)
	status, b, err = ms.ValidateMemberStatus(members)
	require.Nil(t, err)
	require.False(t, b)
	require.Empty(t, status)

	member3 := &pbac.Member{
		OrgId:      "org3",
		MemberType: 0,
		MemberInfo: nil,
	}
	members = []*pbac.Member{member3}
	mockACProvider.On("GetMemberStatus", member3).Return(1, errors.New("err to get status"))
	status, b, err = ms.ValidateMemberStatus(members)
	assert.NoError(t, err)
	require.False(t, b)
	require.Empty(t, status)
}

func TestMemberStatusValidator_ValidateMemberStatusWithChain(t *testing.T) {
	ms := &MemberStatusValidator{
		accessControls: sync.Map{},
	}
	var members []*pbac.Member
	chainId := "chainId"
	ok, err := ms.ValidateMemberStatusWithChain(members, chainId)
	require.Nil(t, err)
	require.False(t, ok)

	ms.AddAC(chainId, nil)
	ok, err = ms.ValidateMemberStatusWithChain(members, chainId)
	require.Nil(t, err)
	require.False(t, ok)

	mockACProvider := new(MockAccessControlProvider)
	ms.accessControls.Delete(chainId)
	ms.AddAC(chainId, mockACProvider)

	member1 := &pbac.Member{
		OrgId:      "org1",
		MemberType: 0,
		MemberInfo: nil,
	}
	members = append(members, member1)
	mockACProvider.On("GetMemberStatus", member1).Return(1, errors.New("err to get status"))
	ok, err = ms.ValidateMemberStatusWithChain(members, chainId)
	require.Nil(t, err)
	require.False(t, ok)

	member2 := &pbac.Member{
		OrgId:      "org2",
		MemberType: 0,
		MemberInfo: nil,
	}
	members = []*pbac.Member{member2}
	mockACProvider.On("GetMemberStatus", member2).Return(1, nil)
	ok, err = ms.ValidateMemberStatusWithChain(members, chainId)
	require.Nil(t, err)
	require.False(t, ok)

	member3 := &pbac.Member{
		OrgId:      "org3",
		MemberType: 0,
		MemberInfo: nil,
	}
	members = []*pbac.Member{member3}
	mockACProvider.On("GetMemberStatus", member3).Return(0, nil)
	ok, err = ms.ValidateMemberStatusWithChain(members, chainId)
	require.Nil(t, err)
	require.True(t, ok)
}

var _ protocol.AccessControlProvider = (*MockAccessControlProvider)(nil)

type MockAccessControlProvider struct {
	mock.Mock
}

func (m *MockAccessControlProvider) GetHashAlg() string {
	panic("implement me")
}

func (m *MockAccessControlProvider) ValidateResourcePolicy(resourcePolicy *config.ResourcePolicy) bool {
	panic("implement me")
}

func (m *MockAccessControlProvider) LookUpPolicy(resourceName string) (*pbac.Policy, error) {
	panic("implement me")
}

func (m *MockAccessControlProvider) LookUpExceptionalPolicy(resourceName string) (*pbac.Policy, error) {
	panic("implement me")
}

func (m *MockAccessControlProvider) CreatePrincipal(resourceName string, endorsements []*common.EndorsementEntry, message []byte) (protocol.Principal, error) {
	panic("implement me")
}

func (m *MockAccessControlProvider) CreatePrincipalForTargetOrg(resourceName string, endorsements []*common.EndorsementEntry, message []byte, targetOrgId string) (protocol.Principal, error) {
	panic("implement me")
}

func (m *MockAccessControlProvider) GetValidEndorsements(principal protocol.Principal) ([]*common.EndorsementEntry, error) {
	panic("implement me")
}

func (m *MockAccessControlProvider) VerifyPrincipal(principal protocol.Principal) (bool, error) {
	panic("implement me")
}

func (m *MockAccessControlProvider) NewMember(member *pbac.Member) (protocol.Member, error) {
	panic("implement me")
}

func (m *MockAccessControlProvider) GetMemberStatus(member *pbac.Member) (pbac.MemberStatus, error) {
	args := m.Called(member)
	return pbac.MemberStatus(args.Int(0)), args.Error(1)
}

func (m *MockAccessControlProvider) VerifyRelatedMaterial(verifyType pbac.VerifyType, data []byte) (bool, error) {
	panic("implement me")
}

func (m *MockAccessControlProvider) GetAllPolicy() (map[string]*pbac.Policy, error) {
	panic("implement me")
}
