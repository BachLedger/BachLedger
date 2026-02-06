# BachLedger

<p align="center">
  <img src="assets/bach.jpg" alt="Johann Sebastian Bach" width="180">
</p>

**BachLedger** is named after [Johann Sebastian Bach](https://en.wikipedia.org/wiki/Johann_Sebastian_Bach), the legendary composer renowned for his mastery of polyphony — the art of weaving multiple independent musical voices into a harmonious whole. Just as Bach orchestrated complex fugues where each voice moves independently yet contributes to a unified composition, BachLedger orchestrates parallel transaction execution where multiple transactions run concurrently while maintaining blockchain consistency.

BachLedger is a high-performance blockchain system that achieves this through **Seamless Scheduling** — dynamic dependency detection and cross-block transaction scheduling that maximizes parallel computing resource utilization.

## Project Structure

```
bachledger/
├── rust/                   # 🦀 Rust native implementation (NEW)
│   ├── crates/             #    Modular crate workspace
│   └── docs/PLAN.md        #    Implementation plan
│
├── chainmaker/             # 🔗 ChainMaker-based implementation (Legacy)
│   ├── yzchain-go/         #    Main blockchain node
│   ├── chain-module/       #    Core modules
│   └── yzchain-cryptogen/  #    Certificate generator
│
├── paper/                  # 📄 Research paper (IEEE ICPADS 2024)
└── assets/                 #    Project assets
```

## Implementations

### Rust Implementation (Recommended)

A complete rewrite using Rust, featuring:

- **Native OEV Architecture**: Ordering-Execution-Validation pipeline built from scratch
- **Seamless Scheduling**: Core algorithm with Ownership Table and Priority Codes
- **Minimal Dependencies**: Pure Rust implementation where possible
- **EVM Compatible**: Full Solidity smart contract support

```bash
cd rust
cargo build --release
```

📖 See [rust/docs/PLAN.md](rust/docs/PLAN.md) for detailed design and roadmap.

### ChainMaker Implementation (Legacy)

The original implementation based on [ChainMaker](https://chainmaker.org.cn/), supporting Go and EVM contracts.

```bash
cd chainmaker/yzchain-go
# Run tests
./test-all-evm.sh      # EVM contract tests
./test-all-dockervm.sh # Go contract tests
```

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
