/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package logger

import (
	"testing"

	"github.com/spf13/viper"
	"github.com/stretchr/testify/assert"
)

func TestKafkaLog(t *testing.T) {
	t.SkipNow()
	path := LOG_YML
	cmViper := viper.New()
	cmViper.SetConfigFile(path)
	err := cmViper.ReadInConfig()
	assert.NoError(t, err)
	config := &MainConfig{}
	err = cmViper.Unmarshal(config)
	assert.Nil(t, err)
	SetLogConfig(&config.Log)
	log := GetLoggerByChain("store", "chain1")
	log.Debug("Test Debug 1")
	log.Info("Test Info 2")
	log.Warn("Test Warning 3")
	log.Error("Test Error 4")
	log.Close()
}
