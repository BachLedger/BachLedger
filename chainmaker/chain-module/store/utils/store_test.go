/*
 *  Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
 *
 *  SPDX-License-Identifier: Apache-2.0
 */

package utils

import (
	"fmt"
	"os"
	"path/filepath"
	"testing"
	"time"

	storePb "chainmaker.org/chainmaker/pb-go/v2/store"
	"chainmaker.org/chainmaker/protocol/v2/test"
	leveldbprovider "chainmaker.org/chainmaker/store-leveldb/v2"
	"github.com/stretchr/testify/assert"
)

func TestUtilsStore(t *testing.T) {
	startT := time.Now()
	tLogger := &test.GoLogger{}
	dirPath := filepath.Join(os.TempDir(), fmt.Sprintf("%d", time.Now().Nanosecond()))
	assert.Nil(t, CreatePath(dirPath))
	assert.Nil(t, CheckPathRWMod(dirPath))

	si := &storePb.StoreInfo{
		FileName: "00000000000000000001",
		Offset:   0,
		ByteLen:  10,
	}

	indexBytes := ConstructDBIndexInfo(si, si.Offset, si.ByteLen)
	assert.True(t, len(indexBytes) > 0)
	si2, err := DecodeValueToIndex(indexBytes)
	assert.Nil(t, err)
	assert.Equal(t, si.FileName, si2.FileName)

	CompactRange(leveldbprovider.NewMemdbHandle(), tLogger)
	tLogger.Infof("ElapsedMillisSeconds: %d", ElapsedMillisSeconds(startT))
}
