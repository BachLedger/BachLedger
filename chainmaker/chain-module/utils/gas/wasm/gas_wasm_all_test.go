package wasm

import (
	"testing"

	"chainmaker.org/chainmaker/protocol/v2"
	commongas "chainmaker.org/chainmaker/utils/v2/gas/common"
	mockgas "chainmaker.org/chainmaker/utils/v2/gas/mock"
	"github.com/stretchr/testify/assert"
)

func newTxSimContextMock(gasLimit uint64, defaultGas uint64) protocol.TxSimContext {
	return mockgas.MockTxSimContext(blockVersion232, gasLimit, defaultGas)
}

var (
	contractName        = "TestContractName1"
	defaultTxGas uint64 = 3000
	gasLimit     uint64 = 1000000
)

func TestGetStateGas(t *testing.T) {

	stateKey := []byte("test-key")
	value := []byte("test-value")

	txSimContextMock := newTxSimContextMock(gasLimit, defaultTxGas)
	err := SubtractGasForGetState(contractName, stateKey, value, txSimContextMock)
	if err != nil {
		t.Fatalf("SubtraceGas failed: err = %v", err)
	}

	var gasUsed, gasExpected uint64
	gasUsed = gasLimit - txSimContextMock.GetGasRemaining()
	gasExpected = wasmSyscallGas232[commongas.GET_STATE_SYSCALL] + defaultTxGas +
		uint64(len(stateKey)+len(contractName))*1 + uint64(len(value))*1
	assert.Equal(t, gasExpected, gasUsed,
		"subtract gas error. expected: %v, actual: %v", gasExpected, gasUsed)
}

func TestPutStateGas(t *testing.T) {

	stateKey := []byte("test-key")
	value := []byte("test-value")

	txSimContextMock := newTxSimContextMock(gasLimit, defaultTxGas)
	err := SubtractGasForPutState(contractName, stateKey, value, txSimContextMock)
	if err != nil {
		t.Fatalf("SubtraceGas failed: err = %v", err)
	}

	var gasUsed, gasExpected uint64
	gasUsed = gasLimit - txSimContextMock.GetGasRemaining()
	gasExpected = wasmSyscallGas232[commongas.PUT_STATE_SYSCALL] + defaultTxGas +
		uint64(len(stateKey)+len(contractName))*1 + uint64(len(value))*1
	assert.Equal(t, gasExpected, gasUsed,
		"subtract gas error. expected: %v, actual: %v", gasExpected, gasUsed)
}

func TestEmitEventGas(t *testing.T) {

	topic := "test-topic"
	events := []string{
		"test-event1",
		"test-event2",
	}

	txSimContextMock := newTxSimContextMock(gasLimit, defaultTxGas)
	err := SubtractGasForEmitEvent(topic, events, txSimContextMock)
	if err != nil {
		t.Fatalf("SubtraceGas failed: err = %v", err)
	}

	var gasUsed, gasExpected uint64
	gasUsed = gasLimit - txSimContextMock.GetGasRemaining()
	gasExpected = wasmSyscallGas232[commongas.EMIT_EVENT_SYSCALL] + defaultTxGas +
		uint64(len(topic))*1 + uint64(mockgas.CalcStringListDataSize(events))*1
	assert.Equal(t, gasExpected, gasUsed,
		"subtract gas error. expected: %v, actual: %v", gasExpected, gasUsed)
}

func TestCallContractGas(t *testing.T) {

	method := "test_method"
	params := make(map[string][]byte)
	returns := []byte("OK")

	txSimContextMock := newTxSimContextMock(gasLimit, defaultTxGas)
	err := SubtractGasForCallContract(contractName, method, params, returns, txSimContextMock)
	if err != nil {
		t.Fatalf("SubtraceGas failed: err = %v", err)
	}

	var gasUsed, gasExpected uint64
	gasUsed = gasLimit - txSimContextMock.GetGasRemaining()
	gasExpected = wasmSyscallGas232[commongas.CALL_CONTRACT_SYSCALL] + defaultTxGas +
		uint64(len(contractName)+len(method)+mockgas.CalcBytesMapDataSize(params))*1 + uint64(len(returns))*1
	assert.Equal(t, gasExpected, gasUsed,
		"subtract gas error. expected: %v, actual: %v", gasExpected, gasUsed)
}

func TestKvIteratorGas(t *testing.T) {

	key := []byte("test-key")
	limit := []byte("test-value")

	txSimContextMock := newTxSimContextMock(gasLimit, defaultTxGas)
	err := SubtractGasForKvIterator(key, limit, txSimContextMock)
	if err != nil {
		t.Fatalf("SubtraceGas failed: err = %v", err)
	}

	var gasUsed, gasExpected uint64
	gasUsed = gasLimit - txSimContextMock.GetGasRemaining()
	gasExpected = wasmSyscallGas232[commongas.KV_ITER_SYSCALL] + defaultTxGas +
		uint64(len(key)+len(limit))*1
	assert.Equal(t, gasExpected, gasUsed,
		"subtract gas error. expected: %v, actual: %v", gasExpected, gasUsed)
}

func TestKvIteratorHasNextGas(t *testing.T) {

	kvIndex := int32(0)
	txSimContextMock := newTxSimContextMock(gasLimit, defaultTxGas)
	err := SubtractGasForKvIteratorHasNext(kvIndex, txSimContextMock)
	if err != nil {
		t.Fatalf("SubtraceGas failed: err = %v", err)
	}

	var gasUsed, gasExpected uint64
	gasUsed = gasLimit - txSimContextMock.GetGasRemaining()
	gasExpected = wasmSyscallGas232[commongas.KV_ITER_HAS_NEXT_SYSCALL] + defaultTxGas
	assert.Equal(t, gasExpected, gasUsed,
		"subtract gas error. expected: %v, actual: %v", gasExpected, gasUsed)
}

func TestKvIteratorNextGas(t *testing.T) {

	key := "test-key"
	field := "test-field"
	value := []byte("test-value")

	txSimContextMock := newTxSimContextMock(gasLimit, defaultTxGas)
	err := SubtractGasForKvIteratorNext(key, field, value, txSimContextMock)
	if err != nil {
		t.Fatalf("SubtraceGas failed: err = %v", err)
	}

	var gasUsed, gasExpected uint64
	gasUsed = gasLimit - txSimContextMock.GetGasRemaining()
	gasExpected = wasmSyscallGas232[commongas.KV_ITER_NEXT_SYSCALL] + defaultTxGas +
		uint64(len(key)+len(field)+len(value))*1
	assert.Equal(t, gasExpected, gasUsed,
		"subtract gas error. expected: %v, actual: %v", gasExpected, gasUsed)
}

func TestKvIteratorCloseGas(t *testing.T) {

	kvIndex := int32(0)
	txSimContextMock := newTxSimContextMock(gasLimit, defaultTxGas)
	err := SubtractGasForKvIteratorClose(kvIndex, txSimContextMock)
	if err != nil {
		t.Fatalf("SubtraceGas failed: err = %v", err)
	}

	var gasUsed, gasExpected uint64
	gasUsed = gasLimit - txSimContextMock.GetGasRemaining()
	gasExpected = wasmSyscallGas232[commongas.KV_ITER_CLOSE_SYSCALL] + defaultTxGas
	assert.Equal(t, gasExpected, gasUsed,
		"subtract gas error. expected: %v, actual: %v", gasExpected, gasUsed)
}
