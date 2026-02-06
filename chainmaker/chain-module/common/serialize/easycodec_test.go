/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

/*
go test -v -run ^TestUnmarshal$
go test -v -run ^TestField$
go test -v -run ^TestEasyCodec$
*/
package serialize

import (
	"bytes"
	"fmt"
	"io/ioutil"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
)

// nolint: varcheck
var ecHeader = []byte{99, 109, 101, 99, 118, 49, 46, 48, 255, 255, 255, 255, 255, 255, 255, 255}

const (
	strVal = "chainmaker长安链!@#$%^&*()_+-={}|:?><"
)

// go test -v -run ^TestEasyCodec$
func TestEasyCodec(t *testing.T) {
	fmt.Println("ecMagicNum", ecMagicNum)
	fmt.Println("ecVersion", ecVersion)
	fmt.Println("ecReserved", ecReserved)
}

// go test -v -run ^TestField$
func TestField(t *testing.T) {
	keyType := "keyBytes"
	t.Run(keyType, func(t *testing.T) {
		ec := NewEasyCodec()
		ec.AddBytes("keyBytes", []byte(strVal))
		data := ec.Marshal()
		origin := []byte{1, 0, 0, 0, 1, 0, 0, 0, 8, 0, 0, 0, 107, 101, 121, 66, 121, 116, 101, 115, 2, 0, 0, 0, 40, 0, 0, 0, 99, 104, 97, 105, 110, 109, 97, 107, 101, 114, 233, 149, 191, 229, 174, 137, 233, 147, 190, 33, 64, 35, 36, 37, 94, 38, 42, 40, 41, 95, 43, 45, 61, 123, 125, 124, 58, 63, 62, 60}
		assert.True(t, bytes.Equal(data, origin), "keyBytes marshal result err")
		fmt.Println("keyBytes pass")
	})

	keyType = "keyInt32"
	t.Run(keyType, func(t *testing.T) {
		ec := NewEasyCodec()
		ec.AddInt32("keyInt32", 123456789)
		data := ec.Marshal()
		origin := []byte{1, 0, 0, 0, 1, 0, 0, 0, 8, 0, 0, 0, 107, 101, 121, 73, 110, 116, 51, 50, 0, 0, 0, 0, 4, 0, 0, 0, 21, 205, 91, 7}
		assert.True(t, bytes.Equal(data, origin), "keyInt32 marshal result err")
		fmt.Println("keyInt32 pass")
	})

	keyType = "keyStr"
	t.Run(keyType, func(t *testing.T) {
		ec := NewEasyCodec()
		ec.AddString("keyBytes", strVal)
		data := ec.Marshal()
		origin := []byte{1, 0, 0, 0, 1, 0, 0, 0, 8, 0, 0, 0, 107, 101, 121, 66, 121, 116, 101, 115, 1, 0, 0, 0, 40, 0, 0, 0, 99, 104, 97, 105, 110, 109, 97, 107, 101, 114, 233, 149, 191, 229, 174, 137, 233, 147, 190, 33, 64, 35, 36, 37, 94, 38, 42, 40, 41, 95, 43, 45, 61, 123, 125, 124, 58, 63, 62, 60}
		assert.True(t, bytes.Equal(data, origin), "keyStr marshal result err")
		fmt.Println("keyStr pass")
	})

	keyType = "all"
	t.Run(keyType, func(t *testing.T) {
		ec := NewEasyCodec()
		ec.AddBytes("keyBytes", []byte(strVal))
		ec.AddInt32("keyInt32", 123456789)
		ec.AddString("keyStr", strVal)
		data := ec.Marshal()
		origin := []byte{3, 0, 0, 0, 1, 0, 0, 0, 8, 0, 0, 0, 107, 101, 121, 66, 121, 116, 101, 115, 2, 0, 0, 0, 40, 0, 0, 0, 99, 104, 97, 105, 110, 109, 97, 107, 101, 114, 233, 149, 191, 229, 174, 137, 233, 147, 190, 33, 64, 35, 36, 37, 94, 38, 42, 40, 41, 95, 43, 45, 61, 123, 125, 124, 58, 63, 62, 60, 1, 0, 0, 0, 8, 0, 0, 0, 107, 101, 121, 73, 110, 116, 51, 50, 0, 0, 0, 0, 4, 0, 0, 0, 21, 205, 91, 7, 1, 0, 0, 0, 6, 0, 0, 0, 107, 101, 121, 83, 116, 114, 1, 0, 0, 0, 40, 0, 0, 0, 99, 104, 97, 105, 110, 109, 97, 107, 101, 114, 233, 149, 191, 229, 174, 137, 233, 147, 190, 33, 64, 35, 36, 37, 94, 38, 42, 40, 41, 95, 43, 45, 61, 123, 125, 124, 58, 63, 62, 60}
		assert.True(t, bytes.Equal(data, origin), "all marshal result err")
		fmt.Println("all pass")
	})
}

//TODO 这个测试用例断言是错误的。
// go test -v -run ^TestUnmarshal$
//func TestUnmarshal(t *testing.T) {
//	origin := []byte{3, 0, 0, 0, 1, 0, 0, 0, 8, 0, 0, 0, 107, 101, 121, 66, 121, 116, 101, 115, 2, 0, 0, 0, 40, 0, 0, 0, 99, 104, 97, 105, 110, 109, 97, 107, 101, 114, 233, 149, 191, 229, 174, 137, 233, 147, 190, 33, 64, 35, 36, 37, 94, 38, 42, 40, 41, 95, 43, 45, 61, 123, 125, 124, 58, 63, 62, 60, 1, 0, 0, 0, 8, 0, 0, 0, 107, 101, 121, 73, 110, 116, 51, 50, 0, 0, 0, 0, 4, 0, 0, 0, 21, 205, 91, 7, 1, 0, 0, 0, 6, 0, 0, 0, 107, 101, 121, 83, 116, 114, 1, 0, 0, 0, 40, 0, 0, 0, 99, 104, 97, 105, 110, 109, 97, 107, 101, 114, 233, 149, 191, 229, 174, 137, 233, 147, 190, 33, 64, 35, 36, 37, 94, 38, 42, 40, 41, 95, 43, 45, 61, 123, 125, 124, 58, 63, 62, 60}
//	ec := NewEasyCodecWithBytes(origin)
//	data := ec.ToJson()
//	require.Equal(t, "{\"keyBytes\":\"chainmaker长安链!@#$%^&*()_+-={}|:?><\",\"keyInt32\":123456789,\"keyStr\":\"chainmaker长安链!@#$%^&*()_+-={}|:?><\"}", data)
//	fmt.Println("Unmarshal toJson success")
//	keyInt32, _ := ec.GetInt32("keyInt32")
//	keyBytes, _ := ec.GetBytes("keyBytes")
//	keyStr, _ := ec.GetString("keyStr")
//	assert.Equal(t, keyInt32, int32(123456789))
//	assert.True(t, bytes.Equal(keyBytes, []byte(strVal)))
//	assert.Equal(t, keyBytes, []byte(strVal))
//	assert.Equal(t, keyStr, strVal)
//
//	// object field
//	ec = NewEasyCodec()
//	ec.AddBytes("keyBytes", []byte(strVal))
//	ec.AddInt32("keyInt32", 123456789)
//	ec.AddString("keyStr", strVal)
//	data = ec.ToJson()
//	assert.Equal(t, "{\"keyBytes\":\"chainmaker长安链!@#$%^&*()_+-={}|:?><\",\"keyInt32\":123456789,\"keyStr\":\"chainmaker长安链!@#$%^&*()_+-={}|:?><\"}", data)
//
//	// map
//	jsonSr := "{\"keyStr1\":\"val str\",\"keyStr2\":\"val str\",\"keyStr3\":\"val str\",\"keyStr4\":\"val str\",\"keyStr5\":\"val str\",\"keyStr6\":\"val str\"}"
//	for i := 0; i < 1000; i++ {
//		mapVal := make(map[string][]byte)
//		mapVal["keyStr1"] = []byte("val str")
//		mapVal["keyStr5"] = []byte("val str")
//		mapVal["keyStr6"] = []byte("val str")
//		mapVal["keyStr2"] = []byte("val str")
//		mapVal["keyStr3"] = []byte("val str")
//		mapVal["keyStr4"] = []byte("val str")
//		ec = NewEasyCodecWithMap(mapVal)
//		data = ec.ToJson()
//		assert.Equal(t, jsonSr, data)
//	}
//	fmt.Println("map sort marshal success")
//}

// nolint: gosec
func writeToFile(bytes []byte) {
	path := "C:\\Users\\51578\\Desktop\\临时\\go-java-byte\\tmpGo"
	ioutil.WriteFile(path, bytes, 0666)
}

func readFromFile() {
	path := "C:\\Users\\51578\\Desktop\\临时\\go-java-byte\\tmpJava"
	f, err := os.Open(path)
	if err != nil {
		panic(err)
	}
	bytes, err := ioutil.ReadAll(f)
	fmt.Println(bytes)
	//json := EasyCodecItemToJsonStr(EasyUnmarshal(bytes))
	//fmt.Println("unmarshal from java file", json)

}
