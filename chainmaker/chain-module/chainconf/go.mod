module chainmaker.org/chainmaker/chainconf/v2

go 1.16

require (
	chainmaker.org/chainmaker/common/v2 v2.3.2
	chainmaker.org/chainmaker/logger/v2 v2.3.0
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3
	chainmaker.org/chainmaker/protocol/v2 v2.3.3
	chainmaker.org/chainmaker/utils/v2 v2.3.2
	github.com/gogo/protobuf v1.3.2
	github.com/golang/groupcache v0.0.0-20200121045136-8c9f03a8e57e
	github.com/golang/mock v1.6.0
	github.com/modern-go/reflect2 v1.0.2 // indirect
	github.com/spf13/viper v1.9.0
	github.com/stretchr/testify v1.7.0
	github.com/test-go/testify v1.1.4
)

replace (
	chainmaker.org/chainmaker/common/v2 v2.3.2 => ../common
	chainmaker.org/chainmaker/logger/v2 v2.3.0 => ../logger
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3 => ../pb-go
	chainmaker.org/chainmaker/protocol/v2 v2.3.3 => ../protocol
	chainmaker.org/chainmaker/utils/v2 v2.3.2 => ../utils
)
