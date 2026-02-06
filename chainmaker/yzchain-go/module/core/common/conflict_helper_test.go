package common

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"math/rand"
	"sync"
	"testing"
	"time"
)

func TestReserveTable_ConcurrentTryTakeOwnership(t *testing.T) {
	rst := &ReserveTable{
		Table: make(map[string]*RTableItem),
		mutex: sync.RWMutex{},
	}

	const goroutines = 100
	rand.Seed(time.Now().UnixNano())

	// 生成一百个随机数
	numbers := make([]int, goroutines)
	for i := range numbers {
		numbers[i] = rand.Intn(goroutines * 10) // 假设随机数在 0 到 999 之间
	}

	// 找出最小值
	min := numbers[0]
	for _, num := range numbers {
		if num < min {
			min = num
		}
	}
	var wg sync.WaitGroup
	wg.Add(goroutines)

	storageKey := "key1"
	who := make([]byte, 9)
	who[0] = 0
	binary.BigEndian.PutUint64(who[1:], uint64(min))

	successCount := 0
	var successMutex sync.Mutex

	for i := 0; i < goroutines; i++ {
		go func(id int) {
			defer wg.Done()
			w := make([]byte, 9)
			w[0] = 0
			time.Sleep(time.Duration(time.Millisecond * time.Duration(id)))
			binary.BigEndian.PutUint64(w[1:9], uint64(numbers[id]))
			fmt.Printf("goroutine %d: w %v\n", id, w)
			if id == 0 {
				who = w
			}
			if rst.TryTakeOwnership(storageKey, w) {
				successMutex.Lock()
				successCount++
				successMutex.Unlock()
			}
		}(i)
	}

	wg.Wait()

	if successCount == 0 {
		t.Errorf("expected more than one success, got %d", successCount)
	}

	if !bytes.Equal(rst.Table[storageKey].Owner, who) {
		t.Errorf("expected owner to be %x, got %x", who, rst.Table[storageKey].Owner)
	}
}

func TestReserveTable_SingleRoutineTakeOwnership(t *testing.T) {
	rst := &ReserveTable{
		Table: make(map[string]*RTableItem),
		mutex: sync.RWMutex{},
	}

	storageKey := "key1"

	id := 0
	who := make([]byte, 9, 100)
	who[0] = 0
	binary.BigEndian.PutUint64(who[1:9], uint64(id))
	fmt.Printf("goroutine %d: who %v\n", id, who)

	if !rst.TryTakeOwnership(storageKey, who) {
		t.Errorf("expected success, got false")
	}

	if !bytes.Equal(rst.Table[storageKey].Owner, who) {
		t.Errorf("expected owner to be %x, got %x", who, rst.Table[storageKey].Owner)
	}
}

func TestReserveTable_SingleRoutineTakeOwnershipAgain(t *testing.T) {
	rst := &ReserveTable{
		Table: make(map[string]*RTableItem),
		mutex: sync.RWMutex{},
	}

	storageKey := "key1"

	id := 234
	who := make([]byte, 9, 100)
	who[0] = 0
	binary.BigEndian.PutUint64(who[1:9], uint64(id))
	fmt.Printf("goroutine %d: who %v\n", id, who)

	if !rst.TryTakeOwnership(storageKey, who) {
		t.Errorf("expected success, got false")
	}
	if !rst.TryTakeOwnership(storageKey, who) {
		t.Errorf("expected success again, got false")
	}
	if ok, _ := rst.CheckDirtyWrite(storageKey, who); !ok {
		t.Errorf("Check fail")
	}

	if !bytes.Equal(rst.Table[storageKey].Owner, who) {
		t.Errorf("expected owner to be %x, got %x", who, rst.Table[storageKey].Owner)
	}
}

func TestOrderedMutex(t *testing.T) {
	om := NewOrderedMutex(TestLogger{T: t})

	var wg sync.WaitGroup
	var result []int
	var mu sync.Mutex

	worker := func(seq int) {
		defer wg.Done()
		om.Lock(seq)

		mu.Lock()
		result = append(result, seq)
		mu.Unlock()

		om.Unlock(seq)
	}

	// Create 5 workers with sequence numbers from 1 to 5
	for i := 1; i <= 5; i++ {
		wg.Add(1)
		go worker(i)
	}

	wg.Wait()

	// Check if the results are in the correct order
	expected := []int{1, 2, 3, 4, 5}
	for i, v := range expected {
		if result[i] != v {
			t.Fatalf("expected %d, but got %d", v, result[i])
		}
	}
}

type TestLogger struct {
	T *testing.T
}

func (t TestLogger) Infof(format string, args ...interface{}) {
	t.T.Logf(format, args...)
}

func (t TestLogger) Debug(args ...interface{}) {
	t.T.Log(args...)
}

func (t TestLogger) Debugf(format string, args ...interface{}) {
	t.T.Logf(format, args...)
}

func (t TestLogger) Debugw(msg string, keysAndValues ...interface{}) {
	t.T.Logf(msg, keysAndValues...)
}

func (t TestLogger) Error(args ...interface{}) {
	t.T.Log(args...)
}

func (t TestLogger) Errorf(format string, args ...interface{}) {
	t.T.Errorf(format, args...)
}

func (t TestLogger) Errorw(msg string, keysAndValues ...interface{}) {
	t.T.Errorf(msg, keysAndValues...)
}

func (t TestLogger) Fatal(args ...interface{}) {
	t.T.Fatal(args...)
}

func (t TestLogger) Fatalf(format string, args ...interface{}) {
	t.T.Fatalf(format, args...)
}

func (t TestLogger) Fatalw(msg string, keysAndValues ...interface{}) {
	t.T.Fatalf(msg, keysAndValues...)
}

func (t TestLogger) Info(args ...interface{}) {
	t.T.Log(args...)
}

func (t TestLogger) Infow(msg string, keysAndValues ...interface{}) {
	t.T.Logf(msg, keysAndValues...)
}

func (t TestLogger) Panic(args ...interface{}) {
	t.T.Log(args...)
}

func (t TestLogger) Panicf(format string, args ...interface{}) {
	t.T.Fatalf(format, args...)
}

func (t TestLogger) Panicw(msg string, keysAndValues ...interface{}) {
	t.T.Logf(msg, keysAndValues...)
}

func (t TestLogger) Warn(args ...interface{}) {
	t.T.Log(args...)
}

func (t TestLogger) Warnf(format string, args ...interface{}) {
	t.T.Logf(format, args...)
}

func (t TestLogger) Warnw(msg string, keysAndValues ...interface{}) {
	t.T.Logf(msg, keysAndValues...)
}

func (t TestLogger) DebugDynamic(getStr func() string) {
	t.T.Log(getStr())
}

func (t TestLogger) InfoDynamic(getStr func() string) {
	t.T.Log(getStr())
}
func TestOrderedMutexParallel(t *testing.T) {
	om := NewOrderedMutex(TestLogger{T: t})

	var wg sync.WaitGroup
	var result []int
	var mu sync.Mutex

	worker := func(seq int, delay time.Duration) {
		defer wg.Done()
		time.Sleep(delay)
		om.Lock(seq)

		mu.Lock()
		result = append(result, seq)
		mu.Unlock()

		om.Unlock(seq)
	}

	// Create workers with random delays
	wg.Add(1)
	go worker(3, 50*time.Millisecond)

	wg.Add(1)
	go worker(1, 10*time.Millisecond)

	wg.Add(1)
	go worker(2, 30*time.Millisecond)

	wg.Add(1)
	go worker(5, 70*time.Millisecond)

	wg.Add(1)
	go worker(4, 60*time.Millisecond)

	wg.Wait()

	// Check if the results are in the correct order
	expected := []int{1, 2, 3, 4, 5}
	for i, v := range expected {
		if result[i] != v {
			t.Fatalf("expected %d, but got %d", v, result[i])
		}
	}
}
