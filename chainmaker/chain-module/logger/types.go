/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package logger

import "path/filepath"

// LogConfig the config of log module
type LogConfig struct {
	ConfigFile string                   `mapstructure:"config_file"`
	SystemLog  LogNodeConfig            `mapstructure:"system"`
	BriefLog   LogNodeConfig            `mapstructure:"brief"`
	EventLog   LogNodeConfig            `mapstructure:"event"`
	ModuleLog  map[string]LogNodeConfig `mapstructure:"module"`
}

// GetConfigByModuleName 获得某个模块的Log配置
// @param pureName
// @return LogNodeConfig
func (c *LogConfig) GetConfigByModuleName(pureName string) LogNodeConfig {
	if config, ok := c.ModuleLog[pureName]; ok {
		return config
	}
	return c.SystemLog
}

// LogNodeConfig the log config of node
type LogNodeConfig struct {
	LogLevelDefault string            `mapstructure:"log_level_default"`
	LogLevels       map[string]string `mapstructure:"log_levels"`
	FilePath        string            `mapstructure:"file_path"`
	MaxAge          int               `mapstructure:"max_age"`
	RotationTime    int               `mapstructure:"rotation_time"`
	RotationSize    int64             `mapstructure:"rotation_size"`
	LogInConsole    bool              `mapstructure:"log_in_console"`
	LogByChain      bool              `mapstructure:"log_by_chain"` //不同的链ID，存储在不同的日志文件夹中
	JsonFormat      bool              `mapstructure:"json_format"`
	ShowColor       bool              `mapstructure:"show_color"`
	StackTraceLevel string            `mapstructure:"stack_trace_level"`
	Kafka           *KafkaLogConfig   `mapstructure:"kafka"`
}

// GetFilePath calculate log file path by chainId and config
// @param chainId
// @return string new file path
func (cfg LogNodeConfig) GetFilePath(chainId string) string {
	if !cfg.LogByChain || len(chainId) == 0 {
		return cfg.FilePath
	}
	return filepath.Join(filepath.Dir(cfg.FilePath), chainId, filepath.Base(cfg.FilePath))
}

// KafkaLogConfig Kafka记录日志时的配置
type KafkaLogConfig struct {
	Servers []string `mapstructure:"servers"`
	// 0: None, 1: Gzip
	Compression int `mapstructure:"compression"`
	//默认的Kafka Topic
	Topic string `mapstructure:"topic"`
	// key: chainId value: topic
	TopicMapping map[string]string `mapstructure:"topic_mapping"`
	KafkaVersion string            `mapstructure:"kafka_version"`
	Sasl         *KafkaSaslConfig  `mapstructure:"sasl"`
}

// GetTopic 根据链ID，获得配置的Kafka的主题
// @param chainId
// @return string
func (c *KafkaLogConfig) GetTopic(chainId string) string {
	if topic, found := c.TopicMapping[chainId]; found {
		return topic
	}
	return c.Topic
}

// KafkaSaslConfig Kafka需要安全认证时的配置
type KafkaSaslConfig struct {
	Enable   bool   `mapstructure:"enable"`
	UserName string `mapstructure:"username"`
	Password string `mapstructure:"password"`
	// 默认是PLAIN
	Mechanism string `mapstructure:"mechanism"`
	Version   int    `mapstructure:"version"`
}
