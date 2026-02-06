/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package accountmgr

import (
	"encoding/hex"
	"errors"
	"fmt"
	"reflect"
	"strconv"
	"testing"

	utilNative "chainmaker.org/chainmaker/vm-native/v2/common"

	crypto2 "chainmaker.org/chainmaker/common/v2/crypto"
	"chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	configPb "chainmaker.org/chainmaker/pb-go/v2/config"
	"chainmaker.org/chainmaker/pb-go/v2/consensus"
	"chainmaker.org/chainmaker/pb-go/v2/syscontract"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/protocol/v2/mock"
	"github.com/gogo/protobuf/proto"
	"github.com/golang/mock/gomock"
)

var (
	address = "ZX958f7550fe53d96e708b0fc95212812bec3141ed"
	pk      = `-----BEGIN PUBLIC KEY-----
MIGJAoGBALC7ewHv7ksky2qw/Zjeyfl5bxuv92/V31ZdrqnRQSxShVjhuANnjvVf
17SG2dlRZy68jkHTAeShomIqFTFKTkAnw5jki8UjQ6pBLTsgyoiPf+7eIRFWW65r
ADkhSM8WYKK5sn89v8lkKkUNBtlvbT2HtWw6QPW/3haU0k3yybupAgMBAAE=
-----END PUBLIC KEY-----`
	cert = `-----BEGIN CERTIFICATE-----
MIICrzCCAlWgAwIBAgIDDsPeMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ
MA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt
b3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD
ExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTMw
MTIwNjA2NTM0M1owgYoxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw
DgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn
MRIwEAYDVQQLEwlyb290LWNlcnQxIjAgBgNVBAMTGWNhLnd4LW9yZzEuY2hhaW5t
YWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAAT7NyTIKcjtUVeMn29b
GKeEmwbefZ7g9Uk5GROl+o4k7fiIKNuty1rQHLQUvAvkpxqtlmOpPOZ0Qziu6Hw6
hi19o4GnMIGkMA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMA8GA1Ud
EwEB/wQFMAMBAf8wKQYDVR0OBCIEIDUkP3EcubfENS6TH3DFczH5dAnC2eD73+wc
UF/bEIlnMEUGA1UdEQQ+MDyCDmNoYWlubWFrZXIub3Jngglsb2NhbGhvc3SCGWNh
Lnd4LW9yZzEuY2hhaW5tYWtlci5vcmeHBH8AAAEwCgYIKoZIzj0EAwIDSAAwRQIg
ar8CSuLl7pA4Iy6ytAMhR0kzy0WWVSElc+koVY6pF5sCIQCDs+vTD/9V1azmbDXX
bjoWeEfXbFJp2X/or9f4UIvMgg==
-----END CERTIFICATE-----
`
)

func TestAccountManagerRuntime_ChargeGasVm(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()

	mockChainConfig := &configPb.ChainConfig{
		Vm: &configPb.Vm{
			AddrType: configPb.AddrType_ZXL,
		},
		Crypto: &configPb.CryptoConfig{
			Hash: crypto2.CRYPTO_ALGO_SHA256,
		},
	}
	mockBlockchainStore := mock.NewMockBlockchainStore(c)
	mockBlockchainStore.EXPECT().GetLastChainConfig().Return(mockChainConfig, nil).AnyTimes()
	mockTxSimContext := mock.NewMockTxSimContext(c)
	mockTxSimContext.EXPECT().GetBlockchainStore().Return(mockBlockchainStore)

	logger := mock.NewMockLogger(c)
	logger.EXPECT().Debugf(gomock.Any(), gomock.Any()).Return().AnyTimes()
	logger.EXPECT().Debugf(gomock.Any(), gomock.Any(), gomock.Any()).Return().AnyTimes()
	gomock.InOrder(
		mockTxSimContext.EXPECT().Get(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(FrozenPrefix+address)).Return([]byte("0"), nil).AnyTimes(),
		mockTxSimContext.EXPECT().Get(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(AccountPrefix+address)).Return([]byte("10"), nil).AnyTimes(),
		mockTxSimContext.EXPECT().Put(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(AccountPrefix+address), []byte("5")).Return(nil).AnyTimes(),
	)

	type fields struct {
		log protocol.Logger
	}

	type args struct {
		txSimContext protocol.TxSimContext
		params       map[string][]byte
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    []byte
		wantErr bool
	}{
		{
			name: "good",
			fields: fields{
				log: logger,
			},
			args: args{
				txSimContext: mockTxSimContext,
				params: map[string][]byte{
					ChargePublicKey: []byte(pk),
					ChargeGasAmount: []byte("5"),
				},
			},
			want:    []byte(Success),
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			g := &AccountManagerRuntime{
				log: tt.fields.log,
			}
			got, err := g.ChargeGasVm(tt.args.txSimContext, tt.args.params)
			if (err != nil) != tt.wantErr {
				t.Errorf("ChargeGasVm() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("ChargeGasVm() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_FrozenAccount(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()

	mockChainConfig := &configPb.ChainConfig{
		Vm: &configPb.Vm{
			AddrType: configPb.AddrType_ZXL,
		},
		Crypto: &configPb.CryptoConfig{
			Hash: crypto2.CRYPTO_ALGO_SHA256,
		},
	}
	mockBlockchainStore := mock.NewMockBlockchainStore(c)
	mockBlockchainStore.EXPECT().GetLastChainConfig().Return(mockChainConfig, nil).AnyTimes()
	mockTxSimContext := mock.NewMockTxSimContext(c)
	mockTxSimContext.EXPECT().GetBlockchainStore().Return(mockBlockchainStore).AnyTimes()

	bcCtrl := gomock.NewController(t)
	bcTest := mock.NewMockBlockchainStore(bcCtrl)
	cfgTest := &configPb.ChainConfig{Vm: &configPb.Vm{AddrType: configPb.AddrType_ZXL}}
	bcTest.EXPECT().GetLastChainConfig().Return(cfgTest, nil).AnyTimes()
	mockTxSimContext.EXPECT().GetBlockchainStore().Return(bcTest).AnyTimes()
	mockTxSimContext.EXPECT().GetBlockVersion().Return(uint32(2300)).AnyTimes()
	member := &accesscontrol.Member{
		MemberType: accesscontrol.MemberType_PUBLIC_KEY,
		MemberInfo: []byte(pk),
	}

	chainConfigContractName := syscontract.SystemContract_CHAIN_CONFIG.String()
	chainConfig := &configPb.ChainConfig{
		AccountConfig: &configPb.GasAccountConfig{
			GasAdminAddress: address,
		},
	}

	chainConfigBytes, err := proto.Marshal(chainConfig)
	if err != nil {
		t.Error(err.Error())
		return
	}

	gomock.InOrder(
		mockTxSimContext.EXPECT().GetSender().Return(member).AnyTimes(),
		mockTxSimContext.EXPECT().Get(chainConfigContractName, []byte(chainConfigContractName)).Return(chainConfigBytes, nil).AnyTimes(),
		mockTxSimContext.EXPECT().Put(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(FrozenPrefix+address), []byte("1")).Return(nil).AnyTimes(),
	)

	logger := mock.NewMockLogger(c)
	logger.EXPECT().Infof(gomock.Any()).Return().AnyTimes().Return().AnyTimes()
	logger.EXPECT().Debugf(gomock.Any(), gomock.Any(), gomock.Any()).Return().AnyTimes()
	type fields struct {
		log protocol.Logger
	}
	type args struct {
		txSimContext protocol.TxSimContext
		params       map[string][]byte
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    []byte
		wantErr bool
	}{
		{
			name: "good",
			fields: fields{
				log: logger,
			},
			args: args{
				txSimContext: mockTxSimContext,
				params: map[string][]byte{
					AddressKey: []byte(address),
				},
			},
			want:    []byte(Success),
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			g := &AccountManagerRuntime{
				log: tt.fields.log,
			}
			got, err := g.FrozenAccount(tt.args.txSimContext, tt.args.params)
			if (err != nil) != tt.wantErr {
				t.Errorf("FrozenAccount() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("FrozenAccount() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_GetAccountStatus(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()

	mockChainConfig := &configPb.ChainConfig{
		Vm: &configPb.Vm{
			AddrType: configPb.AddrType_ZXL,
		},
		Crypto: &configPb.CryptoConfig{
			Hash: crypto2.CRYPTO_ALGO_SHA256,
		},
	}
	mockBlockchainStore := mock.NewMockBlockchainStore(c)
	mockBlockchainStore.EXPECT().GetLastChainConfig().Return(mockChainConfig, nil).AnyTimes()
	mockTxSimContext := mock.NewMockTxSimContext(c)
	mockTxSimContext.EXPECT().GetBlockchainStore().Return(mockBlockchainStore).AnyTimes()

	bcCtrl := gomock.NewController(t)
	bcTest := mock.NewMockBlockchainStore(bcCtrl)
	cfgTest := &configPb.ChainConfig{Vm: &configPb.Vm{AddrType: configPb.AddrType_ZXL}}
	bcTest.EXPECT().GetLastChainConfig().Return(cfgTest, nil).AnyTimes()
	mockTxSimContext.EXPECT().GetBlockchainStore().Return(bcTest).AnyTimes()
	mockTxSimContext.EXPECT().GetBlockVersion().Return(uint32(2300)).AnyTimes()
	mockTxSimContext.EXPECT().Get(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(FrozenPrefix+address)).Return([]byte("0"), nil).AnyTimes()

	logger := mock.NewMockLogger(c)
	logger.EXPECT().Debugf(gomock.Any(), gomock.Any(), gomock.Any()).Return().AnyTimes()
	logger.EXPECT().Infof(gomock.Any()).Return().AnyTimes().Return().AnyTimes()
	type fields struct {
		log protocol.Logger
	}
	type args struct {
		txSimContext protocol.TxSimContext
		params       map[string][]byte
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    []byte
		wantErr bool
	}{
		{
			name:   "unlock",
			fields: fields{log: logger},
			args: args{
				txSimContext: mockTxSimContext,
				params: map[string][]byte{
					AddressKey: []byte(address),
				},
			},
			want:    []byte(unlock),
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			g := &AccountManagerRuntime{
				log: tt.fields.log,
			}
			got, err := g.GetAccountStatus(tt.args.txSimContext, tt.args.params)
			if (err != nil) != tt.wantErr {
				t.Errorf("GetAccountStatus() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("GetAccountStatus() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_GetAdmin(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()

	chainConfigContractName := syscontract.SystemContract_CHAIN_CONFIG.String()
	chainConfig := &configPb.ChainConfig{
		AccountConfig: &configPb.GasAccountConfig{
			GasAdminAddress: address,
		},
	}

	chainConfigBytes, err := proto.Marshal(chainConfig)
	if err != nil {
		t.Error(err.Error())
		return
	}

	mockChainConfig := &configPb.ChainConfig{
		Vm: &configPb.Vm{
			AddrType: configPb.AddrType_ZXL,
		},
		Crypto: &configPb.CryptoConfig{
			Hash: crypto2.CRYPTO_ALGO_SHA256,
		},
	}
	mockBlockchainStore := mock.NewMockBlockchainStore(c)
	mockBlockchainStore.EXPECT().GetLastChainConfig().Return(mockChainConfig, nil).AnyTimes()
	mockTxSimContext := mock.NewMockTxSimContext(c)
	mockTxSimContext.EXPECT().GetBlockchainStore().Return(mockBlockchainStore).AnyTimes()
	mockTxSimContext.EXPECT().GetBlockVersion().Return(uint32(2300)).AnyTimes()
	mockTxSimContext.EXPECT().Get(chainConfigContractName, []byte(chainConfigContractName)).Return(chainConfigBytes, nil).AnyTimes()
	logger := mock.NewMockLogger(c)
	logger.EXPECT().Debugf(gomock.Any(), gomock.Any(), gomock.Any()).Return().AnyTimes()
	logger.EXPECT().Infof(gomock.Any()).Return().AnyTimes().Return().AnyTimes()

	type fields struct {
		log protocol.Logger
	}
	type args struct {
		txSimContext protocol.TxSimContext
		params       map[string][]byte
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    []byte
		wantErr bool
	}{
		{
			name:   "good",
			fields: fields{log: logger},
			args: args{
				txSimContext: mockTxSimContext,
				params:       nil,
			},
			want:    []byte(address),
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			g := &AccountManagerRuntime{
				log: tt.fields.log,
			}
			got, err := g.GetAdmin(tt.args.txSimContext, tt.args.params)
			if (err != nil) != tt.wantErr {
				t.Errorf("GetAdmin() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("GetAdmin() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_GetBalance(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()

	mockChainConfig := &configPb.ChainConfig{
		Vm: &configPb.Vm{
			AddrType: configPb.AddrType_ZXL,
		},
		Crypto: &configPb.CryptoConfig{
			Hash: crypto2.CRYPTO_ALGO_SHA256,
		},
	}
	mockBlockchainStore := mock.NewMockBlockchainStore(c)
	mockBlockchainStore.EXPECT().GetLastChainConfig().Return(mockChainConfig, nil).AnyTimes()
	mockTxSimContext := mock.NewMockTxSimContext(c)
	mockTxSimContext.EXPECT().GetBlockchainStore().Return(mockBlockchainStore).AnyTimes()
	mockTxSimContext.EXPECT().GetBlockVersion().Return(uint32(2300)).AnyTimes()
	mockTxSimContext.EXPECT().Get(syscontract.SystemContract_ACCOUNT_MANAGER.String(),
		[]byte(AccountPrefix+address)).Return([]byte("10"), nil).AnyTimes()
	logger := mock.NewMockLogger(c)
	logger.EXPECT().Debugf(gomock.Any(), gomock.Any(), gomock.Any()).Return().AnyTimes()
	logger.EXPECT().Infof(gomock.Any()).Return().AnyTimes().Return().AnyTimes()

	type fields struct {
		log protocol.Logger
	}
	type args struct {
		txSimContext protocol.TxSimContext
		params       map[string][]byte
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    []byte
		wantErr bool
	}{
		{
			name:   "good",
			fields: fields{log: logger},
			args: args{
				txSimContext: mockTxSimContext,
				params: map[string][]byte{
					AddressKey: []byte(address),
				},
			},
			want:    []byte("10"),
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			g := &AccountManagerRuntime{
				log: tt.fields.log,
			}
			got, err := g.GetBalance(tt.args.txSimContext, tt.args.params)
			if (err != nil) != tt.wantErr {
				t.Errorf("GetBalance() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("GetBalance() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_RechargeGas(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()
	member := &accesscontrol.Member{
		MemberType: accesscontrol.MemberType_PUBLIC_KEY,
		MemberInfo: []byte(pk),
	}
	chainConfigContractName := syscontract.SystemContract_CHAIN_CONFIG.String()
	chainConfig := &configPb.ChainConfig{
		AccountConfig: &configPb.GasAccountConfig{
			GasAdminAddress: address,
		},
	}

	chainConfigBytes, err := proto.Marshal(chainConfig)
	if err != nil {
		t.Error(err.Error())
		return
	}

	mockChainConfig := &configPb.ChainConfig{
		Vm: &configPb.Vm{
			AddrType: configPb.AddrType_ZXL,
		},
		Crypto: &configPb.CryptoConfig{
			Hash: crypto2.CRYPTO_ALGO_SHA256,
		},
	}
	mockBlockchainStore := mock.NewMockBlockchainStore(c)
	mockBlockchainStore.EXPECT().GetLastChainConfig().Return(mockChainConfig, nil).AnyTimes()
	mockTxSimContext := mock.NewMockTxSimContext(c)
	mockTxSimContext.EXPECT().GetBlockchainStore().Return(mockBlockchainStore).AnyTimes()
	mockTxSimContext.EXPECT().GetBlockVersion().Return(uint32(2300)).AnyTimes()
	gomock.InOrder(
		mockTxSimContext.EXPECT().GetSender().Return(member).AnyTimes(),
		mockTxSimContext.EXPECT().Get(chainConfigContractName, []byte(chainConfigContractName)).Return(chainConfigBytes, nil).AnyTimes(),
		mockTxSimContext.EXPECT().Get(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(AccountPrefix+address)).Return([]byte("100"), nil).AnyTimes(),
		mockTxSimContext.EXPECT().Put(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(AccountPrefix+address), []byte("200")).Return(nil).AnyTimes(),
	)

	logger := mock.NewMockLogger(c)
	logger.EXPECT().Infof(gomock.Any()).Return().AnyTimes().Return().AnyTimes()
	logger.EXPECT().Error().Return().AnyTimes()
	logger.EXPECT().Debugf(gomock.Any(), gomock.Any(), gomock.Any()).Return().AnyTimes()

	rechargeGasReq := &syscontract.RechargeGasReq{
		BatchRechargeGas: []*syscontract.RechargeGas{
			{
				Address:   address,
				GasAmount: 100,
			},
		},
	}
	rechargeGasReqBytes, err := rechargeGasReq.Marshal()
	if err != nil {
		t.Error(err.Error())
		return
	}
	type fields struct {
		log protocol.Logger
	}
	type args struct {
		txSimContext protocol.TxSimContext
		params       map[string][]byte
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    []byte
		wantErr bool
	}{
		{
			name:   "good",
			fields: fields{log: logger},
			args: args{
				txSimContext: mockTxSimContext,
				params: map[string][]byte{
					BatchRecharge: rechargeGasReqBytes,
				},
			},
			want:    []byte(Success),
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			g := &AccountManagerRuntime{
				log: tt.fields.log,
			}
			got, err := g.RechargeGas(tt.args.txSimContext, tt.args.params)
			if (err != nil) != tt.wantErr {
				t.Errorf("RechargeGas() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("RechargeGas() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_RefundGas(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()

	mockChainConfig := &configPb.ChainConfig{
		Vm: &configPb.Vm{
			AddrType: configPb.AddrType_ZXL,
		},
		Crypto: &configPb.CryptoConfig{
			Hash: crypto2.CRYPTO_ALGO_SHA256,
		},
	}
	mockBlockchainStore := mock.NewMockBlockchainStore(c)
	mockBlockchainStore.EXPECT().GetLastChainConfig().Return(mockChainConfig, nil).AnyTimes()
	mockTxSimContext := mock.NewMockTxSimContext(c)
	mockTxSimContext.EXPECT().GetBlockchainStore().Return(mockBlockchainStore).AnyTimes()
	mockTxSimContext.EXPECT().GetBlockVersion().Return(uint32(2300)).AnyTimes()
	member := &accesscontrol.Member{
		MemberType: accesscontrol.MemberType_PUBLIC_KEY,
		MemberInfo: []byte(pk),
	}

	chainConfigContractName := syscontract.SystemContract_CHAIN_CONFIG.String()
	chainConfig := &configPb.ChainConfig{
		AccountConfig: &configPb.GasAccountConfig{
			GasAdminAddress: address,
		},
	}

	chainConfigBytes, err := proto.Marshal(chainConfig)
	if err != nil {
		t.Error(err.Error())
		return
	}

	gomock.InOrder(
		mockTxSimContext.EXPECT().GetSender().Return(member).AnyTimes(),
		mockTxSimContext.EXPECT().Get(chainConfigContractName, []byte(chainConfigContractName)).Return(chainConfigBytes, nil).AnyTimes(),
		mockTxSimContext.EXPECT().Get(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(FrozenPrefix+address)).Return([]byte("0"), nil).AnyTimes(),
		mockTxSimContext.EXPECT().Get(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(AccountPrefix+address)).Return([]byte("100"), nil).AnyTimes(),
		mockTxSimContext.EXPECT().Put(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(AccountPrefix+address), []byte("90")).Return(nil).AnyTimes(),
	)

	logger := mock.NewMockLogger(c)
	logger.EXPECT().Debugf(gomock.Any(), gomock.Any(), gomock.Any()).Return().AnyTimes()
	logger.EXPECT().Infof(gomock.Any()).Return().AnyTimes().Return().AnyTimes()
	logger.EXPECT().Error(fmt.Errorf(" %s accout is frozened", address).Error()).Return().AnyTimes()
	type fields struct {
		log protocol.Logger
	}
	type args struct {
		txSimContext protocol.TxSimContext
		params       map[string][]byte
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    []byte
		wantErr bool
	}{
		{
			name: "good",
			fields: fields{
				log: logger,
			},
			args: args{
				txSimContext: mockTxSimContext,
				params: map[string][]byte{
					AddressKey:      []byte(address),
					ChargeGasAmount: []byte("10"),
				},
			},
			want:    []byte(Success),
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			g := &AccountManagerRuntime{
				log: tt.fields.log,
			}
			got, err := g.RefundGas(tt.args.txSimContext, tt.args.params)
			if (err != nil) != tt.wantErr {
				t.Errorf("RefundGas() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("RefundGas() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_RefundGasVm(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()

	mockChainConfig := &configPb.ChainConfig{
		Vm: &configPb.Vm{
			AddrType: configPb.AddrType_ZXL,
		},
		Crypto: &configPb.CryptoConfig{
			Hash: crypto2.CRYPTO_ALGO_SHA256,
		},
	}
	mockBlockchainStore := mock.NewMockBlockchainStore(c)
	mockBlockchainStore.EXPECT().GetLastChainConfig().Return(mockChainConfig, nil).AnyTimes()
	mockTxSimContext := mock.NewMockTxSimContext(c)
	mockTxSimContext.EXPECT().GetBlockchainStore().Return(mockBlockchainStore).AnyTimes()
	mockTxSimContext.EXPECT().GetBlockVersion().Return(uint32(2300)).AnyTimes()

	gomock.InOrder(
		mockTxSimContext.EXPECT().Get(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(AccountPrefix+address)).Return([]byte("100"), nil).AnyTimes(),
		mockTxSimContext.EXPECT().Put(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(AccountPrefix+address), []byte("110")).Return(nil).AnyTimes(),
	)
	logger := mock.NewMockLogger(c)
	logger.EXPECT().Infof(gomock.Any()).Return().AnyTimes().Return().AnyTimes()
	logger.EXPECT().Error(errors.New("error")).Return().AnyTimes()
	logger.EXPECT().Infof(gomock.Any(), gomock.Any(), gomock.Any()).Return().AnyTimes()
	type fields struct {
		log protocol.Logger
	}
	type args struct {
		txSimContext protocol.TxSimContext
		params       map[string][]byte
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    []byte
		wantErr bool
	}{
		{
			name: "good",
			fields: fields{
				log: logger,
			},
			args: args{
				txSimContext: mockTxSimContext,
				params: map[string][]byte{
					RechargeKey:       []byte(pk),
					RechargeAmountKey: []byte("10"),
				},
			},
			want:    []byte(Success),
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			g := &AccountManagerRuntime{
				log: tt.fields.log,
			}
			got, err := g.RefundGasVm(tt.args.txSimContext, tt.args.params)
			if (err != nil) != tt.wantErr {
				t.Errorf("RefundGasVm() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("RefundGasVm() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_SetAdmin(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()

	chainConfigContractName := syscontract.SystemContract_CHAIN_CONFIG.String()
	chainConfig := &configPb.ChainConfig{

		AccountConfig: &configPb.GasAccountConfig{
			GasAdminAddress: address,
		},
		TrustRoots: []*configPb.TrustRootConfig{
			{
				OrgId: address,
			},
		},

		Consensus: &configPb.ConsensusConfig{
			Type: consensus.ConsensusType_SOLO,
			Nodes: []*configPb.OrgConfig{
				{
					Address: []string{address},
					OrgId:   "chainmaker.org1",
					NodeId:  []string{"node1"},
				},
			},
		},
		Block: &configPb.BlockConfig{
			TxTimestampVerify: false,
		},
		ChainId: "chain1",
	}

	chainConfigBytes, err := proto.Marshal(chainConfig)
	if err != nil {
		t.Error(err.Error())
		return
	}
	mockTxSimContext := mock.NewMockTxSimContext(c)
	bcCtrl := gomock.NewController(t)
	bcTest := mock.NewMockBlockchainStore(bcCtrl)
	cfgTest := &configPb.ChainConfig{Vm: &configPb.Vm{AddrType: configPb.AddrType_ZXL}}
	bcTest.EXPECT().GetLastChainConfig().Return(cfgTest, nil).AnyTimes()
	mockTxSimContext.EXPECT().GetBlockchainStore().Return(bcTest).AnyTimes()
	gomock.InOrder(
		mockTxSimContext.EXPECT().Get(chainConfigContractName, []byte(chainConfigContractName)).Return(chainConfigBytes, nil).AnyTimes(),
	)
	logger := mock.NewMockLogger(c)
	logger.EXPECT().Infof(gomock.Any()).Return().AnyTimes().Return().AnyTimes()
	logger.EXPECT().Error("org id(chainmaker.org1) not in trust roots config").Return().AnyTimes()
	logger.EXPECT().Infof(gomock.Any(), gomock.Any(), gomock.Any()).Return().AnyTimes()

	type fields struct {
		log protocol.Logger
	}
	type args struct {
		txSimContext protocol.TxSimContext
		params       map[string][]byte
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    []byte
		wantErr bool
	}{
		//{
		//	name: "bad",
		//	fields: fields{
		//		log: logger,
		//	},
		//	args: args{
		//		txSimContext: mockTxSimContext,
		//		params: map[string][]byte{
		//			AddressKey: []byte(address),
		//		},
		//	},
		//	want:    []byte("org id(chainmaker.org1) not in trust roots config"),
		//	wantErr: true,
		//},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			g := &AccountManagerRuntime{
				log: tt.fields.log,
			}
			got, err := g.SetAdmin(tt.args.txSimContext, tt.args.params)
			if (err != nil) != tt.wantErr {
				t.Errorf("SetAdmin() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("SetAdmin() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_UnFrozenAccount(t *testing.T) {

	c := gomock.NewController(t)
	defer c.Finish()

	mockChainConfig := &configPb.ChainConfig{
		Vm: &configPb.Vm{
			AddrType: configPb.AddrType_ZXL,
		},
		Crypto: &configPb.CryptoConfig{
			Hash: crypto2.CRYPTO_ALGO_SHA256,
		},
	}
	mockBlockchainStore := mock.NewMockBlockchainStore(c)
	mockBlockchainStore.EXPECT().GetLastChainConfig().Return(mockChainConfig, nil).AnyTimes()
	mockTxSimContext := mock.NewMockTxSimContext(c)
	mockTxSimContext.EXPECT().GetBlockchainStore().Return(mockBlockchainStore).AnyTimes()
	mockTxSimContext.EXPECT().GetBlockVersion().Return(uint32(2300)).AnyTimes()

	member := &accesscontrol.Member{
		MemberType: accesscontrol.MemberType_PUBLIC_KEY,
		MemberInfo: []byte(pk),
	}

	chainConfigContractName := syscontract.SystemContract_CHAIN_CONFIG.String()
	chainConfig := &configPb.ChainConfig{
		AccountConfig: &configPb.GasAccountConfig{
			GasAdminAddress: address,
		},
	}

	chainConfigBytes, err := proto.Marshal(chainConfig)
	if err != nil {
		t.Error(err.Error())
		return
	}

	gomock.InOrder(
		mockTxSimContext.EXPECT().GetSender().Return(member).AnyTimes(),
		mockTxSimContext.EXPECT().Get(chainConfigContractName, []byte(chainConfigContractName)).
			Return(chainConfigBytes, nil).AnyTimes(),
		mockTxSimContext.EXPECT().Get(syscontract.SystemContract_ACCOUNT_MANAGER.String(),
			[]byte(FrozenPrefix+address)).Return([]byte("1"), nil).AnyTimes(),
		mockTxSimContext.EXPECT().Put(syscontract.SystemContract_ACCOUNT_MANAGER.String(),
			[]byte(FrozenPrefix+address), []byte(unlock)).Return(nil).AnyTimes(),
	)
	logger := mock.NewMockLogger(c)
	logger.EXPECT().Infof(gomock.Any()).Return().AnyTimes().Return().AnyTimes()
	logger.EXPECT().Error(errors.New("error")).Return().AnyTimes()
	logger.EXPECT().Debugf(gomock.Any(), gomock.Any(), gomock.Any()).Return().AnyTimes()

	type fields struct {
		log protocol.Logger
	}
	type args struct {
		txSimContext protocol.TxSimContext
		params       map[string][]byte
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    []byte
		wantErr bool
	}{
		{
			name: "good",
			fields: fields{
				log: logger,
			},
			args: args{
				txSimContext: mockTxSimContext,
				params: map[string][]byte{
					AddressKey: []byte(address),
				},
			},
			want:    []byte(Success),
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			g := &AccountManagerRuntime{
				log: tt.fields.log,
			}
			got, err := g.UnFrozenAccount(tt.args.txSimContext, tt.args.params)
			if (err != nil) != tt.wantErr {
				t.Errorf("UnFrozenAccount() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("UnFrozenAccount() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_chargeGas(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()
	mockTxSimContext := mock.NewMockTxSimContext(c)
	logger := mock.NewMockLogger(c)
	logger.EXPECT().Debugf(gomock.Any(), gomock.Any(), gomock.Any()).Return().AnyTimes()
	gomock.InOrder(
		mockTxSimContext.EXPECT().Get(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(FrozenPrefix+address)).Return([]byte("0"), nil).AnyTimes(),
		mockTxSimContext.EXPECT().Get(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(AccountPrefix+address)).Return([]byte("200"), nil).AnyTimes(),
		mockTxSimContext.EXPECT().Put(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(AccountPrefix+address), []byte("100")).Return(nil).AnyTimes(),
	)
	type fields struct {
		log protocol.Logger
	}
	type args struct {
		txSimContext         protocol.TxSimContext
		address              string
		chargeGasAmountBytes []byte
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    []byte
		wantErr bool
	}{
		{
			name:   "good",
			fields: fields{log: logger},
			args: args{
				txSimContext:         mockTxSimContext,
				address:              address,
				chargeGasAmountBytes: []byte("100"),
			},
			want:    []byte(Success),
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			g := &AccountManagerRuntime{
				log: tt.fields.log,
			}
			got, err := g.chargeGas(tt.args.txSimContext, tt.args.address, tt.args.chargeGasAmountBytes)
			if (err != nil) != tt.wantErr {
				t.Errorf("chargeGas() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("chargeGas() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_chargeGasForMultiUsers(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()
	mockTxSimContext := mock.NewMockTxSimContext(c)
	logger := mock.NewMockLogger(c)
	gomock.InOrder(
		mockTxSimContext.EXPECT().Get(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(FrozenPrefix+address)).Return([]byte("0"), nil).AnyTimes(),
		mockTxSimContext.EXPECT().Get(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(AccountPrefix+address)).Return([]byte("200"), nil).AnyTimes(),
		mockTxSimContext.EXPECT().Put(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(AccountPrefix+address), []byte("100")).Return(nil).AnyTimes(),
	)
	type fields struct {
		log protocol.Logger
	}
	type args struct {
		txSimContext         protocol.TxSimContext
		address              string
		chargeGasAmountBytes []byte
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    []byte
		wantErr bool
	}{
		{
			name:   "good",
			fields: fields{log: logger},
			args: args{
				txSimContext:         mockTxSimContext,
				address:              address,
				chargeGasAmountBytes: []byte("100"),
			},
			want:    []byte(Success),
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			g := &AccountManagerRuntime{
				log: tt.fields.log,
			}
			gasAmount, _ := strconv.ParseInt(string(tt.args.chargeGasAmountBytes), 10, 64)
			got, err := g.chargeGasForMultiAccount(tt.args.txSimContext, tt.args.address, gasAmount)
			if (err != nil) != tt.wantErr {
				t.Errorf("chargeGas() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("chargeGas() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_checkAdmin(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()
	logger := mock.NewMockLogger(c)
	//logger.EXPECT().Infof(gomock.Any()).Return().AnyTimes()
	logger.EXPECT().Debugf(gomock.Any(), gomock.Any(), gomock.Any()).Return().AnyTimes()
	loggerNotAdmin := mock.NewMockLogger(c)
	gomock.InOrder(
		//loggerNotAdmin.EXPECT().Infof("verify account address is:%v", "Zx958f7550fe53d96e708b0fc95212812bec3141ea"),
		loggerNotAdmin.EXPECT().Error(" gas admin address is illegal"),
	)

	chainConfigContractName := syscontract.SystemContract_CHAIN_CONFIG.String()
	chainConfig := &configPb.ChainConfig{
		AccountConfig: &configPb.GasAccountConfig{
			GasAdminAddress: address,
		},
		Crypto: &configPb.CryptoConfig{
			Hash: crypto2.CRYPTO_ALGO_SHA256,
		},
		Vm: &configPb.Vm{
			AddrType: configPb.AddrType_ZXL,
		},
	}
	chainConfigBytes, err := proto.Marshal(chainConfig)
	if err != nil {
		t.Fatalf("marshal error: %v", err)
	}

	chainConfigNotAdmin := &configPb.ChainConfig{
		AccountConfig: &configPb.GasAccountConfig{
			GasAdminAddress: "Zx958f7550fe53d96e708b0fc95212812bec3141ea",
		},
		Crypto: &configPb.CryptoConfig{
			Hash: crypto2.CRYPTO_ALGO_SHA256,
		},
		Vm: &configPb.Vm{
			AddrType: configPb.AddrType_ZXL,
		},
	}
	chainConfigNotAdminBytes, err := proto.Marshal(chainConfigNotAdmin)
	if err != nil {
		t.Fatalf("marshal error: %v", err)
	}

	mockBlockchainStore := mock.NewMockBlockchainStore(c)
	mockBlockchainStore.EXPECT().GetLastChainConfig().Return(chainConfig, nil).AnyTimes()

	mockTxSimContext := mock.NewMockTxSimContext(c)
	mockTxSimContext.EXPECT().GetBlockchainStore().Return(mockBlockchainStore).AnyTimes()
	mockTxSimContext.EXPECT().GetBlockVersion().Return(uint32(2300)).AnyTimes()
	mockTxSimContext.EXPECT().Get(chainConfigContractName, []byte(chainConfigContractName)).Return(chainConfigBytes, nil).AnyTimes()

	mockBlockchainStoreNotAmin := mock.NewMockBlockchainStore(c)
	mockBlockchainStoreNotAmin.EXPECT().GetLastChainConfig().Return(chainConfigNotAdmin, nil).AnyTimes()
	mockTxSimContextNotAdmin := mock.NewMockTxSimContext(c)
	mockTxSimContextNotAdmin.EXPECT().GetBlockchainStore().Return(mockBlockchainStore).AnyTimes()
	mockTxSimContextNotAdmin.EXPECT().GetBlockVersion().Return(uint32(2300)).AnyTimes()
	mockTxSimContextNotAdmin.EXPECT().Get(chainConfigContractName, []byte(chainConfigContractName)).Return(chainConfigNotAdminBytes, nil).AnyTimes()

	type fields struct {
		log protocol.Logger
	}
	type args struct {
		txSimContext  protocol.TxSimContext
		userPublicKey []byte
	}
	tests := []struct {
		name   string
		fields fields
		args   args
		want   bool
	}{
		{
			name: "admin",
			fields: fields{
				log: logger,
			},
			args: args{
				txSimContext:  mockTxSimContext,
				userPublicKey: []byte(pk),
			},
			want: true,
		},
		{
			name: "notAdmin",
			fields: fields{
				log: loggerNotAdmin,
			},
			args: args{
				txSimContext:  mockTxSimContextNotAdmin,
				userPublicKey: []byte(pk),
			},
			want: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			g := &AccountManagerRuntime{
				log: tt.fields.log,
			}
			if got := g.checkAdmin(tt.args.txSimContext, tt.args.userPublicKey); got != tt.want {
				t.Errorf("checkAdmin() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_checkFrozen(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()
	logger := mock.NewMockLogger(c)
	mockTxSimContext := mock.NewMockTxSimContext(c)
	mockTxSimContextUnfrozen := mock.NewMockTxSimContext(c)
	gomock.InOrder(
		mockTxSimContext.EXPECT().Get(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(FrozenPrefix+address)).Return([]byte("1"), nil),
		mockTxSimContextUnfrozen.EXPECT().Get(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(FrozenPrefix+address)).Return([]byte("0"), nil),
	)

	type fields struct {
		log protocol.Logger
	}
	type args struct {
		txSimContext protocol.TxSimContext
		address      string
	}
	tests := []struct {
		name   string
		fields fields
		args   args
		want   bool
	}{
		{
			name:   "frozen",
			fields: fields{log: logger},
			args: args{
				txSimContext: mockTxSimContext,
				address:      address,
			},
			want: true,
		},
		{
			name:   "unfrozen",
			fields: fields{log: logger},
			args: args{
				txSimContext: mockTxSimContextUnfrozen,
				address:      address,
			},
			want: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			g := &AccountManagerRuntime{
				log: tt.fields.log,
			}
			if got := g.checkFrozen(tt.args.txSimContext, tt.args.address); got != tt.want {
				t.Errorf("checkFrozen() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_getAccountBalance(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()
	logger := mock.NewMockLogger(c)
	mockTxSimContext := mock.NewMockTxSimContext(c)
	mockTxSimContext.EXPECT().Get(syscontract.SystemContract_ACCOUNT_MANAGER.String(), []byte(AccountPrefix+address)).Return([]byte("100"), nil)
	type fields struct {
		log protocol.Logger
	}
	type args struct {
		txSimContext protocol.TxSimContext
		accountKey   string
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    int64
		wantErr bool
	}{
		{
			name:   "good",
			fields: fields{log: logger},
			args: args{
				txSimContext: mockTxSimContext,
				accountKey:   AccountPrefix + address,
			},
			want:    100,
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			g := &AccountManagerRuntime{
				log: tt.fields.log,
			}
			got, err := g.getAccountBalance(tt.args.txSimContext, tt.args.accountKey)
			if (err != nil) != tt.wantErr {
				t.Errorf("getAccountBalance() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != tt.want {
				t.Errorf("getAccountBalance() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_getAdmin(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()
	logger := mock.NewMockLogger(c)
	logger.EXPECT().Infof(gomock.Any()).Return().AnyTimes()

	chainConfigContractName := syscontract.SystemContract_CHAIN_CONFIG.String()
	chainConfig := &configPb.ChainConfig{
		AccountConfig: &configPb.GasAccountConfig{
			GasAdminAddress: address,
		},
	}

	chainConfigBytes, err := proto.Marshal(chainConfig)
	if err != nil {
		t.Error(err.Error())
		return
	}
	mockTxSimContext := mock.NewMockTxSimContext(c)
	bcCtrl := gomock.NewController(t)
	bcTest := mock.NewMockBlockchainStore(bcCtrl)
	cfgTest := &configPb.ChainConfig{Vm: &configPb.Vm{AddrType: configPb.AddrType_ZXL}}
	bcTest.EXPECT().GetLastChainConfig().Return(cfgTest, nil).AnyTimes()
	mockTxSimContext.EXPECT().GetBlockchainStore().Return(bcTest).AnyTimes()
	mockTxSimContext.EXPECT().GetBlockVersion().Return(uint32(2300)).AnyTimes()
	mockTxSimContext.EXPECT().Get(chainConfigContractName, []byte(chainConfigContractName)).Return(chainConfigBytes, nil).AnyTimes()

	type fields struct {
		log protocol.Logger
	}
	type args struct {
		txSimContext protocol.TxSimContext
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    []byte
		wantErr bool
	}{
		{
			name:    "good",
			fields:  fields{log: logger},
			args:    args{txSimContext: mockTxSimContext},
			want:    []byte(address),
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			g := &AccountManagerRuntime{
				log: tt.fields.log,
			}
			got, err := g.getAdmin(tt.args.txSimContext)
			if (err != nil) != tt.wantErr {
				t.Errorf("getAdmin() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("getAdmin() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_getSenderPublicKey(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()
	logger := mock.NewMockLogger(c)
	mockTxSimContext := mock.NewMockTxSimContext(c)

	member := &accesscontrol.Member{
		MemberType: accesscontrol.MemberType_PUBLIC_KEY,
		MemberInfo: []byte(pk),
	}

	mockTxSimContext.EXPECT().GetSender().Return(member).AnyTimes()

	type fields struct {
		log protocol.Logger
	}
	type args struct {
		txSimContext protocol.TxSimContext
	}

	tests := []struct {
		name    string
		fields  fields
		args    args
		want    []byte
		wantErr bool
	}{
		{
			name:   "good",
			fields: fields{log: logger},
			args: args{
				txSimContext: mockTxSimContext,
			},
			want:    []byte(pk),
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := utilNative.GetSenderPublicKey(tt.args.txSimContext)
			if (err != nil) != tt.wantErr {
				t.Errorf("getSenderPublicKey() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("getSenderPublicKey() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAccountManagerRuntime_verifyAddress(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()
	loggerGood := mock.NewMockLogger(c)
	logger := mock.NewMockLogger(c)
	txSimContext := mock.NewMockTxSimContext(c)

	bcCtrl := gomock.NewController(t)
	bcTest := mock.NewMockBlockchainStore(bcCtrl)
	cfgTest := &configPb.ChainConfig{Vm: &configPb.Vm{AddrType: configPb.AddrType_ZXL}}
	bcTest.EXPECT().GetLastChainConfig().Return(cfgTest, nil).AnyTimes()
	txSimContext.EXPECT().GetBlockchainStore().Return(bcTest).AnyTimes()
	txSimContext.EXPECT().GetBlockVersion().Return(uint32(2300)).AnyTimes()
	gomock.InOrder(
		loggerGood.EXPECT().Infof("verify account address is:%v", address).Return().AnyTimes(),
		logger.EXPECT().Infof("verify account address is:%v", "ZX958f7550fe53d96e708b0fc95212812bec3141ed").Return().AnyTimes(),
	)

	type fields struct {
		log protocol.Logger
	}
	type args struct {
		address string
	}
	tests := []struct {
		name   string
		fields fields
		args   args
		want   string
		want1  bool
	}{
		{
			name: "good",
			fields: fields{
				log: loggerGood,
			},
			args: args{
				address: address,
			},
			want:  address,
			want1: true,
		},

		{
			name: "bad",
			fields: fields{
				log: logger,
			},
			args: args{
				address: "ZX958f7550fe53d96e708b0fc95212812bec3141ed",
			},
			want:  address,
			want1: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, got1 := utilNative.VerifyAndToLowerAddress(txSimContext, tt.args.address)
			if got != tt.want {
				t.Errorf("verifyAddress() got = %v, want %v", got, tt.want)
			}
			if got1 != tt.want1 {
				t.Errorf("verifyAddress() got1 = %v, want %v", got1, tt.want1)
			}
		})
	}
}

func Test_publicKeyFromCert(t *testing.T) {
	type args struct {
		member []byte
	}
	tests := []struct {
		name    string
		args    args
		want    string
		wantErr bool
	}{
		{
			name: "good",
			args: args{member: []byte(cert)},
			want: "2d2d2d2d2d424547494e205055424c4943204b45592d2d2d2d2d0a4d466b77457759484b6f5a497a6a3043415159494b6f5a497a6a304441516344516741452b7a636b79436e49375646586a4a39765778696e684a7347336e32650a3450564a4f526b547066714f4a4f333469436a627263746130427930464c774c354b6361725a5a6a71547a6d64454d34727568384f6f597466513d3d0a2d2d2d2d2d454e44205055424c4943204b45592d2d2d2d2d0a",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := publicKeyFromCert(tt.args.member)
			if (err != nil) != tt.wantErr {
				t.Errorf("publicKeyFromCert() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(hex.EncodeToString(got), tt.want) {
				t.Errorf("publicKeyFromCert() got = %v, want %v", hex.EncodeToString(got), tt.want)
			}
		})
	}
}

func Test_publicKeyToAddress(t *testing.T) {
	type args struct {
		publicKey []byte
	}
	tests := []struct {
		name    string
		args    args
		want    string
		wantErr bool
	}{
		{
			name:    "good",
			args:    args{publicKey: []byte(pk)},
			want:    address,
			wantErr: false,
		},
	}

	chainCfg := &configPb.ChainConfig{
		Vm: &configPb.Vm{
			AddrType: configPb.AddrType_ZXL,
		},
		Crypto: &configPb.CryptoConfig{
			Hash: crypto2.CRYPTO_ALGO_SHA256,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := utilNative.PublicKeyToAddress(tt.args.publicKey, chainCfg)
			if (err != nil) != tt.wantErr {
				t.Errorf("publicKeyToAddress() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != tt.want {
				t.Errorf("publicKeyToAddress() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_wholeCertInfo(t *testing.T) {
	c := gomock.NewController(t)
	defer c.Finish()
	mockTxSimContext := mock.NewMockTxSimContext(c)
	certHash := hex.EncodeToString([]byte(pk))
	mockTxSimContext.EXPECT().Get(syscontract.SystemContract_CERT_MANAGE.String(), []byte(certHash)).Return([]byte(pk), nil).AnyTimes()

	logger := mock.NewMockLogger(c)
	logger.EXPECT().Infof(gomock.Any()).Return().AnyTimes().Return().AnyTimes()
	logger.EXPECT().Error(errors.New("error")).Return().AnyTimes()

	type args struct {
		txSimContext protocol.TxSimContext
		certHash     string
	}
	tests := []struct {
		name    string
		args    args
		want    *commonPb.CertInfo
		wantErr bool
	}{
		{
			name: "good",
			args: args{
				txSimContext: mockTxSimContext,
				certHash:     certHash,
			},
			wantErr: false,
			want: &commonPb.CertInfo{
				Hash: certHash,
				Cert: []byte(pk),
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := wholeCertInfo(tt.args.txSimContext, tt.args.certHash)
			if (err != nil) != tt.wantErr {
				t.Errorf("wholeCertInfo() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("wholeCertInfo() got = %v, want %v", got, tt.want)
			}
		})
	}
}
