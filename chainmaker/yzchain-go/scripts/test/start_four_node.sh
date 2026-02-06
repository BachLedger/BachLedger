#!/usr/bin/env bash
#
# Copyright (C) BABEC. All rights reserved.
#
# SPDX-License-Identifier: Apache-2.0
#
## deploy yzchain and test

CURRENT_PATH=$(pwd)
PROJECT_PATH=$(dirname $(dirname "${CURRENT_PATH}"))
#echo "PROJECT_PATH $PROJECT_PATH"

cd $PROJECT_PATH
if [ -e bin/yzchain ]; then
    echo "skip make, yzchain binary already exist"
else
    make
fi

cd bin
nohup ./yzchain start -c ../config/yz-org1/yzchain.yml start_four_node > panic1.log 2>&1 &
nohup ./yzchain start -c ../config/yz-org2/yzchain.yml start_four_node > panic2.log 2>&1 &
nohup ./yzchain start -c ../config/yz-org3/yzchain.yml start_four_node > panic3.log 2>&1 &
nohup ./yzchain start -c ../config/yz-org4/yzchain.yml start_four_node > panic4.log 2>&1 &
echo "start yzchain..."