/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package main

import (
	"strings"

	"chainmaker.org/chainmaker-go/tools/yzc/address"
	"chainmaker.org/chainmaker-go/tools/yzc/archive"
	"chainmaker.org/chainmaker-go/tools/yzc/cert"
	"chainmaker.org/chainmaker-go/tools/yzc/client"
	commandutil "chainmaker.org/chainmaker-go/tools/yzc/command_util"
	"chainmaker.org/chainmaker-go/tools/yzc/console"
	"chainmaker.org/chainmaker-go/tools/yzc/key"
	"chainmaker.org/chainmaker-go/tools/yzc/parallel"
	"chainmaker.org/chainmaker-go/tools/yzc/payload"
	"chainmaker.org/chainmaker-go/tools/yzc/query"
	"chainmaker.org/chainmaker-go/tools/yzc/txpool"
	"chainmaker.org/chainmaker-go/tools/yzc/version"
	"github.com/spf13/cobra"
)

func main() {
	mainCmd := &cobra.Command{
		Use:   "cmc",
		Short: "ChainMaker CLI",
		Long: strings.TrimSpace(`Command line interface for interacting with ChainMaker daemon.
For detailed logs, please see ./sdk.log
`),
	}

	mainCmd.AddCommand(key.KeyCMD())
	mainCmd.AddCommand(cert.CertCMD())
	mainCmd.AddCommand(client.ClientCMD())
	mainCmd.AddCommand(archive.NewArchiveCMD())
	mainCmd.AddCommand(query.NewQueryOnChainCMD())
	mainCmd.AddCommand(payload.NewPayloadCMD())
	mainCmd.AddCommand(console.NewConsoleCMD(mainCmd))

	//mainCmd.AddCommand(tee.NewTeeCMD())
	//mainCmd.AddCommand(pubkey.NewPubkeyCMD())
	mainCmd.AddCommand(parallel.ParallelCMD())
	mainCmd.AddCommand(address.NewAddressCMD())
	//mainCmd.AddCommand(gas.NewGasManageCMD())
	mainCmd.AddCommand(txpool.NewTxPoolCMD())
	mainCmd.AddCommand(version.VersionCMD())
	mainCmd.AddCommand(commandutil.NewUtilCMD())
	// 后续改成go-sdk
	//mainCmd.AddCommand(payload.PayloadCMD())
	//mainCmd.AddCommand(log.LogCMD())

	mainCmd.Execute()

}
