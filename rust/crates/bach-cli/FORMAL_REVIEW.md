# Formal CLI Review Report

**Reviewer:** reviewer
**Date:** 2026-02-06
**Task:** #7 - Review CLI implementation
**Verdict:** APPROVED with security recommendation

---

## Executive Summary

The bach-cli crate is **approved for merge** with one security recommendation that should be addressed before production deployment. The CLI demonstrates good code quality and usability. The security concern is acceptable for the current demo/development phase.

---

## Test Results

- **Total Tests:** 31 passing
- **Clippy:** No warnings
- **Build:** Clean compilation

---

## Security Review

### Passed

| Criterion | Status | Notes |
|-----------|--------|-------|
| Private key not in output | PASS | `test_private_key_not_in_output` verified |
| Error doesn't leak key | PASS | `test_error_no_key_leak` verified |
| Invalid key handled | PASS | Returns error, no panic |
| No unwrap in library code | PASS | All unwrap in tests |

### Security Recommendation (Major - Not Blocking)

**Issue:** Private keys accepted via `--key` CLI argument

**Location:** `src/commands/tx.rs` lines 22, 40, 60

**Problem:** CLI arguments appear in shell history (`~/.bash_history`, `~/.zsh_history`), process listings (`ps aux`), and may be logged by system auditing tools.

**Current behavior:**
```bash
bach tx send --to 0x... --amount 1.0 --key 0xac0974bec39a...
# Key is now in shell history!
```

**Recommended behavior:**
```bash
# Option 1: Environment variable
BACH_PRIVATE_KEY=0xac0974... bach tx send --to 0x... --amount 1.0

# Option 2: Keyfile
bach tx send --to 0x... --amount 1.0 --keyfile ~/.bachledger/keystore/my-wallet.json

# Option 3: Interactive prompt
bach tx send --to 0x... --amount 1.0
Enter private key: ********
```

**Accepted for now because:**
1. This is a demo/development tool
2. Code comments note "for demo purposes" (tx.rs:20)
3. The warning is printed (account.rs:73)
4. No production deployment planned yet

**Must fix before production.**

---

## Code Quality

| Criterion | Status | Notes |
|-----------|--------|-------|
| Clean clap structure | PASS | Well-organized subcommands |
| JSON output support | PASS | Global `--json` flag |
| Error handling | PASS | Proper error types |
| Config file support | PASS | TOML config in ~/.bachledger/ |
| Help text | PASS | All commands documented |

---

## Usability Review

### Strengths

1. **Intuitive command structure**
   ```
   bach account create/list/balance/import
   bach tx send/deploy/call
   bach query block/tx/chain-id/gas-price
   bach config --show/--set-rpc/--set-chain-id
   ```

2. **Flexible output**
   - Human-readable by default
   - JSON with `--json` flag
   - Proper exit codes (0 success, 1 error)

3. **Good defaults**
   - Default RPC: `http://localhost:8545`
   - Default chain ID: 1
   - Default gas limit: 21000

### Minor Recommendations (not blocking)

1. Add confirmation prompt before `tx send`
2. Add `--yes/-y` flag to skip confirmation (for scripting)
3. Support `BACH_RPC_URL` environment variable
4. Add `--verbose/-v` flag for debug output

---

## Files Reviewed

| File | Lines | Status |
|------|-------|--------|
| src/main.rs | 161 | APPROVED |
| src/error.rs | 44 | APPROVED |
| src/config.rs | 133 | APPROVED |
| src/output.rs | 100+ | APPROVED |
| src/commands/account.rs | 214 | APPROVED |
| src/commands/tx.rs | 287 | APPROVED (with security note) |
| src/commands/query.rs | 150+ | APPROVED |
| tests/cli_tests.rs | 330+ | APPROVED |

---

## Issues Summary

### Critical: 0

### Major: 1 (Not blocking for demo phase)

**M1: Private key via CLI argument**
- See Security Recommendation above
- Must be addressed before production

### Minor: 2

**m1: Missing confirmation prompt**
- Transactions execute without confirmation
- Could accidentally send funds

**m2: No env var override for RPC URL**
- Only supports config file and `--rpc-url` flag
- Should support `BACH_RPC_URL` for CI/CD

---

## Conclusion

The bach-cli crate is suitable for demo and development use:
- Clean, well-organized code
- Good error handling
- Comprehensive tests (31 passing)
- Proper security for error messages

The private key handling via CLI argument is acceptable for the demo phase but must be addressed before any production deployment.

**Verdict: APPROVED for demo/development use**

---

*Signed: reviewer*
