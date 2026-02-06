/*
 * Copyright (C) BABEC. All rights reserved.
 * Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

package certmgr220

import (
	"fmt"
	"testing"

	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"github.com/golang/protobuf/proto"

	"github.com/stretchr/testify/assert"
)

func Test_AddAlias(t *testing.T) {
	mgrRuntime, txSimContext, fn := initEnv(t)
	defer fn()
	// normal
	params := make(map[string][]byte)
	params[paramNameAlias] = []byte("alias01")
	result, err := mgrRuntime.AddAlias(txSimContext, params)
	assert.Nil(t, err)
	assert.Equal(t, "ok", string(result))
	fmt.Printf("add alias success. ")

	// no param
	_, err = mgrRuntime.AddAlias(txSimContext, nil)
	assert.NotNil(t, err)

	// bad param
	params[paramNameAlias] = []byte("alias01$*^&*(")
	_, err = mgrRuntime.AddAlias(txSimContext, params)
	assert.NotNil(t, err)

	// to lang alias
	params[paramNameAlias] = []byte("rtyuighjksdfisdfsuf65s76f89dsfsdf878s9df87sd6f78sd8fs7d76fs8df8s76df78sdf86sd7f78dsfs8df78")
	_, err = mgrRuntime.AddAlias(txSimContext, params)
	assert.NotNil(t, err)
}

func Test_UpdateAlias(t *testing.T) {
	mgrRuntime, txSimContext, fn := initEnv(t)
	defer fn()

	// no params
	result, err := mgrRuntime.UpdateAlias(txSimContext, nil)
	assert.NotNil(t, err)
	assert.Nil(t, result)

	// bad params
	params := make(map[string][]byte)
	params[paramNameAlias] = []byte("alias02%^&")
	params[paramNameCert] = getOrg1Client1Signer().MemberInfo
	result, err = mgrRuntime.UpdateAlias(txSimContext, params)
	assert.NotNil(t, err)

	// no alias add
	params[paramNameAlias] = []byte("alias02")
	params[paramNameCert] = getOrg1Client1Signer().MemberInfo
	result, err = mgrRuntime.UpdateAlias(txSimContext, params)
	assert.NotNil(t, err)

	// add alias
	params2 := make(map[string][]byte)
	params2[paramNameAlias] = []byte("alias02")
	result2, err2 := mgrRuntime.AddAlias(txSimContext, params2)
	assert.Nil(t, err2)
	assert.Equal(t, "ok", string(result2))
	fmt.Printf("add alias success. ")

	// bad cert
	params[paramNameAlias] = []byte("alias02")
	params[paramNameCert] = []byte("getOrg1Client1Signer().MemberInfo")
	result, err = mgrRuntime.UpdateAlias(txSimContext, params)
	assert.NotNil(t, err)

	// bad org
	params[paramNameAlias] = []byte("alias02")
	params[paramNameCert] = getOrg2Client1Signer().MemberInfo
	result, err = mgrRuntime.UpdateAlias(txSimContext, params)
	assert.NotNil(t, err)

	// same cert
	params[paramNameAlias] = []byte("alias02")
	params[paramNameCert] = getOrg1Client1Signer().MemberInfo
	result, err = mgrRuntime.UpdateAlias(txSimContext, params)
	assert.NotNil(t, err)

	// normal
	params[paramNameAlias] = []byte("alias02")
	params[paramNameCert] = getOrg1Admin1Signer().MemberInfo
	result, err = mgrRuntime.UpdateAlias(txSimContext, params)
	assert.Nil(t, err)
	assert.Equal(t, "ok", string(result2))

	for i := 0; i < 11; i++ {
		params[paramNameCert] = getOrg1Client1Signer().MemberInfo
		result, err = mgrRuntime.UpdateAlias(txSimContext, params)
		params[paramNameCert] = getOrg1Admin1Signer().MemberInfo
		result, err = mgrRuntime.UpdateAlias(txSimContext, params)
	}
	params[paramNameAliases] = []byte("alias02")
	result, err = mgrRuntime.QueryAlias(txSimContext, params)
	assert.Nil(t, err)
	a := commonPb.AliasInfos{}
	err = proto.Unmarshal(result, &a)
	assert.Nil(t, err)
	assert.Equal(t, len(a.AliasInfos[0].HisCerts), maxHisCertsLen)
}

func Test_DeleteAlias(t *testing.T) {
	mgrRuntime, txSimContext, fn := initEnv(t)
	defer fn()

	// no params
	result, err := mgrRuntime.DeleteAlias(txSimContext, nil)
	assert.NotNil(t, err)
	assert.Nil(t, result)

	// no alias add
	params := make(map[string][]byte)
	params[paramNameAliases] = []byte("alias03")
	result, err = mgrRuntime.DeleteAlias(txSimContext, params)
	assert.NotNil(t, err)
	assert.Nil(t, result)

	// add alias
	params2 := make(map[string][]byte)
	params2[paramNameAlias] = []byte("alias03")
	result2, err2 := mgrRuntime.AddAlias(txSimContext, params2)
	assert.Nil(t, err2)
	assert.Equal(t, "ok", string(result2))
	fmt.Printf("add alias success. ")

	// bad org
	mgrRuntime2, txSimContext2, fn2 := initEnvSender2(t)
	defer fn2()
	params[paramNameAliases] = []byte("alias03")
	params[paramNameCert] = getOrg2Client1Signer().MemberInfo
	result, err = mgrRuntime2.DeleteAlias(txSimContext2, params)
	assert.NotNil(t, err)

	// normal
	params[paramNameAliases] = []byte("alias03")
	params[paramNameCert] = getOrg1Client1Signer().MemberInfo
	result, err = mgrRuntime.DeleteAlias(txSimContext, params)
	assert.Nil(t, err)
	assert.Equal(t, "ok", string(result2))

	// repeat
	params[paramNameAliases] = []byte("alias03")
	params[paramNameCert] = getOrg1Client1Signer().MemberInfo
	result, err = mgrRuntime.DeleteAlias(txSimContext, params)
	assert.NotNil(t, err)
}

func Test_QueryAlias(t *testing.T) {
	mgrRuntime, txSimContext, fn := initEnv(t)
	defer fn()

	// no params
	result, err := mgrRuntime.QueryAlias(txSimContext, nil)
	assert.NotNil(t, err)

	// not found
	params := make(map[string][]byte)
	params[paramNameAliases] = []byte("alias04")
	result, err = mgrRuntime.QueryAlias(txSimContext, params)
	assert.NotNil(t, err)

	// add alias
	params[paramNameAlias] = []byte("alias04")
	result2, err2 := mgrRuntime.AddAlias(txSimContext, params)
	assert.Nil(t, err2)
	assert.Equal(t, "ok", string(result2))
	fmt.Printf("add alias success. ")

	// normal found
	params[paramNameAliases] = []byte("alias04")
	result, err = mgrRuntime.QueryAlias(txSimContext, params)
	assert.Nil(t, err)
	a := commonPb.AliasInfos{}
	err = proto.Unmarshal(result, &a)
	assert.Nil(t, err)
	assert.Equal(t, a.AliasInfos[0].Alias, "alias04")
}
