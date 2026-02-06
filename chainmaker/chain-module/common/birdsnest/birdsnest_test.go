/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package birdsnest

import (
	"fmt"
	"io/ioutil"
	"os"
	"reflect"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"go.uber.org/atomic"
)

func TestBirdsNestImpl_Add(t *testing.T) {
	_ = os.RemoveAll("./data")
	_, _ = ioutil.ReadDir("./data")
	type fields struct {
		bn *BirdsNestImpl
	}
	type args struct {
		key Key
	}
	tests := []struct {
		name   string
		fields fields
		args   args
		want   bool
	}{
		{
			name:   "正常流 添加后存在",
			fields: fields{bn: getTBN(TestDir+"add01", t)},
			args:   args{key: GetTimestampKey()},
			want:   true,
		},
		{
			name:   "正常流 添加后不存在",
			fields: fields{bn: getTBN(TestDir+"add11", t)},
			args:   args{key: GetTimestampKey()},
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

func TestBirdsNestImpl_Adds(t *testing.T) {
	_, _ = ioutil.ReadDir("./data")
	var keys []Key
	type fields struct {
		bn *BirdsNestImpl
	}
	type args struct {
		keys []Key
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		wantErr bool
	}{
		{
			name:   "正常流 插入超过容量大小的key",
			fields: fields{bn: getTBN(TestDir+"adds0", t)},
			args: args{keys: func() []Key {
				return GetTimestampKeys(200)
			}()},
			wantErr: false,
		},
		{
			name:   "正常流 检查不存在",
			fields: fields{bn: getTBN(TestDir+"adds1", t)},
			args: args{keys: func() []Key {
				keys = append(keys, GetTimestampKeys(200)...)
				return keys
			}()},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.fields.bn.Adds(tt.args.keys)
			if (err != nil) != tt.wantErr {
				t.Errorf("Adds() err = %v, wantErr %v", err, tt.wantErr)
			}
			for _, key := range tt.args.keys {
				_, _ = tt.fields.bn.Contains(key)
			}
		})
	}
}

func TestBirdsNestImpl_Contains(t *testing.T) {
	_, _ = ioutil.ReadDir("./data")
	key1 := GetTimestampKey()
	type fields struct {
		bn *BirdsNestImpl
	}
	type args struct {
		key Key
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
			fields: fields{bn: func() *BirdsNestImpl {
				tbn := getTBN(TestDir+"contains0", t)
				_ = tbn.Add(key1)
				return tbn
			}()},
			args:    args{key: key1},
			want:    true,
			wantErr: false,
		},
		{
			name: "正常流 不存在",
			fields: fields{bn: func() *BirdsNestImpl {
				tbn := getTBN(TestDir+"contains1", t)
				_ = tbn.Add(key1)
				return tbn
			}()},
			args:    args{key: GetTimestampKey()},
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

func TestBirdsNestImpl_GetHeight(t *testing.T) {
	_, _ = ioutil.ReadDir("./data")
	type fields struct {
		bn *BirdsNestImpl
	}
	tests := []struct {
		name   string
		fields fields
		want   uint64
	}{
		{
			name: "正常流",
			fields: fields{bn: func() *BirdsNestImpl {
				tbn := getTBN(TestDir+"GetHeight", t)
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
func TestBirdsNestImpl_Info(t *testing.T) {
}

func TestBirdsNestImpl_Serialize(t *testing.T) {
	_, _ = ioutil.ReadDir("./data")
	type fields struct {
		bn *BirdsNestImpl
	}
	tests := []struct {
		name    string
		fields  fields
		wantErr bool
	}{
		{
			name:    "正常流",
			fields:  fields{bn: getTBN(TestDir+"serialize011", t)},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if err := tt.fields.bn.Serialize(tt.fields.bn.currentIndex); (err != nil) != tt.wantErr {
				t.Errorf("Serialize() error = %v, wantErr %v", err, tt.wantErr)
			}
			err := os.RemoveAll(TestDir)
			if (err != nil) != tt.wantErr {
				t.Errorf("RemoveAll() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestBirdsNestImpl_SetHeight(t *testing.T) {
	_, _ = ioutil.ReadDir("./data")
	type fields struct {
		bn *BirdsNestImpl
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
			fields: fields{bn: getTBN(TestDir+"SetHeight", t)},
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
func TestBirdsNestImpl_monitorExitSign(t *testing.T) {
}

// This test is not implemented
func TestBirdsNestImpl_timedSerialize(t *testing.T) {
}

func TestNewBirdsNest(t *testing.T) {
	_ = os.RemoveAll("./data")
	_, _ = ioutil.ReadDir("./data")
	tbn := getTBN(TestDir+"NewBirdsNest", t)
	type args struct {
		config   *BirdsNestConfig
		exitC    chan struct{}
		strategy Strategy
	}
	tests := []struct {
		name    string
		args    args
		want    *BirdsNestImpl
		wantErr bool
	}{
		{
			name: "正常流",
			args: args{
				config:   tbn.config,
				exitC:    tbn.exitC,
				strategy: tbn.strategy,
			},
			want:    tbn,
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := NewBirdsNest(tt.args.config, tt.args.exitC, tt.args.strategy, &TestLogger{t})
			if (err != nil) != tt.wantErr {
				t.Errorf("NewBirdsNest() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			//if !reflect.DeepEqual(got, tt.want) {
			//	T.Errorf("NewBirdsNest() got = %v, want %v", got, tt.want)
			//}
			if !reflect.DeepEqual(got.filters, tt.want.filters) {
				t.Errorf("NewBirdsNest() filters got = %v, want %v", got, tt.want)
			}
			if !reflect.DeepEqual(got.config, tt.want.config) {
				t.Errorf("NewBirdsNest() filters got = %v, want %v", got, tt.want)
			}
			//if !reflect.DeepEqual(got.snapshot, tt.want.snapshot) {
			//	T.Errorf("NewBirdsNest() filters got = %v, want %v", got, tt.want)
			//}
		})
	}
}

func getTBN(path string, t *testing.T) *BirdsNestImpl {
	config := &BirdsNestConfig{
		ChainId: "chain1",
		Length:  10,
		Rules: &RulesConfig{
			AbsoluteExpireTime: 100000,
		},
		Cuckoo: &CuckooConfig{
			KeyType:       KeyType_KTDefault,
			TagsPerBucket: 4,
			BitsPerItem:   9,
			MaxNumKeys:    10,
			TableType:     1,
		},
		Snapshot: &SnapshotSerializerConfig{
			Type:        SerializeIntervalType_Timed,
			Timed:       &TimedSerializeIntervalConfig{Interval: 5},
			BlockHeight: &BlockHeightSerializeIntervalConfig{Interval: 5},
			Path:        path,
		},
	}
	nest, err := NewBirdsNest(config, make(chan struct{}), LruStrategy, &TestLogger{t})
	if err != nil {
		panic(err)
	}
	return nest
}

func TestBirdsNestImpl_ValidateRule(t *testing.T) {
	type fields struct {
		height       uint64
		preHeight    *atomic.Uint64
		filters      []CuckooFilter
		config       *BirdsNestConfig
		strategy     Strategy
		currentIndex int
		rules        map[RuleType]Rule
		log          Logger
		exitC        chan struct{}
		serializeC   chan serializeSignal
		snapshot     *filterSnapshot
	}
	type args struct {
		key   Key
		rules []RuleType
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		wantErr assert.ErrorAssertionFunc
	}{
		{
			name: "正常流 在绝对超时时间范围之内",
			fields: fields{
				rules: func() map[RuleType]Rule {
					m := make(map[RuleType]Rule)
					m[RuleType_AbsoluteExpireTime] = NewAETRule(1, TestLogger{t})
					return m
				}(),
			},
			args: args{
				key:   GetTimestampKey(),
				rules: []RuleType{RuleType_AbsoluteExpireTime},
			},
			wantErr: assert.NoError,
		},
		{
			name: "异常流 在绝对超时时间范围之外",
			fields: fields{

				rules: func() map[RuleType]Rule {
					m := make(map[RuleType]Rule)
					m[RuleType_AbsoluteExpireTime] = NewAETRule(1, TestLogger{t})
					return m
				}(),
			},
			args: args{
				key:   GetTimestampKeyByNano(time.Now().UnixNano() + time.Second.Nanoseconds()*2),
				rules: []RuleType{RuleType_AbsoluteExpireTime},
			},
			wantErr: assert.Error,
		},
		{
			name: "正常流 不填写绝对超时时间范围",
			fields: fields{
				rules: func() map[RuleType]Rule {
					m := make(map[RuleType]Rule)
					m[RuleType_AbsoluteExpireTime] = NewAETRule(1, TestLogger{t})
					return m
				}(),
			},
			args: args{
				key: GetTimestampKey(),
			},
			wantErr: assert.NoError,
		},
		{
			name: "异常流 不填写绝对超时时间范围",
			fields: fields{
				rules: func() map[RuleType]Rule {
					m := make(map[RuleType]Rule)
					m[RuleType_AbsoluteExpireTime] = NewAETRule(1, TestLogger{t})
					return m
				}(),
			},
			args: args{
				key: GetTimestampKeyByNano(time.Now().UnixNano() + time.Second.Nanoseconds()*2),
			},
			wantErr: assert.NoError,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			b := &BirdsNestImpl{
				height:       tt.fields.height,
				preHeight:    tt.fields.preHeight,
				filters:      tt.fields.filters,
				config:       tt.fields.config,
				strategy:     tt.fields.strategy,
				currentIndex: tt.fields.currentIndex,
				rules:        tt.fields.rules,
				log:          tt.fields.log,
				exitC:        tt.fields.exitC,
				serializeC:   tt.fields.serializeC,
				snapshot:     tt.fields.snapshot,
			}
			tt.wantErr(t, b.ValidateRule(tt.args.key, tt.args.rules...), fmt.Sprintf("ValidateRule(%v, %v)", tt.args.key, tt.args.rules))
		})
	}
}
