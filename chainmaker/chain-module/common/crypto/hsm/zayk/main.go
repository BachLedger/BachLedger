/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package main

/**
  This is a demo which implements hsm.IHSMAdapter.
*/
import (
	"errors"
	"fmt"
)

//go build -buildmode=plugin -o plugin1.so plugin1.go

// Adapter variable must be defined
// nolint
var Adapter hsmAdapter

// adapter must implement IHSMAdapter interface
type hsmAdapter struct {
}

// key prefix for zayk
var (
	sm2priv_prefix = "ZAYK_SM2_PRI"
	sm2pub_prefix  = "ZAYK_SM2_PUB"
)

func (a hsmAdapter) PKCS11_GetSM2KeyId(keyIdex int, isPrivate bool) (string, error) {
	if isPrivate {
		return fmt.Sprintf("%s%d", sm2priv_prefix, keyIdex), nil
	}
	return fmt.Sprintf("%s%d", sm2pub_prefix, keyIdex), nil
}

func (a hsmAdapter) PKCS11_GetRSAKeyId(keyIdex int, isPrivate bool) (string, error) {
	return "", errors.New("not implemented")
}

func (a hsmAdapter) PKCS11_GetECCKeyId(keyIdex int, isPrivate bool) (string, error) {
	return "", errors.New("not implemented")
}

func (a hsmAdapter) PKCS11_GetSM4KeyId(keyIdex int) (string, error) {
	return "", errors.New("not implemented")
}

func (a hsmAdapter) PKCS11_GetAESKeyId(keyIdex int) (string, error) {
	return "", errors.New("not implemented")
}

func (a hsmAdapter) PKCS11_GetSM3SM2CKM() uint {
	return 0
}

func (a hsmAdapter) SDF_GetSM2KeyAccessRight(keyIdex int) (newKeyIdex int, need bool) {
	return keyIdex, false
}

func (a hsmAdapter) SDF_GetSM4KeyAccessRight(keyIdex int) (newKeyIdex int, need bool) {
	return keyIdex, false
}

func (a hsmAdapter) SDF_GetRSAKeyAccessRight(keyIdex int) (newKeyIdex int, need bool) {
	return keyIdex, false
}

func (a hsmAdapter) SDF_GetAESKeyAccessRight(keyIdex int) (newKeyIdex int, need bool) {
	return keyIdex, false
}

func main() {}
