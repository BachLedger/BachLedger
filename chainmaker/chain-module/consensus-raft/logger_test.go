/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package raft

import (
	"reflect"
	"testing"

	"chainmaker.org/chainmaker/logger/v2"
	"chainmaker.org/chainmaker/protocol/v2"
	"go.uber.org/zap"
)

func TestNewLogger(t *testing.T) {
	type args struct {
		lg *zap.SugaredLogger
	}
	tests := []struct {
		name string
		args args
		want *Logger
	}{
		// TODO: Add test cases.
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := NewLogger(tt.args.lg); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("NewLogger() = %v, want %v", got, tt.want)
			}
		})
	}
}

func newMockLogger() protocol.Logger {
	return logger.GetLoggerByChain(logger.MODULE_CONSENSUS, "test_chain_id")
}
