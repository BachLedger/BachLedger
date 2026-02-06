#!/bin/bash

cd /mnt/mydisk/yzchain/yzchain-go/tools/yzc

./yzc client contract user create \
--contract-name=balance001 \
--runtime-type=EVM \
--byte-code-path=./testdata/balance-evm-demo/ledger_balance.bin \
--version=1.0 \
--sdk-conf-path=./testdata/sdk_config.yml \
--admin-key-file-paths=./testdata/crypto-config/yz-org1.yzchain.org/user/admin1/admin1.tls.key,./testdata/crypto-config/yz-org2.yzchain.org/user/admin1/admin1.tls.key,./testdata/crypto-config/yz-org3.yzchain.org/user/admin1/admin1.tls.key \
--admin-crt-file-paths=./testdata/crypto-config/yz-org1.yzchain.org/user/admin1/admin1.tls.crt,./testdata/crypto-config/yz-org2.yzchain.org/user/admin1/admin1.tls.crt,./testdata/crypto-config/yz-org3.yzchain.org/user/admin1/admin1.tls.crt \
--sync-result=true \
--abi-file-path=./testdata/balance-evm-demo/ledger_balance.abi

./yzc client contract user invoke \
--contract-name=balance001 \
--method=increaseBalance \
--sdk-conf-path=./testdata/sdk_config.yml \
--params="[{\"address\": \"0xed6decd8f7d56700daf84c4915fb250bef73f194\"}]" \
--sync-result=true \
--abi-file-path=./testdata/balance-evm-demo/ledger_balance.abi



