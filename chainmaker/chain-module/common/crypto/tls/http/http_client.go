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

	cmtls "chainmaker.org/chainmaker/common/v2/crypto/tls"
)

func NewClient(config *cmtls.Config) *http.Client {
	if config == nil {
		panic("config must not be nil")
	}
	return &http.Client{
		Transport: &http.Transport{
			DialTLSContext: func(ctx context.Context, network, addr string) (net.Conn, error) {
				dialer := &net.Dialer{}
				conn, err := cmtls.DialWithDialer(dialer, network, addr, config)
				if err != nil {
					return nil, err
				}

				return conn, nil
			},
			Dial: (&net.Dialer{
				Timeout:   30 * time.Second,
				KeepAlive: 30 * time.Second,
			}).Dial,
			ForceAttemptHTTP2:     true,
			MaxIdleConns:          100,
			TLSHandshakeTimeout:   10 * time.Second,
			IdleConnTimeout:       90 * time.Second,
			ExpectContinueTimeout: 1 * time.Second,
		},
	}
}
