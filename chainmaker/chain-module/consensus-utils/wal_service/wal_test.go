/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package wal_service

import (
	"sync"
	"testing"
	"time"

	"chainmaker.org/chainmaker/common/v2/wal"
	"github.com/stretchr/testify/require"
)

// TestNewDefaultConsensusWalOption
func TestNewDefaultConsensusWalOption(t *testing.T) {
	walOption := NewDefaultConsensusWalOption()
	require.Equal(t, NonWalWrite, walOption.walWriteMode)
	require.Equal(t, "", walOption.walWritePath)
}

// TestWithWrite
func TestWithWrite(t *testing.T) {
	writePath := WithWritePath("utils")
	writeMode := WithWriteMode(SyncWalWrite)
	walOption := NewDefaultConsensusWalOption()
	writePath(&walOption)
	writeMode(&walOption)
	require.Equal(t, SyncWalWrite, walOption.walWriteMode)
	require.Equal(t, "utils", walOption.walWritePath)
}

// TestNonWalService
func TestNonWalService(t *testing.T) {
	walService, err := NewWalService(nil, WithWriteMode(NonWalWrite))
	require.Nil(t, err)
	require.NotNil(t, walService)
	lastIndex, err := walService.LastIndex()
	require.Nil(t, err)
	require.Equal(t, uint64(0), lastIndex)
	err = walService.Write([]byte{0x00})
	require.Nil(t, err)
	lastIndex, err = walService.LastIndex()
	require.Equal(t, uint64(0), lastIndex)
	require.Nil(t, err)
	data, err := walService.Read(0)
	require.Equal(t, wal.ErrNotFound, err)
	require.Nil(t, data)
	err = walService.TruncateFront(0)
	require.Nil(t, err)
	err = walService.Sync()
	require.Nil(t, err)
	err = walService.Close()
	require.Nil(t, err)
}

// TestAsyncWalService
func TestAsyncWalService(t *testing.T) {
	walService, err := NewWalService(nil, WithWriteMode(NonWalWrite), WithWritePath("./wal.log"))
	require.NotNil(t, err)
	require.Nil(t, walService)
	walService, err = NewWalService(nil, WithWriteMode(AsyncWalWrite), WithWritePath("./wal.log"))
	require.Nil(t, err)
	require.NotNil(t, walService)
	wg := sync.WaitGroup{}
	wg.Add(10)
	for i := 0; i < 10; i++ {
		go func() {
			defer wg.Done()
			for i := 0; i < 100; i++ {
				require.NoError(t, walService.Write([]byte{0x00}))
			}
		}()
	}
	wg.Wait()
	err = walService.Sync()
	require.Nil(t, err)
	err = walService.Close()
	require.Nil(t, err)
	time.Sleep(1 * time.Second)
}

// TestSyncWalService
func TestSyncWalService(t *testing.T) {
	walService, err := NewWalService(nil, WithWriteMode(SyncWalWrite), WithWritePath("./wal.log"))
	require.Nil(t, err)
	require.NotNil(t, walService)
	wg := sync.WaitGroup{}
	wg.Add(10)
	for i := 0; i < 10; i++ {
		go func() {
			defer wg.Done()
			for i := 0; i < 100; i++ {
				require.NoError(t, walService.Write([]byte{0x00}))
			}
		}()
	}
	wg.Wait()
	err = walService.Sync()
	require.Nil(t, err)
	err = walService.Close()
	require.Nil(t, err)
	time.Sleep(1 * time.Second)
}

// TestDefaultMarshalFunc
func TestDefaultMarshalFunc(t *testing.T) {
	data := make([]byte, 2, 10)
	data[0] = 0x01
	data[1] = 0x02
	bytes := DefaultMarshalFunc(data)
	require.Equal(t, bytes, data)
}
