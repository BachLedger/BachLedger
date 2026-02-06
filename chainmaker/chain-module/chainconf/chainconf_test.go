/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package chainconf

import (
	"fmt"
	"reflect"
	"testing"

	msgbusMock "chainmaker.org/chainmaker/common/v2/msgbus/mock"
	"chainmaker.org/chainmaker/logger/v2"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/config"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/protocol/v2/mock"
	"github.com/golang/groupcache/lru"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"
	"github.com/test-go/testify/assert"
)

func TestGenesis(t *testing.T) {
	genesis, err := Genesis("./testdata/bc1.yml")
	require.Nil(t, err)
	fmt.Println(genesis)

	genesi2, err := Genesis("")
	require.NotNil(t, err)
	fmt.Println(genesi2)
}

func TestSetChainConf(t *testing.T) {
	chainConf, err := NewChainConf(WithChainId("chain1"))
	assert.Nil(t, err)
	for i := 0; i < 100; i++ {
		go func() {
			chainConfig := &config.ChainConfig{Contract: &config.ContractConfig{EnableSqlSupport: false}, ChainId: "chain1"}
			err = chainConf.SetChainConfig(chainConfig)
			assert.Nil(t, err)
			assert.Equal(t, chainConf.ChainConfig().ChainId, "chain1")
		}()
	}
}
func TestNewChainConf(t *testing.T) {
	type args struct {
		opts []Option
	}
	tests := []struct {
		name    string
		args    args
		want    *ChainConf
		wantErr bool
	}{
		{
			name: "test0",
			args: args{
				opts: []Option{
					func(f *options) error {
						f.chainId = "chain1"
						return nil
					},
				},
			},
			want: &ChainConf{
				watchers:   make([]protocol.Watcher, 0),
				vmWatchers: make(map[string][]protocol.VmWatcher),
				lru:        lru.New(100),
				configLru:  lru.New(10),
				log:        logger.GetLoggerByChain(logger.MODULE_CHAINCONF, "chain1"),
				options: options{
					chainId: "chain1",
				},
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := NewChainConf(tt.args.opts...)
			if (err != nil) != tt.wantErr {
				t.Errorf("NewChainConf() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("NewChainConf() got = %v, want %v", got, tt.want)
			}
		})
	}
}

//func TestGetBlockInCache(t *testing.T) {
//	t.Log("TestGetBlockInCache")
//	chainConf := newChainConf(t, 1)
//	commonBlock := getBlockInCache(chainConf.lru, chainConf.configLru, 1)
//	t.Log(commonBlock)
//}

func TestCallbackChainConfigWatcher(t *testing.T) {

	chainConf := newChainConf(t, 1)

	err := chainConf.callbackChainConfigWatcher()
	require.NotNil(t, err)
}

func TestCallbackContractVmWatcher(t *testing.T) {

	chainConf := newChainConf(t, 1)

	tx := &commonPb.Transaction{
		Result: &commonPb.Result{
			ContractResult: &commonPb.ContractResult{
				Result: []byte("sdddddd"),
			},
		},
	}

	contract, ok := IsNativeTxSucc(tx)

	if ok {
		payloadData, _ := tx.Payload.Marshal()
		if err := chainConf.callbackContractVmWatcher(contract, payloadData); err != nil {
			require.NotNil(t, err)
		}
	}
}

func TestChainConf_AddWatch(t *testing.T) {
	type fields struct {
		log        protocol.Logger
		options    options
		ChainConf  *config.ChainConfig
		watchers   []protocol.Watcher
		vmWatchers map[string][]protocol.VmWatcher
		lru        *lru.Cache
		configLru  *lru.Cache
	}
	type args struct {
		w protocol.Watcher
	}
	tests := []struct {
		name   string
		fields fields
		args   args
	}{
		{
			name:   "test0", // watch is nil
			fields: fields{},
			args:   args{},
		},
		{
			name: "test1", // chainconfig watcher existed
			fields: fields{
				log:       newMockLogger(t),
				options:   options{},
				ChainConf: nil,
				watchers: []protocol.Watcher{
					func() protocol.Watcher {
						watcher := newMockWatcher(t)
						watcher.EXPECT().Module().Return("test").AnyTimes()
						return watcher
					}(),
				},
				vmWatchers: nil,
				lru:        nil,
				configLru:  nil,
			},
			args: args{
				w: func() protocol.Watcher {
					watcher := newMockWatcher(t)
					watcher.EXPECT().Module().Return("test").AnyTimes()
					return watcher
				}(),
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &ChainConf{
				log:        tt.fields.log,
				options:    tt.fields.options,
				ChainConf:  tt.fields.ChainConf,
				watchers:   tt.fields.watchers,
				vmWatchers: tt.fields.vmWatchers,
				lru:        tt.fields.lru,
				configLru:  tt.fields.configLru,
			}
			c.AddWatch(tt.args.w)
		})
	}
}

func TestChainConf_AddVmWatch(t *testing.T) {
	type fields struct {
		log        protocol.Logger
		options    options
		ChainConf  *config.ChainConfig
		watchers   []protocol.Watcher
		vmWatchers map[string][]protocol.VmWatcher
		lru        *lru.Cache
		configLru  *lru.Cache
	}
	type args struct {
		w protocol.VmWatcher
	}
	tests := []struct {
		name   string
		fields fields
		args   args
	}{
		{
			name: "test0", // w is nil
			fields: fields{
				log: newMockLogger(t),
			},
			args: args{
				w: nil,
			},
		},
		{
			name: "test1", // w is not nil, contractNames is nil
			fields: fields{
				log:      newMockLogger(t),
				watchers: []protocol.Watcher{newMockWatcher(t)},
				vmWatchers: map[string][]protocol.VmWatcher{
					allContract: {func() protocol.VmWatcher {
						vmWatcher := newMockVmWatcher(t)
						vmWatcher.EXPECT().Module().Return("test").AnyTimes()
						vmWatcher.EXPECT().ContractNames().Return([]string{"test1"}).AnyTimes()
						return vmWatcher
					}()},
				},
			},
			args: args{
				w: func() protocol.VmWatcher {
					vmWatcher := newMockVmWatcher(t)
					vmWatcher.EXPECT().Module().Return("test").AnyTimes()
					vmWatcher.EXPECT().ContractNames().Return(nil).AnyTimes() // contractNames == nil
					return vmWatcher
				}(),
			},
		},
		{
			name: "test2", // w is not nil, contractNames is not nil
			fields: fields{
				log:      newMockLogger(t),
				watchers: []protocol.Watcher{newMockWatcher(t)},
				vmWatchers: map[string][]protocol.VmWatcher{
					allContract: {func() protocol.VmWatcher {
						vmWatcher := newMockVmWatcher(t)
						vmWatcher.EXPECT().Module().Return("test").AnyTimes()
						vmWatcher.EXPECT().ContractNames().Return([]string{"test1"}).AnyTimes()
						return vmWatcher
					}()},
				},
			},
			args: args{
				w: func() protocol.VmWatcher {
					vmWatcher := newMockVmWatcher(t)
					vmWatcher.EXPECT().Module().Return("test").AnyTimes()
					vmWatcher.EXPECT().ContractNames().Return([]string{"test2"}).AnyTimes() // contractNames is not nil
					return vmWatcher
				}(),
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &ChainConf{
				log:        tt.fields.log,
				options:    tt.fields.options,
				ChainConf:  tt.fields.ChainConf,
				watchers:   tt.fields.watchers,
				vmWatchers: tt.fields.vmWatchers,
				lru:        tt.fields.lru,
				configLru:  tt.fields.configLru,
			}
			c.AddVmWatch(tt.args.w)
		})
	}
}

func TestChainConf_addVmWatcherWithAllContract(t *testing.T) {
	type fields struct {
		log        protocol.Logger
		options    options
		ChainConf  *config.ChainConfig
		watchers   []protocol.Watcher
		vmWatchers map[string][]protocol.VmWatcher
		lru        *lru.Cache
		configLru  *lru.Cache
	}
	type args struct {
		w protocol.VmWatcher
	}

	tests := []struct {
		name   string
		fields fields
		args   args
	}{
		{
			name: "test0", // c.vmWatchers[allContract] is not ok
			fields: fields{
				log: newMockLogger(t),
				vmWatchers: map[string][]protocol.VmWatcher{
					"test": {func() protocol.VmWatcher {
						vmWatcher := newMockVmWatcher(t)
						vmWatcher.EXPECT().Module().Return("test").AnyTimes()
						return vmWatcher
					}()},
				},
			},
			args: args{
				w: nil,
			},
		},
		{
			name: "test1", // vm watcher existed
			fields: fields{
				log: newMockLogger(t),
				watchers: []protocol.Watcher{
					func() protocol.Watcher {
						watcher := newMockWatcher(t)
						watcher.EXPECT().Module().Return("test").AnyTimes()
						return watcher
					}(),
				},
				vmWatchers: map[string][]protocol.VmWatcher{
					allContract: {func() protocol.VmWatcher {
						vmWatcher := newMockVmWatcher(t)
						vmWatcher.EXPECT().Module().Return("test").AnyTimes()
						return vmWatcher
					}()},
				},
			},
			args: args{
				w: func() protocol.VmWatcher {
					vmWatcher := newMockVmWatcher(t)
					vmWatcher.EXPECT().Module().Return("test").AnyTimes()
					return vmWatcher
				}(),
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &ChainConf{
				log:        tt.fields.log,
				options:    tt.fields.options,
				ChainConf:  tt.fields.ChainConf,
				watchers:   tt.fields.watchers,
				vmWatchers: tt.fields.vmWatchers,
				lru:        tt.fields.lru,
				configLru:  tt.fields.configLru,
			}
			c.addVmWatcherWithAllContract(tt.args.w)
		})
	}
}

func TestChainConf_addVmWatcherWithContracts(t *testing.T) {
	type fields struct {
		log        protocol.Logger
		options    options
		ChainConf  *config.ChainConfig
		watchers   []protocol.Watcher
		vmWatchers map[string][]protocol.VmWatcher
		lru        *lru.Cache
		configLru  *lru.Cache
	}
	type args struct {
		w protocol.VmWatcher
	}
	tests := []struct {
		name   string
		fields fields
		args   args
	}{
		{
			name: "test0", // c.vmWatchers[contractName] not ok
			fields: fields{
				log:      newMockLogger(t),
				watchers: []protocol.Watcher{newMockWatcher(t)},
				vmWatchers: map[string][]protocol.VmWatcher{
					allContract: {func() protocol.VmWatcher {
						vmWatcher := newMockVmWatcher(t)
						vmWatcher.EXPECT().Module().Return("test").AnyTimes()
						vmWatcher.EXPECT().ContractNames().Return([]string{"test1"}).AnyTimes()
						return vmWatcher
					}()},
				},
			},
			args: args{
				w: func() protocol.VmWatcher {
					vmWatcher := newMockVmWatcher(t)
					vmWatcher.EXPECT().Module().Return("test").AnyTimes()
					vmWatcher.EXPECT().ContractNames().Return([]string{"test2"}).AnyTimes()
					return vmWatcher
				}(),
			},
		},
		{
			name: "test1", // vm watcher existed
			fields: fields{
				log:      newMockLogger(t),
				watchers: []protocol.Watcher{newMockWatcher(t)},
				vmWatchers: map[string][]protocol.VmWatcher{
					"test1": {func() protocol.VmWatcher {
						vmWatcher := newMockVmWatcher(t)
						vmWatcher.EXPECT().Module().Return("test").AnyTimes()
						vmWatcher.EXPECT().ContractNames().Return([]string{"test1"}).AnyTimes()
						return vmWatcher
					}()},
				},
			},
			args: args{
				w: func() protocol.VmWatcher {
					vmWatcher := newMockVmWatcher(t)
					vmWatcher.EXPECT().Module().Return("test").AnyTimes()
					vmWatcher.EXPECT().ContractNames().Return([]string{"test1"}).AnyTimes()
					return vmWatcher
				}(),
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &ChainConf{
				log:        tt.fields.log,
				options:    tt.fields.options,
				ChainConf:  tt.fields.ChainConf,
				watchers:   tt.fields.watchers,
				vmWatchers: tt.fields.vmWatchers,
				lru:        tt.fields.lru,
				configLru:  tt.fields.configLru,
			}
			c.addVmWatcherWithContracts(tt.args.w)
		})
	}
}

func newMockWatcher(t *testing.T) *mock.MockWatcher {
	ctrl := gomock.NewController(t)
	watcher := mock.NewMockWatcher(ctrl)
	return watcher
}

func newMockVmWatcher(t *testing.T) *mock.MockVmWatcher {
	ctrl := gomock.NewController(t)
	vmWatcher := mock.NewMockVmWatcher(ctrl)
	return vmWatcher
}

func newChainConf(t *testing.T, height uint64) *ChainConf {

	var chainId = "test1"

	ctrl := gomock.NewController(t)

	mock.NewMockWatcher(ctrl)
	bcStore := mock.NewMockBlockchainStore(ctrl)
	//block := createNewBlock(height, int64(height), chainId)
	//bcStore.EXPECT().GetBlock(height).Return(block, nil)
	//bcStore.EXPECT().ReadObject(syscontract.SystemContract_CHAIN_CONFIG.String(), []byte(syscontract.SystemContract_CHAIN_CONFIG.String())).Return([]byte("QmcQHCuAXaFkbcsPUj7e37hXXfZ9DdN7bozseo5oX4qiC4"), nil)

	bcStore.EXPECT().ReadObject(gomock.Any(), gomock.Any()).AnyTimes().Return(nil, nil)

	chainConf, err := NewChainConf(func(f *options) error {
		f.chainId = chainId
		f.msgBus = msgbusMock.NewMockMessageBus(ctrl)
		f.blockchainStore = bcStore

		return nil
	})

	if err != nil {
		t.Error("NewChainConf err", err)
		return nil
	}

	watcher := mock.NewMockWatcher(ctrl)

	watcher.EXPECT().Module().Return("test watcher").AnyTimes()

	//watcher.Module()

	chainConf.watchers = []protocol.Watcher{
		watcher,
	}

	return chainConf
}

//func createNewBlock(height uint64, timeStamp int64, chainId string) *commonPb.Block {
//	block := &commonPb.Block{
//		Header: &commonPb.BlockHeader{
//			BlockHeight:    height,
//			PreBlockHash:   nil,
//			BlockHash:      nil,
//			BlockVersion:   0,
//			DagHash:        nil,
//			RwSetRoot:      nil,
//			BlockTimestamp: timeStamp,
//			Proposer:       &acPb.Member{MemberInfo: []byte{1, 2, 3}},
//			ConsensusArgs:  nil,
//			TxCount:        0,
//			Signature:      nil,
//			ChainId:        chainId,
//		},
//		Dag: &commonPb.DAG{
//			Vertexes: nil,
//		},
//		Txs: nil,
//	}
//	block.Header.PreBlockHash = nil
//	return block
//}
