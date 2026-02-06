#!/usr/bin/env bash
#
# Copyright (C) BABEC. All rights reserved.
#
# SPDX-License-Identifier: Apache-2.0
#
## deploy yzchain and test

module=$1

CURRENT_PATH=$(pwd)
PROJECT_PATH=$(dirname $(dirname "${CURRENT_PATH}"))

# stop yzchain node
for i in {1..5} ; do
  stoping=0
  pid1=`ps -ef | grep yzchain | grep "\-c ../config/yz-org1/yzchain.yml start_four_node" | grep -v grep |  awk  '{print $2}'`
  pid2=`ps -ef | grep yzchain | grep "\-c ../config/yz-org2/yzchain.yml start_four_node" | grep -v grep |  awk  '{print $2}'`
  pid3=`ps -ef | grep yzchain | grep "\-c ../config/yz-org3/yzchain.yml start_four_node" | grep -v grep |  awk  '{print $2}'`
  pid4=`ps -ef | grep yzchain | grep "\-c ../config/yz-org4/yzchain.yml start_four_node" | grep -v grep |  awk  '{print $2}'`
  if [ ! -z ${pid1} ];then
      kill $pid1
      echo "yzchain yz-org1 is stopping..."
      stoping=1
  fi
  if [ ! -z ${pid2} ];then
      kill $pid2
      echo "yzchain yz-org2 is stopping..."
      stoping=1
  fi
  if [ ! -z ${pid3} ];then
      kill $pid3
      echo "yzchain yz-org3 is stopping..."
      stoping=1
  fi
  if [ ! -z ${pid4} ];then
      kill $pid4
      echo "yzchain yz-org4 is stopping..."
      stoping=1
  fi
  if [ ${stoping} == 0 ]; then
      echo "yzchain stopped"
      if [ ${module} == "clean" ];then
        cd $PROJECT_PATH
        rm -rf log data
        echo "rm -rf $PROJECT_PATH/log $PROJECT_PATH/data"
      fi
      exit 0
  fi
  sleep 1
done
echo
echo "yzchain stop fail"
echo
ps -ef|grep "yzchain.yml start_four_node"