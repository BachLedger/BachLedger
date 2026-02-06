/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package testframework

import (
	"testing"

	"github.com/stretchr/testify/require"
)

// TestInitConfigs
// config Path should not remove
func TestInitConfigs(t *testing.T) {
	configPath := "./config_tmp"
	err := initConfigs(configPath)
	//defer os.RemoveAll(configPath)
	require.Nil(t, err)
}
