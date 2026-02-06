module chainmaker.org/chainmaker/vm-evm/v2

go 1.15

require (
	chainmaker.org/chainmaker/common/v2 v2.3.2
	chainmaker.org/chainmaker/logger/v2 v2.3.0
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3
	chainmaker.org/chainmaker/protocol/v2 v2.3.3
	chainmaker.org/chainmaker/utils/v2 v2.3.2
	github.com/go-sql-driver/mysql v1.6.0 // indirect
	github.com/golang/mock v1.6.0
	github.com/google/uuid v1.3.0 // indirect
	github.com/modern-go/reflect2 v1.0.2 // indirect
	github.com/pingcap/errors v0.11.5-0.20201126102027-b0a155152ca3 // indirect
	github.com/pingcap/log v0.0.0-20201112100606-8f1e84a3abc8 // indirect
	github.com/prometheus/procfs v0.6.0 // indirect
	github.com/shirou/gopsutil v3.21.4-0.20210419000835-c7a38de76ee5+incompatible // indirect
	github.com/tjfoc/gmsm v1.4.1
	github.com/tklauser/go-sysconf v0.3.10 // indirect
	golang.org/x/crypto v0.0.0-20220214200702-86341886e292
	google.golang.org/grpc v1.47.0 // indirect

)

replace (
	chainmaker.org/chainmaker/common/v2 v2.3.2 => ../common
	chainmaker.org/chainmaker/logger/v2 v2.3.0 => ../logger
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3 => ../pb-go
	chainmaker.org/chainmaker/protocol/v2 v2.3.3 => ../protocol
	chainmaker.org/chainmaker/utils/v2 v2.3.2 => ../utils
	google.golang.org/grpc => google.golang.org/grpc v1.26.0 //with test error google.golang.org/grpc/naming: module google.golang.org/grpc@latest found (v1.47.0), but does not contain package google.golang.org/grpc/naming
)
