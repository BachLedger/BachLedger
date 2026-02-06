module chainmaker.org/chainmaker/consensus-utils/v2

go 1.15

require (
	chainmaker.org/chainmaker/common/v2 v2.3.2
	chainmaker.org/chainmaker/localconf/v2 v2.3.2
	chainmaker.org/chainmaker/logger/v2 v2.3.0
	chainmaker.org/chainmaker/net-common v1.3.1
	chainmaker.org/chainmaker/net-libp2p v1.2.3
	chainmaker.org/chainmaker/net-liquid v1.1.1
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3
	chainmaker.org/chainmaker/protocol/v2 v2.3.3
	chainmaker.org/chainmaker/utils/v2 v2.3.2
	github.com/gogo/protobuf v1.3.2
	github.com/golang/mock v1.6.0
	github.com/modern-go/reflect2 v1.0.2 // indirect
	github.com/prometheus/procfs v0.6.0 // indirect
	github.com/stretchr/testify v1.8.0
)

replace (
	chainmaker.org/chainmaker/common/v2 v2.3.2 => ../common
	chainmaker.org/chainmaker/localconf/v2 v2.3.2 => ../localconf
	chainmaker.org/chainmaker/logger/v2 v2.3.0 => ../logger
	chainmaker.org/chainmaker/net-common v1.3.1 => ../net-common
	chainmaker.org/chainmaker/net-libp2p v1.2.3 => ../net-libp2p
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3 => ../pb-go
	chainmaker.org/chainmaker/protocol/v2 v2.3.3 => ../protocol
	chainmaker.org/chainmaker/utils/v2 v2.3.2 => ../utils
	github.com/libp2p/go-libp2p-core => chainmaker.org/chainmaker/libp2p-core v1.0.0
	github.com/linvon/cuckoo-filter => chainmaker.org/third_party/cuckoo-filter v1.0.0
	github.com/lucas-clemente/quic-go v0.26.0 => chainmaker.org/third_party/quic-go v1.0.0
	github.com/marten-seemann/qtls-go1-15 => chainmaker.org/third_party/qtls-go1-15 v1.0.0
	github.com/marten-seemann/qtls-go1-16 => chainmaker.org/third_party/qtls-go1-16 v1.0.0
	github.com/marten-seemann/qtls-go1-17 => chainmaker.org/third_party/qtls-go1-17 v1.0.0
	github.com/marten-seemann/qtls-go1-18 => chainmaker.org/third_party/qtls-go1-18 v1.0.0
	google.golang.org/grpc => google.golang.org/grpc v1.26.0
)
