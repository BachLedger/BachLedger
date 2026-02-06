/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package cmtlssupport

import (
	"crypto/rand"
	"crypto/x509"
	"errors"

	"math/big"
	"time"

	"chainmaker.org/chainmaker/common/v2/crypto"
	cmTls "chainmaker.org/chainmaker/common/v2/crypto/tls"
	cmx509 "chainmaker.org/chainmaker/common/v2/crypto/x509"
	"chainmaker.org/chainmaker/common/v2/helper"
)

const (
	certValidityPeriod = 100 * 365 * 24 * time.Hour // ~100 years
)

// NewTlsConfigWithCertMode create a new tls config with tls certificates for tls handshake.
func NewTlsConfigWithCertMode(
	certificates []cmTls.Certificate,
	certValidator *CertValidator,
) (*cmTls.Config, error) {
	if certValidator.pkMode {
		return nil, errors.New("cert validator in public key mode, but tls config with cert mode creating")
	}
	tlsConfig := &cmTls.Config{
		Certificates:          certificates,
		InsecureSkipVerify:    true,
		ClientAuth:            cmTls.RequireAnyClientCert,
		VerifyPeerCertificate: certValidator.VerifyPeerCertificateFunc(),
	}
	//len(certificates) == 2 means enc cert is set, use gmtls
	if len(certificates) == 2 {
		tlsConfig.GMSupport = cmTls.NewGMSupport()
	}
	return tlsConfig, nil
}

// GetCertAndPeerIdWithKeyPair will create a tls cert with x509 key pair and load the peer id from cert.
func GetCertAndPeerIdWithKeyPair(certPEMBlock []byte, keyPEMBlock []byte) (*cmTls.Certificate, string, error) {
	certificate, err := cmTls.X509KeyPair(certPEMBlock, keyPEMBlock)
	if err != nil {
		return nil, "", err
	}
	peerID, err2 := helper.GetLibp2pPeerIdFromCert(certPEMBlock)
	if err2 != nil {
		return nil, "", err2
	}
	return &certificate, peerID, nil
}

// PrivateKeyToCertificate create a certificate simply with a private key.
func PrivateKeyToCertificate(privateKey crypto.PrivateKey) (*cmTls.Certificate, error) {
	sn, err := rand.Int(rand.Reader, big.NewInt(1<<62))
	if err != nil {
		return nil, err
	}
	tmpl := &x509.Certificate{
		SerialNumber: sn,
		NotBefore:    time.Time{},
		NotAfter:     time.Now().Add(certValidityPeriod),
	}
	certDER, err := cmx509.CreateCertificate(rand.Reader, tmpl, tmpl,
		privateKey.PublicKey().ToStandardKey(), privateKey.ToStandardKey())
	if err != nil {
		return nil, err
	}
	return &cmTls.Certificate{
		Certificate: [][]byte{certDER},
		PrivateKey:  privateKey.ToStandardKey(),
	}, nil
}
