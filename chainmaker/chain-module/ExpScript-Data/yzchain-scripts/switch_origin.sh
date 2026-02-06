#!/bin/bash

# 遍历当前文件夹下的每一个子文件夹
for dir in */; do
  # 去掉子文件夹名称后的斜杠
  dir=${dir%/}
  # 输出当前处理的子文件夹名称
  echo "Processing directory: $dir"
  # 进入子文件夹
  cd "$dir" || continue
  # 执行 git remote set-url 命令
  git remote set-url origin "http://166.111.80.46:9080/chain-core/chain-module/${dir}"
  # 返回上一级目录
  cd ..
done

