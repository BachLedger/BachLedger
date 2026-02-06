/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package engine

import (
	"fmt"

	"chainmaker.org/chainmaker/common/v2/opencrypto"
)

var (
	CryptoEngine = opencrypto.TjfocGM

	//IsTls this flag is used to skip p2p tls, because p2p tls use the tjfoc, should be refactor! TODO
	IsTls = false
)

func InitCryptoEngine(eng string, tls bool) {
	CryptoEngine = opencrypto.ToEngineType(eng)
	switch CryptoEngine {
	case opencrypto.GmSSL, opencrypto.TjfocGM, opencrypto.TencentSM:
		fmt.Printf("using crypto CryptoEngine = %s\n", eng)
	default:
		CryptoEngine = opencrypto.TjfocGM
		fmt.Printf("using default crypto CryptoEngine = %s\n", string(opencrypto.TjfocGM))
	}
	IsTls = tls
}
