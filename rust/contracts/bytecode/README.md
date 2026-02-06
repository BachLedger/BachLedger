# Contract Bytecode

This directory contains compiled Solidity bytecode for deployment.

## Compilation

To compile contracts, run:

```bash
./scripts/compile_contracts.sh
```

This requires `solc` (Solidity compiler) to be installed.

## Files

- `AssetToken.bin` - Compiled bytecode for AssetToken ERC-20 contract

## Manual Compilation

If the script doesn't work, you can compile manually:

```bash
# Install solc (macOS)
brew install solidity

# Compile
solc --bin --optimize --optimize-runs 200 \
    -o contracts/bytecode --overwrite \
    contracts/src/AssetToken.sol
```

## Online Compilation

Alternatively, use Remix IDE (https://remix.ethereum.org/):
1. Copy `AssetToken.sol` content to Remix
2. Compile with optimization enabled (200 runs)
3. Copy the bytecode (without 0x prefix)
4. Paste into `AssetToken.bin`
