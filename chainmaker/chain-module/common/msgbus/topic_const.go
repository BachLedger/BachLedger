/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package msgbus

//go:generate stringer -type=Topic
type Topic int

const (
	Invalid Topic = iota
	ProposedBlock
	VerifyBlock
	VerifyResult
	CommitBlock
	ProposeState
	TxPoolSignal
	BlockInfo
	ContractEventInfo

	// For Net Service
	SendConsensusMsg
	RecvConsensusMsg
	SendSyncBlockMsg
	RecvSyncBlockMsg
	SendTxPoolMsg
	RecvTxPoolMsg

	BuildProposal

	// The following are contractual events topics
	// ChainConfig BlockVerifier Blockchain net cert_ac pk_ac pwk_ac
	ChainConfig
	// net cert_ac
	CertManageCertsDelete
	CertManageCertsFreeze
	CertManageCertsUnfreeze
	CertManageCertsRevoke
	CertManageCertsAliasUpdate
	CertManageCertsAliasDelete
	// net pk_ac
	PubkeyManageAdd
	PubkeyManageDelete

	// For Consistent Engine
	SendConsistentMsg
	RecvConsistentMsg

	// For new transactions signal for maxbft
	ProposeBlock

	MaxbftEpochConf

	// solve random tx
	RwSetVerifyFailTxs

	ExeSig
)

const (
	BlacklistTxIdAdd Topic = iota + 100
	BlacklistTxIdDel
	BlacklistStateKeyAdd
	BlacklistStateKeyDel
)
