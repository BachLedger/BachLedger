#
# Copyright (C) BABEC. All rights reserved.
# Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
#
# SPDX-License-Identifier: Apache-2.0
#

export LD_LIBRARY_PATH=$(dirname $PWD)/:$LD_LIBRARY_PATH
export PATH=$(dirname $PWD)/prebuilt/linux:$(dirname $PWD)/prebuilt/win64:$PATH
export WASMER_BACKTRACE=1
cp ../../main/yzchain ./

pid=`ps -ef | grep yzchain | grep "\-c ./config-sql/yz-org1/yzchain.yml ci-sql-tbft" | grep -v grep |  awk  '{print $2}'`
if [ -z ${pid} ];then
    nohup ./yzchain start -c ./config-sql/yz-org1/yzchain.yml ci-sql-tbft > panic1.log 2>&1 &
    echo "yz-org1 yzchain is starting, pls check log..."
else
    echo "yz-org1 yzchain is already started"
fi

pid2=`ps -ef | grep yzchain | grep "\-c ./config-sql/yz-org2/yzchain.yml ci-sql-tbft" | grep -v grep |  awk  '{print $2}'`
if [ -z ${pid2} ];then
    nohup ./yzchain start -c ./config-sql/yz-org2/yzchain.yml ci-sql-tbft > panic2.log 2>&1 &
    echo "yz-org2 yzchain is starting, pls check log..."
else
    echo "yz-org2 yzchain is already started"
fi



pid3=`ps -ef | grep yzchain | grep "\-c ./config-sql/yz-org3/yzchain.yml ci-sql-tbft" | grep -v grep |  awk  '{print $2}'`
if [ -z ${pid3} ];then
    nohup ./yzchain start -c ./config-sql/yz-org3/yzchain.yml ci-sql-tbft > panic3.log 2>&1 &
    echo "yz-org3 yzchain is starting, pls check log..."
else
    echo "yz-org3 yzchain is already started"
fi


pid4=`ps -ef | grep yzchain | grep "\-c ./config-sql/yz-org4/yzchain.yml ci-sql-tbft" | grep -v grep |  awk  '{print $2}'`
if [ -z ${pid4} ];then
    nohup ./yzchain start -c ./config-sql/yz-org4/yzchain.yml ci-sql-tbft > panic4.log 2>&1 &
    echo "yz-org4 yzchain is starting, pls check log..."
else
    echo "yz-org4 yzchain is already started"
fi

# nohup ./yzchain start -c ./config-sql/yz-org5/yzchain.yml ci-sql-tbft > panic.log &

sleep 4
ps -ef|grep yzchain | grep "ci-sql-tbft"
