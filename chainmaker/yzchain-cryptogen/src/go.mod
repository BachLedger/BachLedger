module yzchain.org/yzchain-cryptogen

go 1.15

require (
	chainmaker.org/chainmaker/common/v2 v2.3.3
	github.com/mr-tron/base58 v1.2.0
	github.com/spf13/cobra v1.1.1
	github.com/spf13/viper v1.9.0
)

replace (
	chainmaker.org/chainmaker/common/v2 v2.3.3 => ../../chain-module/common
	github.com/spf13/afero => github.com/spf13/afero v1.5.1 //for go1.15 build
	github.com/spf13/viper => github.com/spf13/viper v1.7.1 //for go1.15 build
)
