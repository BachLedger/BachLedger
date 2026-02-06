#
# Copyright (C) BABEC. All rights reserved.
# Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
#
# SPDX-License-Identifier: Apache-2.0
#

export LD_LIBRARY_PATH=$(dirname $PWD)/:$LD_LIBRARY_PATH
export PATH=$(dirname $PWD)/prebuilt/linux:$(dirname $PWD)/prebuilt/win64:$PATH
export WASMER_BACKTRACE=1
cp -rf ../../bin ./
export CMC=$PWD/bin
echo "cmc path:" $CMC

cd bin


pid=`ps -ef | grep yzchain | grep "\-c ../config/yz-org1.yzchain.org/yzchain.yml ci-chain1" | grep -v grep |  awk  '{print $2}'`
if [ -z ${pid} ];then
    nohup ./yzchain start -c ../config/yz-org1.yzchain.org/yzchain.yml ci-chain1 > panic1.log 2>&1 &
    echo "yz-org1.yzchain.org yzchain is starting, pls check log..."
else
    echo "yz-org1.yzchain.org yzchain is already started"
fi

pid2=`ps -ef | grep yzchain | grep "\-c ../config/yz-org2.yzchain.org/yzchain.yml ci-chain1" | grep -v grep |  awk  '{print $2}'`
if [ -z ${pid2} ];then
    nohup ./yzchain start -c ../config/yz-org2.yzchain.org/yzchain.yml ci-chain1 > panic2.log 2>&1 &
    echo "yz-org2.yzchain.org yzchain is starting, pls check log..."
else
    echo "yz-org2.yzchain.org yzchain is already started"
fi



pid3=`ps -ef | grep yzchain | grep "\-c ../config/yz-org3.yzchain.org/yzchain.yml ci-chain1" | grep -v grep |  awk  '{print $2}'`
if [ -z ${pid3} ];then
    nohup ./yzchain start -c ../config/yz-org3.yzchain.org/yzchain.yml ci-chain1 > panic3.log 2>&1 &
    echo "yz-org3.yzchain.org yzchain is starting, pls check log..."
else
    echo "yz-org3.yzchain.org yzchain is already started"
fi


pid4=`ps -ef | grep yzchain | grep "\-c ../config/yz-org4.yzchain.org/yzchain.yml ci-chain1" | grep -v grep |  awk  '{print $2}'`
if [ -z ${pid4} ];then
    nohup ./yzchain start -c ../config/yz-org4.yzchain.org/yzchain.yml ci-chain1 > panic4.log 2>&1 &
    echo "yz-org4.yzchain.org yzchain is starting, pls check log..."
else
    echo "yz-org4.yzchain.org yzchain is already started"
fi

# nohup ./yzchain start -c ../config/node5/yzchain.yml ci-chain1 > panic.log &

sleep 4
ps -ef|grep yzchain | grep "ci-chain1"
