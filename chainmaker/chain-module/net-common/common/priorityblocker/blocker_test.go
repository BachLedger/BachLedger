/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package priorityblocker

import (
	"sync"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestBlockerBlock(t *testing.T) {
	a, b, c, d := "A", "B", "C", "D"

	blocker := NewBlocker(nil)
	blocker.SetPriority(a, PriorityLevel9)
	blocker.SetPriority(b, PriorityLevel7)
	blocker.SetPriority(c, PriorityLevel5)
	blocker.SetPriority(d, PriorityLevel3)

	aC := make(chan struct{})
	bC := make(chan struct{})
	cC := make(chan struct{})
	dC := make(chan struct{})

	blockTimes := 100000
	var wgA, wgB, wgC, wgD sync.WaitGroup
	wgA.Add(blockTimes)
	wgB.Add(blockTimes)
	wgC.Add(blockTimes)
	wgD.Add(blockTimes)
	for i := 0; i < blockTimes; i++ {
		go func() {
			blocker.Block(a)
			wgA.Done()
		}()
		go func() {
			blocker.Block(b)
			wgB.Done()
		}()
		go func() {
			blocker.Block(c)
			wgC.Done()
		}()
		go func() {
			blocker.Block(d)
			wgD.Done()
		}()
		if i == blockTimes/2 {
			blocker.Run()
		}
	}

	go func() {
		wgA.Wait()
		aC <- struct{}{}
	}()
	go func() {
		wgB.Wait()
		bC <- struct{}{}
	}()
	go func() {
		wgC.Wait()
		cC <- struct{}{}
	}()
	go func() {
		wgD.Wait()
		dC <- struct{}{}
	}()

	res := [4]int{}
	for i := 0; i < 4; i++ {
		select {
		case <-aC:
			res[i] = 1
		case <-bC:
			res[i] = 2
		case <-cC:
			res[i] = 3
		case <-dC:
			res[i] = 4
		}
	}
	//fmt.Printf("%v\n", res)
	require.True(t, res[0] == 1)
	require.True(t, res[1] == 2)
	require.True(t, res[2] == 3)
	require.True(t, res[3] == 4)
}

func TestBlockerBlockWithOuterTicketsPrinter(t *testing.T) {
	a, b, c, d := "A", "B", "C", "D"
	ticketC := make(chan struct{})
	blocker := NewBlocker(ticketC)
	blocker.SetPriority(a, PriorityLevel9)
	blocker.SetPriority(b, PriorityLevel7)
	blocker.SetPriority(c, PriorityLevel5)
	blocker.SetPriority(d, PriorityLevel3)

	aC := make(chan struct{})
	bC := make(chan struct{})
	cC := make(chan struct{})
	dC := make(chan struct{})

	blockTimes := 100000
	var wgA, wgB, wgC, wgD sync.WaitGroup
	wgA.Add(blockTimes)
	wgB.Add(blockTimes)
	wgC.Add(blockTimes)
	wgD.Add(blockTimes)
	blocker.Run()
	for i := 0; i < blockTimes; i++ {
		go func() {
			blocker.Block(a)
			wgA.Done()
		}()
		go func() {
			blocker.Block(b)
			wgB.Done()
		}()
		go func() {
			blocker.Block(c)
			wgC.Done()
		}()
		go func() {
			blocker.Block(d)
			wgD.Done()
		}()
		if i == blockTimes/2 {
			go func() {
				for {
					ticketC <- struct{}{}
				}
			}()
		}
	}

	go func() {
		wgA.Wait()
		aC <- struct{}{}
	}()
	go func() {
		wgB.Wait()
		bC <- struct{}{}
	}()
	go func() {
		wgC.Wait()
		cC <- struct{}{}
	}()
	go func() {
		wgD.Wait()
		dC <- struct{}{}
	}()

	res := [4]int{}
	for i := 0; i < 4; i++ {
		select {
		case <-aC:
			res[i] = 1
		case <-bC:
			res[i] = 2
		case <-cC:
			res[i] = 3
		case <-dC:
			res[i] = 4
		}
	}
	//fmt.Printf("%v\n", res)
	require.True(t, res[0] == 1)
	require.True(t, res[1] == 2)
	require.True(t, res[2] == 3)
	require.True(t, res[3] == 4)

}
