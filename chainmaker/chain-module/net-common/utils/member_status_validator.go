/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package utils

import (
	"chainmaker.org/chainmaker/net-common/common"
	pbac "chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
)

// MemberStatusValidateWithCertMode check the member status in the cert mode
func MemberStatusValidateWithCertMode(
	memberStatusValidator *common.MemberStatusValidator,
	certBytes []byte) (chainIds []string, passed bool, err error) {
	m := &pbac.Member{
		OrgId:      "",
		MemberType: pbac.MemberType_CERT,
		MemberInfo: certBytes,
	}
	return memberStatusValidator.ValidateMemberStatus([]*pbac.Member{m})
}

// ChainMemberStatusValidateWithCertMode check the member status in the cert mode with the chain
func ChainMemberStatusValidateWithCertMode(
	chainId string,
	memberStatusValidator *common.MemberStatusValidator,
	certBytes []byte) (passed bool, err error) {
	m := &pbac.Member{
		OrgId:      "",
		MemberType: pbac.MemberType_CERT,
		MemberInfo: certBytes,
	}
	return memberStatusValidator.ValidateMemberStatusWithChain([]*pbac.Member{m}, chainId)
}
