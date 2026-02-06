# Asset Token Contract Design Document

## Overview

This document defines the design for an on-chain asset trading contract on BachLedger, exposing mint, burn, and transfer interfaces.

**Contract Type**: Fully decentralized, permissionless ERC-20 token with open minting.

---

## User Confirmed Decisions

| Question | Decision | Selected Option |
|----------|----------|-----------------|
| Q1: Token Standard | **Full ERC-20** | A |
| Q2: Mint Permission | **Open minting (anyone can mint)** | C |
| Q3: Burn Permission | **Self-burn only** | A |
| Q4: Max Supply | **Unlimited** | A |
| Q5: Token Metadata | name="AssetToken", symbol="AST", decimals=18 | Confirmed |
| Q6: Security Features | **None** (no Ownable/Pausable/ReentrancyGuard) | None |
| Q7: Initial Supply | **No initial supply** | A |

---

## Final Design Specification

### Contract Characteristics

- **Standard**: Full ERC-20 compliant
- **Permissioning**: Completely permissionless
  - Anyone can mint tokens to any address
  - Users can only burn their own tokens
  - No owner, no admin, no special roles
- **Supply**: Unlimited, no cap
- **Initial State**: Zero total supply at deployment

### Interface Definition

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IAssetToken {
    // ERC-20 Standard (read)
    function name() external view returns (string memory);
    function symbol() external view returns (string memory);
    function decimals() external view returns (uint8);
    function totalSupply() external view returns (uint256);
    function balanceOf(address account) external view returns (uint256);
    function allowance(address owner, address spender) external view returns (uint256);

    // ERC-20 Standard (write)
    function transfer(address to, uint256 amount) external returns (bool);
    function approve(address spender, uint256 amount) external returns (bool);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);

    // Extended: Mint (permissionless)
    function mint(address to, uint256 amount) external;

    // Extended: Burn (self only)
    function burn(uint256 amount) external;

    // Events
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
}
```

### State Variables

```solidity
// Metadata (immutable after deployment)
string public constant name = "AssetToken";
string public constant symbol = "AST";
uint8 public constant decimals = 18;

// Supply tracking
uint256 private _totalSupply;

// Balances and allowances
mapping(address => uint256) private _balances;
mapping(address => mapping(address => uint256)) private _allowances;
```

### Function Specifications

#### `mint(address to, uint256 amount)`
- **Access**: Public, anyone can call
- **Behavior**: Creates `amount` tokens and assigns to `to`
- **Validation**: `to != address(0)`
- **Events**: `Transfer(address(0), to, amount)`

#### `burn(uint256 amount)`
- **Access**: Public, caller burns their own tokens
- **Behavior**: Destroys `amount` tokens from `msg.sender`
- **Validation**: `_balances[msg.sender] >= amount`
- **Events**: `Transfer(msg.sender, address(0), amount)`

#### `transfer(address to, uint256 amount)`
- **Access**: Public
- **Behavior**: Transfers `amount` from `msg.sender` to `to`
- **Validation**: `to != address(0)`, `_balances[msg.sender] >= amount`
- **Events**: `Transfer(msg.sender, to, amount)`

#### `approve(address spender, uint256 amount)`
- **Access**: Public
- **Behavior**: Sets allowance for `spender` to spend `msg.sender`'s tokens
- **Events**: `Approval(msg.sender, spender, amount)`

#### `transferFrom(address from, address to, uint256 amount)`
- **Access**: Public
- **Behavior**: Transfers `amount` from `from` to `to` using allowance
- **Validation**: `to != address(0)`, `_balances[from] >= amount`, `_allowances[from][msg.sender] >= amount`
- **Events**: `Transfer(from, to, amount)`, updates allowance

---

## Risk Analysis (Updated for Permissionless Design)

### Risk 1: Unlimited Minting (ACCEPTED)
- **Threat**: Anyone can mint unlimited tokens
- **Impact**: Token has no scarcity, no monetary value
- **Status**: **Accepted by design** - this is intentional for the use case
- **Note**: This token is NOT suitable for value storage; suitable for testing, faucets, or utility tokens

### Risk 2: Reentrancy Attack
- **Threat**: Malicious contract calls back during transfer
- **Likelihood**: Low (no external calls in implementation)
- **Mitigation**: Checks-Effects-Interactions pattern; no external calls in transfer/mint/burn

### Risk 3: Integer Overflow/Underflow
- **Threat**: Arithmetic errors in balance calculations
- **Likelihood**: Low
- **Mitigation**: Solidity ^0.8.20 automatic overflow checks

### Risk 4: Approval Front-Running
- **Threat**: Approve race condition
- **Likelihood**: Medium
- **Mitigation**: Document safe approval pattern (set to 0 first, then new value)

### Risk 5: Zero Address Operations
- **Threat**: Tokens sent to 0x0 address
- **Mitigation**: Require checks on `to` address in transfer/mint

---

## Event Definitions

```solidity
// Standard ERC-20 Events only (no custom events needed)
event Transfer(address indexed from, address indexed to, uint256 value);
event Approval(address indexed owner, address indexed spender, uint256 value);
```

Note: Mint emits `Transfer(address(0), to, amount)`, Burn emits `Transfer(from, address(0), amount)` per ERC-20 convention.

---

## Implementation Notes

### BachLedger Compatibility
- BachLedger uses `bach-evm` (Yellow Paper compliant)
- Supports Solidity ^0.8.x compiled bytecode
- Deploy via `bach-cli tx deploy` or `bach-sdk`
- Interact via `bach-rpc` (eth_call, eth_sendRawTransaction)

### Testing Strategy
- Unit tests via `bach-e2e` harness
- Test cases:
  - Happy path: mint, transfer, burn
  - Edge cases: zero amounts, self-transfer
  - Overflow: large amount minting (near uint256 max)
  - Allowance: approve, transferFrom, over-spend rejection

### Gas Optimization
- Use `constant` for metadata (name, symbol, decimals)
- Minimize storage writes
- No access control checks = lower gas

---

## Summary

This is a **fully permissionless ERC-20 token** where:
- Anyone can mint any amount to any address
- Users can only burn their own tokens
- No admin, no owner, no special privileges
- No supply cap
- Standard ERC-20 transfer/approve/transferFrom functionality

**Use cases**: Testing, faucets, utility tokens, experimental applications.

**NOT suitable for**: Value storage, DeFi collateral, anything requiring scarcity.

---

## Next Steps

1. ~~User confirms design decisions (Q1-Q7)~~ DONE
2. ~~Architect finalizes contract specification~~ DONE
3. Developer implements Solidity contract (Task #2)
4. Tester creates test cases (Task #3)
5. Deploy and verify on BachLedger
