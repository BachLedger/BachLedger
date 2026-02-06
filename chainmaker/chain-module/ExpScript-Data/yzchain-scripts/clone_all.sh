#!/usr/bin/env bash

# Clone all repos above with "http://124.220.1.170:7006/chain-core/chain-module/$repo.git"
# into the current directory.
repos=(
    'chainconf'
    'common'
    'consensus-raft'
    'consensus-solo'
    'consensus-tbft'
    'consensus-utils'
    'localconf'
    'logger'
    'libp2p-pubsub'
    'libp2p-core'
    'lws'
    'net-common'
    'net-libp2p'
    'pb'
    'pb-go'
    'protocol'
    'sdk-go'
    'store-leveldb'
    'store'
    'txpool-batch'
    'txpool-normal'
    'utils'
    'vm-engine'
    'vm-evm'
    'vm-native'
    'vm'
    # 'yzchain-go'
)

for repo in "${repos[@]}"; do
    git clone --branch v2-dev-yy4 --depth 3 "http://166.111.80.46:9080/chain-core/chain-module/$repo.git"
done

