/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package testframework

////TestBlockCommitterForTest
//func TestBlockCommitterForTest(t *testing.T) {
//	msgbus := msgbus2.NewMessageBus()
//	ctrl := gomock.NewController(t)
//	ledgerCache := mock.NewMockLedgerCache(ctrl)
//	//test newBlockCommitter
//	blockCommit := newBlockCommitterForTest(msgbus, ledgerCache)
//	block := &commonPb.Block{
//		Header: &commonPb.BlockHeader{
//			ChainId:      "chain1",
//			BlockHeight:  100,
//			PreBlockHash: nil,
//		},
//		Txs: fetchTxBatch(txNum),
//	}
//	//test AddBlock
//	blockCommit.AddBlock(block)
//}
