/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package alert

import (
	"errors"
	"fmt"
	"net/http"

	"chainmaker.org/chainmaker/common/v2/httputils"
	"chainmaker.org/chainmaker/common/v2/json"
)

const (
	USER_ALERT_LEVEL_INFO = "info"
	USER_ALERT_LEVEL_WARN = "warning"
	USER_ALERT_URI        = "/cm-alert-api/v1/alert/user"
)

type ErrCode uint32

const (
	ErrCodeOK ErrCode = 0
)

type BaseResp struct {
	RetCode uint32 `json:"retcode"`
	RetMsg  string `json:"retmsg"`
}

type UserAlertModel struct {
	// (1)必填字段
	// 告警级别，支持：USER_ALERT_LEVEL_INFO、USER_ALERT_LEVEL_WARN
	// 仅部分告警触达方式下有效
	Level string `json:"level"`
	// 告警标题
	Subject string `json:"subject"`
	// 告警内容
	Content string `json:"content"`

	// (2)选填字段
	// 告警邮件接收者邮箱，设置该字段后，将发送邮件
	Receivers []string `json:"receivers"`
	// 告警webhooks，当前仅支持企业微信群，设置该字段，将发送到指定企业微信群
	Webhooks []string `json:"webhooks"`
	// 附件名(邮件)
	AttachName string `json:"attach_name"`
	// 附件base64编码内容（邮件），大小限制在配置文件attach_file_size指定
	AttachFile string `json:"attach_file"`
}

type AlertClient struct {
	//自定义Client
	customClient *http.Client

	// 告警中心地址
	alarmCenterUrl string

	// 告警信息
	alertInfo UserAlertModel
}

func (client AlertClient) SendUserAlertSync() error {
	if err := client.checkParams(); err != nil {
		return err
	}

	resp, err := httputils.POST(client.customClient, client.alarmCenterUrl+USER_ALERT_URI, client.alertInfo)
	if err != nil {
		return err
	}

	var alertResp BaseResp
	err = json.Unmarshal(resp, &alertResp)
	if err != nil {
		return fmt.Errorf("unmarshal resp failed, %s", err)
	}

	if alertResp.RetCode != uint32(ErrCodeOK) {
		return fmt.Errorf("recv err, errCode:%d, errMsg:%s", alertResp.RetCode, alertResp.RetMsg)
	}

	return nil
}

func (client AlertClient) SendUserAlert() error {

	if err := client.checkParams(); err != nil {
		return err
	}

	go func() {
		_, _ = httputils.POST(client.customClient, client.alarmCenterUrl+USER_ALERT_URI, client.alertInfo)
	}()

	return nil
}

func (client AlertClient) checkParams() error {
	if client.alarmCenterUrl == "" {
		return errors.New("alert center url is empty")
	}

	if client.alertInfo.Subject == "" {
		return errors.New("alert subject is empty")
	}

	if client.alertInfo.Content == "" {
		return errors.New("alert content is empty")
	}

	if client.alertInfo.Level != "" {
		if client.alertInfo.Level != USER_ALERT_LEVEL_INFO && client.alertInfo.Level != USER_ALERT_LEVEL_WARN {
			return errors.New("alert level is invalid")
		}
	} else {
		client.alertInfo.Level = USER_ALERT_LEVEL_WARN
	}

	if len(client.alertInfo.Receivers) == 0 && len(client.alertInfo.Webhooks) == 0 {
		return errors.New("all receivers and webhooks are empty")
	}

	return nil
}
