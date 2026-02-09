# BachLedger

<p align="center">
  <img src="assets/bach.jpg" alt="Johann Sebastian Bach" width="180">
</p>

**BachLedger** is named after [Johann Sebastian Bach](https://en.wikipedia.org/wiki/Johann_Sebastian_Bach), the legendary composer renowned for his mastery of polyphony — the art of weaving multiple independent musical voices into a harmonious whole. Just as Bach orchestrated complex fugues where each voice moves independently yet contributes to a unified composition, BachLedger orchestrates parallel transaction execution where multiple transactions run concurrently while maintaining blockchain consistency.

BachLedger is a high-performance blockchain system that achieves this through **Seamless Scheduling** — dynamic dependency detection and cross-block transaction scheduling that maximizes parallel computing resource utilization.

## Project Structure

```
bachledger/
├── rust/                   # 🦀 Rust native implementation
│   ├── bach-primitives/    #    Core types (Address, H256, U256)
│   ├── bach-crypto/        #    Cryptographic operations
│   ├── bach-types/         #    Blockchain types
│   ├── bach-evm/           #    EVM interpreter
│   ├── bach-consensus/     #    TBFT consensus
│   ├── bach-network/       #    P2P networking
│   ├── bach-storage/       #    Persistent storage
│   ├── bach-rpc/           #    JSON-RPC server
│   ├── bach-node/          #    Full node binary
│   └── bach-contracts/     #    Smart contract templates
│
├── deployment/             # 🐳 Docker deployment
│   ├── docker-compose.yml  #    4-node network config
│   ├── Dockerfile          #    Node image
│   └── setup.sh            #    Key generation
│
├── docs/                   # 📚 Documentation
│   └── DEPLOYMENT_GUIDE.md #    Deployment & operation guide
│
├── contracts/              # 📜 Solidity contracts
│   ├── MedicalRecord.sol   #    Medical record management
│   ├── AccessControl.sol   #    Role-based access
│   └── AuditLog.sol        #    Compliance logging
│
├── paper/                  # 📄 Research paper (IEEE ICPADS 2024)
└── assets/                 #    Project assets
```

## Implementations

### Rust Implementation (Recommended)

A complete rewrite using Rust, featuring:

- **Native OEV Architecture**: Ordering-Execution-Validation pipeline built from scratch
- **Seamless Scheduling**: Core algorithm with Ownership Table and Priority Codes
- **TBFT Consensus**: Byzantine fault-tolerant consensus (n > 3f+1)
- **EVM Compatible**: Full Ethereum Virtual Machine support
- **JSON-RPC API**: Ethereum-compatible API interface
- **800+ Tests**: Comprehensive test coverage

#### Quick Start

```bash
# Build from source
cd rust
cargo build --release

# Or use Docker
docker pull youngyee/bachledger-node:latest
```

#### Run Single Node

```bash
./target/release/bach-node \
  --data-dir ./data \
  --rpc --rpc-addr 127.0.0.1:8545 \
  run
```

#### Run 4-Node Network (Docker)

```bash
cd deployment
./setup.sh           # Generate validator keys
docker compose up -d # Start network
```

📖 See [docs/DEPLOYMENT_GUIDE.md](docs/DEPLOYMENT_GUIDE.md) for detailed deployment and operation guide.

### ChainMaker Implementation (Legacy)

The original implementation based on [ChainMaker](https://chainmaker.org.cn/), supporting Go and EVM contracts.

```bash
cd chainmaker/yzchain-go
# Run tests
./test-all-evm.sh      # EVM contract tests
./test-all-dockervm.sh # Go contract tests
```

## AI-Generated Implementation Experiments

The Rust implementation of BachLedger is being developed through fully AI-generated coding experiments. **The code on `main` was primarily generated using the [ICDD (Interface-Contract-Driven Development)](.claude/skills/icdd/SKILL.md) skill** — a multi-agent TDD workflow that orchestrates isolated AI agents (Architect → Tester → Coder → Reviewers → Attacker) with strict role separation and interface-first design to produce production-quality, well-tested code.

| Branch | AI Method | Description |
|--------|-----------|-------------|
| **`main`** | **ICDD Skill** | Generated via `.claude/skills/icdd/` — multi-agent TDD with interface contracts, adversarial testing, and validator scripts |
| [`trial-1`](../../tree/trial-1) | Claude Code | Trial 1 — First all-AI generated implementation |

## Paper

This is the official implementation of the research paper published at **IEEE ICPADS 2024**:

> **BachLedger: Orchestrating Parallel Execution with Dynamic Dependency Detection and Seamless Scheduling**
>
> Yi Yang, Guangyong Shang, Guangpeng Qi, Zhen Ma, Yaxiong Liu, Jiazhou Tian, Aocheng Duan, Meng Zhang, Jingying Li, Xuan Ding
>
> *2024 IEEE 30th International Conference on Parallel and Distributed Systems (ICPADS)*
>
> DOI: [10.1109/ICPADS63350.2024.00087](https://doi.org/10.1109/ICPADS63350.2024.00087)

### Abstract

BachLedger addresses the performance bottleneck of sequential transaction execution in blockchain systems by introducing a novel approach that combines dynamic dependency detection with seamless scheduling. The system automatically identifies transaction dependencies at runtime and orchestrates concurrent execution while maintaining correctness and consistency, significantly improving blockchain throughput.

### Key Innovations

1. **Seamless Scheduling**: Utilizes idle thread time between blocks to execute subsequent block transactions in advance
2. **Ownership Table**: Fine-grained conflict detection with minimal lock contention
3. **Priority Codes**: Deterministic transaction ordering using semantic prefixes and hash-derived values

## Citation

If you use BachLedger in your research, please cite:

```bibtex
@inproceedings{yang2024bachledger,
  title={BachLedger: Orchestrating Parallel Execution with Dynamic Dependency Detection and Seamless Scheduling},
  author={Yang, Yi and Shang, Guangyong and Qi, Guangpeng and Ma, Zhen and Liu, Yaxiong and Tian, Jiazhou and Duan, Aocheng and Zhang, Meng and Li, Jingying and Ding, Xuan},
  booktitle={2024 IEEE 30th International Conference on Parallel and Distributed Systems (ICPADS)},
  year={2024},
  organization={IEEE},
  doi={10.1109/ICPADS63350.2024.00087}
}
```

## License

This project is licensed under the [Apache License 2.0](LICENSE).
