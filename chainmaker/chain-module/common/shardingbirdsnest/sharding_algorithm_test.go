/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package shardingbirdsnest

import (
	"fmt"
	"reflect"
	"sync"
	"testing"

	"go.uber.org/atomic"

	bn "chainmaker.org/chainmaker/common/v2/birdsnest"
)

func TestName(t *testing.T) {
	t.Log(bn.GenTimestampKey())
}

func BenchmarkChecksumKeyModulo(b *testing.B) {
	for i := 0; i < b.N; i++ {
		ChecksumKeyModulo(bn.TimestampKey("1647247990575-d0f62b32ad5343b88d6a95ef56634e50"), 4)
	}
}

func TestModuloShardingAlgorithm_DoSharding(t *testing.T) {
	keys := bn.GetTimestampKeys(200)
	type fields struct {
		Length int
	}
	type args struct {
		shardingValues []bn.Key
	}
	tests := []struct {
		name   string
		fields fields
		args   args
		want   [][]bn.Key
	}{
		{
			name:   "正常流 分片数组",
			fields: fields{5},
			args:   args{shardingValues: keys},
			want: func() [][]bn.Key {
				result := make([][]bn.Key, 5)
				// sharding
				for i := range keys {
					modulo := keys[i].Key()[keys[i].Len()-1] % 5
					if result[modulo] == nil {
						result[modulo] = []bn.Key{keys[i]}
					} else {
						result[modulo] = append(result[modulo], keys[i])
					}
				}
				return result
			}(),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			a := ModuloShardingAlgorithm{
				Length: tt.fields.Length,
			}
			if got := a.DoSharding(tt.args.shardingValues); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("DoSharding() = %v, want %v", got, tt.want)
			}
		})
	}
}

// 测试分片是否均匀
func TestChecksumKeyModulo(t *testing.T) {
	length := 5
	size := 100
	group := sync.WaitGroup{}
	group.Add(size)
	var m []*atomic.Uint32
	for i := 0; i < length; i++ {
		m = append(m, atomic.NewUint32(0))
	}

	keyC := make(chan bn.Key)
	go func() {
		for i := 0; i < size; i++ {
			keyC <- bn.GetTimestampKey()
		}
	}()
	for i := 0; i < 5; i++ {
		go func() {
			for {
				select {
				case key := <-keyC:
					modulo := ChecksumKeyModulo(key, length)
					m[modulo].Inc()
					group.Done()
				default:
				}
			}
		}()
	}
	group.Wait()
	for i := 0; i < len(m); i++ {
		fmt.Println(m[i].Load())
	}

}
