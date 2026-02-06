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

func TestPeerIdPubKeyStore(t *testing.T) {
	store := &PeerIdPubKeyStore{
		logger: nil,
		lock:   sync.RWMutex{},
		store:  make(map[string][]byte),
	}

	pubKey := []byte("pubKey")
	store.SetPeerPubKey(peerId, nil)
	require.NotEmpty(t, store.store)

	store.SetPeerPubKey(peerId, pubKey)
	require.Equal(t, 1, len(store.store))

	keys := store.GetCertByPeerId(peerId)
	require.Equal(t, []byte("pubKey"), keys)

	keys = store.GetCertByPeerId("peerId2")
	require.Nil(t, keys)

	copyStore := store.StoreCopy()
	require.Equal(t, 1, len(copyStore))

	store.RemoveByPeerId(peerId)
	require.Empty(t, store.store)
}
