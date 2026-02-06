/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package errors

type Error interface {
	Error() string  // error information
	ErrorCode() int // error code
}
