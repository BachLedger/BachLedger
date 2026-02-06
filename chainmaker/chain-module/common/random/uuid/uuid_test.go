/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package uuid

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestGetUUID(t *testing.T) {
	uuid1 := GetUUID()
	t.Log("UUID: ", uuid1)
	assert.Equal(t, 32, len(uuid1))

	uuid2 := GetUUID()
	//t.Log("UUID: ", uuid2)
	assert.NotEqual(t, uuid1, uuid2)
}

func TestGetUUIDWithSeed(t *testing.T) {
	uuid1 := GetUUIDWithSeed(int64(-1))
	t.Log("UUID: ", uuid1)
	uuid2 := GetUUIDWithSeed(int64(-1))
	//t.Log("UUID: ", uuid2)
	assert.Equal(t, uuid1, uuid2)
}
