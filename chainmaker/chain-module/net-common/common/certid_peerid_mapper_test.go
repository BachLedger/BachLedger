/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package common

import (
	"testing"

	"github.com/stretchr/testify/require"
)

var (
	certId = "cert id"
	peerId = "peer id"
)

func TestCertIdPeerIdMapper_Add(t *testing.T) {
	certIdPeerIdMapper := NewCertIdPeerIdMapper(nil)
	require.Empty(t, certIdPeerIdMapper.mapper)
	certIdPeerIdMapper.Add(certId, peerId)
	require.Contains(t, certIdPeerIdMapper.mapper, certId)
}

func TestCertIdPeerIdMapper_RemoveByPeerId(t *testing.T) {
	certIdPeerIdMapper := NewCertIdPeerIdMapper(nil)
	certIdPeerIdMapper.Add(certId, peerId)
	require.Contains(t, certIdPeerIdMapper.mapper, certId)
	certIdPeerIdMapper.RemoveByPeerId(peerId)
	require.Empty(t, certIdPeerIdMapper.mapper)
}
