/*
 * Copyright (C) BABEC. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

package transactionmgr

import (
	"fmt"
	"sync"
	"testing"

	crypto2 "chainmaker.org/chainmaker/common/v2/crypto"

	"chainmaker.org/chainmaker/pb-go/v2/accesscontrol"

	configPb "chainmaker.org/chainmaker/pb-go/v2/config"

	"chainmaker.org/chainmaker/pb-go/v2/common"

	"chainmaker.org/chainmaker/protocol/v2/mock"
	"chainmaker.org/chainmaker/protocol/v2/test"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/assert"
)

func Test_AddBlacklistTxIds(t *testing.T) {
	ctrl := gomock.NewController(t)
	txSimContext := mock.NewMockTxSimContext(ctrl)
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
	pk := `-----BEGIN PUBLIC KEY-----
MIGJAoGBALC7ewHv7ksky2qw/Zjeyfl5bxuv92/V31ZdrqnRQSxShVjhuANnjvVf
17SG2dlRZy68jkHTAeShomIqFTFKTkAnw5jki8UjQ6pBLTsgyoiPf+7eIRFWW65r
ADkhSM8WYKK5sn89v8lkKkUNBtlvbT2HtWw6QPW/3haU0k3yybupAgMBAAE=
-----END PUBLIC KEY-----`
	member := &accesscontrol.Member{
		MemberType: accesscontrol.MemberType_PUBLIC_KEY,
		MemberInfo: []byte(pk),
	}
	txSimContext.EXPECT().GetSender().Return(member).AnyTimes()

	txSimContext.EXPECT().GetTx().DoAndReturn(
		func() *common.Transaction {
			return &common.Transaction{Payload: &common.Payload{TxId: "1", ChainId: "chain1"}}
		}).AnyTimes()

	mockChainConfig := &configPb.ChainConfig{
		AccountConfig: &configPb.GasAccountConfig{
			GasAdminAddress: "2d7e6a54bcbdd64fcb9c094961895be148d742a3",
		},
		Vm: &configPb.Vm{
			AddrType: configPb.AddrType_ETHEREUM,
		},
		Crypto: &configPb.CryptoConfig{
			Hash: crypto2.CRYPTO_ALGO_SHA256,
		},
	}
	txSimContext.EXPECT().GetLastChainConfig().Return(mockChainConfig).AnyTimes()

	defer ctrl.Finish()

	runtime := &TransactionMgr{log: &test.GoLogger{}}

	param := map[string][]byte{
		paramNameBlackTxIdList: []byte("txId1"),
	}
	paramEmpty := make(map[string][]byte, 0)

	// add
	r := runtime.AddBlacklistTxIds(txSimContext, param)
	assert.NotNil(t, r)
	assert.Equal(t, string(r.Result), "ok")

	r = runtime.AddBlacklistTxIds(txSimContext, paramEmpty)
	assert.NotNil(t, r)
	assert.Nil(t, r.Result)

	// query txId1 = "1"
	param[paramNameBlackTxId] = []byte("txId1")
	result := runtime.GetBlacklistTxIds(txSimContext, param)
	assert.NotNil(t, result)
	assert.Equal(t, "[\"1\"]", string(result.Result))

	// delete txId1
	result = runtime.DeleteBlacklistTxIds(txSimContext, param)
	assert.Equal(t, "ok", string(result.Result))
	result = runtime.DeleteBlacklistTxIds(txSimContext, paramEmpty)
	assert.Equal(t, 1, int(result.Code))

	// query  txId1 = ""
	result = runtime.GetBlacklistTxIds(txSimContext, param)
	assert.Equal(t, "[\"\"]", string(result.Result))
	result = runtime.GetBlacklistTxIds(txSimContext, paramEmpty)
	assert.Equal(t, 1, int(result.Code))
}

var cache = NewCacheMock()

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
