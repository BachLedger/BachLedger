/*
 * Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 */

package normal

import (
	"fmt"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"chainmaker.org/chainmaker/protocol/v2/mock"
	"github.com/golang/mock/gomock"
)

func TestNewLogHelper(t *testing.T) {
	ctrl := gomock.NewController(t)
	logger := mock.NewMockLogger(ctrl)
	logger.EXPECT().Warnf(gomock.Any(), gomock.Any()).DoAndReturn(
		func(format string, args ...interface{}) {
			fmt.Println(fmt.Sprintf(format, args...))
		}).AnyTimes()
	logHelper := newLogHelper(time.Millisecond, 10, logger, "AddTx TxPool is full, txIds:[%s]")
	logHelper.start()
	goSize := 100
	loopSize := 10
	counter := uint64(0)
	var wg sync.WaitGroup
	wg.Add(goSize)
	for i := 0; i < goSize; i++ {
		go func() {
			defer wg.Done()
			for j := 0; j < loopSize; j++ {
				x, y := atomic.AddUint64(&counter, 1), atomic.AddUint64(&counter, 1)
				logHelper.append(fmt.Sprintf("%d-test-%d", x, y))
			}
		}()
	}
	wg.Wait()
	time.Sleep(2 * time.Second)
	logHelper.stop()
}
