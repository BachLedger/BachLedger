#!/bin/bash
# BachLedger 手动测试脚本
#
# 使用方法:
#   ./scripts/run_manual_test.sh
#
# 这个脚本会:
# 1. 清理旧的测试数据
# 2. 在后台启动节点
# 3. 等待节点就绪
# 4. 运行测试
# 5. 关闭节点

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
DATA_DIR="$ROOT_DIR/testdata"
PID_FILE="$DATA_DIR/node.pid"
RPC_URL="http://localhost:8545"

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║           BachLedger 手动测试                                ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo

# 清理函数
cleanup() {
    echo -e "\n${YELLOW}清理中...${NC}"
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if kill -0 "$PID" 2>/dev/null; then
            echo "停止节点 (PID: $PID)"
            kill "$PID" 2>/dev/null || true
            sleep 1
        fi
        rm -f "$PID_FILE"
    fi
}

trap cleanup EXIT

# 1. 清理旧数据
echo -e "${YELLOW}1. 清理旧测试数据...${NC}"
rm -rf "$DATA_DIR"
mkdir -p "$DATA_DIR"

# 2. 编译
echo -e "${YELLOW}2. 编译项目...${NC}"
cd "$ROOT_DIR"
cargo build -p bach-node --release 2>&1 | tail -5

# 3. 启动节点
echo -e "${YELLOW}3. 启动节点...${NC}"
cargo run -p bach-node --release -- \
    --datadir "$DATA_DIR" \
    --chain-id 1337 \
    --rpc-addr 0.0.0.0:8545 \
    --log-level info \
    > "$DATA_DIR/node.log" 2>&1 &

NODE_PID=$!
echo $NODE_PID > "$PID_FILE"
echo "节点已启动 (PID: $NODE_PID)"

# 4. 等待节点就绪
echo -e "${YELLOW}4. 等待节点就绪...${NC}"
MAX_RETRIES=30
RETRY_COUNT=0

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -s -X POST "$RPC_URL" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
        > /dev/null 2>&1; then
        echo -e "${GREEN}节点已就绪!${NC}"
        break
    fi
    sleep 1
    RETRY_COUNT=$((RETRY_COUNT + 1))
    echo -n "."
done

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
    echo -e "${RED}节点启动超时!${NC}"
    cat "$DATA_DIR/node.log"
    exit 1
fi

echo

# 5. 快速连接测试
echo -e "${YELLOW}5. 连接测试...${NC}"
CHAIN_ID=$(curl -s -X POST "$RPC_URL" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
    | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
echo "   Chain ID: $CHAIN_ID"

BLOCK=$(curl -s -X POST "$RPC_URL" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
echo "   区块高度: $BLOCK"

BALANCE=$(curl -s -X POST "$RPC_URL" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266","latest"],"id":1}' \
    | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
echo "   测试账户余额: $BALANCE"

echo

# 6. 运行完整测试
echo -e "${YELLOW}6. 运行完整测试...${NC}"
echo
RPC_URL="$RPC_URL" cargo run --example manual_test --release

echo
echo -e "${GREEN}测试完成!${NC}"
echo
echo "节点日志: $DATA_DIR/node.log"
echo "数据目录: $DATA_DIR"
