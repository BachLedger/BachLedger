/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package utils

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestGetNodeUidFromAddr(t *testing.T) {
	addr := "/ip4/127.0.0.1/tcp/11301/p2p/QmcQHCuAXaFkbcsPUj7e37hXXfZ9DdN7bozseo5oX4qiC4"
	result, err := GetNodeUidFromAddr(addr)
	assert.Nil(t, err)
	t.Log(result)
	assert.True(t, strings.HasPrefix(result, "Qm"))
}
