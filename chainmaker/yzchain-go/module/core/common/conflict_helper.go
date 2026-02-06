package common

import (
	"bytes"
	hash2 "chainmaker.org/chainmaker/common/v2/crypto/hash"
	"chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/protocol/v2"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"runtime"
	"sort"
	"sync"
)

type RTableItem struct {
	Owner []byte
	mutex sync.RWMutex
}

const PREVIOUS_BLK_DIRTY_WRITE_LEADING_BYTE = 1
const NORMAL_LEADING_BYTE = 0

// CheckOwnership is basically similar with NaiveCheckOwnership,
// but takes leading 0/1 into consideration.
//
// the leading 0/1 of ownership id: 1 means it has been written by txs in previous block.
func (rstt *RTableItem) CheckOwnership(who []byte) (bool, []byte) {
	rstt.mutex.RLock()
	defer rstt.mutex.RUnlock()
	if rstt.Owner[0] != NORMAL_LEADING_BYTE {
		return false, rstt.Owner
	} // the leading 0/1 of ownership id: 1 means it has been written by txs in previous block.
	return bytes.Compare(who, rstt.Owner) == 0, rstt.Owner
}

func (rstt *RTableItem) ReleaseOwnership() {
	rstt.mutex.Lock()
	defer rstt.mutex.Unlock()
	rstt.Owner[0] = PREVIOUS_BLK_DIRTY_WRITE_LEADING_BYTE
}

func (rstt *RTableItem) NaiveCheckOwnership(who []byte) bool {
	rstt.mutex.RLock()
	defer rstt.mutex.RUnlock()
	return bytes.Compare(who, rstt.Owner) <= 0
}

func (rstt *RTableItem) NaiveTrySetOwner(who []byte) bool {
	if !rstt.NaiveCheckOwnership(who) {
		return false
	}
	rstt.mutex.Lock()
	defer rstt.mutex.Unlock()
	if bytes.Compare(who, rstt.Owner) <= 0 {
		copy(rstt.Owner, who)
		return true
	}
	return false
}

func (rstt *RTableItem) TrySetOwner(who []byte) bool {
	if ok, _ := rstt.CheckOwnership(who); !ok {
		return false
	}
	rstt.mutex.Lock()
	defer rstt.mutex.Unlock()
	if rstt.Owner[0] == PREVIOUS_BLK_DIRTY_WRITE_LEADING_BYTE {
		return false
	}
	if bytes.Compare(who, rstt.Owner) <= 0 {
		copy(rstt.Owner, who)
		return true
	}
	return false
}

type ReserveTable struct {
	Table map[string]*RTableItem
	mutex sync.RWMutex
}

//
//func (rst *ReserveTable) NaiveCheckOwnership(storageKey string, who []byte) bool {
//	return rst.Table[storageKey].NaiveCheckOwnership(who)
//}

func (rst *ReserveTable) ensureItemExist(storageKey string, who []byte) {
	if func() bool {
		rst.mutex.RLock()
		defer rst.mutex.RUnlock()
		if rst.Table[storageKey] != nil {
			return true
		}
		return false
	}() {
		return
	}
	rst.mutex.Lock()
	defer rst.mutex.Unlock()
	if rst.Table[storageKey] == nil {
		rst.Table[storageKey] = &RTableItem{
			Owner: make([]byte, len(who)),
			mutex: sync.RWMutex{},
		}
		copy(rst.Table[storageKey].Owner, who)
		return
	}
	return
}

func (rst *ReserveTable) NaiveTryTakeOwnership(storageKey string, who []byte) bool {
	rst.ensureItemExist(storageKey, who)
	rst.mutex.RLock()
	defer rst.mutex.RUnlock()
	return rst.Table[storageKey].NaiveTrySetOwner(who)
}

func (rst *ReserveTable) CheckDirtyWrite(storageKey string, who []byte) (bool, []byte) {
	rst.mutex.RLock()
	defer rst.mutex.RUnlock()
	return rst.Table[storageKey].CheckOwnership(who)
}

func (rst *ReserveTable) TryTakeOwnership(storageKey string, who []byte) bool {
	rst.ensureItemExist(storageKey, who)
	rst.mutex.RLock()
	defer rst.mutex.RUnlock()
	return rst.Table[storageKey].TrySetOwner(who)
}

func (rst *ReserveTable) ReleaseOwnership(storageKey string, who []byte) {
	rst.ensureItemExist(storageKey, who)
	rst.mutex.RLock()
	defer rst.mutex.RUnlock()
	rst.Table[storageKey].ReleaseOwnership()
}

func CalReserveTxID(hashType string, transaction *common.Transaction, hashBatch []byte, batchNum uint64) ([]byte, error) {
	selfBytes, _ := json.Marshal(&transaction)
	hashBatch = append(hashBatch, selfBytes...)
	idv, err := hash2.GetByStrType(hashType, hashBatch)
	if err != nil {
		return nil, err
	}
	id := make([]byte, 9, len(idv)+9)
	id[0] = NORMAL_LEADING_BYTE
	binary.BigEndian.PutUint64(id[1:9], batchNum)
	id = append(id, idv...)
	return id, nil
}

func GetBatchHash(hashType string, batch []*common.Transaction) ([]byte, error) {
	hashBatch := make([]byte, 0)
	for _, tx := range batch {
		txBytes, _ := json.Marshal(tx)
		hash, err := hash2.GetByStrType(hashType, txBytes)
		if err != nil {
			_, file, line, _ := runtime.Caller(1)
			return nil, fmt.Errorf("GetByStrType err at %s:%d: %s", file, line, err.Error())
		}
		hashBatch = append(hashBatch, hash...)
	}
	sort.Slice(hashBatch, func(i, j int) bool { return i > j })
	hashBatch, err := hash2.GetByStrType(hashType, hashBatch)
	if err != nil {
		_, file, line, _ := runtime.Caller(1)
		return nil, fmt.Errorf("GetByStrType err at %s:%d: %s", file, line, err.Error())
	}
	return hashBatch, nil
}

type TxTask struct {
	Tx        *common.Transaction
	ReserveId []byte
	exeResult *protocol.ExecuteTxResult
	uniqueTag int64
}

type OrderedMutex struct {
	mu      sync.Mutex
	queue   map[int]chan bool
	current int
	log     protocol.Logger
}

func NewOrderedMutex(logger protocol.Logger) *OrderedMutex {
	om := &OrderedMutex{
		current: 1,
		queue:   make(map[int]chan bool),
		log:     logger,
	}
	return om
}

func (o *OrderedMutex) Lock(id int) {
	o.mu.Lock()
	if id < o.current {
		o.mu.Unlock()
		o.log.Errorf("id[%d] already passed, current[%d]", id, o.current)
		return
	}

	if id == o.current {
		o.mu.Unlock()
		//fmt.Printf("got unlocked lock id[%d]\n", id)
		return
	}
	//fmt.Printf("id[%d] have to wait\n", id)
	ch := make(chan bool)
	o.queue[id] = ch
	o.mu.Unlock()

	<-ch
	//fmt.Printf("got lock id[%d]\n", id)
}

func (o *OrderedMutex) Unlock(id int) {
	o.mu.Lock()
	if id != o.current {
		o.mu.Unlock()
		o.log.Errorf("unlock out of order. got[%d], current[%d]", id, o.current)
		return
	}

	o.current++
	//fmt.Printf("current added[%d] id[%d]\n", o.current, id)
	if ch, ok := o.queue[o.current]; ok {
		close(ch)
		delete(o.queue, o.current)
	}

	o.mu.Unlock()
}
