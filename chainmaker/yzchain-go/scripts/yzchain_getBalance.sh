#!/bin/bash

cd /mnt/mydisk/yzchain/yzchain-go/tools/yzc
./yzc client contract user invoke \
--contract-name=balance001 \
--method=increaseBalance \
--sdk-conf-path=./testdata/sdk_config.yml \
--params="[{\"address\": \"0xed6decd8f7d56700daf84c4915fb250bef73f194\"}]" \
--sync-result=true \
--abi-file-path=./testdata/balance-evm-demo/ledger_balance.abi

./yzc client contract user invoke \
--contract-name=balance001 \
--method=getBalance \
--sdk-conf-path=./testdata/sdk_config.yml \
--params="[{\"address\": \"0xed6decd8f7d56700daf84c4915fb250bef73f194\"}]" \
--sync-result=true \
--abi-file-path=./testdata/balance-evm-demo/ledger_balance.abi



