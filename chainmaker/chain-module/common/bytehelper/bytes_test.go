/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package bytehelper

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestBytesToString(t *testing.T) {
	b := []byte("Hello World")
	str := BytesToString(b)
	assert.Equal(t, string(b), str)
}
func TestStringToBytes(t *testing.T) {
	str := "Hello World"
	b := StringToBytes(str)
	assert.Equal(t, []byte(str), b)
}
