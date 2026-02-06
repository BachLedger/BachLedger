#!/bin/bash

cd tools/cmc || exit

# 定义样例数据数量常量
NUM_SAMPLES=600

# 生成随机16进制地址的函数
generate_random_address() {
  hex_characters="0123456789abcdef"
  address="0x"
  for _ in {1..40}; do
    address+="${hex_characters:$((RANDOM % 16)):1}"
  done
  echo $address
}

# 定义一个数组来存储预先生成的params
params_list=()

# 生成指定数量的params并存储在数组中
for (( i=0; i<$NUM_SAMPLES; i++ )); do
  uint256_value=$((i * 100))
  address_value=$(generate_random_address)
  params_list+=("[{\"key\": \"_newBalance\", \"value\": \"$uint256_value\"},{\"key\": \"_to\", \"value\": \"$address_value\"}]")
  echo ${address_value}: ${uint256_value}
done

# 循环中直接使用预先生成的 params 进行调用
for (( i=0; i<$NUM_SAMPLES; i++ )); do
  ./cmc client contract user invoke \
    --contract-name=balance001 \
    --method=updateBalance \
    --sdk-conf-path=./testdata/sdk_config.yml \
    --params="${params_list[$i]}" \
    --sync-result=false \
    --abi-file-path=./testdata/balance-evm-demo/ledger_balance.abi &
#  ./cmc parallel invoke
##  ants.NewPoolWithFunc(threadNum, subscribeChainBlock) (区块订阅)
#  --pairs="${params_list[$i]}" \
#  --contract-name= evidence # 被调用的合约名称
#  --method=balance001 #调用合约的存证方法
#  --loopNum=1000  # 线程循环次数
#  --printTime=5  #日志打印时间间隔（s)
#  --threadNum=10000 #并发用户数
#  --timeout=10000 # 请求超时时间
#  --sleepTime=1000 # 线程的请求发送间隔
#  --climbTime=5  # 线程启动的爬坡度
done

# 等待所有后台任务完成
wait
