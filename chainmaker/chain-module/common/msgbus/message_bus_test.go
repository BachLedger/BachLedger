/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package msgbus

import (
	"reflect"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestUnRegister(t *testing.T) {
	topic := ProposedBlock
	bus := NewMessageBus()
	impl := bus.(*messageBusImpl)

	sub1 := &sub{}
	sub2 := &sub{}
	sub3 := &sub{}
	bus.Register(topic, sub1)
	bus.Register(topic, sub2)

	subscribers, ok := impl.topicMap.Load(topic)
	require.True(t, ok)

	subs := subscribers.([]Subscriber)
	require.Equal(t, 2, len(subs))
	require.Equal(t, sub1, subs[0])
	require.Equal(t, sub2, subs[1])

	// UnRegister sub1 from msgbus.
	bus.UnRegister(topic, sub1)
	subscribers, ok = impl.topicMap.Load(topic)
	require.True(t, ok)

	subs = subscribers.([]Subscriber)
	require.Equal(t, 1, len(subs))
	require.Equal(t, sub2, subs[0])

	// UnRegister sub3 from msgbus.
	bus.UnRegister(topic, sub3)
	subscribers, ok = impl.topicMap.Load(topic)
	require.True(t, ok)

	subs = subscribers.([]Subscriber)
	require.Equal(t, 1, len(subs))
	require.Equal(t, sub2, subs[0])
}

func TestMessageBusWithoutSubscriber(_ *testing.T) {
	bus := NewMessageBus()
	bus.Publish(ProposedBlock, nil)
}

func TestConcurrentPubSub(t *testing.T) {
	bus := NewMessageBus()
	sub := &sub{Counter: 0}
	bus.Register(ProposedBlock, sub)
	count := 100
	concurrentPub(bus, count)
	time.Sleep(time.Second * 3)
	assert.Equal(t, int32(count), sub.Counter)
}

//TODO
//func TestDuplicateRegister(t *testing.T) {
//	bus := NewMessageBus()
//	sub := &sub{Counter: 0}
//	bus.Register(ProposedBlock, sub)
//	bus.Register(ProposedBlock, sub)
//	count := 100
//	concurrentPub(bus, count)
//	time.Sleep(time.Second * 3)
//	assert.Equal(t, int32(count*2), sub.Counter)
//}
//TODO
//func TestClose(t *testing.T) {
//	bus := NewMessageBus()
//	sub := &sub{Counter: 0}
//	bus.Register(ProposedBlock, sub)
//	count := 1000
//	concurrentPub(bus, count)
//	time.Sleep(time.Second * 3)
//	assert.Equal(t, int32(count), sub.Counter)
//	bus.Close()
//	time.Sleep(time.Second * 1)
//	assert.Equal(t, int32(count+100), sub.Counter)
//}
//TODO
//func TestDuplicateClose(t *testing.T) {
//	bus := NewMessageBus()
//	sub := &sub{Counter: 0}
//	bus.Register(ProposedBlock, sub)
//	count := 1000
//	concurrentPub(bus, count)
//	time.Sleep(time.Second * 3)
//	assert.Equal(t, int32(count), sub.Counter)
//	bus.Close()
//	bus.Close()
//	bus.Close()
//	time.Sleep(time.Second * 3)
//	assert.Equal(t, int32(count+100), sub.Counter)
//}
//
//func TestCloseThenPub(t *testing.T) {
//	bus := NewMessageBus()
//	sub := &sub{Counter: 0}
//	bus.Register(ProposedBlock, sub)
//	count := 1000
//	concurrentPub(bus, count)
//	time.Sleep(time.Second * 3)
//	assert.Equal(t, int32(count), sub.Counter)
//	bus.Close()
//	time.Sleep(time.Second * 1)
//	assert.Equal(t, int32(count+100), sub.Counter)
//	concurrentPub(bus, count)
//	time.Sleep(time.Second * 3)
//	assert.Equal(t, int32(count+100), sub.Counter)
//}
//TODO
//func TestCloseThenSub(t *testing.T) {
//	bus := NewMessageBus()
//	sub := &sub{Counter: 0}
//	bus.Register(ProposedBlock, sub)
//	count := 1000
//	concurrentPub(bus, count)
//	time.Sleep(time.Second * 3)
//	assert.Equal(t, int32(count), sub.Counter)
//	bus.Close()
//	time.Sleep(time.Second * 1)
//	assert.Equal(t, int32(count+100), sub.Counter)
//	concurrentPub(bus, count)
//	time.Sleep(time.Second * 3)
//	assert.Equal(t, int32(count+100), sub.Counter)
//	bus.Register(ProposedBlock, sub)
//	concurrentPub(bus, count)
//	time.Sleep(time.Second * 3)
//	assert.Equal(t, int32(count+100), sub.Counter)
//}

func TestConcurrentSafeAndUnsafe(t *testing.T) {
	bus := NewMessageBus()
	sub := &sub4safe{seq: make([]int, 0)}
	sub4safe := &sub4safe{seq: make([]int, 0)}
	bus.Register(ProposedBlock, sub)
	bus.Register(TxPoolSignal, sub4safe)
	concurrentPubSafe(bus, 10000, true)
	concurrentPubSafe(bus, 10000, false)
	require.True(t, isOrder(sub4safe.seq))
}

func TestSubRedundant(t *testing.T) {
	sub1 := &sub{Counter: 0}
	sub2 := &sub{Counter: 0}
	sub3 := &sub{Counter: 1}
	sub4 := &sub{Counter: 2}
	subs := make([]Subscriber, 0)
	subs = append(subs, sub1)
	subs = append(subs, sub2)
	subs = append(subs, sub3)
	require.True(t, isRedundant(subs, sub1))
	require.True(t, isRedundant(subs, sub2))
	require.True(t, isRedundant(subs, sub3))
	require.False(t, isRedundant(subs, sub4))
	require.True(t, reflect.DeepEqual(sub1, sub2))
	require.False(t, reflect.DeepEqual(sub2, sub3))
}

func concurrentPub(bus MessageBus, count int) {
	var wg sync.WaitGroup
	for i := 0; i < count; i++ {
		wg.Add(1)
		go func() {
			bus.Publish(ProposedBlock, nil)
			wg.Done()
		}()
	}
	wg.Wait()
}
func concurrentPubSafe(bus MessageBus, count int, isSafe bool) {
	for i := 0; i < count; i++ {
		if isSafe {
			bus.PublishSafe(TxPoolSignal, i)
		} else {
			bus.Publish(ProposedBlock, i)
		}
	}
}

type sub struct {
	Counter int32
}

func (s *sub) OnMessage(*Message) {
	atomic.AddInt32(&s.Counter, 1)
}

func (s *sub) OnQuit() {
	atomic.AddInt32(&s.Counter, 100)
}

type sub4safe struct {
	seq []int
}

func (s *sub4safe) OnMessage(m *Message) {
	i, _ := m.Payload.(int)
	s.seq = append(s.seq, i)
}

func (s *sub4safe) OnQuit() {
}

func isOrder(seq []int) bool {
	for i, s := range seq {
		if i != s {
			//fmt.Printf("%d != %d", i, s)
			return false
		}
	}
	return true
}
