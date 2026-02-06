/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package birdsnest

import (
	"reflect"
	"testing"
)

func TestTimestampKey_Key(t *testing.T) {
	key := GenTimestampKey()
	tests := []struct {
		name string
		k    TimestampKey
		want []byte
	}{
		{
			name: "正常流",
			k:    TimestampKey(key),
			want: []byte(key),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.k.Key(); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("Key() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestTimestampKey_Len(t *testing.T) {
	key := GenTimestampKey()
	tests := []struct {
		name string
		k    TimestampKey
		want int
	}{
		{
			name: "正常流",
			k:    TimestampKey(key),
			want: len(key),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.k.Len(); got != tt.want {
				t.Errorf("Len() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestTimestampKey_Parse(t *testing.T) {
	key := GenTimestampKey()
	timestampKey, err := ToTimestampKey(key)
	if err != nil {
		return
	}
	tests := []struct {
		name    string
		k       TimestampKey
		want    [][]byte
		wantErr bool
	}{
		{
			name: "正常流",
			k:    timestampKey,
			want: func() [][]byte {
				timestampKey, err := ToTimestampKey(key)
				if err != nil {
					return nil
				}
				return [][]byte{timestampKey[:8], timestampKey[8:32]}
			}(),
			wantErr: false,
		},
		{
			name:    "异常流 key为空",
			k:       TimestampKey(""),
			want:    nil,
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := tt.k.Parse()
			if (err != nil) != tt.wantErr {
				t.Errorf("Parse() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("Parse() got = %v, want %v", got, tt.want)
			}
		})
	}
}
