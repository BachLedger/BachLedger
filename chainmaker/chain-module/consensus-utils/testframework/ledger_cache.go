/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package testframework

import (
	"errors"
	"sync"

	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/protocol/v2"
)

// ####################################################################################################################
//                                       impls CommitedLedgerCache          (avoid circular references to core module)
// ####################################################################################################################

// Cache is used for cache current block info
type Cache struct {
	chainId            string
	lastCommittedBlock *commonPb.Block
	rwMu               sync.RWMutex
}

// NewCache NewLedgerCache get a ledger cache.
// One ledger cache for one chain.
func NewCache(chainId string) protocol.LedgerCache {
	return &Cache{
		chainId: chainId,
	}
}

// GetLastCommittedBlock get the latest committed block
func (c *Cache) GetLastBlock() *commonPb.Block {
	c.rwMu.RLock()
	defer c.rwMu.RUnlock()
	return c.lastCommittedBlock
}

// SetLastCommittedBlock set the latest committed block
func (c *Cache) SetLastBlock(b *commonPb.Block) {
	c.rwMu.Lock()
	defer c.rwMu.Unlock()
	c.lastCommittedBlock = b
}

// CurrentHeight get current block height
func (c *Cache) CurrentHeight() (uint64, error) {
	c.rwMu.RLock()
	defer c.rwMu.RUnlock()
	if c.lastCommittedBlock == nil {
		return 0, errors.New("last committed block == nil")
	}
	return c.lastCommittedBlock.Header.BlockHeight, nil
}
