/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package priorityblocker

import "testing"

var (
	blocker  = NewBlocker(nil)
	tempFlag = "TEMP"
	ticketC  = make(chan struct{})
	blocker2 = NewBlocker(ticketC)
)

func init() {
	blocker.Run()
	blocker2.Run()
	go func() {
		for {
			ticketC <- struct{}{}
		}
	}()
}

func BenchmarkBlockerBlock(b *testing.B) {
	for i := 0; i < b.N; i++ {
		blocker.Block(tempFlag)
	}
}

func BenchmarkBlockerWithOuterTicketsPrinterBlock(b *testing.B) {
	for i := 0; i < b.N; i++ {
		blocker.Block(tempFlag)
	}
}
