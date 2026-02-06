/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package alert

//const (
//	alarmCenterUrl = "http://127.0.0.1:12091"
//	webhook        = "https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=95788de9-a86a-4fb3-98b3-46419ab73352"
//)

//TODO
//func TestSendUserAlert(t *testing.T) {
//	alertInfo := UserAlertModel{
//		Level:      USER_ALERT_LEVEL_WARN,
//		Subject:    "subject001",
//		Content:    "content001",
//		Receivers:  []string{"jasonruan@tencent.com", "jsrzx@qq.com"},
//		Webhooks:   []string{webhook},
//		AttachName: "",
//		AttachFile: "",
//	}
//
//	client := &AlertClient{
//		alarmCenterUrl: alarmCenterUrl,
//		alertInfo:      alertInfo,
//	}
//
//	err := client.SendUserAlert()
//	require.Nil(t, err)
//
//	err = client.SendUserAlertSync()
//	require.Nil(t, err)
//}
//
//func TestSendUserAlertWithAttach(t *testing.T) {
//	attachName := "sdk_alert_test.go"
//	attachFilePath := "./sdk_alert_test.go"
//	file, err := ioutil.ReadFile(attachFilePath)
//	require.Nil(t, err)
//	attachFile := base64.StdEncoding.EncodeToString(file)
//
//	alertInfo := UserAlertModel{
//		Level:      USER_ALERT_LEVEL_WARN,
//		Subject:    "subject002",
//		Content:    "content002",
//		Receivers:  []string{"jasonruan@tencent.com", "jsrzx@qq.com"},
//		Webhooks:   nil,
//		AttachName: attachName,
//		AttachFile: attachFile,
//	}
//
//	client := &AlertClient{
//		alarmCenterUrl: alarmCenterUrl,
//		alertInfo:      alertInfo,
//	}
//
//	err = client.SendUserAlertSync()
//	require.Nil(t, err)
//}
