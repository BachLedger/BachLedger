#!/bin/bash

cd /mnt/mydisk/yzchain/chain-module/yzchain-cryptogen/
make
cd /mnt/mydisk/yzchain/yzchain-go/scripts/

./prepare.sh 4 1
./build_release.sh
./cluster_quick_start.sh normal
ps -ef | grep yzchain | grep -v grep
cd ../tools/yzc/
go build
cp -rf ../../build/crypto-config ../../tools/yzc/testdata/
echo "-------------finish-------------------"
