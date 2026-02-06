module chainmaker.org/chainmaker/consensus-raft/v2

go 1.15

require (
	chainmaker.org/chainmaker/chainconf/v2 v2.3.2
	chainmaker.org/chainmaker/common/v2 v2.3.2
	chainmaker.org/chainmaker/consensus-utils/v2 v2.3.3
	chainmaker.org/chainmaker/localconf/v2 v2.3.2
	chainmaker.org/chainmaker/logger/v2 v2.3.0
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3
	chainmaker.org/chainmaker/protocol/v2 v2.3.3
	chainmaker.org/chainmaker/raftwal/v2 v2.1.0
	chainmaker.org/chainmaker/utils/v2 v2.3.3
	github.com/gogo/protobuf v1.3.2
	github.com/golang/mock v1.6.0
	github.com/stretchr/testify v1.8.0
	github.com/thoas/go-funk v0.9.1
	go.etcd.io/etcd/client/pkg/v3 v3.5.1
	go.etcd.io/etcd/raft/v3 v3.5.1
	go.etcd.io/etcd/server/v3 v3.5.1
	go.uber.org/zap v1.19.1
)

replace (
	chainmaker.org/chainmaker/chainconf/v2 v2.3.2 => ../chainconf
	chainmaker.org/chainmaker/common/v2 v2.3.2 => ../common
	chainmaker.org/chainmaker/consensus-utils/v2 v2.3.3 => ../consensus-utils
	chainmaker.org/chainmaker/localconf/v2 v2.3.2 => ../localconf
	chainmaker.org/chainmaker/logger/v2 v2.3.0 => ../logger
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3 => ../pb-go
	chainmaker.org/chainmaker/protocol/v2 v2.3.3 => ../protocol
	chainmaker.org/chainmaker/utils/v2 v2.3.3 => ../utils
	github.com/libp2p/go-libp2p-core => chainmaker.org/chainmaker/libp2p-core v1.0.0
	github.com/linvon/cuckoo-filter => chainmaker.org/third_party/cuckoo-filter v1.0.0
	github.com/lucas-clemente/quic-go v0.26.0 => chainmaker.org/third_party/quic-go v1.0.0
	github.com/marten-seemann/qtls-go1-15 => chainmaker.org/third_party/qtls-go1-15 v1.0.0
	github.com/marten-seemann/qtls-go1-16 => chainmaker.org/third_party/qtls-go1-16 v1.0.0
	github.com/marten-seemann/qtls-go1-17 => chainmaker.org/third_party/qtls-go1-17 v1.0.0
	github.com/marten-seemann/qtls-go1-18 => chainmaker.org/third_party/qtls-go1-18 v1.0.0
	google.golang.org/grpc => google.golang.org/grpc v1.26.0
)
