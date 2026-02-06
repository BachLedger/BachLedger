#!/bin/bash

# 仅有镜像 chainmakerofficial/chainmaker-vm-engine:v2.3.2 能够顺利使用

docker run -itd \
--net=host \
-v "/home/data/wx-org1.chainmaker.org/go":/mount \
-v "/home/log/wx-org1.chainmaker.org/go":/log \
-e CHAIN_RPC_PROTOCOL="1" \
-e CHAIN_RPC_PORT="22351" \
-e SANDBOX_RPC_PORT="32351" \
-e MAX_SEND_MSG_SIZE="100" \
-e MAX_RECV_MSG_SIZE="100" \
-e MAX_CONN_TIMEOUT="10" \
-e MAX_ORIGINAL_PROCESS_NUM="20" \
-e DOCKERVM_CONTRACT_ENGINE_LOG_LEVEL="DEBUG" \
-e DOCKERVM_SANDBOX_LOG_LEVEL="DEBUG" \
-e DOCKERVM_LOG_IN_CONSOLE="false" \
--name VM-GO-wx-org1.chainmaker.org \
--privileged chainmakerofficial/chainmaker-vm-engine:v2.3.2 \
> /dev/null

docker run -itd \
--net=host \
-v "/home/data/wx-org2.chainmaker.org/go":/mount \
-v "/home/log/wx-org2.chainmaker.org/go":/log \
-e CHAIN_RPC_PROTOCOL="1" \
-e CHAIN_RPC_PORT="22352" \
-e SANDBOX_RPC_PORT="32352" \
-e MAX_SEND_MSG_SIZE="100" \
-e MAX_RECV_MSG_SIZE="100" \
-e MAX_CONN_TIMEOUT="10" \
-e MAX_ORIGINAL_PROCESS_NUM="20" \
-e DOCKERVM_CONTRACT_ENGINE_LOG_LEVEL="DEBUG" \
-e DOCKERVM_SANDBOX_LOG_LEVEL="DEBUG" \
-e DOCKERVM_LOG_IN_CONSOLE="false" \
--name VM-GO-wx-org2.chainmaker.org \
--privileged chainmakerofficial/chainmaker-vm-engine:v2.3.2 \
> /dev/null

docker run -itd \
--net=host \
-v "/home/data/wx-org3.chainmaker.org/go":/mount \
-v "/home/log/wx-org3.chainmaker.org/go":/log \
-e CHAIN_RPC_PROTOCOL="1" \
-e CHAIN_RPC_PORT="22353" \
-e SANDBOX_RPC_PORT="32353" \
-e MAX_SEND_MSG_SIZE="100" \
-e MAX_RECV_MSG_SIZE="100" \
-e MAX_CONN_TIMEOUT="10" \
-e MAX_ORIGINAL_PROCESS_NUM="20" \
-e DOCKERVM_CONTRACT_ENGINE_LOG_LEVEL="DEBUG" \
-e DOCKERVM_SANDBOX_LOG_LEVEL="DEBUG" \
-e DOCKERVM_LOG_IN_CONSOLE="false" \
--name VM-GO-wx-org3.chainmaker.org \
--privileged chainmakerofficial/chainmaker-vm-engine:v2.3.2 \
> /dev/null

docker run -itd \
--net=host \
-v "/home/data/wx-org4.chainmaker.org/go":/mount \
-v "/home/log/wx-org4.chainmaker.org/go":/log \
-e CHAIN_RPC_PROTOCOL="1" \
-e CHAIN_RPC_PORT="22354" \
-e SANDBOX_RPC_PORT="32354" \
-e MAX_SEND_MSG_SIZE="100" \
-e MAX_RECV_MSG_SIZE="100" \
-e MAX_CONN_TIMEOUT="10" \
-e MAX_ORIGINAL_PROCESS_NUM="20" \
-e DOCKERVM_CONTRACT_ENGINE_LOG_LEVEL="DEBUG" \
-e DOCKERVM_SANDBOX_LOG_LEVEL="DEBUG" \
-e DOCKERVM_LOG_IN_CONSOLE="false" \
--name VM-GO-wx-org4.chainmaker.org \
--privileged chainmakerofficial/chainmaker-vm-engine:v2.3.2 \
> /dev/null
