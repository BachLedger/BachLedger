module chainmaker.org/chainmaker/utils/v2

go 1.16

require (
	chainmaker.org/chainmaker/common/v2 v2.3.2
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3
	chainmaker.org/chainmaker/protocol/v2 v2.3.3
	github.com/gogo/protobuf v1.3.2
	github.com/golang/mock v1.6.0
	github.com/google/uuid v1.1.2
	github.com/kr/text v0.2.0 // indirect
	github.com/mr-tron/base58 v1.2.0
	github.com/pingcap/parser v0.0.0-20200623164729-3a18f1e5dceb
	github.com/pingcap/tidb v1.1.0-beta.0.20200630082100-328b6d0a955c
	github.com/pingcap/tipb v0.0.0-20210425040103-dc47a87b52aa // indirect
	github.com/remyoudompheng/bigfft v0.0.0-20200410134404-eec4a21b6bb0 // indirect
	github.com/shopspring/decimal v1.3.1
	github.com/stretchr/testify v1.7.0
	github.com/tjfoc/gmsm v1.4.1
)
replace (
	chainmaker.org/chainmaker/common/v2 v2.3.2 => ../common
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3 => ../pb-go
	chainmaker.org/chainmaker/protocol/v2 v2.3.3 => ../protocol
)