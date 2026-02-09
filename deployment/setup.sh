#!/bin/bash
set -e

# BachLedger Multi-Node Deployment Setup Script
# Generates validator keys and prepares the deployment environment

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================="
echo "BachLedger Multi-Node Deployment Setup"
echo "=========================================="

# Create directories
echo "Creating directories..."
mkdir -p keys data/node1 data/node2 data/node3 data/node4

# Check if bach-node binary exists, build if not
BACH_NODE_BIN="../rust/target/release/bach-node"
if [ ! -f "$BACH_NODE_BIN" ]; then
    echo "Building bach-node binary..."
    cd ../rust
    cargo build --release --package bach-node
    cd "$SCRIPT_DIR"
fi

# Generate validator keys
echo ""
echo "Generating validator keys..."

for i in 1 2 3 4; do
    KEY_FILE="keys/validator${i}.key"
    if [ ! -f "$KEY_FILE" ]; then
        echo "Generating key for validator $i..."
        $BACH_NODE_BIN gen-key --output "$KEY_FILE" 2>/dev/null
        echo "  Created: $KEY_FILE"
    else
        echo "  Key already exists: $KEY_FILE"
    fi
done

echo ""
echo "=========================================="
echo "Validator Keys Generated:"
echo "=========================================="

for i in 1 2 3 4; do
    KEY_FILE="keys/validator${i}.key"
    if [ -f "$KEY_FILE" ]; then
        echo ""
        echo "Validator $i:"
        KEY_HEX=$(cat "$KEY_FILE")
        # Use openssl to derive address (simplified - in production use proper tooling)
        echo "  Private Key: ${KEY_HEX:0:16}...${KEY_HEX: -16}"
    fi
done

echo ""
echo "=========================================="
echo "Setup Complete!"
echo "=========================================="
echo ""
echo "To start the 4-node network:"
echo "  docker compose up -d"
echo ""
echo "To view logs:"
echo "  docker compose logs -f"
echo ""
echo "To stop the network:"
echo "  docker compose down"
echo ""
echo "RPC Endpoints:"
echo "  Node 1: http://localhost:8545"
echo "  Node 2: http://localhost:8547"
echo "  Node 3: http://localhost:8549"
echo "  Node 4: http://localhost:8551"
echo ""
