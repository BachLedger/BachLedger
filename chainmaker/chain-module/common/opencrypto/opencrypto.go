/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package opencrypto

type Engine string

// supported engine
const (
	GmSSL     Engine = "gmssl"
	TencentSM Engine = "tencentsm"
	TjfocGM   Engine = "tjfoc"
)

func ToEngineType(engine string) Engine {
	return Engine(engine)
}
