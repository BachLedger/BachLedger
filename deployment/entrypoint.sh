#!/bin/bash
set -e

# BachLedger Node Entrypoint Script
# Configures and starts the blockchain node

echo "=========================================="
echo "BachLedger Medical Blockchain Node"
echo "=========================================="
echo "Node ID:    ${NODE_ID:-unknown}"
echo "Node Name:  ${NODE_NAME:-bachledger-node}"
echo "Data Dir:   ${DATA_DIR:-/data}"
echo "P2P Port:   ${P2P_PORT:-30303}"
echo "RPC Port:   ${RPC_PORT:-8545}"
echo "=========================================="

# Build command line arguments
ARGS=""

# Node identity
if [ -n "$NODE_ID" ]; then
    ARGS="$ARGS --node-id $NODE_ID"
fi

if [ -n "$NODE_NAME" ]; then
    ARGS="$ARGS --node-name $NODE_NAME"
fi

# Validator key
if [ -n "$VALIDATOR_KEY_FILE" ] && [ -f "$VALIDATOR_KEY_FILE" ]; then
    ARGS="$ARGS --validator-key $VALIDATOR_KEY_FILE"
    echo "Validator key loaded from: $VALIDATOR_KEY_FILE"
fi

# Data directory
ARGS="$ARGS --data-dir ${DATA_DIR:-/data}"

# Genesis file
if [ -n "$GENESIS_FILE" ] && [ -f "$GENESIS_FILE" ]; then
    ARGS="$ARGS --genesis $GENESIS_FILE"
    echo "Genesis file: $GENESIS_FILE"
fi

# Network configuration
ARGS="$ARGS --p2p-port ${P2P_PORT:-30303}"
ARGS="$ARGS --rpc-port ${RPC_PORT:-8545}"

if [ -n "$WS_PORT" ]; then
    ARGS="$ARGS --ws-port $WS_PORT"
fi

# Bootstrap nodes
if [ -n "$BOOTSTRAP_NODES" ]; then
    ARGS="$ARGS --bootnodes $BOOTSTRAP_NODES"
    echo "Bootstrap nodes: $BOOTSTRAP_NODES"
fi

# Logging
ARGS="$ARGS --log-level ${LOG_LEVEL:-info}"

# Enable RPC APIs
ARGS="$ARGS --rpc-apis eth,net,web3"

# CORS (allow all for development)
ARGS="$ARGS --rpc-cors-origins '*'"

echo "Starting node with args: $ARGS"
echo "=========================================="

# Execute the node
exec /app/bach-node $ARGS "$@"
