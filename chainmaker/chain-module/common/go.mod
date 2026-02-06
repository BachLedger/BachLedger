module chainmaker.org/chainmaker/common/v2

go 1.16

require (
	github.com/btcsuite/btcd v0.21.0-beta
	github.com/davecgh/go-spew v1.1.1
	github.com/go-echarts/go-echarts/v2 v2.2.4
	github.com/gogo/protobuf v1.3.2
	github.com/golang/groupcache v0.0.0-20200121045136-8c9f03a8e57e
	github.com/golang/mock v1.6.0
	github.com/golang/protobuf v1.5.2
	github.com/gomodule/redigo v2.0.0+incompatible
	github.com/google/uuid v1.1.2
	github.com/gorilla/websocket v1.4.3-0.20220104015952-9111bb834a68
	github.com/hyperledger/burrow v0.34.4
	github.com/json-iterator/go v1.1.11
	github.com/lestrrat-go/strftime v1.0.3
	github.com/libp2p/go-libp2p-core v0.6.1
	github.com/libp2p/go-openssl v0.0.7
	github.com/linvon/cuckoo-filter v0.4.0
	github.com/minio/sha256-simd v0.1.1
	github.com/mr-tron/base58 v1.2.0
	github.com/multiformats/go-multiaddr v0.3.1
	github.com/multiformats/go-multihash v0.0.14
	github.com/pkg/errors v0.9.1
	github.com/prometheus/client_golang v1.9.0
	github.com/smartystreets/goconvey v1.6.4
	github.com/spf13/viper v1.9.0
	github.com/stretchr/testify v1.7.0
	github.com/tidwall/gjson v1.10.2
	github.com/tidwall/tinylru v1.1.0
	github.com/tjfoc/gmsm v1.4.1
	github.com/tmthrgd/go-hex v0.0.0-20190904060850-447a3041c3bc
	go.uber.org/atomic v1.7.0
	go.uber.org/zap v1.17.0
	golang.org/x/crypto v0.0.0-20210817164053-32db794688a5
	golang.org/x/net v0.0.0-20210503060351-7fd8e65b6420
	golang.org/x/sys v0.0.0-20220222200937-f2425489ef4c
	google.golang.org/grpc v1.40.0
	github.com/miekg/pkcs11 v1.0.3
)

require (
	github.com/fastly/go-utils v0.0.0-20180712184237-d95a45783239 // indirect
	github.com/hashicorp/golang-lru v0.5.4
	github.com/jehiah/go-strftime v0.0.0-20171201141054-1d33003b3869 // indirect
	github.com/tebeka/strftime v0.1.5 // indirect
)

replace github.com/linvon/cuckoo-filter => chainmaker.org/third_party/cuckoo-filter v1.0.0
