/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package main

import (
	"log"

	"github.com/spf13/cobra"
	"yzchain.org/yzchain-cryptogen/command"
	"yzchain.org/yzchain-cryptogen/config"
)

func main() {
	mainCmd := &cobra.Command{
		Use: "yzchain-cryptogen",
		PersistentPreRun: func(cmd *cobra.Command, args []string) {
			config.LoadCryptoGenConfig(command.ConfigPath)

		},
	}
	mainFlags := mainCmd.PersistentFlags()
	mainFlags.StringVarP(&command.ConfigPath, "config", "c", "../config/crypto_config_template.yml", "specify config file path")
	mainFlags.StringVarP(&command.P11KeysPath, "pkcs11_keys", "p", "../config/pkcs11_keys.yml", "specify pkcs11 keys file path")

	mainCmd.AddCommand(command.ShowConfigCmd())
	mainCmd.AddCommand(command.GenerateCmd())
	mainCmd.AddCommand(command.ExtendCmd())

	if err := mainCmd.Execute(); err != nil {
		log.Fatalf("failed to execute, err = %s", err)
	}
}
