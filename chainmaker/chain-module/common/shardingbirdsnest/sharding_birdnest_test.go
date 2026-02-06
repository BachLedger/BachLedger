/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package shardingbirdsnest

import (
	"os"
	"strconv"
	"testing"

	bn "chainmaker.org/chainmaker/common/v2/birdsnest"
)

func TestNewShardingBirdsNest(t *testing.T) {
	_ = os.RemoveAll("./data")
	type args struct {
		length   int
		config   *ShardingBirdsNestConfig
		exitC    chan struct{}
		strategy bn.Strategy
		alg      ShardingAlgorithm
	}
	tests := []struct {
		name    string
		args    args
		want    *ShardingBirdsNest
		wantErr bool
	}{
		{
			name: "正常流",
			args: args{
				length: 5,
				config: &ShardingBirdsNestConfig{
					ChainId: "chain1",
					Length:  10,
					Timeout: 10,
					Birdsnest: &bn.BirdsNestConfig{
						ChainId: "chain1",
						Length:  5,
						Rules:   &bn.RulesConfig{AbsoluteExpireTime: 10000},
						Cuckoo: &bn.CuckooConfig{
							KeyType:       bn.KeyType_KTDefault,
							TagsPerBucket: 4,
							BitsPerItem:   9,
							MaxNumKeys:    10,
							TableType:     1,
						},
						Snapshot: &bn.SnapshotSerializerConfig{
							Type:        bn.SerializeIntervalType_Timed,
							Timed:       &bn.TimedSerializeIntervalConfig{Interval: 20},
							BlockHeight: &bn.BlockHeightSerializeIntervalConfig{Interval: 20},
							Path:        bn.TestDir + strconv.Itoa(01),
						},
					},
					Snapshot: &bn.SnapshotSerializerConfig{
						Type:        bn.SerializeIntervalType_Timed,
						Timed:       &bn.TimedSerializeIntervalConfig{Interval: 20},
						BlockHeight: &bn.BlockHeightSerializeIntervalConfig{Interval: 20},
						Path:        bn.TestDir + strconv.Itoa(02),
					},
				},
				exitC:    make(chan struct{}),
				strategy: bn.LruStrategy,
				alg:      NewModuloSA(5),
			},
			want: nil,
			//func() *ShardingBirdsNest {
			//	nest, _ := NewShardingBirdsNest(
			//		5,
			//		&bn.BirdsNestConfig{
			//			Length:             5,
			//			absoluteExpireTime: 5000,
			//			Cuckoo: &bn.CuckooConfig{
			//				TagsPerBucket: 4,
			//				BitsPerItem:   9,
			//				MaxNumKeys:    10,
			//				TableType:     1,
			//			},
			//			Snapshot: &bn.SnapshotSerializerConfig{
			//				SerializeInterval: 5,
			//				Path:              bn.TestDir + strconv.Itoa(0),
			//			},
			//		},
			//		make(chan struct{}),
			//		bn.LruStrategy,
			//		NewModuloSA(5),
			//	)
			//	return nest
			//}()
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := NewShardingBirdsNest(tt.args.config, tt.args.exitC, tt.args.strategy, tt.args.alg, bn.TestLogger{})
			if (err != nil) != tt.wantErr {
				t.Errorf("NewShardingBirdsNest() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got == tt.want {
				t.Errorf("NewShardingBirdsNest() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestShardingBirdsNest_Add(t *testing.T) {
	type fields struct {
		bn *ShardingBirdsNest
	}
	type args struct {
		key bn.Key
	}
	tests := []struct {
		name   string
		fields fields
		args   args
		want   bool
	}{
		{
			name:   "正常流 添加后存在",
			fields: fields{bn: getSBN(10, bn.TestDir+"test_add1", t)},
			args:   args{key: bn.GetTimestampKey()},
			want:   true,
		},
		{
			name:   "正常流 添加后不存在",
			fields: fields{bn: getSBN(2, bn.TestDir+"test_add2", t)},
			args:   args{key: bn.GetTimestampKey()},
			want:   true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {

			err := tt.fields.bn.Add(tt.args.key)
			if err != nil {
				t.Errorf("Add() err = %v", err)
			}
			contains, err := tt.fields.bn.Contains(tt.args.key)
			if err != nil {
				t.Errorf("Contains() err = %v", err)
				return
			}
			if contains != tt.want {
				t.Errorf("Contains() got = %v, want %v", err, tt.want)
			}
		})
	}
}

func TestShardingBirdsNest_Adds(t *testing.T) {
	var keys []bn.Key
	type fields struct {
		bn *ShardingBirdsNest
	}
	type args struct {
		keys []bn.Key
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		wantErr bool
	}{
		{
			name:   "正常流 插入超过容量大小的key",
			fields: fields{bn: getSBN(3, bn.TestDir+"test_adds1", t)},
			args: args{keys: func() []bn.Key {
				return bn.GetTimestampKeys(200)
			}()},
			wantErr: false,
		},
		{
			name:   "正常流 检查不存在",
			fields: fields{bn: getSBN(4, bn.TestDir+"test_adds2", t)},
			args: args{keys: func() []bn.Key {
				keys = append(keys, bn.GetTimestampKeys(200)...)
				return keys
			}()},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.fields.bn.Adds(tt.args.keys)
			if (err != nil) != tt.wantErr {
				t.Errorf("Adds() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			for _, key := range tt.args.keys {
				contains, err := tt.fields.bn.Contains(key)
				t.Logf("Contains() contains = %v, err %v", contains, err)
			}
		})
	}
}

func TestShardingBirdsNest_Contains(t *testing.T) {
	key1 := bn.GetTimestampKey()
	type fields struct {
		bn *ShardingBirdsNest
	}
	type args struct {
		key bn.Key
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    bool
		wantErr bool
	}{
		{
			name: "正常流 存在",
			fields: fields{bn: func() *ShardingBirdsNest {
				tbn := getSBN(5, bn.TestDir+"test_contains1", t)
				_ = tbn.Add(key1)
				return tbn
			}()},
			args:    args{key: key1},
			want:    true,
			wantErr: false,
		},
		{
			name: "正常流 不存在",
			fields: fields{bn: func() *ShardingBirdsNest {
				tbn := getSBN(6, bn.TestDir+"test_contains2", t)
				_ = tbn.Add(key1)
				return tbn
			}()},
			args:    args{key: bn.GetTimestampKey()},
			want:    false,
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := tt.fields.bn.Contains(tt.args.key)
			if (err != nil) != tt.wantErr {
				t.Errorf("Contains() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != tt.want {
				t.Errorf("Contains() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestShardingBirdsNest_GetHeight(t *testing.T) {
	type fields struct {
		bn *ShardingBirdsNest
	}
	tests := []struct {
		name   string
		fields fields
		want   uint64
	}{
		{
			name: "正常流",
			fields: fields{bn: func() *ShardingBirdsNest {
				tbn := getSBN(7, bn.TestDir+"test_getheight1", t)
				tbn.height = 2
				return tbn
			}()},
			want: 2,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.fields.bn.GetHeight(); got != tt.want {
				t.Errorf("GetHeight() = %v, want %v", got, tt.want)
			}
		})
	}
}

// This test is not implemented
func TestShardingBirdsNest_Info(t *testing.T) {
}

func TestShardingBirdsNest_SetHeight(t *testing.T) {
	type fields struct {
		bn *ShardingBirdsNest
	}
	type args struct {
		height uint64
	}
	tests := []struct {
		name   string
		fields fields
		args   args
	}{
		{
			name:   "正常流",
			fields: fields{bn: getSBN(8, bn.TestDir+"test_set_height1", t)},
			args:   args{height: 2},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tt.fields.bn.Start()
			tt.fields.bn.SetHeight(tt.args.height)
			if tt.fields.bn.GetHeight() != tt.args.height {
				t.Errorf("SetHeight() got = %v, want %v", tt.fields.bn.GetHeight(), tt.args.height)
			}
		})
	}
}

// This test is not implemented
func TestShardingBirdsNest_monitorExitSign(t *testing.T) {
}

// This test is not implemented
func TestShardingBirdsNest_timedSerialize(t *testing.T) {
}

func getSBN(i int, path string, t *testing.T) *ShardingBirdsNest {
	config := &ShardingBirdsNestConfig{
		ChainId: "chain1",
		Length:  uint32(i),
		Timeout: 10,
		Birdsnest: &bn.BirdsNestConfig{
			ChainId: "chain1",
			Length:  5,
			Rules:   &bn.RulesConfig{AbsoluteExpireTime: 20},
			Cuckoo: &bn.CuckooConfig{
				KeyType:       bn.KeyType_KTDefault,
				TagsPerBucket: 4,
				BitsPerItem:   9,
				MaxNumKeys:    10,
				TableType:     1,
			},
			Snapshot: &bn.SnapshotSerializerConfig{
				Type:        0,
				Timed:       &bn.TimedSerializeIntervalConfig{Interval: 20},
				BlockHeight: &bn.BlockHeightSerializeIntervalConfig{Interval: 20},
				Path:        bn.TestDir + path,
			},
		},
		Snapshot: &bn.SnapshotSerializerConfig{
			Type:        bn.SerializeIntervalType_Timed,
			Timed:       &bn.TimedSerializeIntervalConfig{Interval: 20},
			BlockHeight: &bn.BlockHeightSerializeIntervalConfig{Interval: 20},
			Path:        path + strconv.Itoa(i),
		},
	}
	nest, err := NewShardingBirdsNest(config, make(chan struct{}), bn.LruStrategy, NewModuloSA(int(i)), bn.TestLogger{T: t})
	if err != nil {
		return nil
	}
	return nest
}
