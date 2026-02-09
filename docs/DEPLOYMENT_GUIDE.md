# BachLedger 医疗区块链部署与操作指南

本指南详细说明如何部署 BachLedger 全节点网络、生成密钥、编译部署合约以及调用合约。

---

## 目录

1. [环境准备](#1-环境准备)
2. [密钥生成与管理](#2-密钥生成与管理)
3. [单节点部署](#3-单节点部署)
4. [多节点网络部署](#4-多节点网络部署)
5. [智能合约编写](#5-智能合约编写)
6. [合约编译](#6-合约编译)
7. [合约部署](#7-合约部署)
8. [合约调用](#8-合约调用)
9. [状态查询](#9-状态查询)
10. [常见问题](#10-常见问题)

---

## 1. 环境准备

### 1.1 系统要求

- **操作系统**: Linux, macOS, 或 Windows (WSL2)
- **内存**: 最少 4GB RAM
- **磁盘**: 最少 10GB 可用空间
- **网络**: 开放 P2P 端口 (默认 30303) 和 RPC 端口 (默认 8545)

### 1.2 安装 Rust 工具链

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 更新到最新稳定版
rustup update stable

# 验证安装
rustc --version
cargo --version
```

### 1.3 编译 BachLedger

```bash
# 克隆代码库
git clone https://github.com/BachLedger/BachLedger.git
cd BachLedger

# 编译发布版本
cd rust
cargo build --release --package bach-node

# 验证编译结果
./target/release/bach-node --version
```

编译产物位于: `rust/target/release/bach-node`

---

## 2. 密钥生成与管理

### 2.1 验证者密钥结构

BachLedger 使用 **secp256k1** 椭圆曲线密码学 (与 Ethereum 兼容):

| 组件 | 长度 | 格式 |
|------|------|------|
| 私钥 | 32 bytes | 64 字符十六进制 |
| 公钥 | 65 bytes | 04 前缀 + 128 字符十六进制 (未压缩) |
| 地址 | 20 bytes | 0x 前缀 + 40 字符十六进制 |

**地址派生**: `Address = keccak256(PublicKey[1:])[12:]`

### 2.2 生成验证者密钥

```bash
# 生成单个验证者密钥
./target/release/bach-node gen-key --output validator.key

# 输出示例:
# Validator key generated successfully
# Private key saved to: "validator.key"
# Address: 0x14dcf07a2302449e945135cbfd10eb9efca955a1
# Public key: 0x04d54316ba67c03078fd860888e4311ccdb4...
```

### 2.3 批量生成密钥 (多节点网络)

```bash
#!/bin/bash
# generate-keys.sh

mkdir -p keys

for i in 1 2 3 4; do
    ./target/release/bach-node gen-key --output "keys/validator${i}.key"
    echo "Generated validator${i}.key"
done

echo "All keys generated in ./keys/"
```

### 2.4 密钥文件格式

密钥文件是纯文本格式，包含 64 字符十六进制私钥:

```
644e18f238e2acaa4ce7ffaea0e1f0f531274ce63cccb610b7ec26a1fac6690a
```

**安全警告**:
- 私钥文件权限应设置为 `600`: `chmod 600 validator.key`
- 切勿将私钥提交到版本控制系统
- 生产环境建议使用 HSM (硬件安全模块)

### 2.5 从私钥派生地址

```bash
# 使用 bach-node 显示密钥信息
./target/release/bach-node info --validator-key validator.key
```

或使用 Rust 代码:

```rust
use bach_crypto::{PrivateKey, derive_address};

let private_key = PrivateKey::from_hex("644e18f238e2acaa...")?;
let public_key = private_key.public_key();
let address = derive_address(&public_key);
println!("Address: 0x{}", hex::encode(address.as_bytes()));
```

---

## 3. 单节点部署

### 3.1 创建数据目录

```bash
mkdir -p ~/bachledger-node/data
```

### 3.2 生成验证者密钥

```bash
./target/release/bach-node gen-key --output ~/bachledger-node/validator.key
```

### 3.3 启动节点

```bash
./target/release/bach-node \
    --data-dir ~/bachledger-node/data \
    --validator-key ~/bachledger-node/validator.key \
    --chain-id 31337 \
    --rpc \
    --rpc-addr 127.0.0.1:8545 \
    --log-level info \
    run
```

### 3.4 命令行参数详解

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--data-dir` | `./data` | 区块链数据存储目录 |
| `--listen-addr` | `0.0.0.0:30303` | P2P 网络监听地址 |
| `--bootnodes` | 无 | 引导节点列表 (逗号分隔) |
| `--validator-key` | 无 | 验证者私钥文件路径 |
| `--chain-id` | `31337` | 链 ID |
| `--block-time` | `3000` | 出块时间 (毫秒) |
| `--rpc` | 禁用 | 启用 JSON-RPC 服务器 |
| `--rpc-addr` | `0.0.0.0:8545` | RPC 监听地址 |
| `--log-level` | `info` | 日志级别 (trace/debug/info/warn/error) |

### 3.5 验证节点运行

```bash
# 检查 RPC 连接
curl -s -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'

# 预期响应: {"jsonrpc":"2.0","id":1,"result":"0x7a69"}
```

### 3.6 后台运行 (systemd)

创建 `/etc/systemd/system/bachledger.service`:

```ini
[Unit]
Description=BachLedger Medical Blockchain Node
After=network.target

[Service]
Type=simple
User=bachledger
ExecStart=/usr/local/bin/bach-node \
    --data-dir /var/lib/bachledger/data \
    --validator-key /var/lib/bachledger/validator.key \
    --rpc --rpc-addr 0.0.0.0:8545 \
    run
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable bachledger
sudo systemctl start bachledger
sudo journalctl -u bachledger -f  # 查看日志
```

---

## 4. 多节点网络部署

### 4.1 网络拓扑

BachLedger 使用 **TBFT (Tendermint BFT)** 共识算法:

- 最少需要 **4 个验证者节点** (容忍 1 个拜占庭节点)
- 公式: `n > 3f + 1` (n=节点数, f=可容忍的拜占庭节点数)

```
       ┌─────────────┐
       │   Node 1    │ (Bootstrap)
       │  :30303     │
       └─────┬───────┘
             │
    ┌────────┼────────┐
    │        │        │
┌───▼───┐ ┌──▼────┐ ┌─▼─────┐
│ Node 2│ │ Node 3│ │ Node 4│
│ :30304│ │ :30305│ │ :30306│
└───────┘ └───────┘ └───────┘
```

### 4.2 使用 Docker Compose 部署

```bash
cd deployment

# 1. 生成验证者密钥
./setup.sh

# 2. 启动 4 节点网络
docker compose up -d

# 3. 查看日志
docker compose logs -f

# 4. 查看节点状态
docker compose ps
```

### 4.3 手动多节点部署

**节点 1 (Bootstrap)**:
```bash
./bach-node \
    --data-dir ./node1/data \
    --listen-addr 0.0.0.0:30303 \
    --validator-key ./keys/validator1.key \
    --rpc --rpc-addr 0.0.0.0:8545 \
    run
```

**节点 2**:
```bash
./bach-node \
    --data-dir ./node2/data \
    --listen-addr 0.0.0.0:30304 \
    --bootnodes "node1.example.com:30303" \
    --validator-key ./keys/validator2.key \
    --rpc --rpc-addr 0.0.0.0:8547 \
    run
```

**节点 3 和 4**: 类似配置，更改端口和密钥文件

### 4.4 创世配置 (genesis.json)

```json
{
  "config": {
    "chainId": 31337,
    "chainName": "BachLedger Medical Blockchain",
    "consensus": {
      "type": "tbft",
      "blockTime": 3000,
      "validators": [
        {
          "address": "0x14dcf07a2302449e945135cbfd10eb9efca955a1",
          "publicKey": "0x04d54316ba67c03078...",
          "power": 100
        },
        {
          "address": "0xc2138e6e6ebf96d5a58fa4e116a3fa7d719fc1c6",
          "publicKey": "0x04341d80530710a57e...",
          "power": 100
        }
      ],
      "proposerPolicy": "roundRobin",
      "epochLength": 100
    }
  },
  "alloc": {
    "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266": {
      "balance": "0x56BC75E2D63100000"
    }
  }
}
```

### 4.5 端口映射表

| 节点 | P2P 端口 | RPC 端口 |
|------|----------|----------|
| Node 1 | 30303 | 8545 |
| Node 2 | 30304 | 8547 |
| Node 3 | 30305 | 8549 |
| Node 4 | 30306 | 8551 |

---

## 5. 智能合约编写

### 5.1 合约结构

BachLedger 支持 EVM 兼容的智能合约。可以使用 Solidity 编写:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// @title SimpleStorage - 简单存储合约
contract SimpleStorage {
    uint256 private storedValue;

    event ValueChanged(uint256 indexed oldValue, uint256 indexed newValue);

    /// @notice 存储一个数值
    /// @param value 要存储的值
    function store(uint256 value) public {
        uint256 oldValue = storedValue;
        storedValue = value;
        emit ValueChanged(oldValue, value);
    }

    /// @notice 获取存储的数值
    /// @return 存储的值
    function retrieve() public view returns (uint256) {
        return storedValue;
    }
}
```

### 5.2 医疗数据合约示例

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// @title MedicalRecord - 医疗记录合约
contract MedicalRecord {
    struct Record {
        bytes32 dataHash;      // IPFS 哈希或加密数据哈希
        address doctor;        // 创建记录的医生
        uint256 timestamp;     // 创建时间
        bool isEncrypted;      // 是否加密
        string recordType;     // 记录类型 (诊断/处方/检验等)
    }

    mapping(address => Record[]) private patientRecords;
    mapping(address => mapping(address => bool)) private accessPermissions;

    event RecordAdded(address indexed patient, bytes32 dataHash, address doctor);
    event AccessGranted(address indexed patient, address indexed grantee);
    event AccessRevoked(address indexed patient, address indexed grantee);

    modifier onlyAuthorized(address patient) {
        require(
            msg.sender == patient || accessPermissions[patient][msg.sender],
            "Not authorized"
        );
        _;
    }

    /// @notice 添加医疗记录
    function addRecord(
        address patient,
        bytes32 dataHash,
        bool isEncrypted,
        string calldata recordType
    ) external {
        Record memory newRecord = Record({
            dataHash: dataHash,
            doctor: msg.sender,
            timestamp: block.timestamp,
            isEncrypted: isEncrypted,
            recordType: recordType
        });

        patientRecords[patient].push(newRecord);
        emit RecordAdded(patient, dataHash, msg.sender);
    }

    /// @notice 授权访问
    function grantAccess(address grantee) external {
        accessPermissions[msg.sender][grantee] = true;
        emit AccessGranted(msg.sender, grantee);
    }

    /// @notice 撤销访问
    function revokeAccess(address grantee) external {
        accessPermissions[msg.sender][grantee] = false;
        emit AccessRevoked(msg.sender, grantee);
    }

    /// @notice 获取记录数量
    function getRecordCount(address patient)
        external
        view
        onlyAuthorized(patient)
        returns (uint256)
    {
        return patientRecords[patient].length;
    }

    /// @notice 获取指定记录
    function getRecord(address patient, uint256 index)
        external
        view
        onlyAuthorized(patient)
        returns (Record memory)
    {
        require(index < patientRecords[patient].length, "Index out of bounds");
        return patientRecords[patient][index];
    }
}
```

---

## 6. 合约编译

### 6.1 使用 solc 编译

```bash
# 安装 solc
npm install -g solc

# 编译合约
solcjs --bin --abi SimpleStorage.sol -o build/

# 输出文件:
# build/SimpleStorage_sol_SimpleStorage.bin  (字节码)
# build/SimpleStorage_sol_SimpleStorage.abi  (ABI)
```

### 6.2 使用 Foundry 编译

```bash
# 安装 Foundry
curl -L https://foundry.paradigm.xyz | bash
foundryup

# 初始化项目
forge init my-contracts
cd my-contracts

# 编译
forge build

# 输出位于 out/ 目录
```

### 6.3 字节码结构

编译后的合约包含两部分:

1. **Init Code (构造函数代码)**: 部署时执行，返回 Runtime Code
2. **Runtime Code (运行时代码)**: 实际存储在链上的代码

```
[Init Code] + [Runtime Code]
     │              │
     ▼              ▼
  部署时执行     链上存储
```

### 6.4 获取部署字节码

```javascript
// 使用 ethers.js
const { ethers } = require("ethers");
const fs = require("fs");

const abi = JSON.parse(fs.readFileSync("build/SimpleStorage.abi"));
const bytecode = fs.readFileSync("build/SimpleStorage.bin", "utf8");

const factory = new ethers.ContractFactory(abi, bytecode);
console.log("Deploy bytecode:", factory.bytecode);
```

---

## 7. 合约部署

### 7.1 使用 curl 部署

```bash
# 合约字节码 (SimpleStorage 示例)
BYTECODE="0x608060405234801561001057600080fd5b5060c78061001f6000396000f3fe6080604052348015600f57600080fd5b506004361060325760003560e01c80632e64cec11460375780636057361d146051575b600080fd5b603d6063565b604051604891906078565b60405180910390f35b6061600435606c565b005b60005490565b600055565b90815260200190565b60006020820190508183525091905056fea2646970667358221220..."

# 发送部署交易
curl -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d "{
        \"jsonrpc\":\"2.0\",
        \"method\":\"eth_sendTransaction\",
        \"params\":[{
            \"from\":\"0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266\",
            \"data\":\"$BYTECODE\",
            \"gas\":\"0x100000\"
        }],
        \"id\":1
    }"

# 响应示例:
# {"jsonrpc":"2.0","id":1,"result":"0xabc123..."}  (交易哈希)
```

### 7.2 获取合约地址

合约地址计算方式: `keccak256(RLP([sender, nonce]))[12:]`

```bash
# 查询交易回执获取合约地址
TX_HASH="0xabc123..."

curl -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d "{
        \"jsonrpc\":\"2.0\",
        \"method\":\"eth_getTransactionReceipt\",
        \"params\":[\"$TX_HASH\"],
        \"id\":1
    }"

# 响应中的 contractAddress 字段即为合约地址
```

### 7.3 使用 ethers.js 部署

```javascript
const { ethers } = require("ethers");

async function deployContract() {
    // 连接到 BachLedger 节点
    const provider = new ethers.JsonRpcProvider("http://127.0.0.1:8545");

    // 使用测试账户 (无需真实私钥签名)
    const wallet = new ethers.Wallet(
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        provider
    );

    // 合约 ABI 和字节码
    const abi = [...];
    const bytecode = "0x...";

    // 部署
    const factory = new ethers.ContractFactory(abi, bytecode, wallet);
    const contract = await factory.deploy();
    await contract.waitForDeployment();

    console.log("Contract deployed at:", await contract.getAddress());
    return contract;
}

deployContract();
```

### 7.4 使用 Foundry 部署

```bash
# 部署脚本 script/Deploy.s.sol
forge script script/Deploy.s.sol \
    --rpc-url http://127.0.0.1:8545 \
    --broadcast
```

---

## 8. 合约调用

### 8.1 函数选择器计算

函数选择器 = `keccak256("functionName(type1,type2,...)")` 的前 4 字节

```bash
# 计算 store(uint256) 的选择器
echo -n "store(uint256)" | keccak-256sum | cut -c1-8
# 结果: 6057361d

# 计算 retrieve() 的选择器
echo -n "retrieve()" | keccak-256sum | cut -c1-8
# 结果: 2e64cec1
```

常用选择器速查:
| 函数 | 选择器 |
|------|--------|
| `store(uint256)` | `0x6057361d` |
| `retrieve()` | `0x2e64cec1` |
| `transfer(address,uint256)` | `0xa9059cbb` |
| `approve(address,uint256)` | `0x095ea7b3` |
| `balanceOf(address)` | `0x70a08231` |

### 8.2 参数编码

参数按 32 字节对齐编码 (ABI 编码):

```
函数调用数据 = 选择器(4字节) + 参数1(32字节) + 参数2(32字节) + ...
```

示例: `store(42)`
```
选择器:     6057361d
参数 (42):  000000000000000000000000000000000000000000000000000000000000002a
完整数据:   0x6057361d000000000000000000000000000000000000000000000000000000000000002a
```

### 8.3 写入调用 (eth_sendTransaction)

修改链上状态的调用:

```bash
CONTRACT="0x5FbDB2315678afecb367f032d93F642f64180aa3"
SENDER="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

# 调用 store(42)
curl -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d "{
        \"jsonrpc\":\"2.0\",
        \"method\":\"eth_sendTransaction\",
        \"params\":[{
            \"from\":\"$SENDER\",
            \"to\":\"$CONTRACT\",
            \"data\":\"0x6057361d000000000000000000000000000000000000000000000000000000000000002a\"
        }],
        \"id\":1
    }"
```

### 8.4 只读调用 (eth_call)

不修改状态的查询:

```bash
# 调用 retrieve()
curl -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d "{
        \"jsonrpc\":\"2.0\",
        \"method\":\"eth_call\",
        \"params\":[{
            \"to\":\"$CONTRACT\",
            \"data\":\"0x2e64cec1\"
        }, \"latest\"],
        \"id\":1
    }"

# 响应: {"jsonrpc":"2.0","id":1,"result":"0x000000000000000000000000000000000000000000000000000000000000002a"}
# 解码: 0x2a = 42
```

### 8.5 使用 ethers.js 调用

```javascript
const { ethers } = require("ethers");

async function interactWithContract() {
    const provider = new ethers.JsonRpcProvider("http://127.0.0.1:8545");

    const abi = [
        "function store(uint256 value)",
        "function retrieve() view returns (uint256)"
    ];

    const contract = new ethers.Contract(
        "0x5FbDB2315678afecb367f032d93F642f64180aa3",
        abi,
        provider
    );

    // 只读调用
    const value = await contract.retrieve();
    console.log("Current value:", value.toString());

    // 写入调用 (需要 signer)
    const wallet = new ethers.Wallet("0xac0974...", provider);
    const contractWithSigner = contract.connect(wallet);

    const tx = await contractWithSigner.store(100);
    await tx.wait();
    console.log("Value stored!");
}

interactWithContract();
```

### 8.6 使用 cast (Foundry) 调用

```bash
# 只读调用
cast call 0x5FbDB2315678afecb367f032d93F642f64180aa3 \
    "retrieve()(uint256)" \
    --rpc-url http://127.0.0.1:8545

# 写入调用
cast send 0x5FbDB2315678afecb367f032d93F642f64180aa3 \
    "store(uint256)" 42 \
    --rpc-url http://127.0.0.1:8545 \
    --private-key 0xac0974...
```

---

## 9. 状态查询

### 9.1 查询账户余额

```bash
curl -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc":"2.0",
        "method":"eth_getBalance",
        "params":["0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", "latest"],
        "id":1
    }'
```

### 9.2 查询合约代码

```bash
curl -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc":"2.0",
        "method":"eth_getCode",
        "params":["0x5FbDB2315678afecb367f032d93F642f64180aa3", "latest"],
        "id":1
    }'
```

### 9.3 查询存储槽

```bash
# 查询存储槽 0
curl -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc":"2.0",
        "method":"eth_getStorageAt",
        "params":[
            "0x5FbDB2315678afecb367f032d93F642f64180aa3",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "latest"
        ],
        "id":1
    }'
```

### 9.4 存储槽计算

对于 mapping 类型:
```
slot = keccak256(key + mappingSlot)
```

对于数组:
```
slot[i] = keccak256(arraySlot) + i
```

### 9.5 查询区块信息

```bash
# 最新区块号
curl -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# 按区块号查询
curl -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc":"2.0",
        "method":"eth_getBlockByNumber",
        "params":["0x1", true],
        "id":1
    }'
```

### 9.6 查询交易回执

```bash
curl -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc":"2.0",
        "method":"eth_getTransactionReceipt",
        "params":["0xabc123..."],
        "id":1
    }'
```

---

## 10. 常见问题

### Q1: 节点无法启动

**检查清单**:
1. 端口是否被占用: `lsof -i :8545` / `lsof -i :30303`
2. 数据目录权限: `ls -la ~/bachledger-node/`
3. 密钥文件是否存在: `cat validator.key`

### Q2: RPC 连接失败

**可能原因**:
- 节点未启动 `--rpc` 参数
- 防火墙阻止连接
- 监听地址不正确 (使用 `0.0.0.0` 允许外部连接)

### Q3: 合约部署失败

**检查**:
- 字节码格式 (必须以 `0x` 开头)
- gas 限制是否足够
- 发送者地址格式

### Q4: 多节点无法同步

**检查**:
- bootnodes 地址是否正确
- P2P 端口是否开放
- 创世配置是否一致
- 验证者集合配置

### Q5: 交易未确认

**可能原因**:
- 节点未出块 (检查验证者配置)
- 网络分区 (检查节点连接)

---

## 附录

### A. RPC 方法速查表

| 方法 | 说明 |
|------|------|
| `eth_chainId` | 获取链 ID |
| `eth_blockNumber` | 获取最新区块号 |
| `eth_getBalance` | 查询余额 |
| `eth_getCode` | 获取合约代码 |
| `eth_getStorageAt` | 查询存储槽 |
| `eth_call` | 只读合约调用 |
| `eth_sendTransaction` | 发送交易 |
| `eth_getTransactionReceipt` | 获取交易回执 |
| `eth_setBalance` | 设置余额 (开发用) |
| `net_version` | 网络版本 |
| `web3_clientVersion` | 客户端版本 |

### B. 开发账户

| 地址 | 用途 |
|------|------|
| `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266` | 开发账户 1 |
| `0x70997970C51812dc3A010C7d01b50e0d17dc79C8` | 开发账户 2 |
| `0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC` | 开发账户 3 |

### C. 相关资源

- [Ethereum JSON-RPC 规范](https://ethereum.org/en/developers/docs/apis/json-rpc/)
- [Solidity 文档](https://docs.soliditylang.org/)
- [Foundry Book](https://book.getfoundry.sh/)
- [ethers.js 文档](https://docs.ethers.org/)

---

*BachLedger Medical Blockchain - Secure Healthcare Data on Chain*
