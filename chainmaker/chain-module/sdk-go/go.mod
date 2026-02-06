module chainmaker.org/chainmaker/sdk-go/v2

go 1.16

require (
	chainmaker.org/chainmaker/common/v2 v2.3.2
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3
	chainmaker.org/chainmaker/protocol/v2 v2.3.3
	chainmaker.org/chainmaker/utils/v2 v2.3.3
	github.com/Rican7/retry v0.1.0
	github.com/go-sql-driver/mysql v1.7.0
	github.com/gogo/protobuf v1.3.2
	github.com/golang/mock v1.6.0
	github.com/golang/protobuf v1.5.2
	github.com/google/uuid v1.3.0
	github.com/gorilla/websocket v1.4.3-0.20220104015952-9111bb834a68
	github.com/hokaccha/go-prettyjson v0.0.0-20201222001619-a42f9ac2ec8e
	github.com/modern-go/reflect2 v1.0.2 // indirect
	github.com/spf13/viper v1.9.0
	github.com/stretchr/testify v1.7.0
	go.uber.org/zap v1.17.0
	google.golang.org/grpc v1.40.0
	gorm.io/driver/mysql v1.4.7
	gorm.io/gorm v1.24.6
)
replace (
	chainmaker.org/chainmaker/common/v2 v2.3.2 => ../common
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3 => ../pb-go
	chainmaker.org/chainmaker/protocol/v2 v2.3.3 => ../protocol
	chainmaker.org/chainmaker/utils/v2 v2.3.3 => ../utils

)