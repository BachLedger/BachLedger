/*
 *  Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
 *
 *  SPDX-License-Identifier: Apache-2.0
 */

package blockfile

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	storePb "chainmaker.org/chainmaker/pb-go/v2/store"
	"chainmaker.org/chainmaker/protocol/v2/test"
	"chainmaker.org/chainmaker/store/v2/utils"
	"github.com/stretchr/testify/assert"
)

func Test_Helper(t *testing.T) {
	segName := "00000000000000000001"
	fdbFile := "00000000000000000001.fdb"
	assert.True(t, !CheckSegmentName(segName))
	assert.True(t, CheckSegmentName(fdbFile))

	fiIndex := &storePb.StoreInfo{}
	indexStr := FileIndexToString(nil)
	assert.Equal(t, "rfile index is nil", indexStr)
	indexStr = FileIndexToString(fiIndex)
	assert.True(t, strings.Contains(indexStr, "fileIndex"))

	assert.Equal(t, segName, Uint64ToSegmentName(1))
	index, err := SegmentNameToUint64(segName)
	assert.Nil(t, err)
	assert.Equal(t, uint64(1), index)

	dirPath := filepath.Join(os.TempDir(), fmt.Sprintf("%d", time.Now().Nanosecond()))
	fdbPath := SegmentPath(segName, dirPath)
	indexPath := SegmentIndexPath(segName, dirPath)
	assert.True(t, strings.Contains(SegmentPath(segName, dirPath), DBFileSuffix))
	assert.True(t, strings.Contains(indexPath, RestoringIndexFileName))

	tLogger := &test.GoLogger{}
	fw, errf := OpenWriteFile(fdbPath, DBFileSuffix, false, 0, tLogger)
	assert.NotNil(t, errf)
	assert.Nil(t, fw)

	assert.Nil(t, utils.CreatePath(dirPath))
	assert.Nil(t, utils.CheckPathRWMod(dirPath))

	wfile, errw := NewFileWriter(fdbPath)
	assert.Nil(t, errw)
	assert.NotNil(t, wfile)

	fw, errf = OpenWriteFile(fdbPath, DBFileSuffix, false, 0, tLogger)
	assert.Nil(t, errf)
	assert.NotNil(t, fw)

	data := []byte("test")
	assert.Nil(t, AppendFileEntry(data, wfile, len(data), tLogger))

	ldata, bpos, errl := LoadSegmentEntriesForRestarting(fdbPath, 1)
	assert.Nil(t, errl)
	assert.Equal(t, 1, len(bpos))
	assert.Equal(t, len(data), len(ldata)-bpos[0].PrefixLen)

	ldata, bpos, errl = LoadEntriesForRestarting(fdbPath)
	assert.Nil(t, errl)
	assert.Equal(t, 1, len(bpos))
	assert.Equal(t, len(data), len(ldata)-bpos[0].PrefixLen)

	fdbPath2 := fmt.Sprintf("%s.bak", fdbPath)
	assert.Nil(t, CopyThenRemoveFile(fdbPath, fdbPath2, nil))
}
