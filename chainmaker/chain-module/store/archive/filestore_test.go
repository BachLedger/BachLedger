/*
 *  Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
 *
 *  SPDX-License-Identifier: Apache-2.0
 */

package archive

// test blockstore.go
import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"os"
	"path"
	"path/filepath"
	"reflect"
	"strings"
	"testing"
	"time"

	acPb "chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	storePb "chainmaker.org/chainmaker/pb-go/v2/store"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/protocol/v2/test"
	"chainmaker.org/chainmaker/store/v2/serialization"
	td "chainmaker.org/chainmaker/store/v2/test"
	tbf "chainmaker.org/chainmaker/store/v2/types/blockfile"
	"github.com/stretchr/testify/assert"
)

var (
	chainId    = "ut1"
	cryptoType = "SHA256"
)

// TestOpenRestore tests OpenRestore function
func TestOpenRestore(t *testing.T) {
	opts := DefaultOptions
	testPath := filepath.Join(os.TempDir(), fmt.Sprintf("%d", time.Now().Nanosecond()))
	type args struct {
		path   string
		opts   *Options
		logger protocol.Logger
	}
	tests := []struct {
		name    string
		args    args
		want    *FileRestore
		wantErr bool
	}{
		{
			name: "open restore",
			args: args{
				path:   testPath,
				opts:   opts,
				logger: &test.GoLogger{},
			},
			want:    &FileRestore{},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := OpenRestore(tt.args.path, tt.args.opts, tt.args.logger)
			if (err != nil) != tt.wantErr {
				t.Errorf("OpenRestore() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got.segments, tt.want.segments) {
				t.Errorf("OpenRestore() got = %v, want %v", got, tt.want)
			}
		})
	}
}

// TestFileRestore_Write tests Write function
func TestFileRestore_Write(t *testing.T) {
	opts := DefaultOptions
	restorePath := filepath.Join(os.TempDir(), fmt.Sprintf("%d", time.Now().Nanosecond()))
	//restorePath := "/Users/zhouhoufa/Code/src/chainmaker.org/chainmaker/store/data/archive/bfdb/restoring"
	fr, err := OpenRestore(restorePath, opts, &test.GoLogger{})
	assert.Nil(t, err)
	defer func() {
		_ = fr.Close()
	}()

	tmb := td.MockBlk
	indexMeta := &serialization.BlockIndexMeta{
		Height:         0,
		BlockHash:      []byte("test"),
		BlockTimestamp: 0,
		TxIds:          make([]string, 0),
		RwSets:         make([]string, 0),
	}
	preLen := uint64(7)
	offset := uint64(0)
	totalBlk := uint64(120)
	byteLen := uint64(len(tmb))
	segName := "00000000000000000001"
	for i := uint64(0); i < totalBlk; i++ {
		indexMeta.Height = i
		bim, errb := fr.Write(i+1, tmb, indexMeta)
		assert.Nil(t, errb)
		assert.Equal(t, bim.Index.FileName, segName)
		assert.Equal(t, bim.Index.Offset, offset+preLen)
		assert.Equal(t, bim.Index.ByteLen, byteLen)
		assert.Equal(t, len(fr.segments), 1)
		assert.Equal(t, fr.segments[0].name, segName)
		assert.Equal(t, fr.segments[0].lastIndex, i+1)
		assert.Equal(t, len(fr.segments[0].segIndexs), int(i+1))
		assert.Equal(t, fr.segments[0].segIndexs[i].Height, i)
		offset = offset + preLen + byteLen
	}
	assert.Equal(t, fr.segments[0].index, uint64(1))
	assert.Nil(t, fr.Sync())
	fr.ToString(fr.segments[0])

	indexMeta.Height = totalBlk - 1
	bim, errb := fr.Write(totalBlk+2, tmb, indexMeta)
	assert.Nil(t, errb)
	assert.Equal(t, bim.Index.FileName, "00000000000000000122")
	assert.Equal(t, bim.Index.Offset, preLen)
	assert.Equal(t, bim.Index.ByteLen, byteLen)
	assert.Equal(t, len(fr.segments), 2)
}

func TestFileRestore_Write_Reverse(t *testing.T) {
	opts := DefaultOptions
	restorePath := filepath.Join(os.TempDir(), fmt.Sprintf("%d", time.Now().Nanosecond()))
	//restorePath := "/Users/zhouhoufa/Code/src/chainmaker.org/chainmaker/store/data/archive/bfdb/restoring"
	fr, err := OpenRestore(restorePath, opts, &test.GoLogger{})
	assert.Nil(t, err)

	tmb := td.MockBlk
	indexMeta := &serialization.BlockIndexMeta{
		Height:         0,
		BlockHash:      []byte("test"),
		BlockTimestamp: 0,
		TxIds:          make([]string, 0),
		RwSets:         make([]string, 0),
	}
	preLen := uint64(7)
	totalBlk := uint64(10)
	byteLen := uint64(len(tmb))
	// test reverse write
	for i := totalBlk - 1; i >= 0; i-- {
		indexMeta.Height = i
		bim, errb := fr.Write(i+1, tmb, indexMeta)
		assert.Nil(t, errb)
		assert.Equal(t, bim.Index.FileName, tbf.Uint64ToSegmentName(i+1))
		assert.Equal(t, bim.Index.Offset, preLen)
		assert.Equal(t, bim.Index.ByteLen, byteLen)
		if i == 0 {
			break
		}
	}

	verifyFileRestore(t, fr, totalBlk)

	// restart restore instance
	assert.Nil(t, fr.Close())
	fr, err = OpenRestore(restorePath, opts, &test.GoLogger{})
	assert.Nil(t, err)

	verifyFileRestore(t, fr, totalBlk)
	assert.Nil(t, fr.Close())
}

func TestFileRestore_ResetMergeSegmentEnv(t *testing.T) {
	opts := DefaultOptions
	bfdbPath := filepath.Join(os.TempDir(), fmt.Sprintf("%d", time.Now().Nanosecond()))
	//bfdbPath := "/Users/zhouhoufa/Code/src/chainmaker.org/chainmaker/store/data/archive/bfdb"
	restorePath := path.Join(bfdbPath, tbf.RestoringPath)
	fr, err := OpenRestore(restorePath, opts, &test.GoLogger{})
	assert.Nil(t, err)

	tmb := td.MockBlk
	indexMeta := &serialization.BlockIndexMeta{
		Height:         0,
		BlockHash:      []byte("test"),
		BlockTimestamp: 0,
		TxIds:          make([]string, 0),
		RwSets:         make([]string, 0),
	}
	totalBlk := uint64(10)
	// build segments
	for i := totalBlk - 1; i >= 0; i-- {
		indexMeta.Height = i
		_, err = fr.Write(i+1, tmb, indexMeta)
		assert.Nil(t, err)
		if i == 0 {
			break
		}
	}

	segName := tbf.Uint64ToSegmentName(2)
	mSeg, errm := fr.newMergeSegment(segName, 2)
	assert.Nil(t, errm)
	assert.Equal(t, mSeg.name, segName)
	assert.True(t, strings.Contains(mSeg.dFile.Path, tbf.MergingFileSuffix))
	assert.True(t, strings.Contains(mSeg.iFile.Path, tbf.MergingFileSuffix))

	// close restore instance
	assert.Nil(t, fr.Close())

	// remove segment 2 .fdb file
	segName = tbf.Uint64ToSegmentName(2)
	assert.Nil(t, os.RemoveAll(tbf.SegmentPath(segName, restorePath)))

	// remove segment 3 .restoring.json file
	segName = tbf.Uint64ToSegmentName(3)
	assert.Nil(t, os.RemoveAll(tbf.SegmentIndexPath(segName, restorePath)))

	// move segment 4 .fdb file to restore path
	segName = tbf.Uint64ToSegmentName(4)
	assert.Nil(t, os.Rename(tbf.SegmentPath(segName, restorePath), tbf.SegmentPath(segName, bfdbPath)))

	fr, err = OpenRestore(restorePath, opts, &test.GoLogger{})
	assert.Nil(t, err)

	assert.Equal(t, len(fr.segments), int(totalBlk-2))

	assert.Nil(t, fr.Close())
}

func TestFileRestore_MergeSegment(t *testing.T) {
	opts := DefaultOptions
	bfdbPath := filepath.Join(os.TempDir(), fmt.Sprintf("%d", time.Now().Nanosecond()))
	//bfdbPath := "/Users/zhouhoufa/Code/src/chainmaker.org/chainmaker/store/data/archive/bfdb"
	restorePath := path.Join(bfdbPath, tbf.RestoringPath)
	fr, err := OpenRestore(restorePath, opts, &test.GoLogger{})
	assert.Nil(t, err)

	totalBlk := uint64(20)
	blocks, txRWSetMp, errb := buildFileStoreDB(int(totalBlk), 10000)
	assert.Nil(t, errb)

	for i := len(blocks) - 1; i >= 0; i-- {
		blkBytes, brw, errs := serialization.SerializeBlock(&storePb.BlockWithRWSet{Block: blocks[i], TxRWSets: txRWSetMp[uint64(i)]})
		assert.Nil(t, errs)
		indexMeta := &serialization.BlockIndexMeta{
			Height:         uint64(i),
			BlockHash:      brw.Block.Header.BlockHash,
			BlockTimestamp: brw.Block.Header.BlockTimestamp,
			TxIds:          brw.Meta.TxIds,
			RwSets:         make([]string, len(brw.TxRWSets)),
		}
		indexMeta.MetaIndex = brw.MetaIndex
		indexMeta.TxsIndex = brw.TxsIndex
		indexMeta.RWSetsIndex = brw.RWSetsIndex
		for k := 0; k < len(brw.SerializedTxRWSets); k++ {
			indexMeta.RwSets[k] = brw.TxRWSets[k].TxId
		}
		_, err = fr.Write(uint64(i)+1, blkBytes, indexMeta)
		assert.Nil(t, err)
		assert.Nil(t, fr.Sync())
		if i == 0 {
			break
		}
	}

	assert.Equal(t, len(fr.GetSegsInfo()), len(fr.segments))
	assert.Nil(t, fr.syncSegment(fr.currSegment))
	assert.Nil(t, fr.mergeSegment())
	assert.Nil(t, fr.closeSegmentByName(fr.currSegment.name))

	modifySeg := fr.segments[len(fr.segments)-1]
	rPath := modifySeg.dFile.Path
	bakPath := fmt.Sprintf("%s%s", rPath, tbf.BakFileSuffix)
	assert.Nil(t, os.Rename(rPath, bakPath))
	assert.Nil(t, fr.resetMergeSegmentEnv())

	// close restore instance
	assert.Nil(t, fr.Close())
}

func verifyFileRestore(t *testing.T, fr *FileRestore, totalBlk uint64) {
	// check segments count
	assert.Equal(t, len(fr.segments), int(totalBlk))

	// check segments index data
	fr.sortSegs()
	for i := uint64(0); i < uint64(len(fr.segments)); i++ {
		assert.Equal(t, fr.segments[i].index, i+1)
		assert.Equal(t, fr.segments[i].lastIndex, i+1)
		assert.Equal(t, fr.segments[i].name, tbf.Uint64ToSegmentName(i+1))
		assert.Equal(t, len(fr.segments[i].segIndexs), 1)
		assert.Equal(t, fr.segments[i].segIndexs[0].Height, i)
	}

	// test get segments info
	fRng := fr.GetSegsInfo()
	assert.NotNil(t, fRng)
	assert.Equal(t, len(fRng), int(totalBlk))
	assert.Equal(t, fr.LastSegment().index, totalBlk)
	assert.Equal(t, len(fr.LastSegment().segIndexs), 1)
	assert.Equal(t, fr.LastSegment().segIndexs[0].Height, totalBlk-1)

	// test del segment
	fr.delSegmentByName(tbf.Uint64ToSegmentName(1))
	assert.Equal(t, len(fr.segments), int(totalBlk-1))
	fr.segments[2] = nil
	fr.removeNilSegments()
	assert.Equal(t, len(fr.segments), int(totalBlk-2))
}

func buildFileStoreDB(totalHeight, txNum int) ([]*commonPb.Block, map[uint64][]*commonPb.TxRWSet, error) {
	var err error
	blocks := make([]*commonPb.Block, 0, totalHeight)
	txRWSetMp := make(map[uint64][]*commonPb.TxRWSet)
	for i := 0; i < totalHeight; i++ {
		var (
			block   *commonPb.Block
			txRWSet []*commonPb.TxRWSet
		)

		block, txRWSet = createBlockAndRWSets(chainId, uint64(i), txNum)
		if i == 0 {
			block.Header.PreConfHeight = 0
		} else {
			block.Header.PreConfHeight = uint64(i) - 1
		}
		blocks = append(blocks, block)
		txRWSetMp[block.Header.BlockHeight] = txRWSet
	}

	return blocks, txRWSetMp, err
}

func createBlockAndRWSets(chainId string, height uint64, txNum int) (*commonPb.Block, []*commonPb.TxRWSet) {
	block := createBlock(chainId, height, txNum)
	var txRWSets []*commonPb.TxRWSet

	for i := 0; i < txNum; i++ {
		key := fmt.Sprintf("key_%d", i)
		value := fmt.Sprintf("value_%d_value_0_e99588bd06b5830d1c968c12cad42e968b4ce6c32a5ce37bad9818c33d659254e99588bd06b5830d1c968c12cad42e968b4ce6c32a5ce37bad9818c33d659254e99588bd06b5830d1c968c12cad42e968b4ce6c32a5ce37bad9818c33d659254e99588bd06b5830d1c968c12cad42e968b4ce6c32a5ce37bad9818c33d659254e99588bd06b5830d1c968c12cad42e968b4ce6c32a5ce37bad9818c33d659254e99588bd06b5830d1c968c12cad42e968b4ce6c32a5ce37bad9818c33d659254e99588bd06b5830d1c968c12cad42e968b4ce6c32a5ce37bad9818c33d659254e99588bd06b5830d1c968c12cad42e968b4ce6c32a5ce37bad9818c33d659254e99588bd06b5830d1c968c12cad42e968b4ce6c32a5ce37bad9818c33d659254e99588bd06b5830d1c968c12cad42e968b4ce6c32a5ce37bad9818c33d659254e99588bd06b5830d1c968c12cad42e968b4ce6c32a5ce37bad9818c33d659254e99588bd06b5830d1c968c12cad42e968b4ce6c32a5ce37bad9818c33d659254", i)
		txRWset := &commonPb.TxRWSet{
			TxId: block.Txs[i].Payload.TxId,
			TxWrites: []*commonPb.TxWrite{
				{
					Key:          []byte(key),
					Value:        []byte(value),
					ContractName: "contract1",
				},
			},
		}
		txRWSets = append(txRWSets, txRWset)
	}

	return block, txRWSets
}

func createBlock(chainId string, height uint64, txNum int) *commonPb.Block {
	block := &commonPb.Block{
		Header: &commonPb.BlockHeader{
			ChainId:      chainId,
			BlockHeight:  height,
			BlockVersion: 3000000,
			Proposer: &acPb.Member{
				OrgId:      "org1",
				MemberInfo: []byte("User1"),
			},
		},
		Txs: []*commonPb.Transaction{},
	}
	for i := 0; i < txNum; i++ {
		tx := &commonPb.Transaction{
			Payload: &commonPb.Payload{
				ChainId: chainId,
				TxType:  commonPb.TxType_INVOKE_CONTRACT,
				TxId:    generateTxId(chainId, height, i),
			},
			Sender: &commonPb.EndorsementEntry{
				Signer: &acPb.Member{
					OrgId:      "org1",
					MemberInfo: []byte("User1"),
				},
				Signature: []byte("signature1"),
			},
			Result: &commonPb.Result{
				Code: commonPb.TxStatusCode_SUCCESS,
				ContractResult: &commonPb.ContractResult{
					Result: []byte("ok"),
				},
			},
		}
		block.Txs = append(block.Txs, tx)
	}
	block.Header.BlockHash = generateBlockHash(chainId, height)
	block.Dag = &commonPb.DAG{
		Vertexes: []*commonPb.DAG_Neighbor{{
			Neighbors: []uint32{0, 1, 2},
		}},
	}

	return block
}

func generateTxId(chainId string, height uint64, index int) string {
	txIdBytes := sha256.Sum256([]byte(fmt.Sprintf("%s-%d-%d", chainId, height, index)))
	return hex.EncodeToString(txIdBytes[:])
}

func generateBlockHash(chainId string, height uint64) []byte {
	blockHash := sha256.Sum256([]byte(fmt.Sprintf("%s-%d", chainId, height)))
	return blockHash[:]
}
