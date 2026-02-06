/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package http

import (
	"io/ioutil"
	"log"
	"net/http"
	"path/filepath"
	"testing"
	"time"

	cmtls "chainmaker.org/chainmaker/common/v2/crypto/tls"

	"chainmaker.org/chainmaker/common/v2/crypto/tls/config"

	"github.com/stretchr/testify/assert"
)

var (
	caCert, _ = filepath.Abs("../testdata/certs/CA.crt")

	ssCert, _ = filepath.Abs("../testdata/certs/SS.crt")
	ssKey, _  = filepath.Abs("../testdata/certs/SS.key")
	seCert, _ = filepath.Abs("../testdata/certs/SE.crt")
	seKey, _  = filepath.Abs("../testdata/certs/SE.key")

	csCert, _ = filepath.Abs("../testdata/certs/CS.crt")
	csKey, _  = filepath.Abs("../testdata/certs/CS.key")
	ceCert, _ = filepath.Abs("../testdata/certs/CE.crt")
	ceKey, _  = filepath.Abs("../testdata/certs/CE.key")
)

var (
	msg = []byte("hello world")
)

func sayHello(w http.ResponseWriter, r *http.Request) {
	w.Write(msg)
}

// TestHttpsServer test ecc certificate
func TestHttpsServer(t *testing.T) {
	finish := make(chan bool, 1)
	go func() {
		err := ListenAndServeTLS(
			":13001",
			"../testdata/server.crt",
			"../testdata/server.key",
			"../testdata/ca.crt",
			http.HandlerFunc(sayHello),
		)
		assert.NoError(t, err)
	}()

	time.Sleep(time.Millisecond * 100)
	go func() {
		cfg, err := config.GetConfig(
			"../testdata/client.crt",
			"../testdata/client.key",
			"../testdata/ca.crt",
			false,
		)
		assert.NoError(t, err)

		client := NewClient(cfg)
		resp, err := client.Get("https://localhost:13001")
		assert.NoError(t, err)

		buf, err := ioutil.ReadAll(resp.Body)
		assert.NoError(t, err)
		assert.Equal(t, msg, buf)
		log.Println("receive from server: " + string(buf))
		finish <- true
	}()

	<-finish
}

func testHttpsServerRun(t *testing.T, addr string) {
	cfg, err := config.GetConfig(ssCert, ssKey, caCert, true)
	assert.NoError(t, err)
	cfg.ClientAuth = cmtls.RequireAndVerifyClientCert

	ln, err := cmtls.Listen("tcp", addr, cfg)
	assert.NoError(t, err)
	defer ln.Close()

	mux := http.NewServeMux()
	mux.HandleFunc("/", sayHello)

	err = http.Serve(ln, mux)
	assert.NoError(t, err)
}

func testHttpsClientRun(t *testing.T, url string, finish chan bool) {
	cfg, err := config.GetConfig(csCert, csKey, caCert, false)
	assert.NoError(t, err)
	cfg.ServerName = "chainmaker.org"

	client := NewClient(cfg)
	resp, err := client.Get(url)
	assert.NoError(t, err)

	buf, err := ioutil.ReadAll(resp.Body)
	assert.NoError(t, err)
	assert.Equal(t, msg, buf)
	log.Println("receive from server: " + string(buf))

	finish <- true
}

func testHttpsServerRun_GM1(t *testing.T, addr string) {
	cfg, err := config.GetGMTLSConfig(ssCert, ssKey, seCert, seKey, caCert, true)
	assert.NoError(t, err)
	cfg.ClientAuth = cmtls.RequireAndVerifyClientCert

	ln, err := cmtls.Listen("tcp", addr, cfg)
	assert.NoError(t, err)

	mux := http.NewServeMux()
	mux.HandleFunc("/", sayHello)

	err = http.Serve(ln, mux)
	assert.NoError(t, err)
}

func testHttpsClientRun_GM1(t *testing.T, url string, finish chan bool) {
	cfg, err := config.GetGMTLSConfig(csCert, csKey, ceCert, ceKey, caCert, false)
	assert.NoError(t, err)
	cfg.ServerName = "chainmaker.org"
	cfg.InsecureSkipVerify = false

	client := NewClient(cfg)
	resp, err := client.Get(url)
	assert.NoError(t, err)

	buf, err := ioutil.ReadAll(resp.Body)
	assert.NoError(t, err)
	assert.Equal(t, msg, buf)
	log.Println("receive from server: " + string(buf))

	finish <- true
}
func TestGMHttps(t *testing.T) {
	finish := make(chan bool, 2)
	go testHttpsServerRun(t, ":13002")
	time.Sleep(time.Second * 2) //wait for server start
	go testHttpsClientRun(t, "https://localhost:13002", finish)

	go testHttpsServerRun_GM1(t, ":13003")
	time.Sleep(time.Second * 2) //wait for server start
	go testHttpsClientRun_GM1(t, "https://localhost:13003", finish)

	for i := 0; i < len(finish); i++ {
		<-finish
	}
}

//
//func checkSigCert(raw []byte) (*cmx509.Certificate, bool, error) {
//	cert, err := cmx509.ParseCertificate(raw)
//	if err != nil {
//		return nil, false, fmt.Errorf("parse certificate failed, %s", err.Error())
//	}
//	if (cert.KeyUsage != 0) && (cert.KeyUsage&x509.KeyUsageDigitalSignature) != 0 {
//		return cert, true, nil
//	}
//	return cert, false, nil
//}
//
//func TestHttpsServerRun_GM2(t *testing.T) {
//	cfg, err := config.GetGMTLSConfig(ssCert, ssKey, seCert, seKey, caCert, true)
//	assert.NoError(t, err)
//	cfg.ClientAuth = cmtls.RequireAndVerifyClientCert
//
//	i := 0
//	cfg.VerifyPeerCertificate = func(rawCerts [][]byte, _ [][]*cmx509.Certificate) error {
//		i++
//		for _, raw := range rawCerts {
//			cert, isSignCert, err := checkSigCert(raw)
//			fmt.Println(cert.Subject.String())
//			fmt.Println(isSignCert)
//			if err != nil {
//				return err
//			}
//		}
//		fmt.Printf("invoke %d times\n", i)
//		return nil
//	}
//
//	ln, err := cmtls.Listen("tcp", ":8080", cfg)
//	assert.NoError(t, err)
//
//	mux := http.NewServeMux()
//	mux.HandleFunc("/", sayHello)
//
//	err = http.Serve(ln, mux)
//	assert.NoError(t, err)
//}
//
//func TestHttpsClientRun_GM2(t *testing.T) {
//	cfg, err := config.GetGMTLSConfig(csCert, csKey, ceCert, ceKey, caCert, false)
//
//	assert.NoError(t, err)
//	//cfg.ServerName = "chainmaker.org"
//	cfg.InsecureSkipVerify = false
//
//	client := NewClient(cfg)
//	resp, err := client.Get("https://localhost:8080")
//	assert.NoError(t, err)
//
//	buf, err := ioutil.ReadAll(resp.Body)
//	assert.NoError(t, err)
//	assert.Equal(t, msg, buf)
//	log.Println("receive from server: " + string(buf))
//}
