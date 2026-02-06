package utils

import (
	"io/ioutil"
	"os"
	"testing"

	"chainmaker.org/chainmaker/common/v2/crypto"
	"chainmaker.org/chainmaker/common/v2/crypto/asym"
	"chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/utils/v2"
	"github.com/stretchr/testify/require"
)

const (
	crt = `-----BEGIN CERTIFICATE-----
MIIChzCCAi2gAwIBAgIDAwGbMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ
MA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt
b3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD
ExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1
MTIwNzA2NTM0M1owgY8xCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw
DgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn
MQ4wDAYDVQQLEwVhZG1pbjErMCkGA1UEAxMiYWRtaW4xLnNpZ24ud3gtb3JnMS5j
aGFpbm1ha2VyLm9yZzBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABORqoYNAw8ax
9QOD94VaXq1dCHguarSKqAruEI39dRkm8Vu2gSHkeWlxzvSsVVqoN6ATObi2ZohY
KYab2s+/QA2jezB5MA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMCkG
A1UdDgQiBCDZOtAtHzfoZd/OQ2Jx5mIMgkqkMkH4SDvAt03yOrRnBzArBgNVHSME
JDAigCA1JD9xHLm3xDUukx9wxXMx+XQJwtng+9/sHFBf2xCJZzAKBggqhkjOPQQD
AgNIADBFAiEAiGjIB8Wb8mhI+ma4F3kCW/5QM6tlxiKIB5zTcO5E890CIBxWDICm
Aod1WZHJajgnDQ2zEcFF94aejR9dmGBB/P//
-----END CERTIFICATE-----`
	privKey = `
-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIEAmnFvzHhYsqScEJJj2jlPFRsYRbYpNlr9LVan1xN4GoAoGCCqGSM49
AwEHoUQDQgAE5Gqhg0DDxrH1A4P3hVperV0IeC5qtIqoCu4Qjf11GSbxW7aBIeR5
aXHO9KxVWqg3oBM5uLZmiFgphpvaz79ADQ==
-----END EC PRIVATE KEY-----
`
)

func TestSignPayload(t *testing.T) {
	privKeyObj, err := asym.PrivateKeyFromPEM([]byte(privKey), nil)
	require.Nil(t, err)
	crtObj, err := utils.ParseCert([]byte(crt))
	require.Nil(t, err)
	payload := &common.Payload{
		ChainId: "chain1",
	}

	_, err = SignPayload(privKeyObj, crtObj, payload)
	require.Nil(t, err)
}

func TestSignPayloadWithHashType(t *testing.T) {
	privKeyObj, err := asym.PrivateKeyFromPEM([]byte(privKey), nil)
	require.Nil(t, err)
	payload := &common.Payload{
		ChainId: "chain1",
	}

	_, err = SignPayloadWithHashType(privKeyObj, crypto.HASH_TYPE_SHA256, payload)
	require.Nil(t, err)
}

func TestSignPayloadWithPath(t *testing.T) {
	keyFile, err := ioutil.TempFile("./", "*.key")
	require.Nil(t, err)
	_, err = keyFile.WriteString(privKey)
	require.Nil(t, err)
	defer os.Remove(keyFile.Name())

	crtFile, err := ioutil.TempFile("./", "*.crt")
	require.Nil(t, err)
	_, err = crtFile.WriteString(crt)
	require.Nil(t, err)
	defer os.Remove(crtFile.Name())

	payload := &common.Payload{
		ChainId: "chain1",
	}

	_, err = SignPayloadWithPath(keyFile.Name(), crtFile.Name(), payload)
	require.Nil(t, err)
}
