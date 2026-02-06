package config

import (
	"io/ioutil"

	cmtls "chainmaker.org/chainmaker/common/v2/crypto/tls"
	cmx509 "chainmaker.org/chainmaker/common/v2/crypto/x509"
)

// GetConfig return a config for tls
func GetConfig(certFile, keyFile, caCertFile string, isServer bool) (*cmtls.Config, error) {
	sigCert, err := cmtls.LoadX509KeyPair(certFile, keyFile)
	if err != nil {
		return nil, err
	}

	// 信任的根证书
	certPool := cmx509.NewCertPool()
	cacert, err := ioutil.ReadFile(caCertFile)
	if err != nil {
		return nil, err
	}
	certPool.AppendCertsFromPEM(cacert)

	if isServer {
		return &cmtls.Config{
			Certificates: []cmtls.Certificate{sigCert},
			ClientCAs:    certPool,
		}, nil
	}
	return &cmtls.Config{
		Certificates: []cmtls.Certificate{sigCert},
		RootCAs:      certPool,
	}, nil
}

////GetGMConfig returns a config for GM signle cert tls
//func GetGMConfig(certFile, keyFile, caCertFile string, isServer bool) (*cmtls.Config, error) {
//	sigCert, err := cmtls.LoadX509KeyPair(certFile, keyFile)
//	if err != nil {
//		return nil, err
//	}
//
//	// 信任的根证书
//	certPool := cmx509.NewCertPool()
//	cacert, err := ioutil.ReadFile(caCertFile)
//	if err != nil {
//		return nil, err
//	}
//	certPool.AppendCertsFromPEM(cacert)
//
//	var config *cmtls.Config
//	if isServer {
//		config = &cmtls.Config{
//			GMSupport:    cmtls.NewGMSupport(),
//			Certificates: []cmtls.Certificate{sigCert},
//			ClientCAs:    certPool,
//		}
//	} else {
//		config = &cmtls.Config{
//			GMSupport:    cmtls.NewGMSupport(),
//			Certificates: []cmtls.Certificate{sigCert},
//			RootCAs:      certPool,
//		}
//	}
//	config.GMSupport.EncCertEnable = false
//	return config, nil
//}

// GetGMConfigForDoubleCert returns a config for GM double cert tls
func GetGMTLSConfig(certFile, keyFile, encCertFile, encKeyFile, caCertFile string, isServer bool) (*cmtls.Config, error) {
	sigCert, err := cmtls.LoadX509KeyPair(certFile, keyFile)
	if err != nil {
		return nil, err
	}
	encCert, err := cmtls.LoadX509KeyPair(encCertFile, encKeyFile)
	if err != nil {
		return nil, err
	}

	// 信任的根证书
	certPool := cmx509.NewCertPool()
	caCert, err := ioutil.ReadFile(caCertFile)
	if err != nil {
		return nil, err
	}
	certPool.AppendCertsFromPEM(caCert)

	if isServer {
		return &cmtls.Config{
			GMSupport:    cmtls.NewGMSupport(),
			Certificates: []cmtls.Certificate{sigCert, encCert},
			ClientCAs:    certPool,
		}, nil
	}
	return &cmtls.Config{
		GMSupport:    cmtls.NewGMSupport(),
		Certificates: []cmtls.Certificate{sigCert, encCert},
		RootCAs:      certPool,
	}, nil
}
