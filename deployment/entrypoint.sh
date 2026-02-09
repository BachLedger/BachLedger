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

# Data directory
ARGS="$ARGS --data-dir ${DATA_DIR:-/data}"

# Network configuration - listen address
LISTEN_ADDR="0.0.0.0:${P2P_PORT:-30303}"
ARGS="$ARGS --listen-addr $LISTEN_ADDR"

# RPC configuration
RPC_ADDR="0.0.0.0:${RPC_PORT:-8545}"
ARGS="$ARGS --rpc --rpc-addr $RPC_ADDR"

# Chain ID
ARGS="$ARGS --chain-id ${CHAIN_ID:-31337}"

# Block time
ARGS="$ARGS --block-time ${BLOCK_TIME:-3000}"

# Validator key
if [ -n "$VALIDATOR_KEY_FILE" ] && [ -f "$VALIDATOR_KEY_FILE" ]; then
    ARGS="$ARGS --validator-key $VALIDATOR_KEY_FILE"
    echo "Validator key loaded from: $VALIDATOR_KEY_FILE"
fi

# Bootstrap nodes
if [ -n "$BOOTSTRAP_NODES" ]; then
    ARGS="$ARGS --bootnodes $BOOTSTRAP_NODES"
    echo "Bootstrap nodes: $BOOTSTRAP_NODES"
fi

# Logging
ARGS="$ARGS --log-level ${LOG_LEVEL:-info}"

echo "Starting node with args: $ARGS"
echo "=========================================="

# Execute the node (global args before subcommand)
exec /app/bach-node $ARGS run "$@"
