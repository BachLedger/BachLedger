/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package lockfreequeue

import (
	"fmt"
	"runtime"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

func TestQueue(t *testing.T) {
	q := NewQueue(8)
	ok, quantity := q.Push(&value)
	if !ok {
		t.Error("TestStack Pull.Fail")
		return
	}
	t.Logf("TestStack Push value:%d[%v], quantity:%v\n", &value, value, quantity)

	val, ok, quantity := q.Pull()
	if !ok {
		t.Error("TestStack Pull.Fail")
		return
	}
	t.Logf("TestStack Pull value:%d[%v], quantity:%v\n", val, *(val.(*int)), quantity)
	if q := q.Quantity(); q != 0 {
		t.Errorf("Quantity Error: [%v] <>[%v]", q, 0)
	}
}

func TestQueuePushPull(t *testing.T) {
	runtime.GOMAXPROCS(runtime.NumCPU())
	const (
		isPrintf = false
	)

	cnt := 10000
	sum := 0
	start := time.Now()
	var putD, getD time.Duration
	for i := 0; i <= runtime.NumCPU()*4; i++ {
		sum += i * cnt
		put, get := testQueuePushPull(t, i, cnt)
		putD += put
		getD += get
	}
	end := time.Now()
	use := end.Sub(start)
	op := use / time.Duration(sum)
	t.Logf("Grp: %d, Times: %d, use: %v, %v/op", runtime.NumCPU()*4, sum, use, op)
	t.Logf("Push: %d, use: %v, %v/op", sum, putD, putD/time.Duration(sum))
	t.Logf("Pull: %d, use: %v, %v/op", sum, getD, getD/time.Duration(sum))
}

//TODO
//func TestQueueGeneral(t *testing.T) {
//	runtime.GOMAXPROCS(runtime.NumCPU())
//	const (
//		isPrintf = false
//	)
//
//	var miss, Sum int
//	var Use time.Duration
//	for i := 1; i <= runtime.NumCPU()*4; i++ {
//		cnt := 10000 * 100
//		if i > 9 {
//			cnt = 10000 * 10
//		}
//		sum := i * cnt
//		start := time.Now()
//		miss = testQueueGeneral(t, i, cnt)
//		end := time.Now()
//		use := end.Sub(start)
//		op := use / time.Duration(sum)
//		fmt.Printf("%v, Grp: %3d, Times: %10d, miss:%6v, use: %12v, %8v/op\n",
//			runtime.Version(), i, sum, miss, use, op)
//		Use += use
//		Sum += sum
//	}
//	op := Use / time.Duration(Sum)
//	fmt.Printf("%v, Grp: %3v, Times: %10d, miss:%6v, use: %12v, %8v/op\n",
//		runtime.Version(), "Sum", Sum, 0, Use, op)
//}
//TODO
//func TestQueuePushGoPull(t *testing.T) {
//	var Sum, miss int
//	var Use time.Duration
//	for i := 1; i <= runtime.NumCPU()*4; i++ {
//		//	for i := 2; i <= 2; i++ {
//		cnt := 10000 * 100
//		if i > 9 {
//			cnt = 10000 * 10
//		}
//		sum := i * cnt
//		start := time.Now()
//		miss = testQueuePushGoPull(i, cnt)
//		_ = miss
//		end := time.Now()
//		use := end.Sub(start)
//		//op := use / time.Duration(sum)
//		//t.Logf("%v, Grp: %3d, Times: %10d, miss:%6v, use: %12v, %8v/op\n",
//		//	runtime.Version(), i, sum, miss, use, op)
//		Use += use
//		Sum += sum
//	}
//	//op := Use / time.Duration(Sum)
//	//fmt.Printf("%v, Grp: %3v, Times: %10d, miss:%6v, use: %12v, %8v/op\n",
//	//	runtime.Version(), "Sum", Sum, 0, Use, op)
//}

func TestQueuePushDoPull(t *testing.T) {
	runtime.GOMAXPROCS(runtime.NumCPU())

	var miss, sum1 int
	var useTime time.Duration
	for i := 1; i <= runtime.NumCPU()*4; i++ {
		//	for i := 2; i <= 2; i++ {
		cnt := 10000 * 100
		if i > 9 {
			cnt = 10000 * 10
		}
		sum := i * cnt
		start := time.Now()
		miss = testQueuePushDoPull(i, cnt)
		end := time.Now()
		use := end.Sub(start)
		op := use / time.Duration(sum)
		t.Logf("%v, Grp: %3d, Times: %10d, miss:%6v, use: %12v, %8v/op\n",
			runtime.Version(), i, sum, miss, use, op)
		useTime += use
		sum1 += sum
	}
	op := useTime / time.Duration(sum1)
	fmt.Printf("%v, Grp: %3v, Times: %10d, miss:%6v, use: %12v, %8v/op\n",
		runtime.Version(), "sum1", sum1, 0, useTime, op)
}

func testQueuePushPull(t *testing.T, grp, cnt int) (
	put time.Duration, get time.Duration) {
	var wg sync.WaitGroup
	var id int32
	wg.Add(grp)
	q := NewQueue(1024 * 1024)
	start := time.Now()
	for i := 0; i < grp; i++ {
		go func(g int) {
			defer wg.Done()
			for j := 0; j < cnt; j++ {
				val := fmt.Sprintf("Node.%d.%d.%d", g, j, atomic.AddInt32(&id, 1))
				ok, _ := q.Push(&val)
				for !ok {
					time.Sleep(time.Microsecond)
					ok, _ = q.Push(&val)
				}
			}
		}(i)
	}
	wg.Wait()
	end := time.Now()
	put = end.Sub(start)

	wg.Add(grp)
	start = time.Now()
	for i := 0; i < grp; i++ {
		go func() {
			defer wg.Done()
			for j := 0; j < cnt; {
				_, ok, _ := q.Pull()
				if !ok {
					runtime.Gosched()
				} else {
					j++
				}
			}
		}()
	}
	wg.Wait()
	end = time.Now()
	get = end.Sub(start)
	if q := q.Quantity(); q != 0 {
		t.Errorf("Grp:%v, Quantity Error: [%v] <>[%v]", grp, q, 0)
	}
	return put, get
}

func testQueueGeneral(t *testing.T, grp, cnt int) int {

	var wg sync.WaitGroup
	var idPush, idPull int32
	var miss int32

	wg.Add(grp)
	q := NewQueue(1024 * 1024)
	for i := 0; i < grp; i++ {
		go func(g int) {
			defer wg.Done()
			for j := 0; j < cnt; j++ {
				val := fmt.Sprintf("Node.%d.%d.%d", g, j, atomic.AddInt32(&idPush, 1))
				ok, _ := q.Push(&val)
				for !ok {
					time.Sleep(time.Microsecond)
					ok, _ = q.Push(&val)
				}
			}
		}(i)
	}

	wg.Add(grp)
	for i := 0; i < grp; i++ {
		go func() {
			defer wg.Done()
			ok := false
			for j := 0; j < cnt; j++ {
				_, ok, _ = q.Pull() //该语句注释掉将导致运行结果不正确
				for !ok {
					atomic.AddInt32(&miss, 1)
					time.Sleep(time.Microsecond * 50)
					_, ok, _ = q.Pull()
				}
				atomic.AddInt32(&idPull, 1)
			}
		}()
	}
	wg.Wait()
	if q := q.Quantity(); q != 0 {
		t.Errorf("Grp:%v, Quantity Error: [%v] <>[%v]", grp, q, 0)
	}
	return int(miss)
}

type QtObj struct {
	getMiss int32
	putMiss int32
	putCnt  int32
	getCnt  int32
}

type QtSum struct {
	Go []QtObj
}

func newQtSum(grp int) *QtSum {
	qt := new(QtSum)
	qt.Go = make([]QtObj, grp)
	return qt
}

func (q *QtSum) PullMiss() (num int32) {
	for i := range q.Go {
		num += q.Go[i].getMiss
	}
	return
}
func (q *QtSum) PushMiss() (num int32) {
	for i := range q.Go {
		num += q.Go[i].putMiss
	}
	return
}
func (q *QtSum) PushCnt() (num int32) {
	for i := range q.Go {
		num += q.Go[i].putCnt
	}
	return
}
func (q *QtSum) PullCnt() (num int32) {
	for i := range q.Go {
		num += q.Go[i].getCnt
	}
	return
}

var (
	value int = 1 // nolint: revive
)

func testQueuePushGoPull(grp, cnt int) int {
	var wg sync.WaitGroup
	//var Qt = newQtSum(grp)
	wg.Add(grp)
	q := NewQueue(1024 * 1024)
	for i := 0; i < grp; i++ {
		go func() {
			ok := false
			for j := 0; j < cnt; j++ {
				ok, _ = q.Push(&value)
				//var miss int32
				for !ok {
					//Qt.Go[g].getMiss++
					//atomic.AddInt32(&miss, 1)
					//time.Sleep(time.Microsecond)
					ok, _ = q.Push(&value)
					//if miss > 10000 {
					//	panic(fmt.Sprintf("Push Fail PushId:%12v, PullId:%12v, "+
					//		"putCnt:%12v, putMis:%12v, "+
					//		"getCnt:%12v, getMis:%12v\n",
					//		q.eqPush, q.eqPull, Qt.PushCnt(), Qt.PushMiss(), Qt.PullCnt(), Qt.PullMiss()))
					//}
				}
				//Qt.Go[g].putCnt++
			}
			wg.Done()
		}()
	}
	wg.Add(grp)
	for i := 0; i < grp; i++ {
		go func() {
			ok := false
			for j := 0; j < cnt; j++ {
				//var miss int32
				_, ok, _ = q.Pull() //该语句注释掉将导致运行结果不正确
				for !ok {
					//Qt.Go[g].putMiss++
					//atomic.AddInt32(&miss, 1)
					//time.Sleep(time.Microsecond * 100)
					_, ok, _ = q.Pull()
					//if miss > 10000 {
					//	panic(fmt.Sprintf("Pull Miss PushId:%12v, PullId:%12v, "+
					//		"putCnt:%12v, putMis:%12v, "+
					//		"getCnt:%12v, getMis:%12v\n",
					//		q.eqPush, q.eqPull, Qt.PushCnt(), Qt.PushMiss(),
					//		Qt.PullCnt(), Qt.PullMiss()))
					//}
					//printf("Pull.Fail\n")
				}
				//Qt.Go[g].getCnt++
			}
			wg.Done()
		}()
	}
	wg.Wait()
	return 0 //int(Qt.PushMiss()) + int(Qt.PullMiss())
}

func testQueuePushDoPull(grp, cnt int) int {
	var wg sync.WaitGroup
	//var Qt = newQtSum(grp)
	wg.Add(grp)
	q := NewQueue(1024 * 1024)
	for i := 0; i < grp; i++ {
		go func() {
			ok := false
			for j := 0; j < cnt; j++ {
				ok, _ = q.Push(&value)
				//var missPush int32
				for !ok {
					//Qt.Go[g].getMiss++
					//missPush++
					//time.Sleep(time.Microsecond)
					ok, _ = q.Push(&value)
					//if missPush > 10000 {
					//	panic(fmt.Sprintf("Push Fail PushId:%12v, PullId:%12v, "+
					//		"putCnt:%12v, putMis:%12v, "+
					//		"getCnt:%12v, getMis:%12v\n",
					//		q.eqPush, q.eqPull, Qt.PushCnt(), Qt.PushMiss(), Qt.PullCnt(), Qt.PullMiss()))
					//}
				}
				//Qt.Go[g].putCnt++

				//var missPull int32
				_, ok, _ = q.Pull() //该语句注释掉将导致运行结果不正确
				for !ok {
					//Qt.Go[g].putMiss++
					//missPull++
					//time.Sleep(time.Microsecond * 100)
					_, ok, _ = q.Pull()
					//if missPull > 10000 {
					//	panic(fmt.Sprintf("Pull Miss PushId:%12v, PullId:%12v, "+
					//		"putCnt:%12v, putMis:%12v, "+
					//		"getCnt:%12v, getMis:%12v\n",
					//		q.eqPush, q.eqPull, Qt.PushCnt(), Qt.PushMiss(),
					//		Qt.PullCnt(), Qt.PullMiss()))
					//}
					//printf("Pull.Fail\n")
				}
				//Qt.Go[g].getCnt++
			}
			wg.Done()
		}()
	}
	wg.Wait()
	return 0 //int(Qt.PushMiss()) + int(Qt.PullMiss())
}

func testQueuePushPullOrder(t *testing.T, grp, cnt int) (
	residue int) {
	var wg sync.WaitGroup
	var idPush, idPull int32
	wg.Add(grp)
	q := NewQueue(1024 * 1024)
	for i := 0; i < grp; i++ {
		go func() {
			defer wg.Done()
			for j := 0; j < cnt; j++ {
				v := atomic.AddInt32(&idPush, 1)
				ok, _ := q.Push(v)
				for !ok {
					time.Sleep(time.Microsecond)
					ok, _ = q.Push(v)
				}
			}
		}()
	}
	wg.Wait()
	wg.Add(grp)
	for i := 0; i < grp; i++ {
		go func() {
			defer wg.Done()
			for j := 0; j < cnt; {
				val, ok, _ := q.Pull()
				if !ok {
					fmt.Printf("Pull.Fail\n")
					runtime.Gosched()
				} else {
					j++
					idPull++
					if idPull != val.(int32) {
						t.Logf("Pull.Err %d <> %d\n", idPull, val)
					}
				}
			}
		}()
	}
	wg.Wait()
	return
}

func TestQueuePushPullOrder(t *testing.T) {
	runtime.GOMAXPROCS(runtime.NumCPU())
	grp := 1
	cnt := 100

	testQueuePushPullOrder(t, grp, cnt)
	t.Logf("Grp: %d, Times: %d", grp, cnt)
}

//TODO
//func TestQueueDataValidity(t *testing.T) {
//	runtime.GOMAXPROCS(runtime.NumCPU())
//	var maxCount uint32 = 10000000
//	goroutineCount := 2000
//	var queueSize uint32 = 1024 * 1024
//	var pushFailedTimes uint32 = 0
//	var pullFailedTimes uint32 = 0
//	var w, r uint32 = 0, 0
//	q := NewQueue(queueSize)
//	fmt.Println("queue capacity", q.Capacity())
//	wg := sync.WaitGroup{}
//	startTime := time.Now().UnixNano() / 1000000
//	wg.Add(1)
//	go func() {
//		for i := 0; i < goroutineCount; i++ {
//			wg.Add(1)
//			go func() {
//				for {
//					j := atomic.AddUint32(&w, 1)
//					if j > maxCount {
//						break
//					}
//					ok, _ := q.Push(&j)
//					for !ok {
//						time.Sleep(time.Millisecond)
//						atomic.AddUint32(&pushFailedTimes, 1)
//						ok, _ = q.Push(&j)
//					}
//				}
//				wg.Done()
//			}()
//		}
//		wg.Done()
//	}()
//
//	wg.Add(1)
//	go func() {
//		for i := 0; i < goroutineCount; i++ {
//			wg.Add(1)
//			go func() {
//				for {
//					j := atomic.AddUint32(&r, 1)
//					if j > maxCount {
//						break
//					}
//					_, ok, _ := q.Pull()
//					for !ok {
//						time.Sleep(time.Millisecond)
//						atomic.AddUint32(&pullFailedTimes, 1)
//						_, ok, _ = q.Pull()
//					}
//				}
//				wg.Done()
//			}()
//		}
//		wg.Done()
//	}()
//	wg.Wait()
//	useTime := time.Now().UnixNano()/1000000 - startTime
//	t.Logf(fmt.Sprintf("push failed times:%d, pull failed times:%d, time use:%d ms", pushFailedTimes, pullFailedTimes, useTime))
//}
