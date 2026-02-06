/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package pkcs11

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestGenerateBytesOTP(t *testing.T) {
	rBytes, err := GenerateBytesOTP(p11, -1)
	assert.NoError(t, err)
	assert.Equal(t, defaultRandomLen, len(rBytes))

	rBytes, err = GenerateBytesOTP(p11, 10)
	assert.NoError(t, err)
	assert.Equal(t, 10, len(rBytes))
}

func TestGenerateIntOTP(t *testing.T) {
	r, err := GenerateOTP(p11, -1)
	assert.NoError(t, err)
	t.Log(r)
	assert.Equal(t, defaultRandomLen, len(r))

	r, err = GenerateOTP(p11, 6)
	assert.NoError(t, err)
	t.Log(r)
	assert.Equal(t, 6, len(r))
}
