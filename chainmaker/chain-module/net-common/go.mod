module chainmaker.org/chainmaker/net-common

go 1.15

require (
	chainmaker.org/chainmaker/common/v2 v2.3.2
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3
	chainmaker.org/chainmaker/protocol/v2 v2.3.3
	github.com/libp2p/go-libp2p-core v0.6.1
	github.com/multiformats/go-multiaddr v0.3.1
	github.com/stretchr/testify v1.7.0
	golang.org/x/sys v0.0.0-20220520151302-bc2c85ada10a
)

replace (
	github.com/libp2p/go-libp2p-core => chainmaker.org/chainmaker/libp2p-core v1.0.0
	chainmaker.org/chainmaker/common/v2 v2.3.2 => ../common
	chainmaker.org/chainmaker/pb-go/v2 v2.3.3 => ../pb-go
	chainmaker.org/chainmaker/protocol/v2 v2.3.3 => ../protocol
)
