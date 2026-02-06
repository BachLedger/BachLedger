/*
 * Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 */

package types

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestSavePoint_GetCreateTableSql(t *testing.T) {
	type fields struct {
		BlockHeight uint64
	}
	type args struct {
		dbType string
	}
	tests := []struct {
		name   string
		fields fields
		args   args
		want   string
	}{
		// TODO: Add test cases.
		{
			name: "mysql", fields: fields{BlockHeight: 1},
			args: args{dbType: "mysql"},
			want: "CREATE TABLE `save_points` (`block_height` bigint unsigned AUTO_INCREMENT,PRIMARY KEY (`block_height`))",
		},
		{
			name: "sqlite", fields: fields{BlockHeight: 1},
			args: args{dbType: "sqlite"},
			want: "CREATE TABLE `save_points` (`block_height` integer,PRIMARY KEY (`block_height`))",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			b := &SavePoint{
				BlockHeight: tt.fields.BlockHeight,
			}
			if got := b.GetCreateTableSql(tt.args.dbType); got != tt.want {
				t.Errorf("GetCreateTableSql() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestSavePoint_GetTableName(t *testing.T) {
	type fields struct {
		BlockHeight uint64
	}
	tests := []struct {
		name   string
		fields fields
		want   string
	}{
		// TODO: Add test cases.
		{
			name:   "save_points",
			fields: fields{},
			want:   "save_points",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			b := &SavePoint{
				BlockHeight: tt.fields.BlockHeight,
			}
			if got := b.GetTableName(); got != tt.want {
				t.Errorf("GetTableName() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestSavePoint_SQL(t *testing.T) {
	b := &SavePoint{
		BlockHeight: 0,
	}
	assert.True(t, len(b.GetCreateTableSql("sqlite")) > 0)
	assert.True(t, len(b.GetCreateTableSql("mysql")) > 0)

	assert.Equal(t, "save_points", b.GetTableName())

	str, interf := b.GetInsertSql("sqlite")
	assert.True(t, len(str) > 0)
	assert.NotNil(t, interf)

	str, interf = b.GetUpdateSql()
	assert.True(t, len(str) > 0)
	assert.NotNil(t, interf)

	str, interf = b.GetCountSql()
	assert.True(t, len(str) > 0)
	assert.NotNil(t, interf)

	str, interf = b.GetSaveSql("")
	assert.True(t, len(str) > 0)
	assert.NotNil(t, interf)
}
