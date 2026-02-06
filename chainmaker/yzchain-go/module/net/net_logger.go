/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package net

import (
	"chainmaker.org/chainmaker/logger/v2"
	"chainmaker.org/chainmaker/protocol/v2"
)

var GlobalNetLogger protocol.Logger

func init() {
	GlobalNetLogger = logger.GetLogger(logger.MODULE_NET)
}
