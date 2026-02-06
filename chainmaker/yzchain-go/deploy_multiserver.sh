#!/bin/bash

start_dir=$(pwd)

rm -rf build
cd scripts
./prepare.sh 4 1

# 定义要处理的文件夹列表
folders=("node1" "node2" "node3" "node4")
cd ../build/config
# 遍历每个文件夹
for folder in "${folders[@]}"; do
  # 检查文件是否存在
  if [[ -f "$folder/chainmaker.yml" ]]; then
    # 检查文件是否已经包含新的IP地址
    already_modified=$(awk 'NR >= 88 && NR <= 91 {
      if (substr($0, 13, 9) == "127.0.0.1") count++;
    } END {print count}' "$folder/chainmaker.yml")
    if [[ "$already_modified" -eq 4 ]]; then
      echo "Processing $folder/chainmaker.yml..."
      # 执行awk命令
      awk 'NR==88 {print substr($0, 1, 12) "100.120.0.32" substr($0, 22); next}
           NR==89 {print substr($0, 1, 12) "100.120.0.33" substr($0, 22); next}
           NR==90 {print substr($0, 1, 12) "100.120.0.34" substr($0, 22); next}
           NR==91 {print substr($0, 1, 12) "100.120.0.35" substr($0, 22); next}
           {print}' "$folder/chainmaker.yml" > "$folder/temp.yml" && mv "$folder/temp.yml" "$folder/chainmaker.yml"
    else
      echo "No changes needed for $folder/chainmaker.yml."
    fi
  else
    echo "File not found in $folder"
  fi
done

cd "$start_dir/scripts"
./build_release.sh
