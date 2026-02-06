# SDK Review Notes (Prepared by Reviewer)

**Status:** Preliminary - awaiting Task #3 (tests) completion for formal review

---

## 1. Security Fixes Verification

| Issue | Status | Location | Notes |
|-------|--------|----------|-------|
| S1: Unsafe pointer cast | FIXED | client.rs | Removed completely |
| S2: Clone on Wallet | FIXED | wallet.rs:16 | Documented why not implemented |
| S3: Zeroize | PARTIAL | - | k256::SigningKey has zeroize, but no explicit Drop impl |
| M1: .unwrap() in transport | OPEN | transport.rs:54,60,78,84 | Low priority, test-only code |

---

## 2. Test Coverage Analysis

### Total Tests: 79

#### By Module (inline tests):
- abi/decode: 10 tests
- abi/encode: 9 tests
- abi/types: 3 tests
- client: 13 tests
- contract: 8 tests
- transport: 3 tests
- tx_builder: 7 tests
- types: 3 tests
- wallet: 9 tests

#### External Test Files:
- client_tests.rs: 16 tests (mostly TODO stubs)
- wallet_tests.rs: 15 tests (mostly TODO stubs)
- signing_tests.rs: 20 tests (mostly TODO stubs)
- security_tests.rs: 13 tests (mostly TODO stubs)

### Coverage Gaps vs TEST_PLAN.md

#### P0 Critical (Must have):
| Test | Status | Notes |
|------|--------|-------|
| Key generation | OK | wallet::tests covers |
| Address derivation | OK | Known test vector verified |
| Transaction signing | OK | tx_builder tests |
| Signature verification | STUB | signing_tests.rs TODO |
| Private key security | PARTIAL | Debug test OK, leak tests TODO |

#### P1 High Priority:
| Test | Status | Notes |
|------|--------|-------|
| Legacy tx building | OK | |
| EIP-1559 tx building | OK | |
| RPC method wrappers | STUB | client_tests.rs TODO |
| Error handling | STUB | error_handling mod TODO |

#### P2 Medium Priority:
| Test | Status | Notes |
|------|--------|-------|
| EIP-2 low-s | STUB | signing_tests.rs TODO |
| Known test vectors | PARTIAL | One vector, need more |

---

## 3. Code Quality Review

### Positive Findings
- Clean module structure
- `#![warn(missing_docs)]` enabled
- Builder patterns used consistently
- No `unwrap()` in library code (only in tests and mock code)
- Error types well-defined with `thiserror`

### Issues

#### Minor (m1): Unused constant warning
- Location: `signing_tests.rs:67`
- Issue: `SECP256K1_N_DIV_2` declared but test not implemented
- Action: Implement test or remove constant

#### Minor (m2): Unused mut warning
- Location: `abi/decode.rs:191`
- Issue: `let mut encoded_false` doesn't need mut
- Action: Remove mut

#### Minor (m3): Missing From impl for some error types
- Would be nice: `impl From<std::io::Error> for SdkError`

---

## 4. API Usability Review

### Good
- TxBuilder fluent API is intuitive
- Contract helper with ERC20 preset
- Mock transport for easy testing
- Clear error messages

### Suggestions
- Add `TxBuilder::auto_nonce()` that fetches from client
- Add `TxBuilder::auto_gas()` that estimates from client
- Add `PendingTransaction::wait()` for receipt polling

---

## 5. Missing Tests (Required Before Approval)

### Critical (Block approval):
1. `test_sign_verify_roundtrip` - Must verify signatures work
2. `test_recover_address` - Must verify recovery works
3. `test_private_key_not_in_debug` - Security test

### High (Should have):
1. RPC mock tests for all client methods
2. Error path testing
3. Cross-chain replay protection

### Medium (Nice to have):
1. Property-based tests
2. More known test vectors
3. Malformed input fuzzing

---

## 6. Documentation Review

### Present:
- lib.rs has module docs with examples
- abi/mod.rs has usage example
- DESIGN.md comprehensive
- TEST_PLAN.md comprehensive

### Missing:
- CHANGELOG.md
- API reference (cargo doc should generate)

---

## 7. CLI Review Notes (for Task #7)

### Security Concern
- `--key` argument exposes private key in shell history
- Recommendation: Accept via `BACH_PRIVATE_KEY` env var or `--keyfile` only

### Missing Features
- No confirmation prompt before sending tx
- No `--yes` flag for scripting

### Positive
- Clean clap structure
- JSON output support
- Config file support

---

## 8. Action Items for Tester

Please prioritize implementing these test stubs:

1. **signing_tests.rs**
   - `test_sign_verify_roundtrip`
   - `test_recover_address`
   - `test_signature_low_s`

2. **security_tests.rs**
   - `test_private_key_not_in_debug`
   - `test_invalid_key_error_no_leak`

3. **client_tests.rs**
   - At least one test per RPC method

---

## 9. Review Checklist Status

Refer to `REVIEW_CHECKLIST.md` for full criteria.

### Must Pass Before Approval:
- [x] No `unwrap()` in library code
- [x] Custom Debug hides private key
- [x] Wallet doesn't implement Clone
- [ ] All P0 tests passing (awaiting implementation)
- [x] No clippy errors (only warnings)
- [x] Docs build without errors

---

*Prepared: Reviewer*
*Last Updated: Awaiting Task #3 completion*
