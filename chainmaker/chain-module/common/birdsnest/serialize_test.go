/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package birdsnest

import (
	"fmt"
	"os"
	"testing"
)

// TODO 偶尔报错
func TestBirdsNestImpl_Deserialize(t *testing.T) {
	err := os.RemoveAll("./data")
	if err != nil {
		fmt.Println(err)
	}
	type fields struct {
		bn *BirdsNestImpl
	}
	tests := []struct {
		name    string
		fields  fields
		wantErr error
	}{
		{
			name: "异常流 修改配置",
			fields: fields{bn: func() *BirdsNestImpl {
				tbn := getTBN(TestDir+"_deserialize101", t)
				return tbn
			}()},
			wantErr: ErrCannotModifyTheNestConfiguration,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			for i := 0; i <= int(tt.fields.bn.config.Length); i++ {
				err := tt.fields.bn.Serialize(i)
				if err != nil {
					t.Errorf("Serialize() error = %v", err)
				}
			}
			tt.fields.bn.config.Snapshot.Timed.Interval = 20
			if err := tt.fields.bn.Deserialize(); err != tt.wantErr {
				t.Errorf("ExtensionDeserialize() error = %v, wantErr %v", err, tt.wantErr)
			} else {
				t.Logf("ExtensionDeserialize() error = %v", err)
			}
		})
	}
}

func TestBirdsNestImpl_Start(t *testing.T) {
}

func TestBirdsNestImpl_serializeExit(t *testing.T) {
}

func TestBirdsNestImpl_serializeHeight(t *testing.T) {
}

func TestBirdsNestImpl_serializeMonitor(t *testing.T) {
}

func TestBirdsNestImpl_serializeTimed(t *testing.T) {
}

func TestBirdsNestImpl_timedAndExitSerialize(t *testing.T) {
}

// nolint gocyclo
func TestSerialize(t *testing.T) {
	_ = os.RemoveAll("./data")
	filePath := TestDir + "_deserialize102"
	tbn := getTBN(filePath, t)
	var keys [][]Key
	// Serialize
	for i := 0; i < len(tbn.filters); i++ {
		var keyArray []Key
		for j := 0; j < 10; j++ {
			//key1, _ := ToTimestampKey(fmt.Sprintf("key_%d_%d", i, j))
			key1 := GetTimestampKey()
			keyArray = append(keyArray, key1)
			_, _ = tbn.filters[i].Add(key1)
		}
		keys = append(keys, keyArray)
	}
	tbn.currentIndex = 3
	for i := 0; i < len(tbn.filters); i++ {
		err := tbn.Serialize(i)
		if err != nil {
			t.Error(err.Error())
		}
	}
	// Deserialize
	tbn2 := getTBN(filePath, t)
	err := tbn2.Deserialize()
	if err != nil {
		t.Error(err.Error())
	}
	if len(tbn.filters) != len(tbn2.filters) {
		t.Error()
	}
	if tbn2.currentIndex != tbn.currentIndex {
		t.Error()
	}
	if tbn2.height != tbn.height {
		t.Error()
	}
	for i := 0; i < len(tbn2.filters); i++ {
		for j := 0; j < 10; j++ {
			//key1, _ := ToTimestampKey(fmt.Sprintf("key_%d_%d", i, j))
			key1 := keys[i][j]
			contain, errC := tbn2.filters[i].Contains(key1)
			if !contain {
				t.Error(key1.String())
			}
			if errC != nil {
				t.Error(key1.String())
			}
		}
	}
	_ = os.RemoveAll("./data")
}
