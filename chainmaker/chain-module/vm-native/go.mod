module chainmaker.org/chainmaker/vm-native/v2

go 1.16

require (
	chainmaker.org/chainmaker/chainconf/v2 v2.3.2
	chainmaker.org/chainmaker/common/v2 v2.3.2
	chainmaker.org/chainmaker/localconf/v2 v2.3.2
	chainmaker.org/chainmaker/logger/v2 v2.3.0
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3
	chainmaker.org/chainmaker/protocol/v2 v2.3.3
	chainmaker.org/chainmaker/utils/v2 v2.3.3
	github.com/gogo/protobuf v1.3.2
	github.com/golang/mock v1.6.0
	github.com/golang/protobuf v1.5.2
	github.com/google/uuid v1.1.2
	github.com/mr-tron/base58 v1.2.0
	github.com/pingcap/goleveldb v0.0.0-20191226122134-f82aafb29989
	github.com/stretchr/testify v1.7.0
)

replace (
	chainmaker.org/chainmaker/chainconf/v2 v2.3.2 => ../chainconf
	chainmaker.org/chainmaker/common/v2 v2.3.2 => ../common
	chainmaker.org/chainmaker/localconf/v2 v2.3.2 => ../localconf
	chainmaker.org/chainmaker/logger/v2 v2.3.0 => ../logger
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3 => ../pb-go
	chainmaker.org/chainmaker/protocol/v2 v2.3.3 => ../protocol
	chainmaker.org/chainmaker/utils/v2 v2.3.3 => ../utils
)
