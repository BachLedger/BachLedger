#!/bin/bash
# Compile Solidity contracts to bytecode
# Usage: ./scripts/compile_contracts.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
CONTRACTS_DIR="$ROOT_DIR/contracts"
BYTECODE_DIR="$CONTRACTS_DIR/bytecode"

echo "==================================="
echo "  Contract Compilation Script"
echo "==================================="

# Check for solc
if ! command -v solc &> /dev/null; then
    echo "ERROR: solc (Solidity compiler) not found."
    echo ""
    echo "Install options:"
    echo "  macOS:   brew install solidity"
    echo "  Ubuntu:  sudo add-apt-repository ppa:ethereum/ethereum && sudo apt-get update && sudo apt-get install solc"
    echo "  npm:     npm install -g solc"
    echo ""
    exit 1
fi

echo "solc version: $(solc --version | head -1)"
echo ""

# Create output directory
mkdir -p "$BYTECODE_DIR"

# Compile AssetToken
echo "Compiling AssetToken.sol..."
solc --bin --optimize --optimize-runs 200 \
    -o "$BYTECODE_DIR" --overwrite \
    "$CONTRACTS_DIR/src/AssetToken.sol"

# Rename output file (solc uses contract name)
if [ -f "$BYTECODE_DIR/AssetToken.bin" ]; then
    echo "  -> AssetToken.bin created"
else
    echo "  ERROR: AssetToken.bin not found"
    exit 1
fi

echo ""
echo "Compilation complete!"
echo "Bytecode files:"
ls -la "$BYTECODE_DIR"/*.bin 2>/dev/null || echo "  (none)"
