/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

// Package main test
package main

import (
	"fmt"
	"os"
	"time"

	bn "chainmaker.org/chainmaker/common/v2/birdsnest"
	"chainmaker.org/chainmaker/common/v2/report"
	"chainmaker.org/chainmaker/common/v2/shardingbirdsnest"
	"github.com/go-echarts/go-echarts/v2/opts"
)

const (
	// blockCap
	blockCap = 10000
	// totalHeight
	totalHeight = 10000
)

func main() {
	// remove data dir
	_ = os.RemoveAll("./data")
	// new logger
	log := TestLog{}
	// sharding bird's nest configuration
	conf := &shardingbirdsnest.ShardingBirdsNestConfig{
		ChainId: "chain1",
		Length:  5,
		Timeout: 4,
		// bird's nest configuration
		Birdsnest: &bn.BirdsNestConfig{
			ChainId: "chain1",
			// length 10
			Length: 10,
			// rules configuration
			Rules: &bn.RulesConfig{AbsoluteExpireTime: 300},
			// cuckoo configuration
			Cuckoo: &bn.CuckooConfig{
				// KeyType 1
				KeyType: 1,
				// TagsPerBucket 4
				TagsPerBucket: 4,
				// BitsPerItem 9
				BitsPerItem: 9,
				// MaxNumKeys 2_000_000
				MaxNumKeys: 2_000_000,
				// TableType 1
				TableType: 1,
			},
			// Snapshot configuration
			Snapshot: &bn.SnapshotSerializerConfig{
				// timed interval configuration
				Type:  bn.SerializeIntervalType_Timed,
				Timed: &bn.TimedSerializeIntervalConfig{Interval: 20},
				Path:  "./data/",
			},
		},
		// Snapshot configuration
		Snapshot: &bn.SnapshotSerializerConfig{
			// timed interval configuration
			Type:  bn.SerializeIntervalType_Timed,
			Timed: &bn.TimedSerializeIntervalConfig{Interval: 20},
			Path:  "./data/",
		},
	}
	// new sharding bird's nest
	sharding, err := shardingbirdsnest.NewShardingBirdsNest(conf, make(chan struct{}), bn.LruStrategy,
		shardingbirdsnest.NewModuloSA(5), log)
	if err != nil {
		log.Errorf("%v", log)
		return
	}
	// make heights cap total height
	heights := make([]uint64, 0, totalHeight)
	// Time consuming
	costs := make([]opts.BarData, 0, totalHeight)

	for i := uint64(0); i < totalHeight; i++ {
		// init timestamp keys
		keys := bn.GetTimestampKeys(blockCap)
		now := time.Now()
		// add to sharding bird's nest
		err = sharding.AddsAndSetHeight(keys, i)
		if err != nil {
			log.Errorf("adds and set height, error: %v", err)
			return
		}
		// Time to take up
		cost := time.Since(now)
		// collection cost
		costs = append(costs, opts.BarData{Value: cost.Nanoseconds()})
		heights = append(heights, i)
	}
	//_ = sharding.Serialize()
	//for _, nest := range sharding.bn {
	//	_ = nest.(bn.Serializer).Serialize()
	//}
	// Time consuming report
	report.Report("Sharding bird's nest after optimization",
		"", heights, report.Series{Name: "Category A", Data: costs})

}

// TestLog log
type TestLog struct {
}

// Debugf DEBUG format
func (t TestLog) Debugf(format string, args ...interface{}) {
	fmt.Println("[DEBUG] " + fmt.Sprintf(format, args...))
}

// Errorf ERROR format
func (t TestLog) Errorf(format string, args ...interface{}) {
	fmt.Println("[ERROR] " + fmt.Sprintf(format, args...))
}

// Infof INFO format
func (t TestLog) Infof(format string, args ...interface{}) {
	fmt.Println("[INFO] " + fmt.Sprintf(format, args...))
}

// Warnf WARN format
func (t TestLog) Warnf(format string, args ...interface{}) {
	fmt.Println("[WARN] " + fmt.Sprintf(format, args...))
}
