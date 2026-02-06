/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package http

import (
	"net"
	"net/http"

	"chainmaker.org/chainmaker/common/v2/crypto/tls/config"

	cmtls "chainmaker.org/chainmaker/common/v2/crypto/tls"
)

// NewTLSListener returns a listener with tls.Config, which support gmtls and tls
func NewTLSListener(inner net.Listener, config *cmtls.Config) net.Listener {
	return cmtls.NewListener(inner, config)
}

// ListenAndServeTLS only supprot gmtls single cert mode. For gmtls, use NewTLSListener
func ListenAndServeTLS(addr, certFile, keyFile, caCertFile string, handler http.Handler) error {
	cfg, err := config.GetConfig(certFile, keyFile, caCertFile, true)
	if err != nil {
		return err
	}
	ln, err := cmtls.Listen("tcp", addr, cfg)
	if err != nil {
		return err
	}
	defer ln.Close()
	return http.Serve(ln, handler)
}
