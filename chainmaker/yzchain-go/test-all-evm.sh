#!/bin/bash

# 使用说明
function usage() {
    echo "Usage: $0 [--help|-h]"
    echo
    echo "Test a list of commands in specified directories, showing each command and its directory before executing."
    echo "Press ENTER to execute each command."
    echo "After completing the basic operations, choose one from different operations to repeat until the user chooses to exit."
    echo
}

# 检查是否有任何参数传递给脚本
if [[ "$1" == "--help" ]] || [[ "$1" == "-h" ]]; then
    usage
    exit 0
fi

# 定义命令及其执行目录的数组，格式为 "指令:目录"
commands=(
    "make clean && make:../yzchain-cryptogen"
    "ln -s ../../yzchain-cryptogen/ .:./tools"
    "./cluster_quick_stop.sh:./scripts"
    "rm -rf bin build:."
    "killall yzchain:."
    "./prepare.sh 4 1:./scripts"
    "./build_release.sh:./scripts"
    "./cluster_quick_start.sh normal:./scripts"
    "ps -ef| grep -v grep | grep yzchain:."
    "netstat -lptn | grep 1230:."
    "cat ./build/release/*/log/system.log |grep \"ERROR\|put block\|all necessary\":."
    "cat ./build/release/*/bin/panic.log:."
    "cp -rf ./build/crypto-config ./tools/yzc/testdata/:."
#    "rm yzc:tools/yzc"
#    "go build:tools/yzc"
    "./yzc client contract user create --contract-name=balance001 --runtime-type=EVM --byte-code-path=./testdata/balance-evm-demo/ledger_balance.bin --version=1.0 --sdk-conf-path=./testdata/sdk_config.yml \
               --admin-key-file-paths=./testdata/crypto-config/wx-org1.yzchain.org/user/admin1/admin1.tls.key,./testdata/crypto-config/wx-org2.yzchain.org/user/admin1/admin1.tls.key,./testdata/crypto-config/wx-org3.yzchain.org/user/admin1/admin1.tls.key \
               --admin-crt-file-paths=./testdata/crypto-config/wx-org1.yzchain.org/user/admin1/admin1.tls.crt,./testdata/crypto-config/wx-org2.yzchain.org/user/admin1/admin1.tls.crt,./testdata/crypto-config/wx-org3.yzchain.org/user/admin1/admin1.tls.crt \
               --sync-result=true --abi-file-path=./testdata/balance-evm-demo/ledger_balance.abi:tools/yzc"
)

# 设置颜色
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 获取脚本开始时的目录
start_dir=$(pwd)

# 按顺序执行命令
for item in "${commands[@]}"; do
    # 分割每个元素为指令和目录
    IFS=":" read -r cmd dir <<< "$item"
    echo -e "${RED}Next command: ${cmd}${NC}"
    echo -e "${BLUE}Directory: ${dir}${NC}"
    echo -e "${BLUE}Press ENTER to execute...${NC}"
#    read
    cd "${dir}" || { echo "Failed to change directory to ${dir}. Exiting."; exit 1; }
    eval $cmd
    cd "$start_dir" || { echo "Failed to change back to start directory. Exiting."; exit 1; }
done
cd tools/yzc || exit
# 选择操作
while true; do
    echo -e "${BLUE}Choose an operation:${NC}"
    echo "1. deploy an EVM contract (balance001)"
    echo "2. invoke updateBalance method (sync call)"
    echo "3. query balance"
    echo "4. updateBalance with 10000000 wei"
    echo "5. updateBalance with 20000000 wei"
    echo "6. invoke updateBalance method for 9 times"
    echo "7. invoke random unrelated transactions."
    echo "0. Exit"
    # shellcheck disable=SC2162
    read -p "Enter your choice (0/1/2/3/4/5/6): " choice
    case $choice in
        1) ./yzc client contract user create --contract-name=balance001 --runtime-type=EVM --byte-code-path=./testdata/balance-evm-demo/ledger_balance.bin --version=1.0 --sdk-conf-path=./testdata/sdk_config.yml \
          --admin-key-file-paths=./testdata/crypto-config/yz-org1.yzchain.org/user/admin1/admin1.tls.key,./testdata/crypto-config/yz-org2.yzchain.org/user/admin1/admin1.tls.key,./testdata/crypto-config/yz-org3.yzchain.org/user/admin1/admin1.tls.key \
          --admin-crt-file-paths=./testdata/crypto-config/yz-org1.yzchain.org/user/admin1/admin1.tls.crt,./testdata/crypto-config/yz-org2.yzchain.org/user/admin1/admin1.tls.crt,./testdata/crypto-config/yz-org3.yzchain.org/user/admin1/admin1.tls.crt \
          --sync-result=true --abi-file-path=./testdata/balance-evm-demo/ledger_balance.abi ;;
        2) ./yzc client contract user invoke --contract-name=balance001 \
          --method=updateBalance --sdk-conf-path=./testdata/sdk_config.yml \
          --params="[{\"uint256\": \"10000000\"},{\"address\": \"0xa166c92f4c8118905ad984919dc683a7bdb295c1\"}]" \
          --sync-result=true --abi-file-path=./testdata/balance-evm-demo/ledger_balance.abi ;;
        3) ./yzc client contract user get --contract-name=balance001 --method=balances \
          --sdk-conf-path=./testdata/sdk_config.yml \
          --params="[{\"address\": \"0xa166c92f4c8118905ad984919dc683a7bdb295c1\"}]"  \
          --abi-file-path=./testdata/balance-evm-demo/ledger_balance.abi ;;
        4) ./yzc client contract user invoke --contract-name=balance001 --method=updateBalance --sdk-conf-path=./testdata/sdk_config.yml --params="[{\"uint256\": \"10000000\"},{\"address\": \"0xa166c92f4c8118905ad984919dc683a7bdb295c1\"}]" --sync-result=true --abi-file-path=./testdata/balance-evm-demo/ledger_balance.abi & ;;
        5) ./yzc client contract user invoke --contract-name=balance001 --method=updateBalance --sdk-conf-path=./testdata/sdk_config.yml --params="[{\"uint256\": \"20000000\"},{\"address\": \"0xa166c92f4c8118905ad984919dc683a7bdb295c1\"}]" --sync-result=true --abi-file-path=./testdata/balance-evm-demo/ledger_balance.abi & ;;
        6) ./yzc client contract user invoke --concurrency=9 --contract-name=balance001 --method=updateBalance --sdk-conf-path=./testdata/sdk_config.yml --params="[{\"uint256\": \"10000000\"},{\"address\": \"0xa166c92f4c8118905ad984919dc683a7bdb295c1\"}]" --sync-result=true --abi-file-path=./testdata/balance-evm-demo/ledger_balance.abi ;;
        7) cd ../.. && ./loop-transfer.sh && cd - || exit ;;
        0) echo "Exiting..."; break ;;
        *) echo "Invalid choice. Please choose again." ;;
    esac
done
