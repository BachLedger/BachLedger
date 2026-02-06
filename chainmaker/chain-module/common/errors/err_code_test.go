/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package errors

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestErrCode_String(t *testing.T) {
	require.Equal(t, int32(0), ERR_CODE_OK.Int())
	fmt.Println(ERR_CODE_OK)
	fmt.Println(ERR_CODE_SYSTEM_CONTRACT_PB_UNMARSHAL)
	fmt.Println(ERR_CODE_SYSTEM_CONTRACT_UNSUPPORT_CONTRACT_NAME)
	fmt.Println(ERR_CODE_SYSTEM_CONTRACT_UNSUPPORT_METHOD_NAME)
}
