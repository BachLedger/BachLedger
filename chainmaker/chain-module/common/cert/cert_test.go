/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package cert

import (
	"errors"
	"os"
	"path/filepath"
	"strconv"
	"testing"

	"chainmaker.org/chainmaker/common/v2/crypto"
	"github.com/stretchr/testify/require"
)

const (
	c            = "CN"
	l            = "Beijing"
	p            = "Beijing"
	ou           = "chainmaker.org-OU"
	o            = "chainmaker.org-O"
	cn           = "jasonruan"
	expireYear   = 8
	testFilePath = "./testdata"
)

var (
	sans = []string{"127.0.0.1", "localhost", "chainmaker.org", "8.8.8.8"}
)

func TestCreatePrivKeyWithDiffKeyTypes(t *testing.T) {
	// 0 & 1 are symmetric, not supported
	// 10 and above are not supported now
	var tests = []struct {
		keyType int
		wantErr error
	}{
		{0, errors.New("generate key pair [AES] failed, wrong signature algorithm type")},
		{1, errors.New("generate key pair [SM4] failed, wrong signature algorithm type")},
		{2, nil},
		{3, nil},
		{4, nil},
		{5, nil},
		{6, nil},
		{7, nil},
		{8, nil},
		{9, nil},
		{10, nil},
		{11, errors.New("generate key pair [ECC_Ed25519] failed, unsupport signature algorithm")},
	}

	for i, tt := range tests {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			_, err := CreatePrivKey(crypto.KeyType(tt.keyType), "", "", true)
			require.Equal(t, err, tt.wantErr)
		})
	}
}

func TestCreatePrivKey(t *testing.T) {
	var err error
	testFileName := "test.key"

	_, err = os.Stat(filepath.Join(testFilePath, testFileName))
	require.True(t, os.IsNotExist(err))

	key, err := CreatePrivKey(crypto.ECC_NISTP256, testFilePath, testFileName, true)
	require.NotEmpty(t, key)
	require.NoError(t, err)

	_, err = os.Stat(filepath.Join(testFilePath, testFileName))
	require.NoError(t, err)

	err = os.Remove(filepath.Join(testFilePath, testFileName))
	require.NoError(t, err)
}

func TestCreateCACertificate(t *testing.T) {
	testFileName := "test.crt"
	cfg := &CACertificateConfig{}
	err := CreateCACertificate(cfg)
	require.Contains(t, err.Error(), "nil key material")

	key, err := CreatePrivKey(crypto.ECC_NISTP256, "", "", true)
	cfg = &CACertificateConfig{
		PrivKey: key,
	}
	err = CreateCACertificate(cfg)
	require.Contains(t, err.Error(), "unknown hash algorithm")

	cfg = &CACertificateConfig{
		PrivKey:  key,
		HashType: crypto.HASH_TYPE_SHA256,
	}

	err = CreateCACertificate(cfg)
	require.Contains(t, err.Error(), "mk cert dir failed")

	cfg = &CACertificateConfig{
		PrivKey:      key,
		HashType:     crypto.HASH_TYPE_SHA256,
		CertPath:     testFilePath,
		CertFileName: testFileName,
	}
	err = CreateCACertificate(cfg)
	require.NoError(t, err)

	cert, err := ParseCertificate(filepath.Join(testFilePath, testFileName))
	require.Equal(t, int(cert.PublicKeyAlgorithm), 3)
	require.Equal(t, int(cert.SignatureAlgorithm), 10)

	cfg = &CACertificateConfig{
		PrivKey:      key,
		HashType:     crypto.HASH_TYPE_SHA256,
		CertPath:     testFilePath,
		CertFileName: testFileName,
		Organization: "A",
	}
	err = CreateCACertificate(cfg)
	require.NoError(t, err)

	cert, err = ParseCertificate(filepath.Join(testFilePath, testFileName))
	require.Equal(t, cert.Subject.Organization[0], "A")
	require.Equal(t, cert.Subject.CommonName, defaultCommonName)

	err = os.Remove(filepath.Join(testFilePath, testFileName))
	require.NoError(t, err)
}

func TestIssueCertificate(t *testing.T) {
	//issueCertificate(t, crypto.SM2)
	//issueCertificate(t, crypto.RSA512)
	//issueCertificate(t, crypto.RSA1024)
	//issueCertificate(t, crypto.RSA2048)
	//issueCertificate(t, crypto.ECC_NISTP256)
	//issueCertificate(t, crypto.ECC_NISTP384)
	//issueCertificate(t, crypto.ECC_NISTP521)
}

//TODO
//func TestParseCertificateToString(t *testing.T) {
//	certStr, err := ParseCertificateToJson(filepath.Join(pathPrefix, "ecc_nistp384_issued.crt"))
//	require.Nil(t, err)
//	fmt.Println(certStr)
//
//	fmt.Println("\n\n===============================================================")
//
//	certStr, err = ParseCertificateToJson(filepath.Join(pathPrefix, "rsa2048_ca.crt"))
//	require.Nil(t, err)
//	fmt.Println(certStr)
//}
//
//func createCACertificate(t *testing.T, keyType crypto.KeyType) {
//	keyName, ok := crypto.KeyType2NameMap[keyType]
//	require.Equal(t, true, ok)
//	keyName = strings.ToLower(keyName)
//
//	privKey, err := CreatePrivKey(keyType, pathPrefix, keyName+"_ca.key")
//	require.Nil(t, err)
//
//	certCfg := &IssueCertificateConfig{
//		IssuerPrivKeyFilePath: filepath.Join(testFilePath, testIssuerPrivKeyFileName),
//		IssuerCertFilePath:    filepath.Join(testFilePath, testIssuerCertFileName),
//	}
//	err = IssueCertificate(certCfg)
//	require.Error(t, err)
//
//	subjectPrivKey, err := CreatePrivKey(crypto.ECC_NISTP256, testFilePath, testSubjectPrivKeyFIleName)
//	csrCfg := &CSRConfig{
//		PrivKey:     subjectPrivKey,
//		CsrPath:     testFilePath,
//		CsrFileName: testSubjectCsrFileName,
//	}
//	err = CreateCSR(csrCfg)
//	require.NoError(t, err)
//
//	certCfg.CsrFilePath = filepath.Join(testFilePath, testSubjectCsrFileName)
//	certCfg.CertPath = testFilePath
//	certCfg.CertFileName = testSubjectCertFileName
//	err = CreateCSR(csrCfg)
//	require.NoError(t, err)
//
//	certCfg.HashType = crypto.HASH_TYPE_SHA256
//	err = IssueCertificate(certCfg)
//	require.NoError(t, err)
//
//	err = os.RemoveAll(testFilePath)
//	require.NoError(t, err)
//}
