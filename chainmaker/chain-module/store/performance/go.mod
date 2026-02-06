module chainmaker.org/chainmaker/store/performance

go 1.15

require (
	chainmaker.org/chainmaker/localconf/v2 v2.3.2
	chainmaker.org/chainmaker/logger/v2 v2.3.0
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3
	chainmaker.org/chainmaker/protocol/v2 v2.3.3
	chainmaker.org/chainmaker/store/v2 v2.1.1
	chainmaker.org/chainmaker/utils/v2 v2.3.2
	github.com/spf13/cobra v1.1.1
	github.com/studyzy/sqlparse v0.0.0-20210520090832-d40c792e1576 // indirect
)

replace (
	chainmaker.org/chainmaker/store/v2 => ../
	chainmaker.org/chainmaker/localconf/v2 v2.3.2 => ../../localconf
	chainmaker.org/chainmaker/logger/v2 v2.3.0 => ../../logger
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3 => ../../pb-go
	chainmaker.org/chainmaker/protocol/v2 v2.3.3 => ../../protocol
	chainmaker.org/chainmaker/utils/v2 v2.3.2 => ../../utils

	google.golang.org/grpc => google.golang.org/grpc v1.26.0
)
