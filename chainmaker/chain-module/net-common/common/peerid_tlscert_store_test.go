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

func TestPeerIdTlsCertStore(t *testing.T) {
	store := &PeerIdTlsCertStore{
		logger: nil,
		lock:   sync.RWMutex{},
		store:  make(map[string][]byte),
	}

	tlsCert := []byte("certs")

	store.SetPeerTlsCert(peerId, nil)
	require.NotEmpty(t, store.store)

	store.SetPeerTlsCert(peerId, tlsCert)
	require.Equal(t, 1, len(store.store))

	certs := store.GetCertByPeerId(peerId)
	require.Equal(t, []byte("certs"), certs)

	certs2 := store.GetCertByPeerId("peerId2")
	require.Nil(t, certs2)

	copyStore := store.StoreCopy()
	require.NotEmpty(t, copyStore)

	store.RemoveByPeerId(peerId)
	require.Empty(t, store.store)
}
