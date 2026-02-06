/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package sm4

import (
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
)

const msg = "js"

func TestSM4(t *testing.T) {
	key := make([]byte, 16)
	_, err := rand.Read(key)
	require.Nil(t, err)

	sm4 := SM4Key{Key: key}

	crypt, err := sm4.Encrypt([]byte(msg))
	require.Nil(t, err)

	fmt.Println("crypt data:", base64.StdEncoding.EncodeToString(crypt))

	decrypt, err := sm4.Decrypt(crypt)
	require.Nil(t, err)

	require.Equal(t, string(decrypt), msg)
}
