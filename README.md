# BachLedger

<p align="center">
  <img src="assets/bach.jpg" alt="Johann Sebastian Bach" width="180">
</p>

**BachLedger** is named after [Johann Sebastian Bach](https://en.wikipedia.org/wiki/Johann_Sebastian_Bach), the legendary composer renowned for his mastery of polyphony — the art of weaving multiple independent musical voices into a harmonious whole. Just as Bach orchestrated complex fugues where each voice moves independently yet contributes to a unified composition, BachLedger orchestrates parallel transaction execution where multiple transactions run concurrently while maintaining blockchain consistency.

BachLedger is a high-performance blockchain system that achieves this through dynamic dependency detection and seamless scheduling. It is developed based on [ChainMaker](https://chainmaker.org.cn/), an open-source blockchain platform.

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

BachLedger is developed based on [ChainMaker](https://github.com/chainmaker-io), which is also licensed under Apache License 2.0.
