#!/bin/bash
# Run AssetToken contract tests
# Usage: ./scripts/run_asset_token_test.sh [RPC_URL]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "==================================="
echo "  AssetToken Test Runner"
echo "==================================="

# Check for bytecode
BYTECODE_FILE="$ROOT_DIR/contracts/bytecode/AssetToken.bin"
if [ ! -f "$BYTECODE_FILE" ]; then
    echo "ERROR: Contract bytecode not found at $BYTECODE_FILE"
    echo ""
    echo "Please compile the contract first:"
    echo "  ./scripts/compile_contracts.sh"
    echo ""
    exit 1
fi

# Set RPC URL
export RPC_URL="${1:-http://localhost:8545}"
echo "RPC URL: $RPC_URL"
echo ""

# Check if node is running
if ! curl -s "$RPC_URL" -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' > /dev/null 2>&1; then
    echo "ERROR: Cannot connect to node at $RPC_URL"
    echo ""
    echo "Please start the node first:"
    echo "  cargo run -p bach-node --release -- --datadir ./testdata --chain-id 1337"
    echo ""
    exit 1
fi

echo "Running tests..."
echo ""

cd "$ROOT_DIR"
cargo run --example asset_token_test --release
