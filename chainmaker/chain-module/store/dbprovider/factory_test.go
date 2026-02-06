/*
 *  Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
 *
 *  SPDX-License-Identifier: Apache-2.0
 */

package dbprovider

import (
	"fmt"
	"os"
	"path/filepath"
	"reflect"
	"testing"
	"time"

	"chainmaker.org/chainmaker/common/v2/crypto"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/protocol/v2/test"
	"chainmaker.org/chainmaker/store/v2/conf"
)

var chainId = "ut1"

func TestDBFactory_NewKvDB(t *testing.T) {
	tPath := filepath.Join(os.TempDir(), fmt.Sprintf("%d", time.Now().Nanosecond()))
	lvlConfig := make(map[string]interface{})
	lvlConfig["store_path"] = tPath
	lvlDBConfig := &conf.DbConfig{
		Provider:      conf.DbconfigProviderLeveldb,
		LevelDbConfig: lvlConfig,
	}

	badgerConfig := make(map[string]interface{})
	badgerConfig["store_path"] = tPath
	badgerDBConfig := &conf.DbConfig{
		Provider:      conf.DbconfigProviderBadgerdb,
		LevelDbConfig: badgerConfig,
	}

	type args struct {
		chainId      string
		providerName string
		dbFolder     string
		config       map[string]interface{}
		logger       protocol.Logger
		encryptor    crypto.SymmetricKey
	}
	tests := []struct {
		name    string
		args    args
		want    protocol.DBHandle
		wantErr bool
	}{
		// TODO: Add test cases.
		{
			name: lvlDBConfig.Provider,
			args: args{
				chainId:      chainId,
				providerName: lvlDBConfig.Provider,
				dbFolder:     lvlDBConfig.Provider,
				config:       lvlDBConfig.LevelDbConfig,
				logger:       &test.GoLogger{},
				encryptor:    nil,
			},
			want:    nil,
			wantErr: false,
		},
		{
			name: badgerDBConfig.Provider,
			args: args{
				chainId:      chainId,
				providerName: badgerDBConfig.Provider,
				dbFolder:     badgerDBConfig.Provider,
				config:       badgerDBConfig.BadgerDbConfig,
				logger:       &test.GoLogger{},
				encryptor:    nil,
			},
			want:    nil,
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			f := &DBFactory{}
			_, err := f.NewKvDB(tt.args.chainId, tt.args.providerName, tt.args.dbFolder, tt.args.config, tt.args.logger, tt.args.encryptor)
			if (err != nil) != tt.wantErr {
				t.Errorf("NewKvDB() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
		})
	}
}

func TestNewDBFactory(t *testing.T) {
	var tests []struct {
		name string
		want *DBFactory
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := NewDBFactory(); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("NewDBFactory() = %v, want %v", got, tt.want)
			}
		})
	}
}
