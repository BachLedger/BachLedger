# BachLedger - ChainMaker Implementation (Legacy)

This directory contains the original BachLedger implementation based on [ChainMaker](https://chainmaker.org.cn/).

> ⚠️ **Note**: This is a legacy implementation. The recommended implementation is the [Rust version](../rust/).

## Structure

```
chainmaker/
├── yzchain-go/         # Main blockchain node
├── chain-module/       # Core modules (submodules)
│   ├── chainconf/      # Chain configuration
│   ├── common/         # Common utilities
│   ├── consensus-*/    # Consensus implementations (TBFT, Raft, Solo)
│   ├── protocol/       # Protocol interfaces
│   ├── store/          # Storage layer
│   ├── txpool-*/       # Transaction pools
│   ├── vm/             # Virtual machine
│   ├── vm-evm/         # EVM implementation
│   └── ...
└── yzchain-cryptogen/  # Certificate generator

```

## Quick Start

### Prerequisites

- Go 1.18+
- Docker (for DockerVM contracts)

### Build

```bash
cd yzchain-go
make build
```

### Run Tests

```bash
# EVM contract tests
./test-all-evm.sh

# Go contract tests (requires Docker)
./test-all-dockervm.sh
```

## Features

- BachLedger Seamless Scheduling algorithm
- Support for EVM/Solidity contracts
- Support for Go contracts (via DockerVM)
- TBFT consensus

## References

- [ChainMaker Documentation](https://docs.chainmaker.org.cn/)
- [BachLedger Paper](../paper/)
