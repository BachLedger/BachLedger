/*
 * Copyright (C) BABEC. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

package utils

import (
	"testing"

	"chainmaker.org/chainmaker/common/v2/crypto"
	"github.com/stretchr/testify/assert"
)

const (
	org1ClientSingCertStr = "-----BEGIN CERTIFICATE-----\nMIICijCCAi+gAwIBAgIDBS9vMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\nMTIwNzA2NTM0M1owgZExCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\nMQ8wDQYDVQQLEwZjbGllbnQxLDAqBgNVBAMTI2NsaWVudDEuc2lnbi53eC1vcmcx\nLmNoYWlubWFrZXIub3JnMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE56xayRx0\n/a8KEXPxRfiSzYgJ/sE4tVeI/ZbjpiUX9m0TCJX7W/VHdm6WeJLOdCDuLLNvjGTy\nt8LLyqyubJI5AKN7MHkwDgYDVR0PAQH/BAQDAgGmMA8GA1UdJQQIMAYGBFUdJQAw\nKQYDVR0OBCIEIMjAiM2eMzlQ9HzV9ePW69rfUiRZVT2pDBOMqM4WVJSAMCsGA1Ud\nIwQkMCKAIDUkP3EcubfENS6TH3DFczH5dAnC2eD73+wcUF/bEIlnMAoGCCqGSM49\nBAMCA0kAMEYCIQCWUHL0xisjQoW+o6VV12pBXIRJgdeUeAu2EIjptSg2GAIhAIxK\nLXpHIBFxIkmWlxUaanCojPSZhzEbd+8LRrmhEO8n\n-----END CERTIFICATE-----\n"
)

func Test_GetCertHash(t *testing.T) {
	orgId := "yz-org1"
	var hash []byte
	var err error
	hash, err = GetCertHash(orgId, []byte(org1ClientSingCertStr), crypto.CRYPTO_ALGO_SHA256)
	assert.Nil(t, err)
	assert.NotNil(t, hash)

	hash, err = GetCertHash(orgId, []byte(org1ClientSingCertStr), string(crypto.CRYPTO_ALGO_SHA3_256))
	assert.Nil(t, err)
	assert.NotNil(t, hash)

	hash, err = GetCertHash("yz-org2", []byte(org1ClientSingCertStr), string(crypto.CRYPTO_ALGO_SM3))
	assert.Nil(t, err)
	assert.NotNil(t, hash)

	hash, err = GetCertHash(orgId, nil, string(crypto.CRYPTO_ALGO_SM3))
	assert.NotNil(t, err)
	assert.Nil(t, hash)
}

func Test_ParseCert(t *testing.T) {
	_, err := ParseCert([]byte(org1ClientSingCertStr))
	assert.Nil(t, err)

	_, err = ParseCert(nil)
	assert.NotNil(t, err)

	badCertStr := "-----BEGIN CERTIFICATE-----\nMIICijCCAi+gAwIBAgIDBS9vMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\nMTIwNzA2NTM0M1oQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\nMQ8wDQYDVQQLEwZabGllbnQxLDAqBgNVBAMTI2NsaWVudDEuc2lnbi53eC1vcmcx\nLmNoYWlubWFrZXIub3JnMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE56xayRx0\n/a8KEXPxRfiSzYgJ/sE4tVeI/ZbjpiUX9m0TCJX7W/VHdm6WeJLOdCDuLLNvjGTy\nt8LLyqyubJI5AKN7MHkwDgYDVR0PAQH/BAQDAgGmMA8GA1UdJQQIMAYGBFUdJQAw\nKQYDVR0OBCIEIMjAiM2eMzlQ9HzV9ePW69rfUiRZVT2pDBOMqM4WVJSAMCsGA1Ud\nIwQkMCKAIDUkP3EcubfENS6TH3DFczH5dAnC2eD73+wcUF/bEIlnMAoGCCqGSM49\nBAMCA0kAMEYCIQCWUHL0xisjQoW+o6VV12pBXIRJgdeUeAu2EIjptSg2GAIhAIxK\nLXpHIBFxIkmWlxUaanCojPSZhzEbd+8LRrmhEO8n\n-----END CERTIFICATE-----\n"
	_, err = ParseCert([]byte(badCertStr))
	assert.NotNil(t, err)
}

func Test_VerifyCertIssue(t *testing.T) {
	caCert := "-----BEGIN CERTIFICATE-----\nMIICrzCCAlWgAwIBAgIDDsPeMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTMw\nMTIwNjA2NTM0M1owgYoxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\nMRIwEAYDVQQLEwlyb290LWNlcnQxIjAgBgNVBAMTGWNhLnd4LW9yZzEuY2hhaW5t\nYWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAAT7NyTIKcjtUVeMn29b\nGKeEmwbefZ7g9Uk5GROl+o4k7fiIKNuty1rQHLQUvAvkpxqtlmOpPOZ0Qziu6Hw6\nhi19o4GnMIGkMA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMA8GA1Ud\nEwEB/wQFMAMBAf8wKQYDVR0OBCIEIDUkP3EcubfENS6TH3DFczH5dAnC2eD73+wc\nUF/bEIlnMEUGA1UdEQQ+MDyCDmNoYWlubWFrZXIub3Jngglsb2NhbGhvc3SCGWNh\nLnd4LW9yZzEuY2hhaW5tYWtlci5vcmeHBH8AAAEwCgYIKoZIzj0EAwIDSAAwRQIg\nar8CSuLl7pA4Iy6ytAMhR0kzy0WWVSElc+koVY6pF5sCIQCDs+vTD/9V1azmbDXX\nbjoWeEfXbFJp2X/or9f4UIvMgg==\n-----END CERTIFICATE-----\n"
	caCertOther := "-----BEGIN CERTIFICATE-----\nMIICrzCCAlWgAwIBAgIDDYpTMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMi5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcyLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTMw\nMTIwNjA2NTM0M1owgYoxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcyLmNoYWlubWFrZXIub3Jn\nMRIwEAYDVQQLEwlyb290LWNlcnQxIjAgBgNVBAMTGWNhLnd4LW9yZzIuY2hhaW5t\nYWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAASlekil12ThyvibHhBn\ncDvu958HOdN5Db9YE8bZ5e7YYHsJ85P6jBhlt0eKTR/hiukIBVfYKYwmhpYq2eCb\nRYqco4GnMIGkMA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMA8GA1Ud\nEwEB/wQFMAMBAf8wKQYDVR0OBCIEIPGP1bPT4/Lns2PnYudZ9/qHscm0pGL6Kfy+\n1CAFWG0hMEUGA1UdEQQ+MDyCDmNoYWlubWFrZXIub3Jngglsb2NhbGhvc3SCGWNh\nLnd4LW9yZzIuY2hhaW5tYWtlci5vcmeHBH8AAAEwCgYIKoZIzj0EAwIDSAAwRQIg\nJV7mg6IeKBVSLrsDFpLOSEMFd9zKIxo3RRZiMAkdC3MCIQD/LG53Sb/IcNsCqjz9\noLXYNanXzZn1c1t4jPtMuE7nSw==\n-----END CERTIFICATE-----\n"
	var err error
	err = VerifyCertIssue([][]byte{[]byte(caCert)}, nil, []byte(org1ClientSingCertStr))
	assert.Nil(t, err)

	err = VerifyCertIssue(nil, [][]byte{[]byte(caCert)}, []byte(org1ClientSingCertStr))
	assert.Nil(t, err)

	err = VerifyCertIssue([][]byte{[]byte(caCertOther)}, [][]byte{[]byte(caCert)}, []byte(org1ClientSingCertStr))
	assert.Nil(t, err)

	err = VerifyCertIssue([][]byte{[]byte(caCertOther)}, nil, []byte(org1ClientSingCertStr))
	assert.NotNil(t, err)

	err = VerifyCertIssue(nil, nil, nil)
	assert.NotNil(t, err)

	err = VerifyCertIssue(nil, nil, nil)
	assert.NotNil(t, err)
}
