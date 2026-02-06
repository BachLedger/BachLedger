/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package logger

import (
	"testing"

	"github.com/spf13/viper"
	"github.com/stretchr/testify/assert"
)

const LOG_YML = "./testdata/log.yml"

type MainConfig struct {
	Log LogConfig
}

func TestParseLogYml(t *testing.T) {
	path := LOG_YML
	cmViper := viper.New()
	cmViper.SetConfigFile(path)
	err := cmViper.ReadInConfig()
	assert.NoError(t, err)
	config := &MainConfig{}
	err = cmViper.Unmarshal(config)
	assert.Nil(t, err)
	t.Logf("%#v", config.Log.SystemLog.Kafka)
	for k, v := range config.Log.ModuleLog {
		t.Logf("module:%s, config: %#v", k, v)
	}
}
func TestKafkaLogConfig_GetTopic(t *testing.T) {
	klconfig := &KafkaLogConfig{
		Servers:      nil,
		Compression:  0,
		Topic:        "T",
		TopicMapping: map[string]string{"A": "ATopic", "B": "BTopic", "C": "CTopic"},
		KafkaVersion: "",
		Sasl:         nil,
	}
	topic := klconfig.GetTopic("x")
	assert.Equal(t, "T", topic)
	topic = klconfig.GetTopic("B")
	assert.Equal(t, "BTopic", topic)
}
func TestLogNodeConfig_GetFilePath(t *testing.T) {
	cfg := LogNodeConfig{FilePath: "/root/a/log.txt", LogByChain: true}
	assert.Equal(t, "/root/a/chain1/log.txt", cfg.GetFilePath("chain1"))
	assert.Equal(t, "/root/a/log.txt", cfg.GetFilePath(""))
	cfg.LogByChain = false
	assert.Equal(t, "/root/a/log.txt", cfg.GetFilePath("chain1"))

}
