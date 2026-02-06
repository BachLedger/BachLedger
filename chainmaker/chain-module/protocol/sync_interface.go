/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package protocol

// SYNC_txHash_set_key additional data key
const (
	SYNC_txHash_set_key = "SYNC_txHash_set_key"
)

// SyncService is the server to sync the blockchain
type SyncService interface {
	//Start Init the sync server, and the sync server broadcast the current block height every broadcastTime
	Start() error

	//Stop the sync server
	Stop()

	//ListenSyncToIdealHeight listen local block height has synced to ideal height
	ListenSyncToIdealHeight() <-chan struct{}

	//StopBlockSync syncing blocks from other nodes, but still process other nodes synchronizing blocks from itself
	StopBlockSync()

	//StartBlockSync start request service
	StartBlockSync()
}
