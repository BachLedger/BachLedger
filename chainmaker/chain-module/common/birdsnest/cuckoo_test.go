/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package birdsnest

import (
	"math"
	"reflect"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/linvon/cuckoo-filter"
)

func TestNewCuckooFilter(t *testing.T) {
	type args struct {
		config *CuckooConfig
	}
	tests := []struct {
		name string
		args args
		want CuckooFilter
	}{
		{
			name: "正常流",
			args: args{
				config: &CuckooConfig{
					KeyType:       1,
					TagsPerBucket: 4,
					BitsPerItem:   9,
					MaxNumKeys:    10,
					TableType:     1,
				},
			},
			want: NewCuckooFilter(&CuckooConfig{
				KeyType:       1,
				TagsPerBucket: 4,
				BitsPerItem:   9,
				MaxNumKeys:    10,
				TableType:     1,
			}),
		},
		{
			name: "正常流",
			args: args{
				config: &CuckooConfig{
					KeyType:       1,
					TagsPerBucket: 4,
					BitsPerItem:   9,
					MaxNumKeys:    10,
					TableType:     1,
				},
			},
			want: NewCuckooFilter(&CuckooConfig{
				KeyType:       1,
				TagsPerBucket: 4,
				BitsPerItem:   9,
				MaxNumKeys:    10,
				TableType:     1,
			}),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := NewCuckooFilter(tt.args.config); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("NewCuckooFilter() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestCuckooFilter_Add(t *testing.T) {
	type fields struct {
		cuckoo    cuckoo.Filter
		extension FilterExtension
		config    *CuckooConfig // cuckoo config
		full      bool          // cuckoo is full
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
			name: "正常流",
			fields: fields{
				cuckoo:    *cuckoo.NewFilter(4, 9, 100, 1),
				extension: NewDefaultFilterExtension(),
				config:    &CuckooConfig{MaxNumKeys: 10},
				full:      false,
			},
			args:    args{key: GetTimestampKey()},
			want:    true,
			wantErr: false,
		},
		{
			name: "异常流 过滤器满",
			fields: fields{
				cuckoo:    GetFullCuckooFilter(&[]Key{}),
				extension: NewDefaultFilterExtension(),
				config:    &CuckooConfig{MaxNumKeys: 10},
				full:      false,
			},
			args:    args{key: GetTimestampKey()},
			want:    false,
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &CuckooFilterImpl{
				cuckoo:    tt.fields.cuckoo,
				extension: tt.fields.extension,
				config:    tt.fields.config,
				full:      tt.fields.full,
			}
			got, err := c.Add(tt.args.key)
			if (err != nil) != tt.wantErr {
				t.Errorf("Add() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != tt.want {
				t.Errorf("Add() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestTimestampCuckooFilter_Contains(t *testing.T) {
	var keys []Key
	type fields struct {
		cuckoo    cuckoo.Filter
		extension FilterExtension
		config    *CuckooConfig // cuckoo config
		full      bool          // cuckoo is full
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
			fields: fields{
				cuckoo:    GetFullCuckooFilter(&keys),
				extension: NewTimestampFilterExtension(),
				config:    &CuckooConfig{MaxNumKeys: 10},
				full:      false,
			},
			args:    args{keys[0]},
			want:    true,
			wantErr: false,
		},
		{
			name: "正常流 key不存在",
			fields: fields{
				cuckoo:    GetFullCuckooFilter(&[]Key{}),
				extension: NewTimestampFilterExtension(),
				config:    &CuckooConfig{MaxNumKeys: 10},
				full:      false,
			},
			args:    args{GetTimestampKey()},
			want:    false,
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &CuckooFilterImpl{
				cuckoo:    tt.fields.cuckoo,
				extension: tt.fields.extension,
				config:    tt.fields.config,
				full:      tt.fields.full,
			}
			got, err := c.Contains(tt.args.key)
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

// The data inside the cuckoo filter is non-linear each time it is inserted
func TestTimestampCuckooFilter_Encode_Linear(t *testing.T) {
	//keys := &[]Key{}
	//filter1 := GetFullCuckooFilter(keys)
	//filter2 := GetFullCuckooFilter(keys)
	//
	//encode1, err := filter1.Encode()
	//if err != nil {
	//	return
	//}
	//encode2, err := filter2.Encode()
	//if err != nil {
	//	return
	//}
	//if !reflect.DeepEqual(encode1, encode2) {
	//	T.Errorf("Encode() got = %v, want %v", encode1, encode2)
	//}
}

func TestTimestampCuckooFilter_Encode(t *testing.T) {
	//var keys2 *[]Key
	type fields struct {
		cuckoo    cuckoo.Filter
		extension FilterExtension
		config    CuckooConfig
	}

	tests := []struct {
		name    string
		fields  fields
		want    FilterEncoder
		wantErr bool
	}{
		{
			name: "正常流 空布谷鸟 DefaultFilterExtension",
			fields: fields{
				cuckoo:    GetNullCuckooFilter(),
				extension: NewDefaultFilterExtension(),
				config: CuckooConfig{
					KeyType:       1,
					TagsPerBucket: 4,
					BitsPerItem:   9,
					MaxNumKeys:    10,
					TableType:     1,
				},
			},
			want: func() FilterEncoder {
				filter := NewCuckooFilter(&CuckooConfig{
					KeyType:       1,
					TagsPerBucket: 4,
					BitsPerItem:   9,
					MaxNumKeys:    10,
					TableType:     1,
				})
				encode, err := filter.Encode()
				if err != nil {
					return FilterEncoder{}
				}
				return encode
			}(),
			wantErr: false,
		},
		{
			name: "正常流 空布谷鸟 TimestampFilterExtension",
			fields: fields{
				cuckoo:    GetNullCuckooFilter(),
				extension: NewDefaultFilterExtension(),
				config: CuckooConfig{
					KeyType:       1,
					TagsPerBucket: 4,
					BitsPerItem:   9,
					MaxNumKeys:    10,
					TableType:     1,
				},
			},
			want: func() FilterEncoder {
				filter := NewCuckooFilter(&CuckooConfig{
					KeyType:       1,
					TagsPerBucket: 4,
					BitsPerItem:   9,
					MaxNumKeys:    10,
					TableType:     1,
				})
				encode, err := filter.Encode()
				if err != nil {
					return FilterEncoder{}
				}
				return encode
			}(),
			wantErr: false,
		},
		// See : TestTimestampCuckooFilter_Encode_Linear
		//{
		//	name: "正常流 满布谷鸟 DefaultFilterExtension",
		//	fields: fields{
		//		cuckoo:    GetFullCuckooFilter(keys1),
		//		extension: NewDefaultFilterExtension(),
		//	},
		//	want: func() []byte {
		//		filter := &CuckooFilterImpl{
		//			cuckoo:    GetFullCuckooFilter(keys1),
		//			extension: NewDefaultFilterExtension(),
		//		}
		//		encode, err := filter.Encode()
		//		if err != nil {
		//			return nil
		//		}
		//		return encode
		//	}(),
		//	wantErr: false,
		//},
		//{
		//	name: "正常流 满布谷鸟 TimestampFilterExtension",
		//	fields: fields{
		//		cuckoo:    GetFullCuckooFilter(keys2),
		//		extension: NewTimestampFilterExtension(),
		//	},
		//	want: func() FilterEncoder {
		//		filter := &CuckooFilterImpl{
		//			cuckoo:    GetFullCuckooFilter(keys2),
		//			extension: NewTimestampFilterExtension(),
		//		}
		//
		//		encode, err := filter.Encode()
		//		if err != nil {
		//			return FilterEncoder{}
		//		}
		//		return encode
		//	}(),
		//	wantErr: false,
		//},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &CuckooFilterImpl{
				cuckoo:    tt.fields.cuckoo,
				extension: tt.fields.extension,
				config:    &tt.fields.config,
			}
			got, err := c.Encode()
			if (err != nil) != tt.wantErr {
				t.Errorf("Encode() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("Encode() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestTimestampCuckooFilter_IsFull(t *testing.T) {
	type fields struct {
		cuckoo    cuckoo.Filter
		extension FilterExtension
		config    *CuckooConfig
	}
	tests := []struct {
		name   string
		fields fields
		want   bool
	}{
		{
			name: "正常流 空",
			fields: fields{
				cuckoo:    GetNullCuckooFilter(),
				extension: DefaultFilterExtension{},
				config: &CuckooConfig{
					KeyType:       1,
					TagsPerBucket: 4,
					BitsPerItem:   9,
					MaxNumKeys:    10,
					TableType:     1,
				},
			},
			want: false,
		},
		{
			name: "正常流 满",
			fields: fields{
				cuckoo:    GetFullCuckooFilter(&[]Key{}),
				extension: DefaultFilterExtension{},
				config: &CuckooConfig{
					KeyType:       1,
					TagsPerBucket: 4,
					BitsPerItem:   9,
					MaxNumKeys:    10,
					TableType:     1,
				},
			},
			want: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &CuckooFilterImpl{
				cuckoo:    tt.fields.cuckoo,
				extension: tt.fields.extension,
				config:    tt.fields.config,
			}
			_, _ = c.Add(GetTimestampKey())
			t.Log(c.cuckoo.Size())
			if got := c.IsFull(); got != tt.want {
				t.Errorf("IsFull() = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_newCuckooFilters(t *testing.T) {
	type args struct {
		config    *CuckooConfig
		size      uint32
		extension FilterExtension
	}
	tests := []struct {
		name string
		args args
		want []CuckooFilter
	}{
		{
			name: "正常流",
			args: args{
				config: &CuckooConfig{
					TagsPerBucket: 4,
					BitsPerItem:   9,
					MaxNumKeys:    10,
					TableType:     1,
				},
				size:      10,
				extension: DefaultFilterExtension{},
			},
			want: newCuckooFilters(&CuckooConfig{
				TagsPerBucket: 4,
				BitsPerItem:   9,
				MaxNumKeys:    10,
				TableType:     1,
			}, 10),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := newCuckooFilters(tt.args.config, tt.args.size); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("newCuckooFilters() = %v, want %v", got, tt.want)
			}
		})
	}
}

func GetFullCuckooFilter(keys *[]Key) (filter cuckoo.Filter) {
	filter = *cuckoo.NewFilter(4, 9, 10, 1)
	if len(*keys) == 0 {
		i := 0
		for ; ; i++ {
			key := GetTimestampKey()
			*keys = append(*keys, key)
			full := filter.Add(key.Key())
			if !full {
				return
			}
		}
	} else {
		for _, key := range *keys {
			*keys = append(*keys, key)
			_ = filter.Add(key.Key())
		}
	}
	return
}

func GetNullCuckooFilter() (filter cuckoo.Filter) {
	filter = *cuckoo.NewFilter(4, 9, 10, 1)
	return
}

func Test_getApproximationMaxNumKeys(t *testing.T) {
	type args struct {
		maxNumKeys uint32
		b          uint32
	}
	tests := []struct {
		name string
		args args
		want uint
	}{
		{
			name: "正常流",
			args: args{
				maxNumKeys: 2_000_000,
				b:          4,
			},
			want: func() uint {
				loadFactor, ok := loadFactorMap[4]
				if !ok {
					loadFactor = DefaultLoadFactor
				}
				got := float64(2_000_000) * 1.25 / loadFactor
				for i := float64(1); true; i++ {
					pow := math.Pow(2, i)
					rl := pow * loadFactor
					if rl > got {
						return uint(rl)
					}
				}
				return uint(2_000_000)
			}(),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			assert.Equalf(t, tt.want, getApproximationMaxNumKeys(tt.args.maxNumKeys, tt.args.b), "getApproximationMaxNumKeys(%v, %v)", tt.args.maxNumKeys, tt.args.b)
		})
	}
}
