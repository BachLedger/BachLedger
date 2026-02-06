/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package birdsnest

import (
	"bytes"
	"encoding/json"
	"os"
	"strconv"
	"testing"

	"github.com/gogo/protobuf/proto"
	"github.com/stretchr/testify/assert"

	birdsnestpb "chainmaker.org/chainmaker/common/v2/birdsnest/pb"
)

func TestReadWrite(t *testing.T) {
	err := os.RemoveAll("./data/wal_snapshot")
	assert.Nil(t, err)
	snapshot, err := NewWalSnapshot("./data/wal_snapshot", Filepath, 0)
	assert.Nil(t, err)
	for i := 0; i < 10; i++ {
		write := []byte("aaa" + strconv.Itoa(i))
		err = snapshot.Write(write)
		assert.Nil(t, err)
		read, err := snapshot.Read()
		assert.Nil(t, err)
		if !bytes.Equal(write, read) {
			t.Errorf("got %v want %v", string(write), string(read))
		}
	}
}

func TestNewWalSnapshot(t *testing.T) {
	type args struct {
		path string
	}
	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name:    "正常流",
			args:    args{"./data/wal_snapshot"},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := NewWalSnapshot(tt.args.path, Filepath, 0)
			assert.Nil(t, err)
			assert.NotEmpty(t, got, "NewWalSnapshot() got is empty")
		})
	}
}

func TestFileSnapshot(t *testing.T) {
	err := os.RemoveAll("./data/wal_snapshot")
	snapshot, err := newFilterSnapshot("./data/wal_snapshot", "birdsnest1", 0)
	assert.Nil(t, err)

	pConfig := &BirdsNestConfig{
		ChainId: "chain1",
		Length:  32,
	}
	marshal, _ := json.Marshal(pConfig)
	birdsNest := &birdsnestpb.BirdsNest{
		Config:       marshal,
		Height:       22,
		CurrentIndex: 3,
	}
	cfg, err := proto.Marshal(birdsNest)
	err = snapshot.WriteCfg(marshal)
	assert.Nil(t, err)
	err = snapshot.WriteGauge(cfg)
	assert.Nil(t, err)
	for i := uint8(0); i < 10; i++ {
		write := []byte("aaa" + strconv.Itoa(int(i)))
		err = snapshot.WriteFilter(write, uint16(i))
		assert.Nil(t, err)
	}

	for i := uint8(2); i < 5; i++ {
		write := []byte("bbb" + strconv.Itoa(int(i)))
		err = snapshot.WriteFilter(write, uint16(i))
		assert.Nil(t, err)
	}

	read, errR := snapshot.Read(9)
	assert.Nil(t, errR)
	for i, data := range read {
		if i == 0 {
			pBN := &birdsnestpb.BirdsNest{}
			err = proto.Unmarshal(data, pBN)
			assert.Nil(t, err)
			println(pBN)
		}
		println(string(data))
	}
}
