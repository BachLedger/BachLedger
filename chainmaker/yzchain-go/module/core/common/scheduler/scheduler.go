/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package scheduler

import (
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"regexp"
	"strconv"
	"sync"
	"time"

	configPb "chainmaker.org/chainmaker/pb-go/v2/config"

	"chainmaker.org/chainmaker/localconf/v2"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/utils/v2"
	"chainmaker.org/chainmaker/vm/v2"

	"chainmaker.org/chainmaker/common/v2/crypto"
	"chainmaker.org/chainmaker/common/v2/crypto/asym"
	"github.com/gogo/protobuf/proto"

	"github.com/hokaccha/go-prettyjson"

	"chainmaker.org/chainmaker-go/module/core/provider/conf"
	"chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/syscontract"
	"chainmaker.org/chainmaker/vm-native/v2/accountmgr"
	"github.com/panjf2000/ants/v2"
	"github.com/prometheus/client_golang/prometheus"
)

const (
	ScheduleTimeout        = 10
	ScheduleWithDagTimeout = 20
	blockVersion2300       = uint32(2300)
	blockVersion2310       = uint32(2030100)
	blockVersion2312       = uint32(2030102)
)

const (
	ErrMsgOfGasLimitNotSet = "field `GasLimit` must be set in payload."
)

// TxScheduler transaction scheduler structure
type TxScheduler struct {
	lock            sync.Mutex
	VmManager       protocol.VmManager
	scheduleFinishC chan bool
	log             protocol.Logger
	chainConf       protocol.ChainConf // chain config

	metricVMRunTime     *prometheus.HistogramVec
	StoreHelper         conf.StoreHelper
	keyReg              *regexp.Regexp
	signer              protocol.SigningMember
	commitedLedgerCache protocol.LedgerCache
	contractCache       *sync.Map
}

// Transaction dependency in adjacency table representation
type dagNeighbors map[int]struct{}

type TxIdAndExecOrderType struct {
	string
	protocol.ExecOrderTxType
}

// Schedule according to a batch of transactions,
// and generating DAG according to the conflict relationship
func (ts *TxScheduler) GetResultMaps(block *commonPb.Block, txBatch []*commonPb.Transaction, snapshot protocol.Snapshot) (map[string]*commonPb.TxRWSet, map[string][]*commonPb.ContractEvent, error) {

	//ts.lock.Lock()
	//defer ts.lock.Unlock()

	defer ts.releaseContractCache()
	// todo: 需要如下方式支持线程池，但是是在全局
	//var goRoutinePool *ants.Pool
	//poolCapacity := ts.StoreHelper.GetPoolCapacity()
	//ts.log.Debugf("GetPoolCapacity() => %v", poolCapacity)
	//if goRoutinePool, err = ants.NewPool(poolCapacity, ants.WithPreAlloc(false)); err != nil {
	//	return nil, nil, err
	//}
	//defer goRoutinePool.Release()
	txRWSetMap := ts.getTxRWSetTable(snapshot, block)
	contractEventMap := ts.getContractEventMap(block)

	return txRWSetMap, contractEventMap, nil
}

// handleTx: run tx and apply tx sim context to snapshot
func handleTx(block *commonPb.Block, snapshot protocol.Snapshot, ts *TxScheduler, tx *commonPb.Transaction, exeTxresult *protocol.ExecuteTxResult) {

	// execute tx, and get
	// 1) the read/write set
	// 2) the result that telling if the invoke success.
	//exeTxresult := ts.ExecuteTx(tx, snapshot, block)
	txSimContext, specialTxType, runVmSuccess := exeTxresult.TxSimCtx, exeTxresult.SpecialTxType, exeTxresult.RunVmSuccess
	tx.Result = txSimContext.GetTxResult()
	ts.log.DebugDynamic(func() string {
		return fmt.Sprintf("handleTx(`%v`) => ExecuteTx(...) => runVmSuccess = %v", tx.GetPayload().TxId, runVmSuccess)
	})

	// Apply failed means this tx's read set conflict with other txs' write set
	_, applySize := snapshot.ApplyTxSimContext(txSimContext, specialTxType,
		runVmSuccess, false)
	ts.log.DebugDynamic(func() string {
		return fmt.Sprintf("handleTx(`%v`) => ApplyTxSimContext(...) => snapshot.txTable = %v, applySize = %v",
			tx.GetPayload().TxId, len(snapshot.GetTxTable()), applySize)
	})

}

func (ts *TxScheduler) initOptimizeTools(
	txBatch []*commonPb.Transaction) (bool, *ConflictsBitWindow) {
	txBatchSize := len(txBatch)
	var conflictsBitWindow *ConflictsBitWindow
	enableConflictsBitWindow := ts.chainConf.ChainConfig().Core.EnableConflictsBitWindow

	ts.log.Infof("enable conflicts bit window: [%t]\n", enableConflictsBitWindow)

	if AdjustWindowSize*MinAdjustTimes > txBatchSize {
		enableConflictsBitWindow = false
	}
	if enableConflictsBitWindow {
		conflictsBitWindow = NewConflictsBitWindow(txBatchSize)
	}

	return enableConflictsBitWindow, conflictsBitWindow
}

// send txs from sender group
func (ts *TxScheduler) sendTxBySenderGroup(conflictsBitWindow *ConflictsBitWindow, senderGroup *SenderGroup,
	runningTxC chan *commonPb.Transaction, enableConflictsBitWindow bool) {
	// first round
	for _, v := range senderGroup.txsMap {
		runningTxC <- v[0]
	}
	// solve done tx channel
	for {
		k := <-senderGroup.doneTxKeyC
		if k == [32]byte{} {
			return
		}
		senderGroup.txsMap[k] = senderGroup.txsMap[k][1:]
		if len(senderGroup.txsMap[k]) > 0 {
			runningTxC <- senderGroup.txsMap[k][0]
		} else {
			delete(senderGroup.txsMap, k)
			if enableConflictsBitWindow {
				conflictsBitWindow.setMaxPoolCapacity(len(senderGroup.txsMap))
			}
		}
	}
}

// apply the read/write set to txSimContext,
// and adjust the go routine size
func (ts *TxScheduler) handleApplyResult(enableConflictsBitWindow bool, enableSenderGroup bool,
	conflictsBitWindow *ConflictsBitWindow, senderGroup *SenderGroup, goRoutinePool *ants.Pool,
	tx *commonPb.Transaction, start time.Time) {
	if localconf.ChainMakerConfig.MonitorConfig.Enabled {
		elapsed := time.Since(start)
		ts.metricVMRunTime.WithLabelValues(tx.Payload.ChainId).Observe(elapsed.Seconds())
	}
}

func (ts *TxScheduler) getTxRWSetTable(snapshot protocol.Snapshot, block *commonPb.Block) map[string]*commonPb.TxRWSet {
	block.Txs = snapshot.GetTxTable()
	txRWSetTable := snapshot.GetTxRWSetTable()
	txRWSetMap := make(map[string]*commonPb.TxRWSet, len(txRWSetTable))
	for _, txRWSet := range txRWSetTable {
		if txRWSet != nil {
			txRWSetMap[txRWSet.TxId] = txRWSet
		}
	}
	//ts.dumpDAG(block.Dag, block.Txs)
	if localconf.ChainMakerConfig.SchedulerConfig.RWSetLog {
		result, _ := prettyjson.Marshal(txRWSetMap)
		ts.log.Infof("schedule rwset :%s, dag:%+v", result, block.Dag)
	}
	return txRWSetMap
}

func (ts *TxScheduler) getContractEventMap(block *commonPb.Block) map[string][]*commonPb.ContractEvent {
	contractEventMap := make(map[string][]*commonPb.ContractEvent, len(block.Txs))
	for _, tx := range block.Txs {
		event := tx.Result.ContractResult.ContractEvent
		contractEventMap[tx.Payload.TxId] = event
	}
	return contractEventMap
}

// SimulateWithDag based on the dag in the block, perform scheduling and execution transactions
func (ts *TxScheduler) SimulateWithDag(block *commonPb.Block, snapshot protocol.Snapshot) (
	map[string]*commonPb.TxRWSet, map[string]*commonPb.Result, error) {
	ts.lock.Lock()
	defer ts.lock.Unlock()

	defer ts.releaseContractCache()

	var (
		startTime  = time.Now()
		txRWSetMap = make(map[string]*commonPb.TxRWSet, len(block.Txs))
	)
	if block.Header.BlockVersion >= blockVersion2300 && len(block.Txs) != len(block.Dag.Vertexes) {
		ts.log.Warnf("found dag size mismatch txs length in "+
			"block[%x] dag:%d, txs:%d", block.Header.BlockHash, len(block.Dag.Vertexes), len(block.Txs))
		return nil, nil, fmt.Errorf("found dag size mismatch txs length in "+
			"block[%x] dag:%d, txs:%d", block.Header.BlockHash, len(block.Dag.Vertexes), len(block.Txs))
	}
	if len(block.Txs) == 0 {
		ts.log.DebugDynamic(func() string {
			return fmt.Sprintf("no txs in block[%x] when simulate", block.Header.BlockHash)
		})
		return txRWSetMap, snapshot.GetTxResultMap(), nil
	}
	ts.log.Infof("simulate with dag start, size %d", len(block.Txs))
	txMapping := make(map[int]*commonPb.Transaction, len(block.Txs))
	for index, tx := range block.Txs {
		txMapping[index] = tx
	}

	// Construct the adjacency list of dag, which describes the subsequent adjacency transactions of all transactions
	dag := block.Dag
	txIndexBatch, dagRemain, reverseDagRemain, err := ts.initSimulateDag(dag)
	if err != nil {
		ts.log.Warnf("initialize simulate dag error:%s", err)
		return nil, nil, err
	}

	txBatchSize := len(dag.Vertexes)
	runningTxC := make(chan int, txBatchSize)
	doneTxC := make(chan int, txBatchSize)

	timeoutC := time.After(ScheduleWithDagTimeout * time.Second)
	finishC := make(chan bool)

	txExecOrderTypeC := make(chan TxIdAndExecOrderType, txBatchSize)

	var goRoutinePool *ants.Pool
	if goRoutinePool, err = ants.NewPool(len(block.Txs), ants.WithPreAlloc(true)); err != nil {
		return nil, nil, err
	}
	defer goRoutinePool.Release()

	ts.log.DebugDynamic(func() string {
		return fmt.Sprintf("block [%d] simulate with dag first batch size:%d, total batch size:%d",
			block.Header.BlockHeight, len(txIndexBatch), txBatchSize)
	})

	blockFingerPrint := string(utils.CalcBlockFingerPrintWithoutTx(block))
	ts.VmManager.BeforeSchedule(blockFingerPrint, block.Header.BlockHeight)
	defer ts.VmManager.AfterSchedule(blockFingerPrint, block.Header.BlockHeight)

	go func() {
		for _, tx := range txIndexBatch {
			runningTxC <- tx
		}
	}()
	go func() {
		for {
			select {
			case txIndex := <-runningTxC:
				tx := txMapping[txIndex]
				ts.log.Debugf("simulate with dag, prepare to submit running task for tx id:%s", tx.Payload.GetTxId())
				err = goRoutinePool.Submit(func() {
					handleTxInSimulateWithDag(block, snapshot, ts, tx, txIndex, doneTxC, finishC, txExecOrderTypeC, txBatchSize)
				})
				if err != nil {
					ts.log.Warnf("failed to submit tx id %s during simulate with dag, %+v",
						tx.Payload.GetTxId(), err)
				}
			case doneTxIndex := <-doneTxC:
				txIndexBatchAfterShrink := ts.shrinkDag(doneTxIndex, dagRemain, reverseDagRemain)
				ts.log.Debugf("block [%d] simulate with dag, pop next tx index batch size:%d, dagRemain size:%d",
					block.Header.BlockHeight, len(txIndexBatchAfterShrink), len(dagRemain))
				for _, tx := range txIndexBatchAfterShrink {
					runningTxC <- tx
				}
			case <-finishC:
				ts.log.Debugf("block [%d] simulate with dag finish", block.Header.BlockHeight)
				ts.scheduleFinishC <- true
				return
			case <-timeoutC:
				ts.log.Errorf("block [%d] simulate with dag timeout", block.Header.BlockHeight)
				ts.scheduleFinishC <- true
				return
			}
		}
	}()

	<-ts.scheduleFinishC
	snapshot.Seal()
	timeUsed := time.Since(startTime)
	ts.log.Infof("simulate with dag finished, block %d, size %d, time used %v, tps %v", block.Header.BlockHeight,
		len(block.Txs), timeUsed, float64(len(block.Txs))/(float64(timeUsed)/1e9))

	// Return the read and write set after the scheduled execution
	for _, txRWSet := range snapshot.GetTxRWSetTable() {
		if txRWSet != nil {
			txRWSetMap[txRWSet.TxId] = txRWSet
		}
	}
	txExecOrderTypeMap := make(map[string]protocol.ExecOrderTxType, len(block.Txs))
	// we only receive fixed number of elements from this channel since we process unreceived things
	// and return error in later parts
	length := len(txExecOrderTypeC)
	for i := 0; i < length; i++ {
		t := <-txExecOrderTypeC
		txExecOrderTypeMap[t.string] = t.ExecOrderTxType
	}
	err = ts.compareDag(block, snapshot, txRWSetMap, txExecOrderTypeMap)
	if err != nil {
		return nil, nil, err
	}
	if localconf.ChainMakerConfig.SchedulerConfig.RWSetLog {
		result, _ := prettyjson.Marshal(txRWSetMap)
		ts.log.Infof("simulate with dag rwset :%s, dag: %+v", result, block.Dag)
	}
	return txRWSetMap, snapshot.GetTxResultMap(), nil
}

func (ts *TxScheduler) initSimulateDag(dag *commonPb.DAG) (
	[]int, map[int]dagNeighbors, map[int]dagNeighbors, error) {
	dagRemain := make(map[int]dagNeighbors, len(dag.Vertexes))
	reverseDagRemain := make(map[int]dagNeighbors, len(dag.Vertexes)*4)
	var txIndexBatch []int
	for txIndex, neighbors := range dag.Vertexes {
		if neighbors == nil {
			return nil, nil, nil, fmt.Errorf("dag has nil neighbor")
		}
		if len(neighbors.Neighbors) == 0 {
			txIndexBatch = append(txIndexBatch, txIndex)
			continue
		}
		dn := make(dagNeighbors)
		for index, neighbor := range neighbors.Neighbors {
			if index > 0 {
				if neighbors.Neighbors[index-1] >= neighbor {
					return nil, nil, nil, fmt.Errorf("dag neighbors not strict increasing, neighbors: %v", neighbors.Neighbors)
				}
			}
			if int(neighbor) >= txIndex {
				return nil, nil, nil, fmt.Errorf("dag has neighbor >= txIndex, txIndex: %d, neighbor: %d", txIndex, neighbor)
			}
			dn[int(neighbor)] = struct{}{}
			if _, ok := reverseDagRemain[int(neighbor)]; !ok {
				reverseDagRemain[int(neighbor)] = make(dagNeighbors)
			}
			reverseDagRemain[int(neighbor)][txIndex] = struct{}{}
		}
		dagRemain[txIndex] = dn
	}
	return txIndexBatch, dagRemain, reverseDagRemain, nil
}

func handleTxInSimulateWithDag(
	block *commonPb.Block, snapshot protocol.Snapshot,
	ts *TxScheduler, tx *commonPb.Transaction, txIndex int,
	doneTxC chan int, finishC chan bool,
	txExecOrderTypeC chan TxIdAndExecOrderType, txBatchSize int) {
	exeTxResult := ts.ExecuteTx(tx, snapshot, block)
	txSimContext, specialTxType, runVmSuccess := exeTxResult.TxSimCtx, exeTxResult.SpecialTxType, exeTxResult.RunVmSuccess
	// send specialTxType BEFORE snapshot.ApplyTxSimContext which has a lock, ensuring that all txs have it
	// and eliminating race condition
	txExecOrderTypeC <- TxIdAndExecOrderType{tx.Payload.GetTxId(), specialTxType}
	// if apply failed means this tx's read set conflict with other txs' write set
	applyResult, applySize := snapshot.ApplyTxSimContext(txSimContext, specialTxType, runVmSuccess, true)
	if !applyResult {
		ts.log.DebugDynamic(func() string {
			return fmt.Sprintf("failed to apply snapshot for tx id:%s, shouldn't have its rwset", tx.Payload.TxId)
		})
		// apply fails in verification, make it done rather than retry it
		doneTxC <- txIndex
	} else {
		ts.log.DebugDynamic(func() string {
			return fmt.Sprintf("apply to snapshot for tx id:%s, result:%+v, apply count:%d, tx batch size:%d",
				tx.Payload.GetTxId(), txSimContext.GetTxResult(), applySize, txBatchSize)
		})
		doneTxC <- txIndex
	}
	// If all transactions in current batch have been successfully added to dag
	if applySize >= txBatchSize {
		ts.log.DebugDynamic(func() string {
			return fmt.Sprintf("finished 1 batch, apply size:%d, tx batch size:%d", applySize, txBatchSize)
		})
		finishC <- true
	}
}

func (ts *TxScheduler) adjustPoolSize(pool *ants.Pool, conflictsBitWindow *ConflictsBitWindow, txExecType TxExecType) {
	newPoolSize := conflictsBitWindow.Enqueue(txExecType, pool.Cap())
	if newPoolSize == -1 {
		return
	}
	pool.Tune(newPoolSize)
}

func (ts *TxScheduler) ExecuteTx(tx *commonPb.Transaction, snapshot protocol.Snapshot, block *commonPb.Block) *protocol.ExecuteTxResult {
	txSimContext := vm.NewTxSimContext(ts.VmManager, snapshot, tx, block.Header.BlockVersion, ts.log)
	ts.log.DebugDynamic(func() string {
		return fmt.Sprintf("NewTxSimContext finished for tx id:%s", tx.Payload.GetTxId())
	})
	//ts.log.DebugDynamic(func() string {
	//	return fmt.Sprintf("tx.Result = %v", tx.Result)
	//})

	enableGas := ts.checkGasEnable()
	enableOptimizeChargeGas := IsOptimizeChargeGasEnabled(ts.chainConf)
	blockVersion := block.GetHeader().BlockVersion

	if blockVersion >= 2300 {
		if !ts.guardForExecuteTx2300(tx, txSimContext, enableGas, enableOptimizeChargeGas, snapshot) {
			//return txSimContext, protocol.ExecOrderTxTypeNormal, false
			return &protocol.ExecuteTxResult{
				TxSimCtx:      txSimContext,
				SpecialTxType: protocol.ExecOrderTxTypeNormal,
				RunVmSuccess:  false,
			}
		}
	} else if blockVersion >= 2220 {
		if !ts.guardForExecuteTx2220(tx, txSimContext, enableGas, enableOptimizeChargeGas) {
			//return txSimContext, protocol.ExecOrderTxTypeNormal, false
			return &protocol.ExecuteTxResult{
				TxSimCtx:      txSimContext,
				SpecialTxType: protocol.ExecOrderTxTypeNormal,
				RunVmSuccess:  false,
			}
		}
	}

	runVmSuccess := true
	var txResult *commonPb.Result
	var err error
	var specialTxType protocol.ExecOrderTxType

	ts.log.Debugf("run vm start for tx:%s", tx.Payload.GetTxId())
	if blockVersion >= 2300 {
		if txResult, specialTxType, err = ts.runVM2300(tx, txSimContext, enableOptimizeChargeGas); err != nil {
			runVmSuccess = false
			ts.log.Errorf("failed to run vm for tx id:%s,contractName:%s, tx result:%+v, error:%+v",
				tx.Payload.GetTxId(), tx.Payload.ContractName, txResult, err)
		}
	} else if blockVersion >= 2220 {
		if txResult, specialTxType, err = ts.runVM2220(tx, txSimContext, enableOptimizeChargeGas); err != nil {
			runVmSuccess = false
			ts.log.Errorf("failed to run vm for tx id:%s,contractName:%s, tx result:%+v, error:%+v",
				tx.Payload.GetTxId(), tx.Payload.ContractName, txResult, err)
		}
	} else {
		if txResult, specialTxType, err = ts.runVM2210(tx, txSimContext); err != nil {
			runVmSuccess = false
			ts.log.Errorf("failed to run vm for tx id:%s,contractName:%s, tx result:%+v, error:%+v",
				tx.Payload.GetTxId(), tx.Payload.ContractName, txResult, err)
		}
	}
	ts.log.Debugf("run vm finished for tx:%s, runVmSuccess:%v, txResult = %v ", tx.Payload.TxId, runVmSuccess, txResult)
	txSimContext.SetTxResult(txResult)
	//return txSimContext, specialTxType, runVmSuccess
	return &protocol.ExecuteTxResult{
		TxSimCtx:      txSimContext,
		SpecialTxType: specialTxType,
		RunVmSuccess:  runVmSuccess,
	}
}

func (ts *TxScheduler) simulateSpecialTxs(dag *commonPb.DAG, snapshot protocol.Snapshot, block *commonPb.Block,
	txBatchSize int) {
	specialTxs := snapshot.GetSpecialTxTable()
	specialTxsLen := len(specialTxs)
	var firstTx *commonPb.Transaction
	runningTxC := make(chan *commonPb.Transaction, specialTxsLen)
	scheduleFinishC := make(chan bool)
	timeoutC := time.After(ScheduleWithDagTimeout * time.Second)
	go func() {
		for _, tx := range specialTxs {
			runningTxC <- tx
		}
	}()

	go func() {
		for {
			select {
			case tx := <-runningTxC:
				// simulate tx
				executeTxResult := ts.ExecuteTx(tx, snapshot, block)
				tx.Result = executeTxResult.TxSimCtx.GetTxResult()
				// apply tx
				applyResult, applySize := snapshot.ApplyTxSimContext(
					executeTxResult.TxSimCtx, executeTxResult.SpecialTxType, executeTxResult.RunVmSuccess, true)
				if !applyResult {
					ts.log.Debugf("failed to apply according to dag with tx %s ", tx.Payload.TxId)
					runningTxC <- tx
					continue
				}
				if firstTx == nil {
					firstTx = tx
					dagNeighbors := &commonPb.DAG_Neighbor{
						Neighbors: make([]uint32, 0, snapshot.GetSnapshotSize()-1),
					}
					for i := uint32(0); i < uint32(snapshot.GetSnapshotSize()-1); i++ {
						dagNeighbors.Neighbors = append(dagNeighbors.Neighbors, i)
					}
					dag.Vertexes = append(dag.Vertexes, dagNeighbors)
				} else {
					dagNeighbors := &commonPb.DAG_Neighbor{
						Neighbors: make([]uint32, 0, 1),
					}
					dagNeighbors.Neighbors = append(dagNeighbors.Neighbors, uint32(snapshot.GetSnapshotSize())-2)
					dag.Vertexes = append(dag.Vertexes, dagNeighbors)
				}
				if applySize >= txBatchSize {
					ts.log.Debugf("block [%d] schedule special txs finished, apply size:%d, len of txs:%d, "+
						"len of special txs:%d", block.Header.BlockHeight, applySize, txBatchSize, specialTxsLen)
					scheduleFinishC <- true
					return
				}
			case <-timeoutC:
				ts.log.Errorf("block [%d] schedule special txs timeout", block.Header.BlockHeight)
				scheduleFinishC <- true
				return
			}
		}
	}()
	<-scheduleFinishC
}

func (ts *TxScheduler) shrinkDag(txIndex int, dagRemain map[int]dagNeighbors,
	reverseDagRemain map[int]dagNeighbors) []int {
	var txIndexBatch []int
	for k := range reverseDagRemain[txIndex] {
		delete(dagRemain[k], txIndex)
		if len(dagRemain[k]) == 0 {
			txIndexBatch = append(txIndexBatch, k)
			delete(dagRemain, k)
		}
	}
	delete(reverseDagRemain, txIndex)
	return txIndexBatch
}

func (ts *TxScheduler) Halt() {
	ts.scheduleFinishC <- true
}

// nolint: unused
func (ts *TxScheduler) dumpDAG(dag *commonPb.DAG, txs []*commonPb.Transaction) {
	dagString := "digraph DAG {\n"
	for i, ns := range dag.Vertexes {
		if len(ns.Neighbors) == 0 {
			dagString += fmt.Sprintf("id_%s -> begin;\n", txs[i].Payload.TxId[:8])
			continue
		}
		for _, n := range ns.Neighbors {
			dagString += fmt.Sprintf("id_%s -> id_%s;\n", txs[i].Payload.TxId[:8], txs[n].Payload.TxId[:8])
		}
	}
	dagString += "}"
	ts.log.Infof("Dump Dag: %s", dagString)
}

func (ts *TxScheduler) chargeGasLimit(accountMangerContract *commonPb.Contract, tx *commonPb.Transaction,
	txSimContext protocol.TxSimContext, contractName, method string, pk []byte,
	result *commonPb.Result) (re *commonPb.Result, err error) {
	if ts.checkGasEnable() &&
		ts.checkNativeFilter(txSimContext.GetBlockVersion(), contractName, method, tx, txSimContext.GetSnapshot()) &&
		tx.Payload.TxType == commonPb.TxType_INVOKE_CONTRACT {
		var code commonPb.TxStatusCode
		var runChargeGasContract *commonPb.ContractResult
		var limit uint64
		if tx.Payload.Limit == nil {
			err = errors.New("tx payload limit is nil")
			ts.log.Error(err.Error())
			result.Message = err.Error()
			return result, err
		}

		limit = tx.Payload.Limit.GasLimit
		chargeParameters := map[string][]byte{
			accountmgr.ChargePublicKey: pk,
			accountmgr.ChargeGasAmount: []byte(strconv.FormatUint(limit, 10)),
		}
		ts.log.Debugf("【chargeGasLimit】%v, pk = %s, amount = %v", tx.Payload.TxId, pk, limit)
		runChargeGasContract, _, code = ts.VmManager.RunContract(
			accountMangerContract, syscontract.GasAccountFunction_CHARGE_GAS.String(),
			nil, chargeParameters, txSimContext, 0, commonPb.TxType_INVOKE_CONTRACT)
		if code != commonPb.TxStatusCode_SUCCESS {
			result.Code = code
			result.ContractResult = runChargeGasContract
			return result, errors.New(runChargeGasContract.Message)
		}
	} else {
		ts.log.Debugf("%s:%s no need to charge gas.", contractName, method)
	}
	return result, nil
}

func (ts *TxScheduler) checkRefundGas(accountMangerContract *commonPb.Contract, tx *commonPb.Transaction,
	txSimContext protocol.TxSimContext, contractName, method string, pk []byte,
	result *commonPb.Result, contractResultPayload *commonPb.ContractResult, enableOptimizeChargeGas bool) error {

	return nil
}

func (ts *TxScheduler) getAccountMgrContractAndPk(txSimContext protocol.TxSimContext, tx *commonPb.Transaction,
	contractName, method string) (accountMangerContract *commonPb.Contract, pk []byte, err error) {
	if ts.checkGasEnable() &&
		ts.checkNativeFilter(txSimContext.GetBlockVersion(), contractName, method, tx, txSimContext.GetSnapshot()) &&
		tx.Payload.TxType == commonPb.TxType_INVOKE_CONTRACT {
		ts.log.Debugf("getAccountMgrContractAndPk => txSimContext.GetContractByName(`%s`)",
			syscontract.SystemContract_ACCOUNT_MANAGER.String())
		accountMangerContract, err = txSimContext.GetContractByName(syscontract.SystemContract_ACCOUNT_MANAGER.String())
		if err != nil {
			ts.log.Error(err.Error())
			return nil, nil, err
		}

		pk, err = ts.getPayerPk(txSimContext, tx)
		if err != nil {
			ts.log.Error(err.Error())
			return accountMangerContract, nil, err
		}
		return accountMangerContract, pk, err
	}
	return nil, nil, nil
}

func (ts *TxScheduler) checkGasEnable() bool {

	return false
}

// checkNativeFilter use snapshot instead of blockchainStore
func (ts *TxScheduler) checkNativeFilter(blockVersion uint32, contractName, method string,
	tx *commonPb.Transaction, snapshot protocol.Snapshot) bool {
	ts.log.Debugf("checkNativeFilter => contractName = %s, method = %s", contractName, method)

	// 用户合约，扣费
	if !utils.IsNativeContract(contractName) {
		return true
	}

	// install & upgrade 系统合约扣费
	if contractName == syscontract.SystemContract_CONTRACT_MANAGE.String() {
		if method == syscontract.ContractManageFunction_INIT_CONTRACT.String() ||
			method == syscontract.ContractManageFunction_UPGRADE_CONTRACT.String() {
			return true
		}
	}

	return ts.checkMultiSignFilter2312(contractName, method, tx, snapshot)
}

func (ts *TxScheduler) checkMultiSignFilter2312(
	contractName string, method string, tx *commonPb.Transaction, snapshot protocol.Snapshot) bool {
	return contractName == syscontract.SystemContract_MULTI_SIGN.String()
}

// todo: merge with getPayerPk
func getPayerPkFromTx(tx *commonPb.Transaction, snapshot protocol.Snapshot) (crypto.PublicKey, error) {

	var err error
	var pk []byte
	var publicKey crypto.PublicKey
	signingMember := getTxPayerSigner(tx)
	if signingMember == nil {
		err = errors.New(" can not find sender from tx ")
		return nil, err
	}

	switch signingMember.MemberType {
	case accesscontrol.MemberType_CERT:
		pk, err = publicKeyFromCert(signingMember.MemberInfo)
		if err != nil {
			return nil, err
		}
		publicKey, err = asym.PublicKeyFromPEM(pk)
		if err != nil {
			return nil, err
		}

	case accesscontrol.MemberType_CERT_HASH:
		var certInfo *commonPb.CertInfo
		infoHex := hex.EncodeToString(signingMember.MemberInfo)
		if certInfo, err = wholeCertInfoFromSnapshot(snapshot, infoHex); err != nil {
			return nil, fmt.Errorf(" can not load the whole cert info,member[%s],reason: %s", infoHex, err)
		}

		pk, err = publicKeyFromCert(certInfo.Cert)
		if err != nil {
			return nil, err
		}

		publicKey, err = asym.PublicKeyFromPEM(pk)
		if err != nil {
			return nil, err
		}

	case accesscontrol.MemberType_PUBLIC_KEY:
		pk = signingMember.MemberInfo
		publicKey, err = asym.PublicKeyFromPEM(pk)
		if err != nil {
			return nil, err
		}

	default:
		err = fmt.Errorf("invalid member type: %s", signingMember.MemberType)
		return nil, err
	}

	return publicKey, nil
}

func (ts *TxScheduler) getPayerPk(txSimContext protocol.TxSimContext, tx *commonPb.Transaction) ([]byte, error) {

	var err error
	var pk []byte
	sender := getTxPayerSigner(tx)
	if sender == nil {
		err = errors.New(" can not find sender from tx ")
		ts.log.Error(err.Error())
		return nil, err
	}

	switch sender.MemberType {
	case accesscontrol.MemberType_CERT:
		pk, err = publicKeyFromCert(sender.MemberInfo)
		if err != nil {
			ts.log.Error(err.Error())
			return nil, err
		}
	case accesscontrol.MemberType_CERT_HASH:
		var certInfo *commonPb.CertInfo
		infoHex := hex.EncodeToString(sender.MemberInfo)
		if certInfo, err = wholeCertInfo(txSimContext, infoHex); err != nil {
			ts.log.Error(err.Error())
			return nil, fmt.Errorf(" can not load the whole cert info,member[%s],reason: %s", infoHex, err)
		}

		if pk, err = publicKeyFromCert(certInfo.Cert); err != nil {
			ts.log.Error(err.Error())
			return nil, err
		}

	case accesscontrol.MemberType_PUBLIC_KEY:
		pk = sender.MemberInfo
	default:
		err = fmt.Errorf("invalid member type: %s", sender.MemberType)
		ts.log.Error(err.Error())
		return nil, err
	}

	return pk, nil
}

// dispatchTxs dispatch txs from:
//  1. senderCollection when flag `enableOptimizeChargeGas` was set
//  2. senderGroup when flag `enableOptimizeChargeGas` was not set, and flag `enableSenderGroup` was set
//  3. txBatch directly where no flags was set
//     to runningTxC
func (ts *TxScheduler) dispatchTxs(
	block *commonPb.Block,
	txBatch []*commonPb.Transaction,
	runningTxC chan *commonPb.Transaction,
	goRoutinePool *ants.Pool,
	enableOptimizeChargeGas bool,
	senderCollection *SenderCollection,
	enableSenderGroup bool,
	senderGroup *SenderGroup,
	enableConflictsBitWindow bool,
	conflictsBitWindow *ConflictsBitWindow,
	snapshot protocol.Snapshot) {
	if enableOptimizeChargeGas {
		ts.log.Debugf("before `SenderCollection` dispatch => ")
		ts.dispatchTxsInSenderCollection(block, senderCollection, runningTxC, snapshot)
		ts.log.Debugf("end `SenderCollection` dispatch => ")

	} else if enableSenderGroup {
		ts.log.Debugf("before `SenderGroup` dispatch => ")
		if enableConflictsBitWindow {
			conflictsBitWindow.setMaxPoolCapacity(len(senderGroup.txsMap))
		}
		goRoutinePool.Tune(len(senderGroup.txsMap))
		ts.sendTxBySenderGroup(conflictsBitWindow, senderGroup, runningTxC, enableConflictsBitWindow)
		ts.log.Debugf("end `SenderGroup` dispatch => ")

	} else {
		ts.log.Debugf("before `Normal` dispatch => ")
		for _, tx := range txBatch {
			runningTxC <- tx
		}
		ts.log.Debugf("end `Normal` dispatch => ")
	}
}

// dispatchTxsInSenderCollection dispatch txs from senderCollection to runningTxC chan
// if the balance less than gas limit, set the result of tx and dispatch this tx.
// use snapshot for newest data
func (ts *TxScheduler) dispatchTxsInSenderCollection(
	block *commonPb.Block,
	senderCollection *SenderCollection,
	runningTxC chan *commonPb.Transaction,
	snapshot protocol.Snapshot) {
	ts.log.Debugf("begin dispatchTxsInSenderCollection(...)")
	for addr, txCollection := range senderCollection.txsMap {
		ts.log.Debugf("%v => {balance: %v, tx size: %v}",
			addr, txCollection.accountBalance, len(txCollection.txs))
	}

	for addr, txCollection := range senderCollection.txsMap {
		balance := txCollection.accountBalance
		for _, tx := range txCollection.txs {
			ts.log.Debugf("dispatch sender collection tx => %s", tx.Payload)
			var gasLimit int64
			limit := tx.Payload.Limit
			txNeedChargeGas := ts.checkNativeFilter(
				block.GetHeader().GetBlockVersion(),
				tx.GetPayload().ContractName,
				tx.GetPayload().Method,
				tx, snapshot)
			ts.log.Debugf("tx need charge gas => %v", txNeedChargeGas)
			if limit == nil && txNeedChargeGas {
				// tx需要扣费，但是limit没有设置
				tx.Result = &commonPb.Result{
					Code: commonPb.TxStatusCode_GAS_LIMIT_NOT_SET,
					ContractResult: &commonPb.ContractResult{
						Code:    uint32(1),
						Result:  nil,
						Message: ErrMsgOfGasLimitNotSet,
					},
					RwSetHash: nil,
					Message:   ErrMsgOfGasLimitNotSet,
				}

				runningTxC <- tx
				continue

			} else if txNeedChargeGas && tx.Result != nil {
				runningTxC <- tx
				continue

			} else if !txNeedChargeGas {
				// tx 不需要扣费
				gasLimit = int64(0)
			} else {
				// tx 需要扣费，limit 正常设置
				gasLimit = int64(limit.GasLimit)
			}

			// if the balance less than gas limit, set the result ahead, working goroutine will never runVM for it.
			if balance-gasLimit < 0 {
				pkStr, _ := txCollection.publicKey.String()
				ts.log.Debugf("balance is too low to execute tx. address = %v, public key = %s", addr, pkStr)
				errMsg := fmt.Sprintf("`%s` has no enough balance to execute tx.", addr)
				tx.Result = &commonPb.Result{
					Code: commonPb.TxStatusCode_GAS_BALANCE_NOT_ENOUGH_FAILED,
					ContractResult: &commonPb.ContractResult{
						Code:    uint32(1),
						Result:  nil,
						Message: errMsg,
					},
					RwSetHash: nil,
					Message:   errMsg,
				}
			} else {
				balance = balance - gasLimit
			}

			runningTxC <- tx
		}
	}
}

// signTxPayload sign charging tx with node's private key
func (ts *TxScheduler) signTxPayload(
	payload *commonPb.Payload) ([]byte, error) {

	payloadBytes, err := proto.Marshal(payload)
	if err != nil {
		return nil, err
	}

	// using the default hash type of the chain
	hashType := ts.chainConf.ChainConfig().GetCrypto().Hash
	return ts.signer.Sign(hashType, payloadBytes)
}

// appendChargeGasTxToDAG append the tx to the DAG with dependencies on all tx.
func (ts *TxScheduler) appendChargeGasTxToDAG(
	dag *commonPb.DAG,
	snapshot protocol.Snapshot) {

	dagNeighbors := &commonPb.DAG_Neighbor{
		Neighbors: make([]uint32, 0, snapshot.GetSnapshotSize()-1),
	}
	for i := uint32(0); i < uint32(snapshot.GetSnapshotSize()-1); i++ {
		dagNeighbors.Neighbors = append(dagNeighbors.Neighbors, i)
	}
	dag.Vertexes = append(dag.Vertexes, dagNeighbors)
}

func errResult(result *commonPb.Result, err error) (*commonPb.Result, protocol.ExecOrderTxType, error) {
	result.ContractResult.Message = err.Error()
	result.Code = commonPb.TxStatusCode_INVALID_PARAMETER
	result.ContractResult.Code = 1
	return result, protocol.ExecOrderTxTypeNormal, err
}

// parseUserAddress
func publicKeyFromCert(member []byte) ([]byte, error) {
	certificate, err := utils.ParseCert(member)
	if err != nil {
		return nil, err
	}
	pubKeyStr, err := certificate.PublicKey.String()
	if err != nil {
		return nil, err
	}
	return []byte(pubKeyStr), nil
}

func wholeCertInfo(txSimContext protocol.TxSimContext, certHash string) (*commonPb.CertInfo, error) {
	certBytes, err := txSimContext.Get(syscontract.SystemContract_CERT_MANAGE.String(), []byte(certHash))
	if err != nil {
		return nil, err
	}

	return &commonPb.CertInfo{
		Hash: certHash,
		Cert: certBytes,
	}, nil
}

type SenderGroup struct {
	txsMap     map[[32]byte][]*commonPb.Transaction
	doneTxKeyC chan [32]byte
}

func NewSenderGroup(txBatch []*commonPb.Transaction) *SenderGroup {
	return &SenderGroup{
		txsMap:     getSenderTxsMap(txBatch),
		doneTxKeyC: make(chan [32]byte, len(txBatch)),
	}
}

func getSenderTxsMap(txBatch []*commonPb.Transaction) map[[32]byte][]*commonPb.Transaction {
	senderTxsMap := make(map[[32]byte][]*commonPb.Transaction, len(txBatch))
	for _, tx := range txBatch {
		hashKey, _ := getSenderHashKey(tx)
		senderTxsMap[hashKey] = append(senderTxsMap[hashKey], tx)
	}
	return senderTxsMap
}

func getSenderHashKey(tx *commonPb.Transaction) ([32]byte, error) {
	sender := getTxPayerSigner(tx)
	keyBytes, err := sender.Marshal()
	if err != nil {
		return [32]byte{}, err
	}
	return sha256.Sum256(keyBytes), nil
}

// publicKeyToAddress: generate address from public key, according to chainconfig parameter
func publicKeyToAddress(pk crypto.PublicKey, chainCfg *configPb.ChainConfig) (string, error) {

	publicKeyString, err := utils.PkToAddrStr(pk, chainCfg.Vm.AddrType, crypto.HashAlgoMap[chainCfg.Crypto.Hash])
	if err != nil {
		return "", err
	}

	if chainCfg.Vm.AddrType == configPb.AddrType_ZXL {
		publicKeyString = "ZX" + publicKeyString
	}
	return publicKeyString, nil
}

func getTxPayerSigner(tx *commonPb.Transaction) *accesscontrol.Member {
	payer := tx.GetPayer()
	// don't need version compatibility
	if payer == nil {
		payer = tx.GetSender()
	}
	return payer.GetSigner()
}

func wholeCertInfoFromSnapshot(snapshot protocol.Snapshot, certHash string) (*commonPb.CertInfo, error) {
	certBytes, err := snapshot.GetKey(-1, syscontract.SystemContract_CERT_MANAGE.String(), []byte(certHash))
	if err != nil {
		return nil, err
	}

	return &commonPb.CertInfo{
		Hash: certHash,
		Cert: certBytes,
	}, nil
}

// getTxGasLimit get the gas limit field from tx, and will return err when the gas limit field is not set.
func getTxGasLimit(tx *commonPb.Transaction) (uint64, error) {
	var limit uint64

	if tx.Payload.Limit == nil {
		return limit, errors.New("tx payload limit is nil")
	}

	limit = tx.Payload.Limit.GasLimit
	return limit, nil
}

func (ts *TxScheduler) verifyExecOrderTxType(block *commonPb.Block,
	txExecOrderTypeMap map[string]protocol.ExecOrderTxType) (uint32, uint32, uint32, error) {

	var txExecOrderNormalCount, txExecOrderIteratorCount, txExecOrderChargeGasCount uint32
	for _, v := range txExecOrderTypeMap {
		switch v {
		case protocol.ExecOrderTxTypeNormal:
			txExecOrderNormalCount++
		case protocol.ExecOrderTxTypeIterator:
			txExecOrderIteratorCount++
		case protocol.ExecOrderTxTypeChargeGas:
			txExecOrderChargeGasCount++
		}
	}
	if (IsOptimizeChargeGasEnabled(ts.chainConf) && txExecOrderChargeGasCount != 1) ||
		(!IsOptimizeChargeGasEnabled(ts.chainConf) && txExecOrderChargeGasCount != 0) {
		return txExecOrderNormalCount, txExecOrderIteratorCount, txExecOrderChargeGasCount,
			fmt.Errorf("charge gas enabled but charge gas tx is not 1")
	}
	// check type are all correct
	for i, tx := range block.Txs {
		t, ok := txExecOrderTypeMap[tx.Payload.GetTxId()]
		if !ok {
			return txExecOrderNormalCount, txExecOrderIteratorCount, txExecOrderChargeGasCount,
				fmt.Errorf("cannot get tx ExecOrderTxType, txId:%s", tx.Payload.GetTxId())
		}
		var typeShouldBe protocol.ExecOrderTxType
		if uint32(i) < txExecOrderNormalCount {
			typeShouldBe = protocol.ExecOrderTxTypeNormal
		} else {
			typeShouldBe = protocol.ExecOrderTxTypeIterator
		}
		if IsOptimizeChargeGasEnabled(ts.chainConf) && uint32(i+1) == uint32(len(block.Txs)) {
			typeShouldBe = protocol.ExecOrderTxTypeChargeGas
		}
		if t != typeShouldBe {
			return txExecOrderNormalCount, txExecOrderIteratorCount, txExecOrderChargeGasCount,
				fmt.Errorf("tx type mismatch, txId:%s, index:%d", tx.Payload.GetTxId(), i)
		}
	}
	return txExecOrderNormalCount, txExecOrderIteratorCount, txExecOrderChargeGasCount, nil
}

func (ts *TxScheduler) compareDag(block *commonPb.Block, snapshot protocol.Snapshot,
	txRWSetMap map[string]*commonPb.TxRWSet, txExecOrderTypeMap map[string]protocol.ExecOrderTxType) error {
	if block.Header.BlockVersion < blockVersion2300 {
		return nil
	}
	startTime := time.Now()
	txExecOrderNormalCount, txExecOrderIteratorCount, txExecOrderChargeGasCount, err :=
		ts.verifyExecOrderTxType(block, txExecOrderTypeMap)
	if err != nil {
		ts.log.Errorf("verifyExecOrderTxType has err:%s, tx type count:%d,%d,%d, block tx count:%d", err,
			txExecOrderNormalCount, txExecOrderIteratorCount, txExecOrderChargeGasCount, block.Header.TxCount)
		return err
	}
	// rebuild and verify dag
	txRWSetTable := utils.RearrangeRWSet(block, txRWSetMap)
	if uint32(len(txRWSetTable)) != txExecOrderNormalCount+txExecOrderIteratorCount+txExecOrderChargeGasCount {
		return fmt.Errorf("txRWSetTable:%d != txExecOrderTypeCount:%d+%d+%d", len(txRWSetTable),
			txExecOrderNormalCount, txExecOrderIteratorCount, txExecOrderChargeGasCount)
	}

	// first, only build dag for normal tx
	txRWSetTable = txRWSetTable[0:txExecOrderNormalCount]
	dag := snapshot.BuildDAG(ts.chainConf.ChainConfig().Contract.EnableSqlSupport, txRWSetTable)
	// then, append special tx into dag
	if txExecOrderIteratorCount > 0 {
		appendSpecialTxsToDag(dag, txExecOrderIteratorCount)
	}

	equal, err := utils.IsDagEqual(block.Dag, dag)
	if err != nil {
		return err
	}
	if !equal {
		ts.log.Warnf("compare block dag (vertex:%d) with simulate dag (vertex:%d)",
			len(block.Dag.Vertexes), len(dag.Vertexes))
		return fmt.Errorf("simulate dag not equal to block dag")
	}
	timeUsed := time.Since(startTime)
	ts.log.Infof("compare dag finished, time used %v", timeUsed)
	return nil
}

func (ts *TxScheduler) releaseContractCache() {
	ts.contractCache.Range(func(key interface{}, value interface{}) bool {
		ts.contractCache.Delete(key)
		return true
	})
}

// appendSpecialTxsToDag similar to ts.simulateSpecialTxs except do not execute tx, only handle dag
// txExecOrderSpecialCount must >0
func appendSpecialTxsToDag(dag *commonPb.DAG, txExecOrderSpecialCount uint32) {
	txExecOrderNormalCount := uint32(len(dag.Vertexes))
	// the first special tx
	dagNeighbors := &commonPb.DAG_Neighbor{
		Neighbors: make([]uint32, 0, txExecOrderNormalCount),
	}
	for i := uint32(0); i < txExecOrderNormalCount; i++ {
		dagNeighbors.Neighbors = append(dagNeighbors.Neighbors, i)
	}
	dag.Vertexes = append(dag.Vertexes, dagNeighbors)
	// other special tx
	for i := uint32(1); i < txExecOrderSpecialCount; i++ {
		dagNeighbors := &commonPb.DAG_Neighbor{
			Neighbors: make([]uint32, 0, 1),
		}
		// this special tx (txExecOrderNormalCount+i) only depend on previous special tx (txExecOrderNormalCount+i-1)
		dagNeighbors.Neighbors = append(dagNeighbors.Neighbors, txExecOrderNormalCount+i-1)
		dag.Vertexes = append(dag.Vertexes, dagNeighbors)
	}
}
