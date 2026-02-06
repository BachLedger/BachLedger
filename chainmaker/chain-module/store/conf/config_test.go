/*
 * Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 */

package conf

import (
	"testing"

	"github.com/mitchellh/mapstructure"
	"github.com/stretchr/testify/assert"
)

func TestNewHistoryDbConfig(t *testing.T) {
	storageConfig := make(map[string]interface{})
	storageConfig["store_path"] = "dbPath1"
	storageConfig["blockdb_config"] = map[string]interface{}{
		"provider": "badgerdb",
		"badgerdb_config": map[string]interface{}{
			"store_path": "badgerPath1",
		},
	}
	storageConfig["statedb_config"] = map[string]interface{}{
		"provider": "badgerdb",
		"badgerdb_config": map[string]interface{}{
			"store_path": "badgerPath2",
		},
	}
	storageConfig["historydb_config"] = map[string]interface{}{
		"provider":                 "badgerdb",
		"disable_contract_history": true,
		"badgerdb_config": map[string]interface{}{
			"store_path": "badgerPath3",
		},
	}
	config := StorageConfig{}
	err := mapstructure.Decode(storageConfig, &config)
	assert.Nil(t, err)
	t.Logf("blockdb:%#v", config.BlockDbConfig)
	t.Logf("statedb:%#v", config.StateDbConfig)
	t.Logf("historydb:%#v", config.HistoryDbConfig)
}

func TestNewStorageConfig(t *testing.T) {
	storageConfig := make(map[string]interface{})
	storageConfig["store_path"] = "./mypath1"
	storageConfig["blockdb_config"] = map[string]interface{}{
		"provider":        DbconfigProviderBadgerdb,
		"badgerdb_config": map[string]interface{}{},
	}
	storageConfig["statedb_config"] = map[string]interface{}{
		"provider":       DbconfigProviderLeveldb,
		"leveldb_config": map[string]interface{}{},
	}
	storageConfig["resultdb_config"] = map[string]interface{}{
		"provider":      DbconfigProviderTikvdb,
		"tikvdb_config": map[string]interface{}{},
	}
	storageConfig["historydb_config"] = map[string]interface{}{
		"provider":     DbconfigProviderSql,
		"sqldb_config": map[string]interface{}{},
	}
	storageConfig["txexistdb_config"] = map[string]interface{}{
		"provider":     DbconfigProviderSqlKV,
		"sqldb_config": map[string]interface{}{},
	}
	storageConfig["disable_contract_eventdb"] = true
	storageConfig["contract_eventdb_config"] = map[string]interface{}{
		"provider":     DbconfigProviderSqlKV,
		"sqldb_config": map[string]interface{}{},
	}

	config, err := NewStorageConfig(storageConfig)
	assert.Nil(t, err)
	t.Log(config.WriteBatchSize)

	assert.Equal(t, DbconfigProviderBadgerdb, config.GetBlockDbConfig().Provider)
	assert.Equal(t, DbconfigProviderLeveldb, config.GetStateDbConfig().Provider)
	assert.Equal(t, DbconfigProviderSql, config.GetHistoryDbConfig().Provider)
	assert.Equal(t, DbconfigProviderTikvdb, config.GetResultDbConfig().Provider)
	assert.Equal(t, DbconfigProviderSqlKV, config.GetContractEventDbConfig().Provider)
	assert.Equal(t, DbconfigProviderSqlKV, config.GetTxExistDbConfig().Provider)
	assert.Equal(t, DbconfigProviderLeveldb, config.GetDefaultDBConfig().Provider)

	assert.Equal(t, 0, len(config.BlockDbConfig.GetDbConfig()))
	assert.Equal(t, 0, len(config.StateDbConfig.GetDbConfig()))
	assert.Equal(t, 0, len(config.HistoryDbConfig.GetDbConfig()))
	assert.Equal(t, 0, len(config.ResultDbConfig.GetDbConfig()))
	assert.Equal(t, 0, len(config.ContractEventDbConfig.GetDbConfig()))

	storageConfig["contract_eventdb_config"] = map[string]interface{}{
		"provider":     DbconfigProviderMemdb,
		"sqldb_config": map[string]interface{}{},
	}
	config, err = NewStorageConfig(storageConfig)
	assert.Nil(t, err)
	assert.Equal(t, 0, len(config.ContractEventDbConfig.GetDbConfig()))
	assert.False(t, NewHistoryDbConfig(config.GetDefaultDBConfig()).DisableKeyHistory)

	assert.Equal(t, 4, config.GetActiveDBCount())
	assert.True(t, config.BlockDbConfig.IsKVDB())
	assert.True(t, config.HistoryDbConfig.IsSqlDB())
}
