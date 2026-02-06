/*
 * Copyright (C) BABEC. All rights reserved.
 * Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

package certmgr220

import (
	"fmt"
	"sync"
	"testing"

	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"

	"chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	"chainmaker.org/chainmaker/protocol/v2/test"
	"github.com/stretchr/testify/assert"

	"chainmaker.org/chainmaker/protocol/v2/mock"
	"github.com/golang/mock/gomock"

	"chainmaker.org/chainmaker/protocol/v2"
)

var (
	errCert = false
)

func Test_AddCert(t *testing.T) {
	mgrRuntime, txSimContext, fn := initEnv(t)
	defer fn()
	result, err := mgrRuntime.Add(txSimContext, nil)
	assert.Nil(t, err)
	assert.Equal(t, "e77c9238c51e3446d942f94bd8803cc4f351254f8771f972146d7bfc6e0be7f4", string(result))
	fmt.Printf("add cert success. cert hash: %s \n", result)

	errCert = true
	_, err = mgrRuntime.Add(txSimContext, nil)
	assert.NotNil(t, err)
	errCert = false
}

func Test_DeleteCert(t *testing.T) {
	mgrRuntime, txSimContext, fn := initEnv(t)
	defer fn()

	result, err := mgrRuntime.Add(txSimContext, nil)
	assert.Equal(t, "e77c9238c51e3446d942f94bd8803cc4f351254f8771f972146d7bfc6e0be7f4", string(result))

	params := make(map[string][]byte)
	params[paramNameCertHashes] = []byte("e77c9238c51e3446d942f94bd8803cc4f351254f8771f972146d7bfc6e0be7f4")
	result, err = mgrRuntime.Delete(txSimContext, params)
	assert.Nil(t, err)
	result, err = mgrRuntime.Delete(txSimContext, params)
	assert.NotNil(t, err)

	_, err = mgrRuntime.Delete(txSimContext, nil)
	assert.NotNil(t, err)
}

var _ protocol.TxSimContext = (*mock.MockTxSimContext)(nil)

func initEnv(t *testing.T) (*CertManageRuntime, *mock.MockTxSimContext, func()) {
	r, m, f := initEnvCore(t)
	m.EXPECT().GetSender().Return(getOrg1Client1Signer()).AnyTimes()
	m.EXPECT().GetTx().DoAndReturn(
		func() *commonPb.Transaction {
			return &commonPb.Transaction{
				Sender: &commonPb.EndorsementEntry{
					Signer: getOrg1Client1Signer(),
				},
				Endorsers: []*commonPb.EndorsementEntry{
					{
						Signer:    getOrg1Client1Signer(),
						Signature: nil,
					},
				},
			}
		}).AnyTimes()
	return r, m, f
}

var cache = NewCacheMock()

func initEnvSender2(t *testing.T) (*CertManageRuntime, *mock.MockTxSimContext, func()) {
	r, m, f := initEnvCore(t)
	m.EXPECT().GetSender().Return(getOrg2Client1Signer()).AnyTimes()
	m.EXPECT().GetTx().DoAndReturn(
		func() *commonPb.Transaction {
			return &commonPb.Transaction{
				Sender: &commonPb.EndorsementEntry{
					Signer: getOrg1Client1Signer(),
				},
				Endorsers: []*commonPb.EndorsementEntry{
					{
						Signer:    getOrg2Client1Signer(),
						Signature: nil,
					},
				},
			}
		}).AnyTimes()
	return r, m, f
}

func initEnvCore(t *testing.T) (*CertManageRuntime, *mock.MockTxSimContext, func()) {
	certMgrRuntime := &CertManageRuntime{NewLogger()}
	ctrl := gomock.NewController(t)
	txSimContext := mock.NewMockTxSimContext(ctrl)

	acTest := mock.NewMockAccessControlProvider(ctrl)
	acTest.EXPECT().GetHashAlg().Return("SHA256").AnyTimes()
	txSimContext.EXPECT().GetAccessControl().Return(acTest, nil).AnyTimes()

	txSimContext.EXPECT().Put(gomock.Any(), gomock.Any(), gomock.Any()).DoAndReturn(
		func(name string, key []byte, value []byte) error {
			cache.Put(name, string(key), value)
			return nil
		}).AnyTimes()
	txSimContext.EXPECT().Get(gomock.Any(), gomock.Any()).DoAndReturn(
		func(name string, key []byte) ([]byte, error) {
			return cache.Get(name, string(key)), nil
		}).AnyTimes()
	txSimContext.EXPECT().Del(gomock.Any(), gomock.Any()).DoAndReturn(
		func(name string, key []byte) error {
			return cache.Del(name, string(key))
		}).AnyTimes()
	txSimContext.EXPECT().GetBlockHeight().Return(uint64(1)).AnyTimes()
	return certMgrRuntime, txSimContext, func() { ctrl.Finish() }
}

func getOrg1Client1Signer() *accesscontrol.Member {
	certStr := "-----BEGIN CERTIFICATE-----\nMIICijCCAi+gAwIBAgIDBS9vMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\nMTIwNzA2NTM0M1owgZExCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\nMQ8wDQYDVQQLEwZjbGllbnQxLDAqBgNVBAMTI2NsaWVudDEuc2lnbi53eC1vcmcx\nLmNoYWlubWFrZXIub3JnMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE56xayRx0\n/a8KEXPxRfiSzYgJ/sE4tVeI/ZbjpiUX9m0TCJX7W/VHdm6WeJLOdCDuLLNvjGTy\nt8LLyqyubJI5AKN7MHkwDgYDVR0PAQH/BAQDAgGmMA8GA1UdJQQIMAYGBFUdJQAw\nKQYDVR0OBCIEIMjAiM2eMzlQ9HzV9ePW69rfUiRZVT2pDBOMqM4WVJSAMCsGA1Ud\nIwQkMCKAIDUkP3EcubfENS6TH3DFczH5dAnC2eD73+wcUF/bEIlnMAoGCCqGSM49\nBAMCA0kAMEYCIQCWUHL0xisjQoW+o6VV12pBXIRJgdeUeAu2EIjptSg2GAIhAIxK\nLXpHIBFxIkmWlxUaanCojPSZhzEbd+8LRrmhEO8n\n-----END CERTIFICATE-----"
	if errCert {
		certStr = "-----BEGIN CERTIFICATE-----\nMIICijCCAi+gAwIBAgIDBS9vMAoCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\nMTIwNzA2NTM0M1owgZExCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\nMQ8wDQYDVQQLEwZjbGllbnQxLDAqBgNVBAMTI2NsaWVudDEuc2lnbi53eC1vcmcx\nLmNoYWlubWFrZXIub3JnMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE56xayRx0/a8KEXPxRfiSzYgJ/sE4tVeI/ZbjpiUX9m0TCJX7W/VHdm6WeJLOdCDuLLNvjGTy\nt8LLyqyubJI5AKN7MHkwDgYDVR0PAQH/BAQDAgGmMA8GA1UdJQQIMAYGBFUdJQAw\nKQYDVR0OBCIEIMjAiM2eMzlQ9HzV9ePW69rfUiRZVT2pDBOMqM4WVJSAMCsGA1Ud\nIwQkMCKAIDUkP3EcubfENS6TH3DFczH5dAnC2eD73+wcUF/bEIlnMAoGCCqGSM49\nBAMCA0kAMEYCIQCWUHisjQoW+o6VV12pBXIRJgdeUeAu2EIjptSg2GAIhAIxK\nLXpHIBFxIkmWlxUaanCojPSEbd+8LRrmhEO8n\n-----END CERTIFICATE-----"
	}
	return &accesscontrol.Member{
		OrgId:      "yz-org1.yzchain.org",
		MemberType: accesscontrol.MemberType_CERT,
		MemberInfo: []byte(certStr),
	}
}

func getOrg1Admin1Signer() *accesscontrol.Member {
	certStr := "-----BEGIN CERTIFICATE-----\nMIIChzCCAi2gAwIBAgIDAwGbMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\nMTIwNzA2NTM0M1owgY8xCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\nMQ4wDAYDVQQLEwVhZG1pbjErMCkGA1UEAxMiYWRtaW4xLnNpZ24ud3gtb3JnMS5j\naGFpbm1ha2VyLm9yZzBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABORqoYNAw8ax\n9QOD94VaXq1dCHguarSKqAruEI39dRkm8Vu2gSHkeWlxzvSsVVqoN6ATObi2ZohY\nKYab2s+/QA2jezB5MA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMCkG\nA1UdDgQiBCDZOtAtHzfoZd/OQ2Jx5mIMgkqkMkH4SDvAt03yOrRnBzArBgNVHSME\nJDAigCA1JD9xHLm3xDUukx9wxXMx+XQJwtng+9/sHFBf2xCJZzAKBggqhkjOPQQD\nAgNIADBFAiEAiGjIB8Wb8mhI+ma4F3kCW/5QM6tlxiKIB5zTcO5E890CIBxWDICm\nAod1WZHJajgnDQ2zEcFF94aejR9dmGBB/P//\n-----END CERTIFICATE-----\n"
	return &accesscontrol.Member{
		OrgId:      "yz-org1.yzchain.org",
		MemberType: accesscontrol.MemberType_CERT,
		MemberInfo: []byte(certStr),
	}
}

func getOrg2Client1Signer() *accesscontrol.Member {
	certStr := "-----BEGIN CERTIFICATE-----\nMIICiTCCAi+gAwIBAgIDA+zYMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMi5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcyLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\nMTIwNzA2NTM0M1owgZExCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcyLmNoYWlubWFrZXIub3Jn\nMQ8wDQYDVQQLEwZjbGllbnQxLDAqBgNVBAMTI2NsaWVudDEuc2lnbi53eC1vcmcy\nLmNoYWlubWFrZXIub3JnMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEZd92CJez\nCiOMzLSTrJfX5vIUArCycg05uKru2qFaX0uvZUCwNxbfSuNvkHRXE8qIBUhTbg1Q\nR9rOlfDY1WfgMaN7MHkwDgYDVR0PAQH/BAQDAgGmMA8GA1UdJQQIMAYGBFUdJQAw\nKQYDVR0OBCIEICfLatSyyebzRsLbnkNKZJULB2bZOtG+88NqvAHCsXa3MCsGA1Ud\nIwQkMCKAIPGP1bPT4/Lns2PnYudZ9/qHscm0pGL6Kfy+1CAFWG0hMAoGCCqGSM49\nBAMCA0gAMEUCIQDzHrEHrGNtoNfB8jSJrGJU1qcxhse74wmDgIdoGjvfTwIgabRJ\nJNvZKRpa/VyfYi3TXa5nhHRIn91ioF1dQroHQFc=\n-----END CERTIFICATE-----\n"
	return &accesscontrol.Member{
		OrgId:      "yz-org2.yzchain.org",
		MemberType: accesscontrol.MemberType_CERT,
		MemberInfo: []byte(certStr),
	}
}
func NewLogger() protocol.Logger {
	cmLogger := &test.GoLogger{}
	return cmLogger
}

const KeyFormat = "%s/%s"

func realKey(name, key string) string {
	return fmt.Sprintf(KeyFormat, name, key)
}

type CacheMock struct {
	content map[string][]byte
	lock    sync.Mutex
}

func NewCacheMock() *CacheMock {
	return &CacheMock{
		content: make(map[string][]byte, 64),
	}
}

func (c *CacheMock) Put(name, key string, value []byte) {
	c.lock.Lock()
	defer c.lock.Unlock()
	c.content[realKey(name, key)] = value
}

func (c *CacheMock) Get(name, key string) []byte {
	c.lock.Lock()
	defer c.lock.Unlock()
	return c.content[realKey(name, key)]
}

func (c *CacheMock) Del(name, key string) error {
	c.lock.Lock()
	defer c.lock.Unlock()
	delete(c.content, realKey(name, key))
	return nil
}

func (c *CacheMock) GetByKey(key string) []byte {
	c.lock.Lock()
	defer c.lock.Unlock()
	return c.content[key]
}

func (c *CacheMock) Keys() []string {
	c.lock.Lock()
	defer c.lock.Unlock()
	sc := make([]string, 0)
	for k := range c.content {
		sc = append(sc, k)
	}
	return sc
}
