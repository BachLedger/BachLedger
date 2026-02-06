/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package birdsnest

import (
	"io/ioutil"
	"testing"
)

func TestLruStrategy(t *testing.T) {
	_, _ = ioutil.ReadDir("./data")
	type args struct {
		bn *BirdsNestImpl
	}
	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name: "正常流",
			args: args{bn: func() *BirdsNestImpl {
				bn := getTBN(TestDir+"LruStrategy", t)
				err := bn.Adds(GetTimestampKeys(200))
				if err != nil {
					return nil
				}
				return bn
			}(),
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if err := LruStrategy(tt.args.bn); (err != nil) != tt.wantErr {
				t.Errorf("LruStrategy() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func Test_loopIReset(t *testing.T) {
	type args struct {
		both uint32
	}
	tests := []struct {
		name string
		args args
		want uint32
	}{
		{
			name: "正常流",
			args: args{
				both: 10,
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var j int
			for i := 0; i < 20; i++ {
				j = seeNextIndex(j, int(tt.args.both+1))
				t.Log(j)
			}
		})
	}
}
