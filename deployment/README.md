# BachLedger Multi-Node Deployment

Docker-based deployment for the BachLedger Medical Blockchain network.

## Overview

This deployment creates a 4-node TBFT (Tendermint BFT) validator network suitable for development, testing, and production environments.

## Architecture

```
+------------------+    +------------------+
|     Node 1       |    |     Node 2       |
|  (Bootstrap)     |<-->|   (Validator)    |
|  RPC: 8545       |    |  RPC: 8547       |
|  P2P: 30303      |    |  P2P: 30304      |
+------------------+    +------------------+
        ^                       ^
        |                       |
        v                       v
+------------------+    +------------------+
|     Node 3       |    |     Node 4       |
|   (Validator)    |<-->|   (Validator)    |
|  RPC: 8549       |    |  RPC: 8551       |
|  P2P: 30305      |    |  P2P: 30306      |
+------------------+    +------------------+
```

## Quick Start

### Prerequisites

- Docker Engine 20.10+
- Docker Compose 2.0+
- 4GB RAM minimum (8GB recommended)
- 20GB disk space

### 1. Generate Validator Keys

```bash
# Generate keys for all 4 validators
./scripts/generate-keys.sh
```

This creates:
- `keys/validator1.key` through `keys/validator4.key`
- Updates `genesis.json` with validator public keys

### 2. Start the Network

```bash
# Start all nodes
docker-compose up -d

# View logs
docker-compose logs -f

# Check node status
docker-compose ps
```

### 3. Verify Network

```bash
# Check node 1 is syncing
curl http://localhost:8545 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Check peer count
curl http://localhost:8545 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}'
```

## Port Mapping

| Node   | RPC HTTP | RPC WS | P2P     |
|--------|----------|--------|---------|
| Node 1 | 8545     | 8546   | 30303   |
| Node 2 | 8547     | 8548   | 30304   |
| Node 3 | 8549     | 8550   | 30305   |
| Node 4 | 8551     | 8552   | 30306   |

## Configuration

### Environment Variables

| Variable           | Description                    | Default       |
|--------------------|--------------------------------|---------------|
| `NODE_ID`          | Unique node identifier         | Required      |
| `NODE_NAME`        | Human-readable node name       | bachledger    |
| `VALIDATOR_KEY_FILE` | Path to validator private key | Required      |
| `P2P_PORT`         | P2P listening port             | 30303         |
| `RPC_PORT`         | JSON-RPC HTTP port             | 8545          |
| `WS_PORT`          | JSON-RPC WebSocket port        | 8546          |
| `BOOTSTRAP_NODES`  | Comma-separated bootstrap nodes| -             |
| `LOG_LEVEL`        | Logging level                  | info          |
| `DATA_DIR`         | Data directory path            | /data         |
| `GENESIS_FILE`     | Genesis configuration file     | /config/genesis.json |

### Genesis Configuration

Edit `genesis.json` to customize:

- `chainId`: Network identifier (default: 31337)
- `consensus.blockTime`: Block interval in milliseconds
- `consensus.validators`: Initial validator set
- `alloc`: Pre-funded accounts

## Operations

### Stop Network

```bash
docker-compose down
```

### Reset Network (Clear Data)

```bash
docker-compose down -v
rm -rf data/*
docker-compose up -d
```

### Scale Validators

To add more validators:

1. Generate new validator key
2. Add node configuration to `docker-compose.yml`
3. Update `genesis.json` with new validator
4. Restart network

### View Logs

```bash
# All nodes
docker-compose logs -f

# Specific node
docker-compose logs -f node1

# Last 100 lines
docker-compose logs --tail=100 node1
```

### Enter Container Shell

```bash
docker-compose exec node1 /bin/bash
```

## Monitoring

### Health Checks

Each node exposes a health endpoint:

```bash
curl http://localhost:8545/health
```

### Metrics (Prometheus)

Metrics are available at:
```
http://localhost:8545/metrics
```

## Security Considerations

### Production Deployment

1. **Key Management**: Never commit validator keys to version control
2. **Network Isolation**: Use private networks and firewalls
3. **TLS**: Enable TLS for RPC endpoints
4. **Access Control**: Restrict RPC access to trusted IPs

### Key Generation

For production, use hardware security modules (HSM) or secure key management:

```bash
# Generate secure random key
openssl rand -hex 32 > validator.key
chmod 600 validator.key
```

## Troubleshooting

### Node Won't Start

1. Check logs: `docker-compose logs node1`
2. Verify key file exists and is readable
3. Ensure ports are not in use

### Nodes Not Connecting

1. Check bootstrap node is healthy
2. Verify network connectivity between containers
3. Check firewall rules

### Consensus Stalled

1. Verify at least 3 of 4 validators are online (2f+1)
2. Check validator keys are correct
3. Review consensus logs for errors

## Development

### Building Locally

```bash
# Build Docker image
docker build -t bachledger:dev -f deployment/Dockerfile ..

# Run single node for testing
docker run -it --rm \
  -p 8545:8545 \
  -p 30303:30303 \
  -v $(pwd)/data:/data \
  bachledger:dev
```

### Running Tests

```bash
# Integration tests against running network
cd ../rust
cargo test --package bach-node --test integration
```

## License

MIT License - See LICENSE file for details.
