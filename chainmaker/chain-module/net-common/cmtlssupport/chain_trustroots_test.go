/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package cmtlssupport

import (
	"encoding/pem"
	"sync"
	"testing"

	"chainmaker.org/chainmaker/common/v2/crypto/x509"

	"github.com/stretchr/testify/require"
)

var (
	chainId  = "chain1"
	chainId2 = "chain2"

	certRoot = "-----BEGIN CERTIFICATE-----\n" +
		"MIICrzCCAlWgAwIBAgIDDsPeMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\n" +
		"MA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\n" +
		"b3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\n" +
		"ExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTMw\n" +
		"MTIwNjA2NTM0M1owgYoxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\n" +
		"DgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\n" +
		"MRIwEAYDVQQLEwlyb290LWNlcnQxIjAgBgNVBAMTGWNhLnd4LW9yZzEuY2hhaW5t\n" +
		"YWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAAT7NyTIKcjtUVeMn29b\n" +
		"GKeEmwbefZ7g9Uk5GROl+o4k7fiIKNuty1rQHLQUvAvkpxqtlmOpPOZ0Qziu6Hw6\n" +
		"hi19o4GnMIGkMA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMA8GA1Ud\n" +
		"EwEB/wQFMAMBAf8wKQYDVR0OBCIEIDUkP3EcubfENS6TH3DFczH5dAnC2eD73+wc\n" +
		"UF/bEIlnMEUGA1UdEQQ+MDyCDmNoYWlubWFrZXIub3Jngglsb2NhbGhvc3SCGWNh\n" +
		"Lnd4LW9yZzEuY2hhaW5tYWtlci5vcmeHBH8AAAEwCgYIKoZIzj0EAwIDSAAwRQIg\n" +
		"ar8CSuLl7pA4Iy6ytAMhR0kzy0WWVSElc+koVY6pF5sCIQCDs+vTD/9V1azmbDXX\n" +
		"bjoWeEfXbFJp2X/or9f4UIvMgg==\n" +
		"-----END CERTIFICATE-----"

	certAdmin = "-----BEGIN CERTIFICATE-----\n" +
		"MIIChzCCAi2gAwIBAgIDAwGbMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\n" +
		"MA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\n" +
		"b3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\n" +
		"ExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\n" +
		"MTIwNzA2NTM0M1owgY8xCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\n" +
		"DgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\n" +
		"MQ4wDAYDVQQLEwVhZG1pbjErMCkGA1UEAxMiYWRtaW4xLnNpZ24ud3gtb3JnMS5j\n" +
		"aGFpbm1ha2VyLm9yZzBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABORqoYNAw8ax\n" +
		"9QOD94VaXq1dCHguarSKqAruEI39dRkm8Vu2gSHkeWlxzvSsVVqoN6ATObi2ZohY\n" +
		"KYab2s+/QA2jezB5MA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMCkG\n" +
		"A1UdDgQiBCDZOtAtHzfoZd/OQ2Jx5mIMgkqkMkH4SDvAt03yOrRnBzArBgNVHSME\n" +
		"JDAigCA1JD9xHLm3xDUukx9wxXMx+XQJwtng+9/sHFBf2xCJZzAKBggqhkjOPQQD\n" +
		"AgNIADBFAiEAiGjIB8Wb8mhI+ma4F3kCW/5QM6tlxiKIB5zTcO5E890CIBxWDICm\n" +
		"Aod1WZHJajgnDQ2zEcFF94aejR9dmGBB/P//\n" +
		"-----END CERTIFICATE-----"

	certAdminBad = "-----BEGIN CERTIFICATE-----\n" +
		"MIIChzCCAi2gAwIBAgIDAwGbMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\n" +
		"MA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\n" +
		"b3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\n" +
		"ExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\n" +
		"MTIwNzA2NTM0M1owgY8xCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\n" +
		"DgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\n" +
		"MQ4wDAYDVQQLEwVhZG1pbjErMCkGA1UEAxMiYWRtaW4xLnNpZ24ud3gtb3JnMS5j\n" +
		"aGFpbm1ha2VyLm9yZzBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABORqoYNAw8ax\n" +
		"9QOD94VaXq1dCHguarSKqAruEI39dRkm8Vu2gSHkeWlxzvSsVVqoN6ATObi2ZohY\n" +
		"KYab2s+/QA2jezB5MA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMCkG\n" +
		"A1UdDgQiBCDZOtAtHzfoZd/OQ2Jx5mIMgkqkMkH4SDvAt03yOrRnBzArBgNVHSME\n" +
		"JDAigCA1JD9xHLm3xDUukx9wxXMx+XQJwtng+9/sHFBf2xCJZzAKBggqhkjOPQQD\n" +
		"AgNIADBFAiEAiGjIB8Wb8mhI+ma4F3kCW/5QM6tlxiKIB5zTcO5E890CIBxWDICm\n" +
		"Aod1WZHJajgnDQ2zEcFF94aejR9dmGBBBBBBBBBB/P//\n" +
		"-----END CERTIFICATE-----"

	certPEMs = [][]byte{
		[]byte("-----BEGIN CERTIFICATE-----\nMIIDFTCCArugAwIBAgIDBOOCMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\nMTIwNzA2NTM0M1owgZYxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\nMRIwEAYDVQQLEwljb25zZW5zdXMxLjAsBgNVBAMTJWNvbnNlbnN1czEudGxzLnd4\nLW9yZzEuY2hhaW5tYWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAAQr\nB6ZGGvO/kZJKazLgRESGdAniOhxq7JacPV1dTH1fxzhXCEbmFZDuhz5wzLPqtc8p\nOtTEoPnRX44HQVWSlju8o4IBADCB/TAOBgNVHQ8BAf8EBAMCAaYwDwYDVR0lBAgw\nBgYEVR0lADApBgNVHQ4EIgQgqzFBKQ6cAvTThFgrn//B/SDhAFEDfW5Y8MOE7hvY\nBf4wKwYDVR0jBCQwIoAgNSQ/cRy5t8Q1LpMfcMVzMfl0CcLZ4Pvf7BxQX9sQiWcw\nUQYDVR0RBEowSIIOY2hhaW5tYWtlci5vcmeCCWxvY2FsaG9zdIIlY29uc2Vuc3Vz\nMS50bHMud3gtb3JnMS5jaGFpbm1ha2VyLm9yZ4cEfwAAATAvBguBJ1iPZAsej2QL\nBAQgMDAxNjQ2ZTY3ODBmNGIwZDhiZWEzMjNlZThjMjQ5MTUwCgYIKoZIzj0EAwID\nSAAwRQIgNVNGr+G8dbYnzmmNMr9GCSUEC3TUmRcS4uOd5/Sw4mECIQDII1R7dCcx\n02YrxI8jEQZhmWeZ5FJhnSG6p6H9pCIWDQ==\n-----END CERTIFICATE-----\n"),
		[]byte("-----BEGIN CERTIFICATE-----\nMIIDFjCCArugAwIBAgIDAdGZMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMi5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcyLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\nMTIwNzA2NTM0M1owgZYxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcyLmNoYWlubWFrZXIub3Jn\nMRIwEAYDVQQLEwljb25zZW5zdXMxLjAsBgNVBAMTJWNvbnNlbnN1czEudGxzLnd4\nLW9yZzIuY2hhaW5tYWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAARJ\nldhjiDOKiRqWJdfkmTBP6J1MILB6yO4m1O6kAG6Eiq7ujaO3VXQ01ZW+lKEgz7zv\nwFpJva/ZOWXBWg5iZ/M+o4IBADCB/TAOBgNVHQ8BAf8EBAMCAaYwDwYDVR0lBAgw\nBgYEVR0lADApBgNVHQ4EIgQgH0PY7Oic1NRq5O64ag3g12d5vI5jqEWW9+MzOOrE\nnhEwKwYDVR0jBCQwIoAg8Y/Vs9Pj8uezY+di51n3+oexybSkYvop/L7UIAVYbSEw\nUQYDVR0RBEowSIIOY2hhaW5tYWtlci5vcmeCCWxvY2FsaG9zdIIlY29uc2Vuc3Vz\nMS50bHMud3gtb3JnMi5jaGFpbm1ha2VyLm9yZ4cEfwAAATAvBguBJ1iPZAsej2QL\nBAQgZjVhODUwYTAzYjFlNDU0NzkzOTg5NzIxYzVjMTc3NjMwCgYIKoZIzj0EAwID\nSQAwRgIhAKvDGBl+17dcTMdOjRW3VTTaGNlQiZepRXYarmAdX3PiAiEA6F6cZjsT\nEpSBfal9mUGlxJNNHhYIxs2SlSL4of4GTBA=\n-----END CERTIFICATE-----\n"),
		[]byte("-----BEGIN CERTIFICATE-----\nMIIDFTCCArugAwIBAgIDCJoJMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMy5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmczLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\nMTIwNzA2NTM0M1owgZYxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmczLmNoYWlubWFrZXIub3Jn\nMRIwEAYDVQQLEwljb25zZW5zdXMxLjAsBgNVBAMTJWNvbnNlbnN1czEudGxzLnd4\nLW9yZzMuY2hhaW5tYWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAAQt\nRBahVpwtR/GUIchzc5qiwePQyefBXrWLrP2W0EcsYHe+W25bb1W369OArZCt6Bct\nBTLOxL/Dl2Wk/KPBszKuo4IBADCB/TAOBgNVHQ8BAf8EBAMCAaYwDwYDVR0lBAgw\nBgYEVR0lADApBgNVHQ4EIgQgEnC2getHs64R4n9VVe1A66N41/5HH63o63aV8Iqq\nk2EwKwYDVR0jBCQwIoAg0Y9lHSxXCu9i0Wd5MPoZTIFB+XClOYnSoKyC90WAif0w\nUQYDVR0RBEowSIIOY2hhaW5tYWtlci5vcmeCCWxvY2FsaG9zdIIlY29uc2Vuc3Vz\nMS50bHMud3gtb3JnMy5jaGFpbm1ha2VyLm9yZ4cEfwAAATAvBguBJ1iPZAsej2QL\nBAQgNzNiMWM4MWJkZjA2NDllNjk4YmI4MTVlNWI3NzM2YmIwCgYIKoZIzj0EAwID\nSAAwRQIhAODEcNO5jIBT+Dd4Fcsxz1ML8pzIzcWlPDeeuD6nfbQMAiARIw6KvJMu\nH9A4TrVomaX3eP0ttXTYwhdqu+5JeA+j2Q==\n-----END CERTIFICATE-----\n"),
		[]byte("-----BEGIN CERTIFICATE-----\nMIIDFTCCArugAwIBAgIDAg4CMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnNC5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmc0LmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\nMTIwNzA2NTM0M1owgZYxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmc0LmNoYWlubWFrZXIub3Jn\nMRIwEAYDVQQLEwljb25zZW5zdXMxLjAsBgNVBAMTJWNvbnNlbnN1czEudGxzLnd4\nLW9yZzQuY2hhaW5tYWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAAQg\nqP7ARZIf1tbrZoGOFkicxqyORGDXM7sdhmFTi/EVl/wI8RiwIwm64Jb0fSmakoEE\nXfXqfcav93s52EvpBUFso4IBADCB/TAOBgNVHQ8BAf8EBAMCAaYwDwYDVR0lBAgw\nBgYEVR0lADApBgNVHQ4EIgQg+OLUREO24tHaUDNz5k2t4FYnm3sY2AMqmIfRk3ns\n6Q4wKwYDVR0jBCQwIoAgucqtaCpx/s+2C0wkXKeYip0W3ShUqYrPEP418+yFwcQw\nUQYDVR0RBEowSIIOY2hhaW5tYWtlci5vcmeCCWxvY2FsaG9zdIIlY29uc2Vuc3Vz\nMS50bHMud3gtb3JnNC5jaGFpbm1ha2VyLm9yZ4cEfwAAATAvBguBJ1iPZAsej2QL\nBAQgM2U1NmNhYTZkNjI2NDM2NWJjN2IxZTEyYmM3ZDljMjYwCgYIKoZIzj0EAwID\nSAAwRQIgYseIev8uXoorRTvz+lDou5GTcnWEvz3yeawlMRMBbDkCIQC2SD9oCjus\n7U2f6ujxCbediFaOo1YdBj1GNSaGfqSFbg==\n-----END CERTIFICATE-----\n"),
	}
)

func getCert(certOpt string) *x509.Certificate {
	block, _ := pem.Decode([]byte(certOpt))
	certificate, _ := x509.ParseCertificate(block.Bytes)
	return certificate
}

func TestChainTrustRoots(t *testing.T) {
	chainTrustRoots := NewChainTrustRoots()
	res, ok := chainTrustRoots.RootsPool(chainId)
	require.False(t, ok)
	require.Nil(t, res)

	chainTrustRoots.AddRoot(chainId, getCert(certRoot))
	require.NotEmpty(t, chainTrustRoots.trustRoots)

	ok = chainTrustRoots.AppendRootsFromPem(chainId, certPEMs[0])
	require.True(t, ok)

	ok = chainTrustRoots.AppendRootsFromPem(chainId2, certPEMs[1])
	require.True(t, ok)

	ok = chainTrustRoots.RefreshRootsFromPem(chainId, certPEMs)
	require.True(t, ok)
}

func TestChainTrustIntermediates(t *testing.T) {
	chainTrustRoots := NewChainTrustRoots()
	pool, ok := chainTrustRoots.IntermediatesPool(chainId)
	require.False(t, ok)
	require.Nil(t, pool)

	chainTrustRoots.AddIntermediates(chainId, getCert(certRoot))
	require.NotEmpty(t, chainTrustRoots.trustIntermediates)

	ok = chainTrustRoots.AppendIntermediatesFromPem(chainId, certPEMs[0])
	require.True(t, ok)

	ok = chainTrustRoots.AppendIntermediatesFromPem(chainId2, certPEMs[1])
	require.True(t, ok)

	ok = chainTrustRoots.RefreshIntermediatesFromPem(chainId, certPEMs)
	require.True(t, ok)
}

func TestChainTrustRoots_VerifyCert(t *testing.T) {
	chainTrustRoots := NewChainTrustRoots()
	res, err := chainTrustRoots.VerifyCert(nil)
	require.Error(t, err)
	require.Nil(t, res)

	chainTrustRoots.AddRoot(chainId, getCert(certRoot))
	chainTrustRoots.AppendIntermediatesFromPem(chainId, certPEMs[0])

	res, err = chainTrustRoots.VerifyCert(getCert(certRoot))
	require.Nil(t, err)
	require.NotEmpty(t, res)

	res, err = chainTrustRoots.VerifyCert(getCert(certAdmin))
	require.Nil(t, err)
	require.NotEmpty(t, res)
	require.True(t, len(res) == 1)

	res, err = chainTrustRoots.VerifyCert(getCert(certAdminBad))
	require.Error(t, err)
	require.Nil(t, res)

}

func TestChainTrustRoots_VerifyCertOfChain(t *testing.T) {
	chainTrustRoots := NewChainTrustRoots()
	ok := chainTrustRoots.VerifyCertOfChain(chainId, nil)
	require.False(t, ok)

	ok = chainTrustRoots.VerifyCertOfChain(chainId, getCert(certRoot))
	require.False(t, ok)

	chainTrustRoots.AddRoot(chainId, getCert(certRoot))
	chainTrustRoots.AppendIntermediatesFromPem(chainId, certPEMs[0])

	ok = chainTrustRoots.VerifyCertOfChain(chainId, getCert(certAdmin))
	require.True(t, ok)

	ok = chainTrustRoots.VerifyCertOfChain(chainId, getCert(certAdminBad))
	require.False(t, ok)

}

func TestGetAllCertsBytes(t *testing.T) {
	res := getAllCertsBytes(nil)
	require.Nil(t, res)

	res = getAllCertsBytes([]byte(certRoot))
	require.NotNil(t, res)
	require.Equal(t, 1, len(res))
}

func TestLoadAllCertsFromCertBytes(t *testing.T) {
	chainTrustRoots := &ChainTrustRoots{
		lock:               sync.Mutex{},
		trustRoots:         make(map[string]*x509.CertPool),
		trustIntermediates: make(map[string]*x509.CertPool),
	}

	ok, err := loadAllCertsFromCertBytes(nil, chainId, chainTrustRoots)
	require.Nil(t, err)
	require.False(t, ok)

	ok, err = loadAllCertsFromCertBytes([]byte(certRoot), chainId, chainTrustRoots)
	require.Nil(t, err)
	require.True(t, ok)
}
