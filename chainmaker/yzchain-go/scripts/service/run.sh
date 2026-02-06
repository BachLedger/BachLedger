#!/usr/bin/env bash

start() {
	export LD_LIBRARY_PATH=$(dirname $PWD)/lib:$LD_LIBRARY_PATH
  export PATH=$(dirname $PWD)/lib:$PATH
  export WASMER_BACKTRACE=1
  pid=`ps -ef | grep yzchain | grep "\-c ../config/{org_id}/yzchain.yml" | grep -v grep |  awk  '{print $2}'`
  if [ -z ${pid} ];then
      nohup ./yzchain start -c ../config/{org_id}/yzchain.yml > panic.log &
      echo "yzchain is starting, pls check log..."
  else
      echo "yzchain is already started"
  fi
}

stop() {
  pid=`ps -ef | grep yzchain | grep "\-c ../config/{org_id}/yzchain.yml" | grep -v grep |  awk  '{print $2}'`
  if [ ! -z ${pid} ];then
      kill $pid
  fi
  echo "yzchain is stopped"
}

case "$1" in
    start)
      start
    	;;
    stop)
      stop
    	;;
    restart)
    	echo "yzchain restart"
    	stop
    	start
    	;;
    *)
        echo "you can use: $0 [start|stop|restart]"
	exit 1 
esac

exit 0
