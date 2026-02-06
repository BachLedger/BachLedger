module chainmaker.org/chainmaker/vm-engine/v2/vm_mgr

go 1.16

require (
	chainmaker.org/chainmaker/common/v2 v2.3.2
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3
	chainmaker.org/chainmaker/protocol/v2 v2.3.3
	github.com/emirpasic/gods v1.12.0
	github.com/gogo/protobuf v1.3.2
	github.com/golang/mock v1.6.0
	github.com/spf13/viper v1.9.0
	github.com/stretchr/testify v1.7.1
	go.uber.org/atomic v1.7.0
	go.uber.org/zap v1.19.1
	google.golang.org/grpc v1.47.0
)

replace (
	chainmaker.org/chainmaker/common/v2 v2.3.2 => ../../common
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3 => ../../pb-go
	chainmaker.org/chainmaker/protocol/v2 v2.3.3 => ../../protocol

)
