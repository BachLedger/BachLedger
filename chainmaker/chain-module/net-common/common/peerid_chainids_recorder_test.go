/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package common

import (
	"sync"
	"testing"

	"github.com/stretchr/testify/require"
)

var (
	chainId = "chain1"

	peerId2 = "peerId2"
)

func TestPeerIdChainIdsRecorder(t *testing.T) {
	recorder := &PeerIdChainIdsRecorder{
		logger:    nil,
		lock:      sync.RWMutex{},
		records:   make(map[string]*StringMapList),
		onAddC:    nil,
		onRemoveC: nil,
	}

	onAddC := make(chan string)
	recorder.OnAddNotifyC(onAddC)
	require.NotNil(t, recorder.onAddC)

	ok := recorder.RemoveAllByPeerId(peerId)
	require.False(t, ok)

	go func() {
		onAddCGet := <-onAddC
		require.NotNil(t, onAddCGet)
	}()
	ok = recorder.AddPeerChainId(peerId, chainId)
	require.True(t, ok)

	ok = recorder.IsPeerBelongToChain(peerId, chainId)
	require.True(t, ok)
	ok = recorder.IsPeerBelongToChain(peerId2, chainId)
	require.False(t, ok)

	ids := recorder.PeerIdsOfChain(chainId)
	require.NotEmpty(t, ids)

	ids = recorder.PeerIdsOfNoChain()
	require.Empty(t, ids)

	ok = recorder.RemovePeerChainId(peerId2, chainId)
	require.False(t, ok)

	onRemoveC := make(chan string)
	recorder.OnRemoveNotifyC(onRemoveC)
	require.NotNil(t, recorder.onRemoveC)

	go func() {
		onRemoveCGet := <-onRemoveC
		require.NotNil(t, onRemoveCGet)
	}()
	ok = recorder.RemovePeerChainId(peerId, chainId)
	require.True(t, ok)

	go func() {
		onRemoveCGet := <-onRemoveC
		require.NotNil(t, onRemoveCGet)
	}()
	ok = recorder.RemoveAllByPeerId(peerId)
	require.True(t, ok)

}
