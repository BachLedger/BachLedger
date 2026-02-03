# BachLedger

## 目录结构

```
bachledger/
├── .memo/
├── yzchain-go/                      # 主链节点
├── yzchain-cryptogen/               # 密钥生成工具
└── chain-module/
    ├── chainconf/                   # 链配置
    ├── common/                      # 公共库
    ├── consensus-raft/              # Raft 共识
    ├── consensus-solo/              # Solo 共识
    ├── consensus-tbft/              # TBFT 共识
    ├── consensus-utils/             # 共识工具
    ├── localconf/                   # 本地配置
    ├── logger/                      # 日志模块
    ├── net-common/                  # 网络公共库
    ├── net-libp2p/                  # libp2p 网络
    ├── pb/                          # Protobuf 定义
    ├── pb-go/                       # Protobuf Go 生成代码
    ├── protocol/                    # 协议定义
    ├── sdk-go/                      # Go SDK
    ├── store/                       # 存储模块
    ├── txpool-batch/                # 批量交易池
    ├── txpool-normal/               # 普通交易池
    ├── utils/                       # 工具库
    ├── vm/                          # 虚拟机
    ├── vm-engine/                   # 虚拟机引擎
    ├── vm-evm/                      # EVM 虚拟机
    ├── vm-native/                   # Native 虚拟机
    └── ExpScript-Data/              # 实验脚本数据
```

## 克隆

```bash
git clone --recursive <repo-url>
```

## 更新子模块

```bash
git submodule update --init --recursive
```
