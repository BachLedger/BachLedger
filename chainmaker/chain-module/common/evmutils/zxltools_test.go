/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package evmutils

import (
	"testing"

	"github.com/stretchr/testify/require"

	"chainmaker.org/chainmaker/common/v2/crypto/asym"
)

const (
	zxlPkPEM = `-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoEcz1UBgi0DQgAESkwkzwN7DHoCfmNLmUpf280PqnGM
6QU+P3X8uahlUjpgWv+Stfmeco9RqSTU8Y1YGcQvm2Jr327qkRlG7+dELQ==
-----END PUBLIC KEY-----`
	zxlAddress = "ZXaaa6f45415493ffb832ca28faa14bef5c357f5f0"

	cmCertPEM = `-----BEGIN CERTIFICATE-----
MIICzjCCAi+gAwIBAgIDCzLUMAoGCCqGSM49BAMCMGoxCzAJBgNVBAYTAkNOMRAw
DgYDVQQIEwdCZWlqaW5nMRAwDgYDVQQHEwdCZWlqaW5nMRAwDgYDVQQKEwd3eC1v
cmcxMRAwDgYDVQQLEwdyb290LWNhMRMwEQYDVQQDEwp3eC1vcmcxLWNhMB4XDTIw
MTAyOTEzMzgxMFoXDTMwMTAyNzEzMzgxMFowcDELMAkGA1UEBhMCQ04xEDAOBgNV
BAgTB0JlaWppbmcxEDAOBgNVBAcTB0JlaWppbmcxEDAOBgNVBAoTB3d4LW9yZzEx
EzARBgNVBAsTCkNoYWluTWFrZXIxFjAUBgNVBAMTDXVzZXIxLnd4LW9yZzEwgZsw
EAYHKoZIzj0CAQYFK4EEACMDgYYABAGLEJZriYzK9Se/vMGfkwjhU55eEZsM2iKM
emSZICh/HY37uR0BFAVUjMYEj84tJBzEEzlpD+AUAe44/b11b+GCMwDXPKcsjHK0
jsAPrN5LH7uptXsjMFpN2bbOqvj6sAIDfTV9chuF91LxCjYnh+Lya0ikextGkpbp
HOvi5eQ/yUHSQaN7MHkwDgYDVR0PAQH/BAQDAgGmMA8GA1UdJQQIMAYGBFUdJQAw
KQYDVR0OBCIEIAp+6tWmoiE0KmdtpLFBZpBj1Ni7JH8g2XPgoQwhQS8qMCsGA1Ud
IwQkMCKAIMsnP+UWEyGuyEHBn7JkJzb+tfBqsRCBUIPyMZH4h1HPMAoGCCqGSM49
BAMCA4GMADCBiAJCAIENc8ip2BP4yJpj9SdR9pvZc4/qbBzKucZQaD/GT2sj0FxH
hp8YLjSflgw1+uWlMb/WCY60MyxZr/RRsTYpHu7FAkIBSMAVxw5RYySsf4J3bpM0
CpIO2ZrxkJ1Nm/FKZzMLQjp7Dm//xEMkpCbqqC6koOkRP2MKGSnEGXGfRr1QgBvr
8H8=
-----END CERTIFICATE-----`
	cmAddressFromCert = "ZX0787b8affa4cbdb9994548010c80d9741113ae78"
)

func TestAddress(t *testing.T) {
	pk, err := asym.PublicKeyFromPEM([]byte(zxlPkPEM))
	require.Nil(t, err)

	addressFromPk, err := ZXAddressFromPublicKey(pk)
	require.Nil(t, err)

	addressFromPEM, err := ZXAddressFromPublicKeyPEM([]byte(zxlPkPEM))
	require.Nil(t, err)

	require.Equal(t, zxlAddress, addressFromPk)
	require.Equal(t, addressFromPEM, addressFromPk)

	addressFromCert, err := ZXAddressFromCertificatePEM([]byte(cmCertPEM))
	require.Nil(t, err)

	require.Equal(t, addressFromCert, cmAddressFromCert)
}
