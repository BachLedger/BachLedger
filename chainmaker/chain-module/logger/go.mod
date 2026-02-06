module chainmaker.org/chainmaker/logger/v2

go 1.16

require (
	chainmaker.org/chainmaker/common/v2 v2.3.2
	github.com/Shopify/sarama v1.33.0
	github.com/mitchellh/mapstructure v1.5.0 // indirect
	github.com/spf13/viper v1.9.0
	github.com/stretchr/testify v1.7.0
	go.uber.org/zap v1.17.0
)


replace (
	chainmaker.org/chainmaker/common/v2 v2.3.2 => ../common
)