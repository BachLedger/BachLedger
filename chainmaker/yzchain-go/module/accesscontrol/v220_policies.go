package accesscontrol

import (
	"chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/syscontract"
	"chainmaker.org/chainmaker/protocol/v2"
)

func (acs *accessControlService) createDefaultResourcePolicy_220(localOrgId string) {

	policyArchive.orgList = []string{localOrgId}

	acs.resourceNamePolicyMap220.Store(protocol.ResourceNameReadData, policyRead)
	acs.resourceNamePolicyMap220.Store(protocol.ResourceNameWriteData, policyWrite)
	acs.resourceNamePolicyMap220.Store(protocol.ResourceNameUpdateSelfConfig, policySelfConfig)
	acs.resourceNamePolicyMap220.Store(protocol.ResourceNameUpdateConfig, policyConfig)
	acs.resourceNamePolicyMap220.Store(protocol.ResourceNameConsensusNode, policyConsensus)
	acs.resourceNamePolicyMap220.Store(protocol.ResourceNameP2p, policyP2P)

	// only used for test
	acs.resourceNamePolicyMap220.Store(protocol.ResourceNameAllTest, policyAllTest)
	acs.resourceNamePolicyMap220.Store("test_2", policyLimitTestAny)
	acs.resourceNamePolicyMap220.Store("test_2_admin", policyLimitTestAdmin)
	acs.resourceNamePolicyMap220.Store("test_3/4", policyPortionTestAny)
	acs.resourceNamePolicyMap220.Store("test_3/4_admin", policyPortionTestAnyAdmin)

	// for txtype
	acs.resourceNamePolicyMap220.Store(common.TxType_QUERY_CONTRACT.String(), policyRead)
	acs.resourceNamePolicyMap220.Store(common.TxType_INVOKE_CONTRACT.String(), policyWrite)
	acs.resourceNamePolicyMap220.Store(common.TxType_SUBSCRIBE.String(), policySubscribe)
	acs.resourceNamePolicyMap220.Store(common.TxType_ARCHIVE.String(), policyArchive)

	// exceptional resourceName opened for light user
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_BLOCK_BY_HEIGHT.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_BLOCK_WITH_TXRWSETS_BY_HEIGHT.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_BLOCK_BY_HASH.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_BLOCK_WITH_TXRWSETS_BY_HASH.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_BLOCK_BY_TX_ID.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_TX_BY_TX_ID.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_LAST_CONFIG_BLOCK.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_LAST_BLOCK.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_FULL_BLOCK_BY_HEIGHT.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_BLOCK_HEIGHT_BY_TX_ID.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_BLOCK_HEIGHT_BY_HASH.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_BLOCK_HEADER_BY_HEIGHT.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_ARCHIVED_BLOCK_HEIGHT.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_GET_CHAIN_CONFIG.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERTS_QUERY.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERT_ADD.String(), policySpecialWrite)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERTS_ALIAS_QUERY.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERT_ALIAS_ADD.String(), policySpecialWrite)

	// system contract interface resource definitions
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_CORE_UPDATE.String(), policyConfig)

	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_BLOCK_UPDATE.String(), policyConfig)

	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_TRUST_ROOT_ADD.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_TRUST_ROOT_DELETE.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_TRUST_ROOT_UPDATE.String(), policySelfConfig)

	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_TRUST_MEMBER_ADD.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_TRUST_MEMBER_DELETE.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_TRUST_MEMBER_UPDATE.String(), policyConfig)

	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_NODE_ID_ADD.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_NODE_ID_DELETE.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_NODE_ID_UPDATE.String(), policySelfConfig)

	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_NODE_ORG_ADD.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_NODE_ORG_UPDATE.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_NODE_ORG_DELETE.String(), policyConfig)

	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_CONSENSUS_EXT_ADD.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_CONSENSUS_EXT_UPDATE.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_CONSENSUS_EXT_DELETE.String(), policyConfig)

	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_PERMISSION_ADD.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_PERMISSION_UPDATE.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_PERMISSION_DELETE.String(), policyConfig)

	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CONTRACT_MANAGE.String()+"-"+
		syscontract.ContractManageFunction_INIT_CONTRACT.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CONTRACT_MANAGE.String()+"-"+
		syscontract.ContractManageFunction_UPGRADE_CONTRACT.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CONTRACT_MANAGE.String()+"-"+
		syscontract.ContractManageFunction_FREEZE_CONTRACT.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CONTRACT_MANAGE.String()+"-"+
		syscontract.ContractManageFunction_UNFREEZE_CONTRACT.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CONTRACT_MANAGE.String()+"-"+
		syscontract.ContractManageFunction_REVOKE_CONTRACT.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CONTRACT_MANAGE.String()+"-"+
		syscontract.ContractManageFunction_GRANT_CONTRACT_ACCESS.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CONTRACT_MANAGE.String()+"-"+
		syscontract.ContractManageFunction_REVOKE_CONTRACT_ACCESS.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CONTRACT_MANAGE.String()+"-"+
		syscontract.ContractManageFunction_VERIFY_CONTRACT_ACCESS.String(), policyConfig)

	// certificate management
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERTS_FREEZE.String(), policyAdmin)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERTS_UNFREEZE.String(), policyAdmin)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERTS_DELETE.String(), policyAdmin)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERTS_REVOKE.String(), policyAdmin)
	// for cert_alias
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERT_ALIAS_UPDATE.String(), policyAdmin)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERTS_ALIAS_DELETE.String(), policyAdmin)

}

func (acs *accessControlService) createDefaultResourcePolicyForPK_220(localOrgId string) {

	policyArchive.orgList = []string{localOrgId}

	acs.resourceNamePolicyMap220.Store(protocol.ResourceNameReadData, policyRead)
	acs.resourceNamePolicyMap220.Store(protocol.ResourceNameWriteData, policyWrite)
	acs.resourceNamePolicyMap220.Store(protocol.ResourceNameUpdateSelfConfig, policySelfConfig)
	acs.resourceNamePolicyMap220.Store(protocol.ResourceNameUpdateConfig, policyConfig)
	acs.resourceNamePolicyMap220.Store(protocol.ResourceNameConsensusNode, policyConsensus)
	acs.resourceNamePolicyMap220.Store(protocol.ResourceNameP2p, policyP2P)

	// only used for test
	acs.resourceNamePolicyMap220.Store(protocol.ResourceNameAllTest, policyAllTest)
	acs.resourceNamePolicyMap220.Store("test_2", policyLimitTestAny)
	acs.resourceNamePolicyMap220.Store("test_2_admin", policyLimitTestAdmin)
	acs.resourceNamePolicyMap220.Store("test_3/4", policyPortionTestAny)
	acs.resourceNamePolicyMap220.Store("test_3/4_admin", policyPortionTestAnyAdmin)

	// for txtype
	acs.resourceNamePolicyMap220.Store(common.TxType_QUERY_CONTRACT.String(), policyRead)
	acs.resourceNamePolicyMap220.Store(common.TxType_INVOKE_CONTRACT.String(), policyWrite)
	acs.resourceNamePolicyMap220.Store(common.TxType_SUBSCRIBE.String(), policySubscribe)
	acs.resourceNamePolicyMap220.Store(common.TxType_ARCHIVE.String(), policyArchive)

	// exceptional resourceName opened for light user
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_BLOCK_BY_HEIGHT.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_BLOCK_WITH_TXRWSETS_BY_HEIGHT.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_BLOCK_BY_HASH.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_BLOCK_WITH_TXRWSETS_BY_HASH.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_BLOCK_BY_TX_ID.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_TX_BY_TX_ID.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_LAST_CONFIG_BLOCK.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_LAST_BLOCK.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_FULL_BLOCK_BY_HEIGHT.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_BLOCK_HEIGHT_BY_TX_ID.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_BLOCK_HEIGHT_BY_HASH.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_BLOCK_HEADER_BY_HEIGHT.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_QUERY.String()+"-"+
		syscontract.ChainQueryFunction_GET_ARCHIVED_BLOCK_HEIGHT.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_GET_CHAIN_CONFIG.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERTS_QUERY.String(), policySpecialRead)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERT_ADD.String(), policySpecialWrite)

	// Disable certificate management for pk mode
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERT_ADD.String(), policyForbidden)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERTS_FREEZE.String(), policyForbidden)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERTS_UNFREEZE.String(), policyForbidden)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERTS_DELETE.String(), policyForbidden)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERTS_REVOKE.String(), policyForbidden)

	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERT_ALIAS_ADD.String(), policyForbidden)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERT_ALIAS_UPDATE.String(), policyForbidden)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CERT_MANAGE.String()+"-"+
		syscontract.CertManageFunction_CERTS_ALIAS_DELETE.String(), policyForbidden)

	// Disable trust member management for pk mode
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_TRUST_MEMBER_ADD.String(), policyForbidden)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_TRUST_MEMBER_DELETE.String(), policyForbidden)
	acs.exceptionalPolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_TRUST_MEMBER_UPDATE.String(), policyForbidden)

	// system contract interface resource definitions
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_CORE_UPDATE.String(), policyConfig)

	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_BLOCK_UPDATE.String(), policyConfig)

	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_TRUST_ROOT_ADD.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_TRUST_ROOT_DELETE.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_TRUST_ROOT_UPDATE.String(), policySelfConfig)

	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_NODE_ID_ADD.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_NODE_ID_DELETE.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_NODE_ID_UPDATE.String(), policySelfConfig)

	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_NODE_ORG_ADD.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_NODE_ORG_UPDATE.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_NODE_ORG_DELETE.String(), policyConfig)

	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_CONSENSUS_EXT_ADD.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_CONSENSUS_EXT_UPDATE.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_CONSENSUS_EXT_DELETE.String(), policyConfig)

	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_PERMISSION_ADD.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_PERMISSION_UPDATE.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CHAIN_CONFIG.String()+"-"+
		syscontract.ChainConfigFunction_PERMISSION_DELETE.String(), policyConfig)

	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CONTRACT_MANAGE.String()+"-"+
		syscontract.ContractManageFunction_INIT_CONTRACT.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CONTRACT_MANAGE.String()+"-"+
		syscontract.ContractManageFunction_UPGRADE_CONTRACT.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CONTRACT_MANAGE.String()+"-"+
		syscontract.ContractManageFunction_FREEZE_CONTRACT.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CONTRACT_MANAGE.String()+"-"+
		syscontract.ContractManageFunction_UNFREEZE_CONTRACT.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CONTRACT_MANAGE.String()+"-"+
		syscontract.ContractManageFunction_REVOKE_CONTRACT.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CONTRACT_MANAGE.String()+"-"+
		syscontract.ContractManageFunction_GRANT_CONTRACT_ACCESS.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CONTRACT_MANAGE.String()+"-"+
		syscontract.ContractManageFunction_REVOKE_CONTRACT_ACCESS.String(), policyConfig)
	acs.resourceNamePolicyMap220.Store(syscontract.SystemContract_CONTRACT_MANAGE.String()+"-"+
		syscontract.ContractManageFunction_VERIFY_CONTRACT_ACCESS.String(), policyConfig)

}
