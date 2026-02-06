/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package birdsnest

import (
	"fmt"
	"reflect"
	"strconv"
	"testing"
	"time"

	"go.uber.org/atomic"
)

func TestDeserializeTimestamp(t *testing.T) {
	type args struct {
		bytes []byte
	}
	tests := []struct {
		name    string
		args    args
		want    *TimestampFilterExtension
		wantErr bool
	}{
		{
			name: "正常流 Min",
			args: args{func() []byte {
				t := &TimestampFilterExtension{
					firstTimestamp: atomic.NewInt64(0),
					lastTimestamp:  atomic.NewInt64(0),
				}
				return t.Serialize()
			}()},
			want: func() *TimestampFilterExtension {
				return &TimestampFilterExtension{
					firstTimestamp: atomic.NewInt64(0),
					lastTimestamp:  atomic.NewInt64(0),
				}
			}(),
			wantErr: false,
		},
		{
			name: "正常流 Max",
			args: args{func() []byte {
				t := &TimestampFilterExtension{
					firstTimestamp: atomic.NewInt64(9223372036854775807),
					lastTimestamp:  atomic.NewInt64(9223372036854775807),
				}
				return t.Serialize()
			}()},
			want: func() *TimestampFilterExtension {
				return &TimestampFilterExtension{
					firstTimestamp: atomic.NewInt64(9223372036854775807),
					lastTimestamp:  atomic.NewInt64(9223372036854775807),
				}
			}(),
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := DeserializeTimestamp(tt.args.bytes)
			if (err != nil) != tt.wantErr {
				t.Errorf("DeserializeTimestamp() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("DeserializeTimestamp() got = %v, want %v", got, tt.want)
			}
		})
	}
}

// TestFilterExtension_Serialize This test is not implemented see: TestDeserialize
func TestTimestampFilterExtension_Serialize(t1 *testing.T) {
}

func TestTimestampFilterExtension_Store(t1 *testing.T) {
	timestamp := CurrentTimestampNano()
	key := GenTimestampKey()
	type fields struct {
		firstTimestamp *atomic.Int64
		lastTimestamp  *atomic.Int64
	}
	type args struct {
		key Key
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		wantErr bool
	}{
		{
			name: "正常流",
			fields: fields{
				firstTimestamp: atomic.NewInt64(timestamp),
				lastTimestamp:  atomic.NewInt64(timestamp),
			},
			args: args{key: func() Key {
				timestampKey, err := ToTimestampKey(key)
				if err != nil {
					t1.Error(err)
					return nil
				}
				return timestampKey
			}()},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t1.Run(tt.name, func(t1 *testing.T) {
			t := &TimestampFilterExtension{
				firstTimestamp: tt.fields.firstTimestamp,
				lastTimestamp:  tt.fields.lastTimestamp,
			}
			if err := t.Store(tt.args.key); (err != nil) != tt.wantErr {
				t1.Errorf("Store() error = %v, wantErr %v", err, tt.wantErr)
			}
			if t.firstTimestamp.Load() != tt.fields.firstTimestamp.Load() {
				t1.Errorf(" ExtensionDeserialize firstTimestamp not equal before = %v after = %v", t.firstTimestamp.Load(), tt.fields.firstTimestamp.Load())
			}
			if t.lastTimestamp.Load() != tt.fields.lastTimestamp.Load() {
				t1.Errorf(" ExtensionDeserialize lastTimestamp not equal before = %v after = %v", t.lastTimestamp.Load(), tt.fields.lastTimestamp.Load())
			}
		})

	}
}

func TestTimestampFilterExtension_Validate(t1 *testing.T) {
	current := CurrentTimestampNano()
	first := atomic.NewInt64(current - time.Second.Nanoseconds())
	last := atomic.NewInt64(current + time.Second.Nanoseconds())
	fmt.Println("first", first, "last", last)
	type fields struct {
		firstTimestamp *atomic.Int64
		lastTimestamp  *atomic.Int64
	}
	type args struct {
		key  Key
		full bool
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		wantErr bool
	}{
		{
			name: "正常流",
			fields: fields{
				firstTimestamp: first,
				lastTimestamp:  last,
			},
			args:    args{key: GetTimestampKey(), full: true},
			wantErr: false,
		},
		{
			name: "异常流 不在时间范围内 上边界",
			fields: fields{
				firstTimestamp: first,
				lastTimestamp:  last,
			},
			args: args{
				key: func() Key {
					return TimestampKey(strconv.FormatInt(current-10000, 10) + SeparatorString + GenTxId())
				}(),
				full: true,
			},
			wantErr: true,
		},
		{
			name: "异常流 不在时间范围内 下边界",
			fields: fields{
				firstTimestamp: first,
				lastTimestamp:  last,
			},
			args: args{
				key: func() Key {
					return TimestampKey(strconv.FormatInt(current+2000, 10) + SeparatorString + GenTxId())
				}(),
				full: true,
			},
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t1.Run(tt.name, func(t1 *testing.T) {
			t := &TimestampFilterExtension{
				firstTimestamp: tt.fields.firstTimestamp,
				lastTimestamp:  tt.fields.lastTimestamp,
			}
			if err := t.Validate(tt.args.key, tt.args.full); (err != nil) != tt.wantErr {
				t1.Errorf("Validate() error = %v, wantErr %v, key = %v, first = %v, last = %v", err, tt.wantErr, tt.args.key.String(), tt.fields.firstTimestamp.Load(), tt.fields.lastTimestamp.Load())
			}
		})
	}
}

func TestDefaultFilterExtension_Serialize(t *testing.T) {
	tests := []struct {
		name string
		want []byte
	}{
		{
			name: "正常流",
			want: []byte{0, 0, 0, 0, 0, 0, 0, 0},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			d := DefaultFilterExtension{}
			if got := d.Serialize(); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("Serialize() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestDefaultFilterExtension_Store(t *testing.T) {
	type args struct {
		in0 Key
	}
	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name:    "正常流",
			args:    args{in0: GetTimestampKey()},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			d := DefaultFilterExtension{}
			if err := d.Store(tt.args.in0); (err != nil) != tt.wantErr {
				t.Errorf("Store() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestDefaultFilterExtension_Validate(t *testing.T) {
	type args struct {
		in0 Key
	}
	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name:    "正常流",
			args:    args{in0: GetTimestampKey()},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			d := DefaultFilterExtension{}
			if err := d.Validate(tt.args.in0, false); (err != nil) != tt.wantErr {
				t.Errorf("Validate() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestNewDefaultFilterExtension(t *testing.T) {
	tests := []struct {
		name string
		want *DefaultFilterExtension
	}{
		{
			name: "正常流",
			want: NewDefaultFilterExtension(),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := NewDefaultFilterExtension(); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("NewDefaultFilterExtension() = %v, want %v", got, tt.want)
			}
		})
	}
}
