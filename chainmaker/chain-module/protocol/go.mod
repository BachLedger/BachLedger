module chainmaker.org/chainmaker/protocol/v2

go 1.16

require (
	chainmaker.org/chainmaker/common/v2 v2.3.2
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3
	github.com/golang/mock v1.6.0
)

replace (
	chainmaker.org/chainmaker/common/v2 v2.3.2 => ../common
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3 => ../pb-go
)
