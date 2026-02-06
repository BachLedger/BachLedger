/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package helper

import (
	"testing"

	"github.com/btcsuite/btcd/btcec"
	"github.com/stretchr/testify/require"
)

func TestGetLibp2pPeerIdFromCert(t *testing.T) {
	certBytes := []byte("-----BEGIN CERTIFICATE-----\n" +
		"MIICHzCCAcSgAwIBAgIRAMR9Zia8ue5OEB/mEJ0B5jYwCgYIKoEcz1UBg3UwYDEL\n" +
		"MAkGA1UEBhMCQ04xCzAJBgNVBAgTAkdEMQswCQYDVQQHEwJTWjEZMBcGA1UEChMQ\n" +
		"b3JnMS5leGFtcGxlLmNvbTEcMBoGA1UEAxMTY2Eub3JnMS5leGFtcGxlLmNvbTAe\n" +
		"Fw0yMDA1MjkxMDMwNDJaFw0zMDA1MjcxMDMwNDJaMGAxCzAJBgNVBAYTAkNOMQsw\n" +
		"CQYDVQQIEwJHRDELMAkGA1UEBxMCU1oxGTAXBgNVBAoTEG9yZzEuZXhhbXBsZS5j\n" +
		"b20xHDAaBgNVBAMTE2NhLm9yZzEuZXhhbXBsZS5jb20wWTATBgcqhkjOPQIBBggq\n" +
		"gRzPVQGCLQNCAAQWXBhGZrChTwqPDfhxeXr930tjVWaiF+bToVSAHpYYAOzAI/7S\n" +
		"B/MMp82P71BDTp+dua4N0VhWWZNYtJRMravvo18wXTAOBgNVHQ8BAf8EBAMCAaYw\n" +
		"DwYDVR0lBAgwBgYEVR0lADAPBgNVHRMBAf8EBTADAQH/MCkGA1UdDgQiBCA48Q7H\n" +
		"PVM6G837SCKsNuxA4VsoeLKxs4//8a65NUiNDzAKBggqgRzPVQGDdQNJADBGAiEA\n" +
		"kSQyih4ax6A7UWiWyzBTv7oNdUL2BGG6I3N5BDZ/040CIQCGlW38vfSntJe1Vvgg\n" +
		"5ctBDSRW9ophuyCuUX6Gx99Ogw==\n" +
		"-----END CERTIFICATE-----\n")
	nodeUid, err := GetLibp2pPeerIdFromCert(certBytes)
	if err != nil {
		t.Fatal(err)
	}
	require.Equal(t, "QmTrsVrof7hvU79LmAMnJrmhTCUdaBoVNYDhHMUGaVQa6m", nodeUid)
}

func TestGetNodeUidFromAddr(t *testing.T) {
	addr := "/ip4/0.0.0.0/tcp/6666/p2p/QmTrsVrof7hvU79LmAMnJrmhTCUdaBoVNYDhHMUGaVQa6m"
	nodeUid, err := GetNodeUidFromAddr(addr)
	if err != nil {
		t.Fatal(err)
	}
	require.Equal(t, "QmTrsVrof7hvU79LmAMnJrmhTCUdaBoVNYDhHMUGaVQa6m", nodeUid)
}

func TestP2pAddressFormatVerify(t *testing.T) {
	bl := P2pAddressFormatVerify("/ip4/0.0.0.0/tcp/6666/p2p/QmTrsVrof7hvU79LmAMnJrmhTCUdaBoVNYDhHMUGaVQa6m")
	require.True(t, bl)
	bl = P2pAddressFormatVerify("/ip4/0.0.0.0/tcp/6666/p2p/QmTrsVrof7hvU79LmAMnJrmhTCUdaBoVNYDhHMUGaVQa6m" +
		"/p2p-circuit/p2p/QmTrsVrof7hvU79LmAMnJrmhTCUdaBoVNYDhHMUGaVQa6m")
	require.True(t, bl)
	bl = P2pAddressFormatVerify("/ip4/0.0.0.0/tcp/6666")
	require.False(t, bl)
	bl = P2pAddressFormatVerify("0.0.0.0:6666")
	require.False(t, bl)
}

func TestSecp256k1Curve(t *testing.T) {
	privk, err := btcec.NewPrivateKey(btcec.S256())
	require.Nil(t, err)
	libp2pPk, err := ParseGoPublicKeyToPubKey(&privk.PublicKey)
	require.Nil(t, err)
	require.NotNil(t, libp2pPk)

}
