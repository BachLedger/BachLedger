package scheduler

import (
	"chainmaker.org/chainmaker/protocol/v2"
)

func IsOptimizeChargeGasEnabled(chainConf protocol.ChainConf) bool {
	enableGas := false
	enableOptimizeChargeGas := false
	if chainConf.ChainConfig() != nil && chainConf.ChainConfig().AccountConfig != nil {
		enableGas = chainConf.ChainConfig().AccountConfig.EnableGas
	}

	if chainConf.ChainConfig() != nil && chainConf.ChainConfig().Core != nil {
		enableOptimizeChargeGas = chainConf.ChainConfig().Core.EnableOptimizeChargeGas
	}
	return enableGas && enableOptimizeChargeGas
}
