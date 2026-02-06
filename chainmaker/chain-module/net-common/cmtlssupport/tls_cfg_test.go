/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package cmtlssupport

import (
	"testing"

	"chainmaker.org/chainmaker/common/v2/crypto/asym"
	"github.com/stretchr/testify/require"
)

func TestPrivateKeyToCertificate(t *testing.T) {
	gmskPem := "-----BEGIN EC PRIVATE KEY-----\n" +
		"MIGTAgEAMBMGByqGSM49AgEGCCqBHM9VAYItBHkwdwIBAQQgy51q+v+RbXRRSTQV\n" +
		"rWzk3A9bNRuZXqetMmdVknhGFpegCgYIKoEcz1UBgi2hRANCAAQqyk3DJRKz+H1/\n" +
		"SzfVu6KXZ9PbKupjebfhd5gVdAjrPmvSMtAbdyzJesEIk68cVh//fenV78oAUVrl\n" +
		"3DOBicVS\n" +
		"-----END EC PRIVATE KEY-----"
	gmsk, err := asym.PrivateKeyFromPEM([]byte(gmskPem), nil)
	require.Nil(t, err)
	_, err = PrivateKeyToCertificate(gmsk)
	require.Nil(t, err)
	skPem := "-----BEGIN EC PRIVATE KEY-----\n" +
		"MHcCAQEEIF4Sy4KANZHi8uU4YkmymbcbF3HHJnGgSjV/0iNOSdy3oAoGCCqGSM49\n" +
		"AwEHoUQDQgAEKwemRhrzv5GSSmsy4EREhnQJ4jocauyWnD1dXUx9X8c4VwhG5hWQ\n" +
		"7oc+cMyz6rXPKTrUxKD50V+OB0FVkpY7vA==\n" +
		"-----END EC PRIVATE KEY-----\n"
	sk, err := asym.PrivateKeyFromPEM([]byte(skPem), nil)
	require.Nil(t, err)
	_, err = PrivateKeyToCertificate(sk)
	require.Nil(t, err)
}
