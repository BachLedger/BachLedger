#!/bin/bash

CORE_RANGE=$1 # 例如 0-7 表示使用前 8 个core

# 创建新的cgroup, 假设cgroup v2
sudo mkdir -p /sys/fs/cgroup/my_cgroup

# 启用cpuset子系统并设置cgroup可以使用的CPU cores和内存节点
sudo bash -c 'echo "+cpuset" > /sys/fs/cgroup/cgroup.subtree_control'
sudo bash -c "echo ${CORE_RANGE} > /sys/fs/cgroup/my_cgroup/cpuset.cpus"
sudo bash -c 'echo 0 > /sys/fs/cgroup/my_cgroup/cpuset.mems'

# 定义命令及其执行目录的数组，格式为 "指令:目录"
commands=(
    "make clean && make:../chainmaker-cryptogen"
    "ln -s ../../chainmaker-cryptogen/ .:./tools"
    "./cluster_quick_stop.sh:./scripts"
    "rm -rf bin build:."
    "killall chainmaker:."
    "./prepare2.sh 4 1:./scripts"
    "./build_release.sh:./scripts"
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

cd scripts || exit
./cluster_quick_start.sh normal &
# 启动任务并获取父进程的PID
PARENT_PID=$!

# 等待任务启动完成，确保所有子进程也加入cgroup
sleep 2

echo PARENT_PID: $PARENT_PID
# 将父进程及其所有子进程加入cgroup
echo $PARENT_PID | sudo tee /sys/fs/cgroup/my_cgroup/cgroup.procs

for CHILD_PID in $(pgrep -P $PARENT_PID); do
  echo $CHILD_PID | sudo tee /sys/fs/cgroup/my_cgroup/cgroup.procs
done

# 验证配置
echo "Current tasks in my_cgroup:"
cat /sys/fs/cgroup/my_cgroup/cgroup.procs

wait