/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package http

import (
	"context"
	"net"
	"net/http"
	"time"

	"github.com/gorilla/websocket"

	cmtls "chainmaker.org/chainmaker/common/v2/crypto/tls"
)

func NewDial(config *cmtls.Config) *websocket.Dialer {
	if config == nil {
		panic("config must not be nil")
	}
	return &websocket.Dialer{
		NetDialTLSContext: func(ctx context.Context, network, addr string) (net.Conn, error) {
			dialer := &net.Dialer{}
			conn, err := cmtls.DialWithDialer(dialer, network, addr, config)
			if err != nil {
				return nil, err
			}
			return conn, nil
		},
		ReadBufferSize:   10,
		WriteBufferSize:  10,
		HandshakeTimeout: 45 * time.Second,
		Proxy:            http.ProxyFromEnvironment,
	}
}
