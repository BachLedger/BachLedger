#
# Copyright (C) BABEC. All rights reserved.
# Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
#
# SPDX-License-Identifier: Apache-2.0
#

export LD_LIBRARY_PATH=$(dirname $PWD)/:$LD_LIBRARY_PATH
export PATH=$(dirname $PWD)/prebuilt/linux:$(dirname $PWD)/prebuilt/win64:$PATH
export WASMER_BACKTRACE=1
cd ../../main

pid=$(ps -ef | grep yzchain | grep "\-c ../config" | grep -v grep | awk '{print $2}')
if [ -z ${pid} ]; then
  nohup ./yzchain start -c ../config/yz-org1-solo/yzchain.yml local-tbft >./panic.log &
  echo "yz-org1 yzchain is starting, pls check log..."
else
  echo "yz-org1 yzchain is already started"
fi

sleep 2
ps -ef | grep yzchain
