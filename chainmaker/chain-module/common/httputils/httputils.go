/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package httputils

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net/http"
	"time"
)

const (
	// 默认超时时间，单位：s
	DEFAULT_TIMEOUT = 5
	MAX_RETRY_CNT   = 3
)

func GET(customClient *http.Client, url string) (respData []byte, err error) {
	var (
		resp *http.Response
	)

	var client = customClient
	if customClient == nil {
		client = &http.Client{
			Timeout: DEFAULT_TIMEOUT * time.Second,
		}
	}

	resp, err = client.Get(url)
	if err != nil {
		return nil, fmt.Errorf("http GET failed, %s", err.Error())
	}
	defer resp.Body.Close()

	respData, err = ioutil.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read body failed, %s", err)
	}

	return
}

func POST(customClient *http.Client, url string, reqObj interface{}) (respData []byte, err error) {

	var (
		data []byte
		req  *http.Request
		resp *http.Response
	)

	data, err = json.Marshal(reqObj)
	if err != nil {
		return nil, fmt.Errorf("json marshal failed, %s", err.Error())
	}

	req, err = http.NewRequest(
		"POST",
		url,
		bytes.NewBuffer(data))

	if err != nil {
		return nil, fmt.Errorf("new request failed, %s", err.Error())
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("charset", "UTF-8")

	var client = customClient
	if customClient == nil {
		client = &http.Client{
			Timeout: DEFAULT_TIMEOUT * time.Second,
		}
	}

	cnt := 0
	for {
		resp, err = client.Do(req)
		if err != nil {
			cnt++
			if cnt <= MAX_RETRY_CNT {
				//log.Warnf("post failed, [cur:%d|max:%d] meet err: %s, and try again",
				//	cnt, MAX_RETRY_CNT, err.Error())
				time.Sleep(200 * time.Millisecond)
				continue
			}

			return nil, fmt.Errorf("post failed, meet too many err: %s", err.Error())
		}

		break
	}

	defer resp.Body.Close()

	respData, err = ioutil.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read body failed, %s", err)
	}

	return
}
