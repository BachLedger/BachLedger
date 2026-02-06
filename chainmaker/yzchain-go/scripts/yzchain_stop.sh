#!/bin/bash

./cluster_quick_stop.sh
cd ..
rm -rf bin build
cd /mnt/mydisk/yzchain/yzchain-go/tools/yzc/testdata/

rm -rf crypto-config