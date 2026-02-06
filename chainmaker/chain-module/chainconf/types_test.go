/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package chainconf

import (
	"reflect"
	"testing"

	"chainmaker.org/chainmaker/common/v2/crypto"
	"chainmaker.org/chainmaker/common/v2/crypto/asym"
	"chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/config"
	"chainmaker.org/chainmaker/pb-go/v2/consensus"
	"chainmaker.org/chainmaker/pb-go/v2/syscontract"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/protocol/v2/mock"
	"chainmaker.org/chainmaker/protocol/v2/test"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

var (
	localLog = &test.GoLogger{}
)

const (
	cert   = "-----BEGIN CERTIFICATE-----\nMIICrzCCAlWgAwIBAgIDDYpTMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMi5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcyLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTMw\nMTIwNjA2NTM0M1owgYoxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcyLmNoYWlubWFrZXIub3Jn\nMRIwEAYDVQQLEwlyb290LWNlcnQxIjAgBgNVBAMTGWNhLnd4LW9yZzIuY2hhaW5t\nYWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAASlekil12ThyvibHhBn\ncDvu958HOdN5Db9YE8bZ5e7YYHsJ85P6jBhlt0eKTR/hiukIBVfYKYwmhpYq2eCb\nRYqco4GnMIGkMA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMA8GA1Ud\nEwEB/wQFMAMBAf8wKQYDVR0OBCIEIPGP1bPT4/Lns2PnYudZ9/qHscm0pGL6Kfy+\n1CAFWG0hMEUGA1UdEQQ+MDyCDmNoYWlubWFrZXIub3Jngglsb2NhbGhvc3SCGWNh\nLnd4LW9yZzIuY2hhaW5tYWtlci5vcmeHBH8AAAEwCgYIKoZIzj0EAwIDSAAwRQIg\nJV7mg6IeKBVSLrsDFpLOSEMFd9zKIxo3RRZiMAkdC3MCIQD/LG53Sb/IcNsCqjz9\noLXYNanXzZn1c1t4jPtMuE7nSw==\n-----END CERTIFICATE-----\n"
	nodeId = "QmTrsVrof7hvU79LmAMnJrmhTCUdaBoVNYDhHMUGaVQa6m"
)

func TestVerifyChainConfig(t *testing.T) {
	type args struct {
		cconfig *config.ChainConfig
	}

	log = newMockLogger(t)
	//cconfig := &config.ChainConfig{
	//	TrustRoots: []*config.TrustRootConfig{
	//		{
	//			OrgId: "org1",
	//			Root:  []string{cert},
	//		},
	//	},
	//	Consensus: &config.ConsensusConfig{
	//		Nodes: []*config.OrgConfig{
	//			{
	//				NodeId: []string{nodeId},
	//				OrgId:  "org1",
	//			},
	//		},
	//		Type: consensus.ConsensusType_TBFT,
	//	},
	//	Block: &config.BlockConfig{
	//		TxTimeout:       minTxTimeout + 100,
	//		BlockTxCapacity: 10,
	//		BlockSize:       10,
	//		BlockInterval:   minBlockInterval + 5,
	//		TxParameterSize: maxTxParameterSize - 10,
	//	},
	//	ChainId: "chain1",
	//	Core: &config.CoreConfig{
	//		ConsensusTurboConfig: nil,
	//	},
	//	Contract: &config.ContractConfig{
	//		EnableSqlSupport: true,
	//	},
	//}
	tests := []struct {
		name    string
		args    args
		want    *ChainConfig
		wantErr bool
	}{
		{
			name: "test0", // validateParams, return err, chainconfig trust_roots is nil
			args: args{
				cconfig: &config.ChainConfig{},
			},
			want:    nil,
			wantErr: true,
		},
		{
			name: "test1", // verifyAuthType, return err, invalid PEM string for certificate
			args: args{
				cconfig: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{"test"},
						},
					},
					Consensus: &config.ConsensusConfig{},
					Block:     &config.BlockConfig{},
					ChainId:   "chain1",
				},
			},
			want:    nil,
			wantErr: true,
		},
		{
			name: "test2", // trustRoots len less than minTrustRoots
			args: args{
				cconfig: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{cert},
						},
					},
					Consensus: &config.ConsensusConfig{},
					Block:     &config.BlockConfig{},
					ChainId:   "chain1",
				},
			},
			want:    nil,
			wantErr: true,
		},
		{
			name: "test3", // nodeIds len less than 1, nodeIds len is 0
			args: args{
				cconfig: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{cert},
						},
					},
					Consensus: &config.ConsensusConfig{},
					Block: &config.BlockConfig{
						TxTimeout: minTxTimeout - 100,
					},
					ChainId: "chain1",
				},
			},
			want:    nil,
			wantErr: true,
		},
		{
			name: "test4", // txTimeout less than minTxTimeout, cconfig.Block.TxTimeout
			args: args{
				cconfig: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{cert},
						},
					},
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{
							{
								NodeId: []string{nodeId},
								OrgId:  "org1",
							},
						},
					},
					Block: &config.BlockConfig{
						TxTimeout: minTxTimeout - 100,
					},
					ChainId: "chain1",
				},
			},
			want:    nil,
			wantErr: true,
		},
		{
			name: "test5", // blockTxCapacity less than minBlockTxCapacity, blockTxCapacity cconfig.Block.BlockTxCapacity
			args: args{
				cconfig: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{cert},
						},
					},
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{
							{
								NodeId: []string{nodeId},
								OrgId:  "org1",
							},
						},
					},
					Block: &config.BlockConfig{
						TxTimeout:       minTxTimeout + 100,
						BlockTxCapacity: 0,
					},
					ChainId: "chain1",
				},
			},
			want:    nil,
			wantErr: true,
		},
		{
			name: "test6", // blockSize less than minBlockSize, blockSize is cconfig.Block.BlockSize
			args: args{
				cconfig: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{cert},
						},
					},
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{
							{
								NodeId: []string{nodeId},
								OrgId:  "org1",
							},
						},
					},
					Block: &config.BlockConfig{
						TxTimeout:       minTxTimeout + 100,
						BlockTxCapacity: 10,
						BlockSize:       0,
					},
					ChainId: "chain1",
				},
			},
			want:    nil,
			wantErr: true,
		},
		{
			name: "test7", // blockInterval less than %minBlockInterval, blockInterval is cconfig.Block.BlockInterval
			args: args{
				cconfig: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{cert},
						},
					},
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{
							{
								NodeId: []string{nodeId},
								OrgId:  "org1",
							},
						},
					},
					Block: &config.BlockConfig{
						TxTimeout:       minTxTimeout + 100,
						BlockTxCapacity: 10,
						BlockSize:       10,
						BlockInterval:   minBlockInterval - 5,
					},
					ChainId: "chain1",
				},
			},
			want:    nil,
			wantErr: true,
		},
		{
			name: "test8", // Contract.EnableSqlSupport return err
			args: args{
				cconfig: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{cert},
						},
					},
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{
							{
								NodeId: []string{nodeId},
								OrgId:  "org1",
							},
						},
						Type: consensus.ConsensusType_MAXBFT,
					},
					Block: &config.BlockConfig{
						TxTimeout:       minTxTimeout + 100,
						BlockTxCapacity: 10,
						BlockSize:       10,
						BlockInterval:   minBlockInterval + 5,
					},
					ChainId: "chain1",
					Core: &config.CoreConfig{
						ConsensusTurboConfig: nil,
					},
					Contract: &config.ContractConfig{
						EnableSqlSupport: true,
					},
				},
			},
			want:    nil,
			wantErr: true,
		},
		{
			name: "test9", // txParameterSize should be (0,100] MB, txParameterSize
			args: args{
				cconfig: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{cert},
						},
					},
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{
							{
								NodeId: []string{nodeId},
								OrgId:  "org1",
							},
						},
						Type: consensus.ConsensusType_TBFT,
					},
					Block: &config.BlockConfig{
						TxTimeout:       minTxTimeout + 100,
						BlockTxCapacity: 10,
						BlockSize:       10,
						BlockInterval:   minBlockInterval + 5,
						TxParameterSize: maxTxParameterSize + 1,
					},
					ChainId: "chain1",
					Core: &config.CoreConfig{
						ConsensusTurboConfig: nil,
					},
					Contract: &config.ContractConfig{
						EnableSqlSupport: true,
					},
				},
			},
			want:    nil,
			wantErr: true,
		},
		//{
		//	name: "test10", // return err nil
		//	args: args{
		//		cconfig: cconfig,
		//	},
		//	want: &ChainConfig{
		//		ChainConfig: cconfig,
		//		NodeOrgIds: map[string][]string{
		//			"org1": {nodeId},
		//		},
		//		NodeIds: map[string]string{
		//			nodeId: nodeId,
		//		},
		//		CaRoots: map[string]struct{}{
		//			"org1": {},
		//		},
		//		ResourcePolicies: map[string]struct{}{
		//		},
		//	},
		//	wantErr: false,
		//},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := VerifyChainConfig(tt.args.cconfig)
			if (err != nil) != tt.wantErr {
				t.Errorf("VerifyChainConfig() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("VerifyChainConfig() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_verifyChainConfigTrustRoots1(t *testing.T) {
	type args struct {
		config  *config.ChainConfig
		mConfig *ChainConfig
		log     protocol.Logger
	}

	cert := "-----BEGIN CERTIFICATE-----\nMIICrzCCAlWgAwIBAgIDDYpTMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMi5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcyLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTMw\nMTIwNjA2NTM0M1owgYoxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcyLmNoYWlubWFrZXIub3Jn\nMRIwEAYDVQQLEwlyb290LWNlcnQxIjAgBgNVBAMTGWNhLnd4LW9yZzIuY2hhaW5t\nYWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAASlekil12ThyvibHhBn\ncDvu958HOdN5Db9YE8bZ5e7YYHsJ85P6jBhlt0eKTR/hiukIBVfYKYwmhpYq2eCb\nRYqco4GnMIGkMA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMA8GA1Ud\nEwEB/wQFMAMBAf8wKQYDVR0OBCIEIPGP1bPT4/Lns2PnYudZ9/qHscm0pGL6Kfy+\n1CAFWG0hMEUGA1UdEQQ+MDyCDmNoYWlubWFrZXIub3Jngglsb2NhbGhvc3SCGWNh\nLnd4LW9yZzIuY2hhaW5tYWtlci5vcmeHBH8AAAEwCgYIKoZIzj0EAwIDSAAwRQIg\nJV7mg6IeKBVSLrsDFpLOSEMFd9zKIxo3RRZiMAkdC3MCIQD/LG53Sb/IcNsCqjz9\noLXYNanXzZn1c1t4jPtMuE7nSw==\n-----END CERTIFICATE-----\n"

	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name: "test0", // trust root nil, return nil
			args: args{
				config: &config.ChainConfig{
					TrustRoots: nil,
				},
				mConfig: nil,
				log:     newMockLogger(t),
			},
			wantErr: false,
		},
		{
			name: "test1", // check root certificate failed, org id already exists
			args: args{
				config: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
						},
					},
				},
				mConfig: &ChainConfig{
					CaRoots: map[string]struct{}{
						"org1": {},
					},
				},
				log: newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test2", // check root certificate failed, org id already exists
			args: args{
				config: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{cert, cert},
						},
					},
				},
				mConfig: &ChainConfig{
					CaRoots: map[string]struct{}{
						"org2": {},
					},
				},
				log: newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test3", // check root certificate failed, org id already exists
			args: args{
				config: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{cert},
						},
					},
				},
				mConfig: &ChainConfig{
					CaRoots: map[string]struct{}{
						"org2": {},
					},
				},
				log: newMockLogger(t),
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if err := verifyChainConfigTrustRoots(tt.args.config, tt.args.mConfig, tt.args.log); (err != nil) != tt.wantErr {
				t.Errorf("verifyChainConfigTrustRoots() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func Test_verifyChainConfigTrustRoots(t *testing.T) {
	var err error
	cfg := &config.ChainConfig{}
	mConfig := &ChainConfig{
		ChainConfig:      cfg,
		NodeOrgIds:       make(map[string][]string),
		NodeIds:          make(map[string]string),
		CaRoots:          make(map[string]struct{}),
		ResourcePolicies: make(map[string]struct{}),
	}
	// nil
	err = verifyChainConfigTrustRoots(cfg, mConfig, localLog)
	assert.Nil(t, err)
	// normal
	cfg.TrustRoots = []*config.TrustRootConfig{
		{
			OrgId: "org1",
			Root:  []string{"-----BEGIN CERTIFICATE-----\nMIICrzCCAlWgAwIBAgIDDsPeMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTMw\nMTIwNjA2NTM0M1owgYoxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\nMRIwEAYDVQQLEwlyb290LWNlcnQxIjAgBgNVBAMTGWNhLnd4LW9yZzEuY2hhaW5t\nYWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAAT7NyTIKcjtUVeMn29b\nGKeEmwbefZ7g9Uk5GROl+o4k7fiIKNuty1rQHLQUvAvkpxqtlmOpPOZ0Qziu6Hw6\nhi19o4GnMIGkMA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMA8GA1Ud\nEwEB/wQFMAMBAf8wKQYDVR0OBCIEIDUkP3EcubfENS6TH3DFczH5dAnC2eD73+wc\nUF/bEIlnMEUGA1UdEQQ+MDyCDmNoYWlubWFrZXIub3Jngglsb2NhbGhvc3SCGWNh\nLnd4LW9yZzEuY2hhaW5tYWtlci5vcmeHBH8AAAEwCgYIKoZIzj0EAwIDSAAwRQIg\nar8CSuLl7pA4Iy6ytAMhR0kzy0WWVSElc+koVY6pF5sCIQCDs+vTD/9V1azmbDXX\nbjoWeEfXbFJp2X/or9f4UIvMgg==\n-----END CERTIFICATE-----\n", "-----BEGIN CERTIFICATE-----\nMIICrzCCAlWgAwIBAgIDDjhZMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMy5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmczLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTMw\nMTIwNjA2NTM0M1owgYoxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmczLmNoYWlubWFrZXIub3Jn\nMRIwEAYDVQQLEwlyb290LWNlcnQxIjAgBgNVBAMTGWNhLnd4LW9yZzMuY2hhaW5t\nYWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAAREQ8bC/Ocg6Nf1c0OG\nQXybPYWXT0fWygGvn2KgrBFQjq8NLOwXQPO4BYY1vYuBTTFl0Qf0uz7OvVPMcrmy\n6ZDXo4GnMIGkMA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMA8GA1Ud\nEwEB/wQFMAMBAf8wKQYDVR0OBCIEINGPZR0sVwrvYtFneTD6GUyBQflwpTmJ0qCs\ngvdFgIn9MEUGA1UdEQQ+MDyCDmNoYWlubWFrZXIub3Jngglsb2NhbGhvc3SCGWNh\nLnd4LW9yZzMuY2hhaW5tYWtlci5vcmeHBH8AAAEwCgYIKoZIzj0EAwIDSAAwRQIg\nOxwZGMwSa58xWiou+Bpi6YcerIwm7Lqsd+4OqjHZp8ACIQCGElUBWJt5EYKkxt3x\nNb1ypMnQXHMFaHZVOIACtGz2GA==\n-----END CERTIFICATE-----\n"},
		},
	}
	err = verifyChainConfigTrustRoots(cfg, mConfig, localLog)
	assert.Nil(t, err)

	// cert repeat for org2
	cfg.TrustRoots = append(cfg.TrustRoots,
		&config.TrustRootConfig{
			OrgId: "org2",
			Root:  []string{"-----BEGIN CERTIFICATE-----\nMIICrzCCAlWgAwIBAgIDDYpTMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMi5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcyLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTMw\nMTIwNjA2NTM0M1owgYoxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcyLmNoYWlubWFrZXIub3Jn\nMRIwEAYDVQQLEwlyb290LWNlcnQxIjAgBgNVBAMTGWNhLnd4LW9yZzIuY2hhaW5t\nYWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAASlekil12ThyvibHhBn\ncDvu958HOdN5Db9YE8bZ5e7YYHsJ85P6jBhlt0eKTR/hiukIBVfYKYwmhpYq2eCb\nRYqco4GnMIGkMA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMA8GA1Ud\nEwEB/wQFMAMBAf8wKQYDVR0OBCIEIPGP1bPT4/Lns2PnYudZ9/qHscm0pGL6Kfy+\n1CAFWG0hMEUGA1UdEQQ+MDyCDmNoYWlubWFrZXIub3Jngglsb2NhbGhvc3SCGWNh\nLnd4LW9yZzIuY2hhaW5tYWtlci5vcmeHBH8AAAEwCgYIKoZIzj0EAwIDSAAwRQIg\nJV7mg6IeKBVSLrsDFpLOSEMFd9zKIxo3RRZiMAkdC3MCIQD/LG53Sb/IcNsCqjz9\noLXYNanXzZn1c1t4jPtMuE7nSw==\n-----END CERTIFICATE-----\n", "-----BEGIN CERTIFICATE-----\nMIICrzCCAlWgAwIBAgIDDYpTMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMi5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcyLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTMw\nMTIwNjA2NTM0M1owgYoxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcyLmNoYWlubWFrZXIub3Jn\nMRIwEAYDVQQLEwlyb290LWNlcnQxIjAgBgNVBAMTGWNhLnd4LW9yZzIuY2hhaW5t\nYWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAASlekil12ThyvibHhBn\ncDvu958HOdN5Db9YE8bZ5e7YYHsJ85P6jBhlt0eKTR/hiukIBVfYKYwmhpYq2eCb\nRYqco4GnMIGkMA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMA8GA1Ud\nEwEB/wQFMAMBAf8wKQYDVR0OBCIEIPGP1bPT4/Lns2PnYudZ9/qHscm0pGL6Kfy+\n1CAFWG0hMEUGA1UdEQQ+MDyCDmNoYWlubWFrZXIub3Jngglsb2NhbGhvc3SCGWNh\nLnd4LW9yZzIuY2hhaW5tYWtlci5vcmeHBH8AAAEwCgYIKoZIzj0EAwIDSAAwRQIg\nJV7mg6IeKBVSLrsDFpLOSEMFd9zKIxo3RRZiMAkdC3MCIQD/LG53Sb/IcNsCqjz9\noLXYNanXzZn1c1t4jPtMuE7nSw==\n-----END CERTIFICATE-----\n"},
		})
	mConfig.CaRoots = make(map[string]struct{})
	err = verifyChainConfigTrustRoots(cfg, mConfig, newMockLogger(t))
	assert.NotNil(t, err)

	// org repeat
	cfg.TrustRoots = append(cfg.TrustRoots[:1], cfg.TrustRoots[2:]...)
	cfg.TrustRoots = append(cfg.TrustRoots,
		&config.TrustRootConfig{
			OrgId: "org1",
			Root:  []string{"-----BEGIN CERTIFICATE-----\nMIICrzCCAlWgAwIBAgIDDYpTMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMi5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcyLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTMw\nMTIwNjA2NTM0M1owgYoxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcyLmNoYWlubWFrZXIub3Jn\nMRIwEAYDVQQLEwlyb290LWNlcnQxIjAgBgNVBAMTGWNhLnd4LW9yZzIuY2hhaW5t\nYWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAASlekil12ThyvibHhBn\ncDvu958HOdN5Db9YE8bZ5e7YYHsJ85P6jBhlt0eKTR/hiukIBVfYKYwmhpYq2eCb\nRYqco4GnMIGkMA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMA8GA1Ud\nEwEB/wQFMAMBAf8wKQYDVR0OBCIEIPGP1bPT4/Lns2PnYudZ9/qHscm0pGL6Kfy+\n1CAFWG0hMEUGA1UdEQQ+MDyCDmNoYWlubWFrZXIub3Jngglsb2NhbGhvc3SCGWNh\nLnd4LW9yZzIuY2hhaW5tYWtlci5vcmeHBH8AAAEwCgYIKoZIzj0EAwIDSAAwRQIg\nJV7mg6IeKBVSLrsDFpLOSEMFd9zKIxo3RRZiMAkdC3MCIQD/LG53Sb/IcNsCqjz9\noLXYNanXzZn1c1t4jPtMuE7nSw==\n-----END CERTIFICATE-----\n", "-----BEGIN CERTIFICATE-----\nMIICrzCCAlWgAwIBAgIDDYpTMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMi5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcyLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTMw\nMTIwNjA2NTM0M1owgYoxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcyLmNoYWlubWFrZXIub3Jn\nMRIwEAYDVQQLEwlyb290LWNlcnQxIjAgBgNVBAMTGWNhLnd4LW9yZzIuY2hhaW5t\nYWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAASlekil12ThyvibHhBn\ncDvu958HOdN5Db9YE8bZ5e7YYHsJ85P6jBhlt0eKTR/hiukIBVfYKYwmhpYq2eCb\nRYqco4GnMIGkMA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMA8GA1Ud\nEwEB/wQFMAMBAf8wKQYDVR0OBCIEIPGP1bPT4/Lns2PnYudZ9/qHscm0pGL6Kfy+\n1CAFWG0hMEUGA1UdEQQ+MDyCDmNoYWlubWFrZXIub3Jngglsb2NhbGhvc3SCGWNh\nLnd4LW9yZzIuY2hhaW5tYWtlci5vcmeHBH8AAAEwCgYIKoZIzj0EAwIDSAAwRQIg\nJV7mg6IeKBVSLrsDFpLOSEMFd9zKIxo3RRZiMAkdC3MCIQD/LG53Sb/IcNsCqjz9\noLXYNanXzZn1c1t4jPtMuE7nSw==\n-----END CERTIFICATE-----\n"},
		})
	mConfig.CaRoots = make(map[string]struct{})
	err = verifyChainConfigTrustRoots(cfg, mConfig, newMockLogger(t))
	assert.NotNil(t, err)

	// cert/pem invalid
	cfg.TrustRoots = append(cfg.TrustRoots[:1], cfg.TrustRoots[2:]...)
	cfg.TrustRoots = append(cfg.TrustRoots,
		&config.TrustRootConfig{
			OrgId: "org3",
			Root:  []string{"-----BEGIN CERTIFICATE-----\nIICrzCCAlWgAwIBAgIDDYpTMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA5GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMi5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcyLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTMw\nMTIwNjA2NTM0M1owgYoxCzAJBgNVBAYTAaNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcyLmNoYWlubWFrZXIub3Jn\nMRIwEAYDVQQLEwlyb290LWNlcnQxIjAgBgNVBAMTGWNhLnd4LW9yZzIuY2hhaW5t\nYWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAASlekil12ThyvibHhBn\ncDvu958HOdN5Db9YE8bZ5e7YYHsJ85P6jBhlt0eKTR/hiukIBVfYKYwmhpYq2eCb\nRYqco4GnMIGkMA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMA8GA1Ud\nEwEB/wQFMAMBAf8wKQYDVR0OBCIEIPGP1bPT4/Lns2PnYudZ9/qHscm0pGL6Kfy+\n1CAFWG0hMEUGA1UdEQQ+MDyCDmNoYWlubWFrZXIub3Jngglsb2NhbGhvc3SCGWNh\nLnd4LW9yZzIuY2hhaW5tYWtlci5vcmeHBH8AAAEwCgYIKoZIzj0EAwIDSAAwRQIg\nJV7mg6IeKBVSLrsDFpLOSEMFd9zKIxo3RRZiMAkdC3MCIQD/LG53Sb/IcNsCqjz9\noLXYNanXzZn1c1t4jPtMuE7nSw==\n-----END CERTIFICATE-----\n", "-----BEGIN CERTIFICATE-----\nMIICrzCCAlWgAwIBAgIDDYpTMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMi5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcyLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTMw\nMTIwNjA2NTM0M1owgYoxCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcyLmNoYWlubWFrZXIub3Jn\nMRIwEAYDVQQLEwlyb290LWNlcnQxIjAgBgNVBAMTGWNhLnd4LW9yZzIuY2hhaW5t\nYWtlci5vcmcwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAASlekil12ThyvibHhBn\ncDvu958HOdN5Db9YE8bZ5e7YYHsJ85P6jBhlt0eKTR/hiukIBVfYKYwmhpYq2eCb\nRYqco4GnMIGkMA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMA8GA1Ud\nEwEB/wQFMAMBAf8wKQYDVR0OBCIEIPGP1bPT4/Lns2PnYudZ9/qHscm0pGL6Kfy+\n1CAFWG0hMEUGA1UdEQQ+MDyCDmNoYWlubWFrZXIub3Jngglsb2NhbGhvc3SCGWNh\nLnd4LW9yZzIuY2hhaW5tYWtlci5vcmeHBH8AAAEwCgYIKoZIzj0EAwIDSAAwRQIg\nJV7mg6IeKBVSLrsDFpLOSEMFd9zKIxo3RRZiMAkdC3MCIQD/LG53Sb/IcNsCqjz9\noLXYNanXzZn1c1t4jPtMuE7nSw==\n-----END CERTIFICATE-----\n"},
		})
	mConfig.CaRoots = make(map[string]struct{})
	err = verifyChainConfigTrustRoots(cfg, mConfig, newMockLogger(t))
	assert.NotNil(t, err)
}

func Test_verifyChainConfigTrustMembers(t *testing.T) {
	var err error
	cfg := &config.ChainConfig{}

	// nil
	err = verifyChainConfigTrustMembers(cfg)
	assert.Nil(t, err)

	// normal
	cfg.TrustMembers = []*config.TrustMemberConfig{
		{
			MemberInfo: "-----BEGIN CERTIFICATE-----\nMIICijCCAi+gAwIBAgIDBS9vMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\nMTIwNzA2NTM0M1owgZExCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\nMQ8wDQYDVQQLEwZjbGllbnQxLDAqBgNVBAMTI2NsaWVudDEuc2lnbi53eC1vcmcx\nLmNoYWlubWFrZXIub3JnMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE56xayRx0\n/a8KEXPxRfiSzYgJ/sE4tVeI/ZbjpiUX9m0TCJX7W/VHdm6WeJLOdCDuLLNvjGTy\nt8LLyqyubJI5AKN7MHkwDgYDVR0PAQH/BAQDAgGmMA8GA1UdJQQIMAYGBFUdJQAw\nKQYDVR0OBCIEIMjAiM2eMzlQ9HzV9ePW69rfUiRZVT2pDBOMqM4WVJSAMCsGA1Ud\nIwQkMCKAIDUkP3EcubfENS6TH3DFczH5dAnC2eD73+wcUF/bEIlnMAoGCCqGSM49\nBAMCA0kAMEYCIQCWUHL0xisjQoW+o6VV12pBXIRJgdeUeAu2EIjptSg2GAIhAIxK\nLXpHIBFxIkmWlxUaanCojPSZhzEbd+8LRrmhEO8n\n-----END CERTIFICATE-----",
			OrgId:      "org1",
			Role:       "client",
			NodeId:     "",
		},
	}
	err = verifyChainConfigTrustMembers(cfg)
	assert.Nil(t, err)

	// repeat
	cfg.TrustMembers = append(cfg.TrustMembers, &config.TrustMemberConfig{
		MemberInfo: "-----BEGIN CERTIFICATE-----\nMIICijCCAi+gAwIBAgIDBS9vMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\nMTIwNzA2NTM0M1owgZExCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\nMQ8wDQYDVQQLEwZjbGllbnQxLDAqBgNVBAMTI2NsaWVudDEuc2lnbi53eC1vcmcx\nLmNoYWlubWFrZXIub3JnMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE56xayRx0\n/a8KEXPxRfiSzYgJ/sE4tVeI/ZbjpiUX9m0TCJX7W/VHdm6WeJLOdCDuLLNvjGTy\nt8LLyqyubJI5AKN7MHkwDgYDVR0PAQH/BAQDAgGmMA8GA1UdJQQIMAYGBFUdJQAw\nKQYDVR0OBCIEIMjAiM2eMzlQ9HzV9ePW69rfUiRZVT2pDBOMqM4WVJSAMCsGA1Ud\nIwQkMCKAIDUkP3EcubfENS6TH3DFczH5dAnC2eD73+wcUF/bEIlnMAoGCCqGSM49\nBAMCA0kAMEYCIQCWUHL0xisjQoW+o6VV12pBXIRJgdeUeAu2EIjptSg2GAIhAIxK\nLXpHIBFxIkmWlxUaanCojPSZhzEbd+8LRrmhEO8n\n-----END CERTIFICATE-----",
		OrgId:      "org1",
		Role:       "client",
		NodeId:     "",
	})
	err = verifyChainConfigTrustMembers(cfg)
	assert.NotNil(t, err)

	// cert bad
	cfg.TrustMembers = append(cfg.TrustMembers[:1], cfg.TrustMembers[2:]...)
	cfg.TrustMembers = append(cfg.TrustMembers, &config.TrustMemberConfig{
		MemberInfo: "-----BEGIN CERTIFICATE-----\nMIICijCCAi+gAwIBAgIDBS9vMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\nMTIwNzA2NTM0M1owgZExCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\nMQ8wDQYDVQQLEwZjbGllbnQxLDAqBgNVBAMTI2NsaWVudDEuc2lnbi53eC1vcmcx\nLmNoYWlubWFrZXIub3JnMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE56xayRx0\n/a8KEXPxRfiSzYgJ/sE4tVeI/ZbjpiUX9m0TCJX7W/VHdm6WeJLOdCDuLLNvjGTy\nt8LLyqyubJI5AKN7MHkwDgYDVR0PAQH/BAQDAgGmMA8GA1UdJQQIMAYGBFUdJQAw\nKQYDVR0OBCIEIMjAiM2eMzlQ9HzV9ePW69rfUiRZVT2pDBOMqM4WVJSAMCsGA1Ud\nIwQkMCKAIDUkP3EcubfENS6TH3DFczH5dAnC2eD73+wcUF/bEIlnMAoGCCqGSM49\nBAMCA0kAMEYCIQCWUHL0xisjQoW+o6VV12pBXIRJgdeUeAu2EIjptSg2GAIhAIxK\nLXpHIBFxIkmWlxUaanCojPSZhzEbd+8LRrmhEO8n\n-----END CERTIFICATE-----",
		OrgId:      "org1",
		Role:       "client",
		NodeId:     "",
	})
	err = verifyChainConfigTrustMembers(cfg)
	assert.NotNil(t, err)

}

func TestVerifyAuthType(t *testing.T) {
	t.Log("TestVerifyAuthType")

	cfg := &config.ChainConfig{}

	if cfg.AuthType == "" {
		cfg.AuthType = protocol.PermissionedWithCert
	}

	mConfig := &ChainConfig{
		ChainConfig:      cfg,
		NodeOrgIds:       make(map[string][]string),
		NodeIds:          make(map[string]string),
		CaRoots:          make(map[string]struct{}),
		ResourcePolicies: make(map[string]struct{}),
	}

	// normal
	cfg.TrustMembers = []*config.TrustMemberConfig{
		{
			MemberInfo: "-----BEGIN CERTIFICATE-----\nMIICijCCAi+gAwIBAgIDBS9vMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\nMTIwNzA2NTM0M1owgZExCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\nMQ8wDQYDVQQLEwZjbGllbnQxLDAqBgNVBAMTI2NsaWVudDEuc2lnbi53eC1vcmcx\nLmNoYWlubWFrZXIub3JnMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE56xayRx0\n/a8KEXPxRfiSzYgJ/sE4tVeI/ZbjpiUX9m0TCJX7W/VHdm6WeJLOdCDuLLNvjGTy\nt8LLyqyubJI5AKN7MHkwDgYDVR0PAQH/BAQDAgGmMA8GA1UdJQQIMAYGBFUdJQAw\nKQYDVR0OBCIEIMjAiM2eMzlQ9HzV9ePW69rfUiRZVT2pDBOMqM4WVJSAMCsGA1Ud\nIwQkMCKAIDUkP3EcubfENS6TH3DFczH5dAnC2eD73+wcUF/bEIlnMAoGCCqGSM49\nBAMCA0kAMEYCIQCWUHL0xisjQoW+o6VV12pBXIRJgdeUeAu2EIjptSg2GAIhAIxK\nLXpHIBFxIkmWlxUaanCojPSZhzEbd+8LRrmhEO8n\n-----END CERTIFICATE-----",
			OrgId:      "org1",
			Role:       "client",
			NodeId:     "",
			//TxParameterSize: 10,
		},
	}

	// repeat
	cfg.TrustMembers = append(cfg.TrustMembers, &config.TrustMemberConfig{
		MemberInfo: "-----BEGIN CERTIFICATE-----\nMIICijCCAi+gAwIBAgIDBS9vMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\nMTIwNzA2NTM0M1owgZExCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\nMQ8wDQYDVQQLEwZjbGllbnQxLDAqBgNVBAMTI2NsaWVudDEuc2lnbi53eC1vcmcx\nLmNoYWlubWFrZXIub3JnMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE56xayRx0\n/a8KEXPxRfiSzYgJ/sE4tVeI/ZbjpiUX9m0TCJX7W/VHdm6WeJLOdCDuLLNvjGTy\nt8LLyqyubJI5AKN7MHkwDgYDVR0PAQH/BAQDAgGmMA8GA1UdJQQIMAYGBFUdJQAw\nKQYDVR0OBCIEIMjAiM2eMzlQ9HzV9ePW69rfUiRZVT2pDBOMqM4WVJSAMCsGA1Ud\nIwQkMCKAIDUkP3EcubfENS6TH3DFczH5dAnC2eD73+wcUF/bEIlnMAoGCCqGSM49\nBAMCA0kAMEYCIQCWUHL0xisjQoW+o6VV12pBXIRJgdeUeAu2EIjptSg2GAIhAIxK\nLXpHIBFxIkmWlxUaanCojPSZhzEbd+8LRrmhEO8n\n-----END CERTIFICATE-----",
		OrgId:      "org1",
		Role:       "client",
		NodeId:     "",
	})

	// cert bad
	cfg.TrustMembers = append(cfg.TrustMembers[:1], cfg.TrustMembers[2:]...)
	cfg.TrustMembers = append(cfg.TrustMembers, &config.TrustMemberConfig{
		MemberInfo: "-----BEGIN CERTIFICATE-----\nMIICijCCAi+gAwIBAgIDBS9vMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ\nMA4GA1UECamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt\nb3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD\nExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1\nMTIwNzA2NTM0M1owgZExCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw\nDgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn\nMQ8wDQYDVQQLEwZjbGllbnQxLDAqBgNVBAMTI2NsaWVudDEuc2lnbi53eC1vcmcx\nLmNoYWlubWFrZXIub3JnMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE56xayRx0\n/a8KEXPxRfiSzYgJ/sE4tVeI/ZbjpiUX9m0TCJX7W/VHdm6WeJLOdCDuLLNvjGTy\nt8LLyqyubJI5AKN7MHkwDgYDVR0PAQH/BAQDAgGmMA8GA1UdJQQIMAYGBFUdJQAw\nKQYDVR0OBCIEIMjAiM2eMzlQ9HzV9ePW69rfUiRZVT2pDBOMqM4WVJSAMCsGA1Ud\nIwQkMCKAIDUkP3EcubfENS6TH3DFczH5dAnC2eD73+wcUF/bEIlnMAoGCCqGSM49\nBAMCA0kAMEYCIQCWUHL0xisjQoW+o6VV12pBXIRJgdeUeAu2EIjptSg2GAIhAIxK\nLXpHIBFxIkmWlxUaanCojPSZhzEbd+8LRrmhEO8n\n-----END CERTIFICATE-----",
		OrgId:      "org1",
		Role:       "client",
		NodeId:     "",
	})

	err := verifyAuthType(cfg, mConfig, cfg.AuthType)
	require.NotNil(t, err)
}

func Test_verifyChainConfigConsensus(t *testing.T) {
	type args struct {
		config  *config.ChainConfig
		mConfig *ChainConfig
		log     protocol.Logger
	}

	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name: "test0", // config.Consensus == nil , return err nil
			args: args{
				config: &config.ChainConfig{
					Consensus: nil,
				},
				mConfig: nil,
				log:     newMockLogger(t),
			},
			wantErr: false,
		},
		{
			name: "test1", // config.Consensus != nil &&  config.Consensus.Nodes == nil, return err nil
			args: args{
				config: &config.ChainConfig{
					Consensus: &config.ConsensusConfig{
						Nodes: nil,
					},
				},
				mConfig: nil,
				log:     newMockLogger(t),
			},
			wantErr: false,
		},
		{
			name: "test2", // there is at least one consensus node
			args: args{
				config: &config.ChainConfig{
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{},
					},
				},
				mConfig: nil,
				log:     newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test3", // org id existed
			args: args{
				config: &config.ChainConfig{
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{
							{
								OrgId: "org1",
							},
						},
					},
				},
				mConfig: &ChainConfig{
					NodeOrgIds: map[string][]string{
						"org1": {},
					},
				},
				log: newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test4", // org id not in trust roots config
			args: args{
				config: &config.ChainConfig{
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{
							{
								OrgId:  "org1",
								NodeId: []string{nodeId},
							},
						},
					},
				},
				mConfig: &ChainConfig{
					NodeOrgIds: map[string][]string{
						"org2": {},
					},
					CaRoots: map[string]struct{}{
						"org2": {},
					},
				},
				log: newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test5", // verifyChainConfigConsensusNodesIds err
			args: args{
				config: &config.ChainConfig{
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{
							{
								OrgId:  "org1",
								NodeId: []string{"test"},
							},
						},
					},
				},
				mConfig: &ChainConfig{
					NodeOrgIds: map[string][]string{
						"org2": {},
					},
					CaRoots: map[string]struct{}{
						"org1": {},
					},
				},
				log: newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test6", // return nil err
			args: args{
				config: &config.ChainConfig{
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{
							{
								OrgId:  "org1",
								NodeId: []string{nodeId},
							},
						},
					},
				},
				mConfig: &ChainConfig{
					NodeOrgIds: map[string][]string{
						"org2": {},
					},
					CaRoots: map[string]struct{}{
						"org1": {},
					},
					NodeIds: map[string]string{
						"test": "",
					},
				},
				log: newMockLogger(t),
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if err := verifyChainConfigConsensus(tt.args.config, tt.args.mConfig, tt.args.log); (err != nil) != tt.wantErr {
				t.Errorf("verifyChainConfigConsensus() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func Test_verifyChainConfigConsensusNodesIds(t *testing.T) {
	type args struct {
		mConfig *ChainConfig
		node    *config.OrgConfig
		log     protocol.Logger
	}

	nodeId := "QmTrsVrof7hvU79LmAMnJrmhTCUdaBoVNYDhHMUGaVQa6m"
	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name: "test0", // node.NodeId > 0, wrong node id set
			args: args{
				mConfig: nil,
				node: &config.OrgConfig{
					NodeId: []string{"node1"},
				},
				log: newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test1", //node.NodeId > 0, node id existed
			args: args{
				mConfig: &ChainConfig{
					NodeIds: map[string]string{
						nodeId: "",
					},
				},
				node: &config.OrgConfig{
					NodeId: []string{nodeId},
				},
				log: newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test2", //node.NodeId > 0, return err nil
			args: args{
				mConfig: &ChainConfig{
					NodeIds: map[string]string{
						"test": "",
					},
				},
				node: &config.OrgConfig{
					NodeId: []string{nodeId},
				},
				log: newMockLogger(t),
			},
			wantErr: false,
		},
		{
			name: "test3", // node.NodeId == 0, return err nil
			args: args{
				mConfig: nil,
				node:    &config.OrgConfig{},
				log:     newMockLogger(t),
			},
			wantErr: false,
		},
		{
			name: "test4", // node.NodeId == 0, return err nil
			args: args{
				mConfig: nil,
				node:    &config.OrgConfig{},
				log:     newMockLogger(t),
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if err := verifyChainConfigConsensusNodesIds(tt.args.mConfig, tt.args.node, tt.args.log); (err != nil) != tt.wantErr {
				t.Errorf("verifyChainConfigConsensusNodesIds() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func Test_verifyChainConfigResourcePolicies(t *testing.T) {
	type args struct {
		config  *config.ChainConfig
		mConfig *ChainConfig
	}
	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name: "test0", // config.ResourcePolicies == nil, return nil
			args: args{
				config: &config.ChainConfig{
					ResourcePolicies: nil,
				},
				mConfig: nil,
			},
			wantErr: false,
		},
		{ //mConfig.ResourcePolicies[resourcePolicy.ResourceName] = struct{}{}
			name: "test1", // verifyPolicy err
			args: args{
				config: &config.ChainConfig{
					ResourcePolicies: []*config.ResourcePolicy{
						{
							ResourceName: syscontract.SystemContract_CHAIN_CONFIG.String() + "-" + syscontract.ChainConfigFunction_NODE_ID_ADD.String(),
							Policy: &accesscontrol.Policy{
								Rule: string(protocol.RuleSelf),
							},
						},
					},
				},
				mConfig: &ChainConfig{
					ResourcePolicies: map[string]struct{}{
						string(protocol.RuleSelf): {},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "test2", // resource name duplicate
			args: args{
				config: &config.ChainConfig{
					ResourcePolicies: []*config.ResourcePolicy{
						{
							ResourceName: syscontract.SystemContract_CHAIN_CONFIG.String() + "-" + syscontract.ChainConfigFunction_NODE_ID_ADD.String(),
							Policy:       nil,
						},
					},
				},
				mConfig: &ChainConfig{
					ResourcePolicies: map[string]struct{}{
						string(protocol.RuleSelf): {},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "test3", // return err nil
			args: args{
				config: &config.ChainConfig{
					ResourcePolicies: []*config.ResourcePolicy{
						{
							ResourceName: syscontract.SystemContract_CHAIN_CONFIG.String() + "-" + syscontract.ChainConfigFunction_NODE_ID_UPDATE.String(),
							Policy: &accesscontrol.Policy{
								Rule:     string(protocol.RuleMajority),
								RoleList: []string{"admin"},
							},
						},
						{
							ResourceName: syscontract.SystemContract_CHAIN_CONFIG.String() + "-" + syscontract.ChainConfigFunction_NODE_ID_UPDATE.String(),
							Policy: &accesscontrol.Policy{
								Rule:     string(protocol.RuleMajority),
								RoleList: []string{"admin"},
							},
						},
					},
				},
				mConfig: &ChainConfig{
					ResourcePolicies: map[string]struct{}{
						string(protocol.RuleMajority): {},
					},
				},
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if err := verifyChainConfigResourcePolicies(tt.args.config, tt.args.mConfig); (err != nil) != tt.wantErr {
				t.Errorf("verifyChainConfigResourcePolicies() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func Test_verifyPolicy(t *testing.T) {
	type args struct {
		resourcePolicy *config.ResourcePolicy
	}
	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name: "test0", // policy == nil, return nil, TODO can verify case policy == nil
			args: args{
				resourcePolicy: &config.ResourcePolicy{
					Policy: nil,
				},
			},
			wantErr: false,
		},
		{
			name: "test1", // self rule can only be used by NODE_ID_UPDATE or TRUST_ROOT_UPDATE
			args: args{
				resourcePolicy: &config.ResourcePolicy{
					Policy: &accesscontrol.Policy{
						Rule: string(protocol.RuleSelf),
					},
					ResourceName: syscontract.SystemContract_CHAIN_CONFIG.String() + "-" + syscontract.ChainConfigFunction_NODE_ID_ADD.String(),
				},
			},
			wantErr: true,
		},
		{
			name: "test2", // config rule[MAJORITY], role can only be admin or null
			args: args{
				resourcePolicy: &config.ResourcePolicy{
					Policy: &accesscontrol.Policy{
						Rule:     string(protocol.RuleMajority),
						RoleList: []string{"client"},
					},
					ResourceName: syscontract.SystemContract_CHAIN_CONFIG.String() + "-" + syscontract.ChainConfigFunction_NODE_ID_UPDATE.String(),
				},
			},
			wantErr: true,
		},
		{
			name: "test3", // config rule[MAJORITY], org_list param not allowed
			args: args{
				resourcePolicy: &config.ResourcePolicy{
					Policy: &accesscontrol.Policy{
						Rule:     string(protocol.RuleMajority),
						RoleList: []string{"admin"},
						OrgList:  []string{"org1"},
					},
					ResourceName: syscontract.SystemContract_CHAIN_CONFIG.String() + "-" + syscontract.ChainConfigFunction_NODE_ID_UPDATE.String(),
				},
			},
			wantErr: true,
		},
		{
			name: "test3", // return err nil
			args: args{
				resourcePolicy: &config.ResourcePolicy{
					Policy: &accesscontrol.Policy{
						Rule:     string(protocol.RuleMajority),
						RoleList: []string{"admin"},
					},
					ResourceName: syscontract.SystemContract_CHAIN_CONFIG.String() + "-" + syscontract.ChainConfigFunction_NODE_ID_UPDATE.String(),
				},
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if err := verifyPolicy(tt.args.resourcePolicy); (err != nil) != tt.wantErr {
				t.Errorf("verifyPolicy() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func Test_verifyChainConfigTrustRootsInPermissionedWithKey(t *testing.T) {
	type args struct {
		config  *config.ChainConfig
		mConfig *ChainConfig
		log     protocol.Logger
	}

	_, pk, _ := asym.GenerateKeyPairPEM(crypto.ECC_Secp256k1)
	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name: "test0", // check root certificate failed, org id already exists
			args: args{
				config: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
						},
					},
				},
				mConfig: &ChainConfig{
					CaRoots: map[string]struct{}{
						"org1": {},
					},
				},
				log: newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test1", //fail to decode public key
			args: args{
				config: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{"test1", "test1"},
						},
					},
				},
				mConfig: &ChainConfig{
					CaRoots: map[string]struct{}{
						"org2": {},
					},
				},
				log: newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test2", // check root certificate failed,trust root already exists in orgId, repeatCertCheck
			args: args{
				config: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{pk, pk},
						},
					},
				},
				mConfig: &ChainConfig{
					CaRoots: map[string]struct{}{
						"org2": {},
					},
				},
				log: newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test3", // return err nil
			args: args{
				config: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{pk},
						},
					},
				},
				mConfig: &ChainConfig{
					CaRoots: map[string]struct{}{
						"org2": {},
					},
				},
				log: newMockLogger(t),
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if err := verifyChainConfigTrustRootsInPermissionedWithKey(tt.args.config, tt.args.mConfig, tt.args.log); (err != nil) != tt.wantErr {
				t.Errorf("verifyChainConfigTrustRootsInPermissionedWithKey() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func Test_verifyChainConfigConsensusInPermissionedWithKey(t *testing.T) {
	type args struct {
		config  *config.ChainConfig
		mConfig *ChainConfig
		log     protocol.Logger
	}
	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name: "test0", // config.Consensus == nil , return err nil
			args: args{
				config: &config.ChainConfig{
					Consensus: nil,
				},
				mConfig: nil,
				log:     newMockLogger(t),
			},
			wantErr: false,
		},
		{
			name: "test1", // config.Consensus != nil &&  config.Consensus.Nodes == nil, return err nil
			args: args{
				config: &config.ChainConfig{
					Consensus: &config.ConsensusConfig{
						Nodes: nil,
					},
				},
				mConfig: nil,
				log:     newMockLogger(t),
			},
			wantErr: false,
		},
		{
			name: "test2", // there is at least one consensus node
			args: args{
				config: &config.ChainConfig{
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{},
					},
				},
				mConfig: nil,
				log:     newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test3", // org id existed
			args: args{
				config: &config.ChainConfig{
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{
							{
								OrgId: "org1",
							},
						},
					},
				},
				mConfig: &ChainConfig{
					NodeOrgIds: map[string][]string{
						"org1": {},
					},
				},
				log: newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test4", // verifyChainConfigConsensusNodesIds err
			args: args{
				config: &config.ChainConfig{
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{
							{
								OrgId:  "org1",
								NodeId: []string{"test"},
							},
						},
					},
				},
				mConfig: &ChainConfig{
					NodeOrgIds: map[string][]string{
						"org2": {},
					},
					CaRoots: map[string]struct{}{
						"org1": {},
					},
				},
				log: newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test5", // return err nil
			args: args{
				config: &config.ChainConfig{
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{
							{
								OrgId: "org1",
							},
						},
					},
				},
				mConfig: &ChainConfig{
					NodeOrgIds: map[string][]string{
						"org2": {},
					},
				},
				log: newMockLogger(t),
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if err := verifyChainConfigConsensusInPermissionedWithKey(tt.args.config, tt.args.mConfig, tt.args.log); (err != nil) != tt.wantErr {
				t.Errorf("verifyChainConfigConsensusInPermissionedWithKey() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func Test_verifyChainConfigConsensusInPublic(t *testing.T) {
	type args struct {
		config  *config.ChainConfig
		mConfig *ChainConfig
		log     protocol.Logger
	}
	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name: "test0", // config.Consensus == nil , return err nil
			args: args{
				config: &config.ChainConfig{
					Consensus: nil,
				},
				mConfig: nil,
				log:     newMockLogger(t),
			},
			wantErr: false,
		},
		{
			name: "test1", // config.Consensus != nil &&  config.Consensus.Nodes == nil, return err nil
			args: args{
				config: &config.ChainConfig{
					Consensus: &config.ConsensusConfig{
						Nodes: nil,
					},
				},
				mConfig: nil,
				log:     newMockLogger(t),
			},
			wantErr: false,
		},
		{
			name: "test2", // there is at least one consensus node
			args: args{
				config: &config.ChainConfig{
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{},
					},
				},
				mConfig: nil,
				log:     newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test3", // org id existed
			args: args{
				config: &config.ChainConfig{
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{
							{
								OrgId: "org1",
							},
						},
					},
				},
				mConfig: &ChainConfig{
					NodeOrgIds: map[string][]string{
						"org1": {},
					},
				},
				log: newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test4", // verifyChainConfigConsensusNodesIds err
			args: args{
				config: &config.ChainConfig{
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{
							{
								OrgId:  "org1",
								NodeId: []string{"test"},
							},
						},
					},
				},
				mConfig: &ChainConfig{
					NodeOrgIds: map[string][]string{
						"org2": {},
					},
					CaRoots: map[string]struct{}{
						"org1": {},
					},
				},
				log: newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test5", // return err nil
			args: args{
				config: &config.ChainConfig{
					Consensus: &config.ConsensusConfig{
						Nodes: []*config.OrgConfig{
							{
								OrgId: "org1",
							},
						},
					},
				},
				mConfig: &ChainConfig{
					NodeOrgIds: map[string][]string{
						"org2": {},
					},
				},
				log: newMockLogger(t),
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if err := verifyChainConfigConsensusInPublic(tt.args.config, tt.args.mConfig, tt.args.log); (err != nil) != tt.wantErr {
				t.Errorf("verifyChainConfigConsensusInPublic() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func Test_verifyChainConfigTrustRootsInPublic(t *testing.T) {
	type args struct {
		config  *config.ChainConfig
		mConfig *ChainConfig
		log     protocol.Logger
	}

	_, _, _ = asym.GenerateKeyPairPEM(crypto.ECC_Secp256k1)

	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name: "test0", // len(config.TrustRoots) != 1
			args: args{
				config:  &config.ChainConfig{},
				mConfig: nil,
				log:     newMockLogger(t),
			},
			wantErr: true,
		},
		{
			name: "test1", // config.TrustRoots[0].OrgId != protocol.Public
			args: args{
				config: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: protocol.PermissionedWithCert,
						},
					},
				},
				mConfig: nil,
				log:     newMockLogger(t),
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if err := verifyChainConfigTrustRootsInPublic(tt.args.config, tt.args.mConfig, tt.args.log); (err != nil) != tt.wantErr {
				t.Errorf("verifyChainConfigTrustRootsInPublic() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func Test_validateParams(t *testing.T) {
	type args struct {
		config *config.ChainConfig
	}
	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name: "test0", // chainconfig trust_roots is nil
			args: args{
				config: &config.ChainConfig{},
			},
			wantErr: true,
		},
		{
			name: "test1", // chainconfig consensus is nil
			args: args{
				config: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{"test"},
						},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "test2", // chainconfig block is nil
			args: args{
				config: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{"test"},
						},
					},
					Consensus: &config.ConsensusConfig{},
				},
			},
			wantErr: true,
		},
		{
			name: "test3", // 1.chainconfig vm is nil  2.chain id can only consist of numbers, letters and underscores and chainId length must less than 30
			args: args{
				config: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{"test"},
						},
					},
					Consensus: &config.ConsensusConfig{},
					Block:     &config.BlockConfig{},
				},
			},
			wantErr: true,
		},
		{
			name: "test4", // return err nil
			args: args{
				config: &config.ChainConfig{
					TrustRoots: []*config.TrustRootConfig{
						{
							OrgId: "org1",
							Root:  []string{"test"},
						},
					},
					Consensus: &config.ConsensusConfig{},
					Block:     &config.BlockConfig{},
					ChainId:   "chain1",
				},
			},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if err := validateParams(tt.args.config); (err != nil) != tt.wantErr {
				t.Errorf("validateParams() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func Test_setDefaultVm(t *testing.T) {
	type args struct {
		chainConf *config.ChainConfig
	}
	tests := []struct {
		name string
		args args
	}{
		{
			name: "test0",
			args: args{
				chainConf: &config.ChainConfig{},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			setDefaultVm(tt.args.chainConf)
		})
	}
}

func TestGetVerifier(t *testing.T) {
	type args struct {
		chainId       string
		consensusType consensus.ConsensusType
	}

	verifier := newMockVerifier(t)
	tests := []struct {
		name string
		args args
		want protocol.Verifier
	}{
		{
			name: "test0",
			args: args{
				chainId:       "chain1",
				consensusType: consensus.ConsensusType_MAXBFT,
			},
			want: nil,
		},
		{
			name: "test1",
			args: args{
				chainId:       "chain2",
				consensusType: consensus.ConsensusType_TBFT,
			},
			want: verifier,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.name == "test1" {
				chainConsensusVerifier = map[string]consensusVerifier{
					"chain2": {
						consensus.ConsensusType_TBFT: verifier,
					},
				}
			}
			if got := GetVerifier(tt.args.chainId, tt.args.consensusType); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("GetVerifier() = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_initChainConsensusVerifier(t *testing.T) {
	type args struct {
		chainId string
	}
	tests := []struct {
		name string
		args args
	}{
		{
			name: "test0",
			args: args{
				chainId: "chain1",
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			initChainConsensusVerifier(tt.args.chainId)
		})
	}
}

func TestIsNativeTxSucc(t *testing.T) {
	type args struct {
		tx *commonPb.Transaction
	}
	tests := []struct {
		name         string
		args         args
		wantContract string
		wantB        bool
	}{
		{
			name: "test0",
			args: args{
				tx: &commonPb.Transaction{
					Payload: &commonPb.Payload{
						TxType:       commonPb.TxType_INVOKE_CONTRACT,
						ContractName: "test",
					},
				},
			},
			wantContract: "",
			wantB:        false,
		},
		{
			name: "test1",
			args: args{
				tx: &commonPb.Transaction{
					Payload: &commonPb.Payload{
						TxType:       commonPb.TxType_QUERY_CONTRACT,
						ContractName: "test",
					},
				},
			},
			wantContract: "",
			wantB:        false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			gotContract, gotB := IsNativeTxSucc(tt.args.tx)
			if gotContract != tt.wantContract {
				t.Errorf("IsNativeTxSucc() gotContract = %v, want %v", gotContract, tt.wantContract)
			}
			if gotB != tt.wantB {
				t.Errorf("IsNativeTxSucc() gotB = %v, want %v", gotB, tt.wantB)
			}
		})
	}
}

func Test_isNativeTx(t *testing.T) {
	type args struct {
		tx *commonPb.Transaction
	}
	tests := []struct {
		name         string
		args         args
		wantContract string
		wantB        bool
	}{
		{
			name: "test0",
			args: args{
				tx: &commonPb.Transaction{
					Payload: &commonPb.Payload{
						TxType:       commonPb.TxType_INVOKE_CONTRACT,
						ContractName: "test",
					},
				},
			},
			wantContract: "test",
			wantB:        false,
		},
		{
			name: "test1",
			args: args{
				tx: &commonPb.Transaction{
					Payload: &commonPb.Payload{
						TxType: commonPb.TxType_QUERY_CONTRACT,
					},
				},
			},
			wantContract: "",
			wantB:        false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			gotContract, gotB := isNativeTx(tt.args.tx)
			if gotContract != tt.wantContract {
				t.Errorf("isNativeTx() gotContract = %v, want %v", gotContract, tt.wantContract)
			}
			if gotB != tt.wantB {
				t.Errorf("isNativeTx() gotB = %v, want %v", gotB, tt.wantB)
			}
		})
	}
}

func newMockVerifier(t *testing.T) protocol.Verifier {
	ctrl := gomock.NewController(t)
	return mock.NewMockVerifier(ctrl)
}

func newMockLogger(t *testing.T) protocol.Logger {
	ctrl := gomock.NewController(t)
	log := mock.NewMockLogger(ctrl)
	log.EXPECT().Errorf(gomock.Any(), gomock.Any()).AnyTimes()
	log.EXPECT().Error(gomock.Any()).AnyTimes()
	return log
}
