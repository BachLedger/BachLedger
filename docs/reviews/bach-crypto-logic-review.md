# Logic Review: bach-crypto

## Summary

| Category | Status | Issues |
|----------|--------|--------|
| Stub Detection | PASS | 0 |
| Unused Code | PASS | 0 |
| Hardcoded Returns | PASS | 0 |
| Unwrap Abuse | MINOR | 5 |
| Interface Drift | PASS | 0 |
| Logic Correctness | PASS | 0 |
| Security | PASS | 0 |

**Overall**: APPROVED

## Revision History

| Date | Verdict | Notes |
|------|---------|-------|
| 2026-02-09 | NEEDS_REVISION | Initial review - double-hashing issue |
| 2026-02-09 | APPROVED | Fix verified (commit f0d495e) - prehash methods now used |

## Fixed Issue (Previously MAJOR)

### Issue #1: Message Hashing - RESOLVED
- **Status**: FIXED in commit f0d495e
- **Original Problem**: Code applied extra Keccak256 hash during sign/verify/recover
- **Fix Applied**: Now uses prehash variants:
  - Line 85: `sign_prehash_recoverable(message.as_bytes())`
  - Line 241: `verify_prehash(message.as_bytes(), &k256_sig)`
  - Line 259: `recover_from_prehash(message.as_bytes(), &k256_sig, recovery_id)`
- Added import: `use k256::ecdsa::signature::hazmat::PrehashVerifier;`

## Minor Issues (Non-blocking)

### Issue #2: Unwrap in `Signature::from_bytes` (safe)
- **File**: line 208-209
- **Severity**: MINOR
- **Description**: `try_into().unwrap()` for array conversion from slice
- **Justification**: Safe - slices are exactly 32 bytes by construction

### Issue #3: Unwrap in `Signature::verify` (safe)
- **File**: lines 230-231
- **Severity**: MINOR
- **Description**: Same `try_into().unwrap()` pattern
- **Justification**: Safe - extracting from fixed-size internal array

### Issue #4: Unwrap in `Signature::recover` (safe)
- **File**: lines 246-247
- **Severity**: MINOR
- **Description**: Same pattern
- **Justification**: Safe for same reason

### Issue #5: Unwrap in `Signature::r()` and `Signature::s()`
- **File**: lines 267, 272
- **Severity**: MINOR
- **Description**: `try_into().unwrap()` for returning references
- **Justification**: Safe - internal array is always 65 bytes

### Issue #6: Expect in `PrivateKey::sign`
- **File**: line 85-86
- **Severity**: MINOR
- **Description**: Uses `.expect("signing should not fail with valid key")`
- **Justification**: Acceptable - signing key validity enforced by construction

## Interface Drift Analysis

### Constants
- `SIGNATURE_LENGTH: usize = 65` - **matches contract**

### CryptoError
- `InvalidPrivateKey` - **matches**
- `InvalidSignature` - **matches**
- `RecoveryFailed` - **matches**
- `InvalidPublicKey` - **matches**

### Functions
- `keccak256(data: &[u8]) -> H256` - **matches**
- `keccak256_concat(data: &[&[u8]]) -> H256` - **matches**

### PrivateKey
- `random() -> Self` - **matches**, uses `OsRng` (secure OS entropy)
- `from_bytes(&[u8; 32]) -> Result<Self, CryptoError>` - **matches**
- `to_bytes() -> [u8; 32]` - **matches**
- `public_key() -> PublicKey` - **matches**
- `sign(&self, message: &H256) -> Signature` - **matches**
- `Debug` impl redacts bytes - **matches security requirement**

### PublicKey
- `from_bytes(&[u8; 64]) -> Result<Self, CryptoError>` - **matches**, validates point
- `to_bytes() -> [u8; 64]` - **matches**
- `to_address() -> Address` - **matches**, correct `keccak256(pubkey)[12..32]`
- `verify(&self, signature: &Signature, message: &H256) -> bool` - **matches**
- `Debug`, `Clone`, `PartialEq`, `Eq` - **matches**

### Signature
- `from_bytes(&[u8; 65]) -> Result<Self, CryptoError>` - **matches**
- `to_bytes() -> [u8; 65]` - **matches**
- `verify(&self, pubkey: &PublicKey, message: &H256) -> bool` - **matches**
- `recover(&self, message: &H256) -> Result<PublicKey, CryptoError>` - **matches**
- `r() -> &[u8; 32]` - **matches**
- `s() -> &[u8; 32]` - **matches**
- `v() -> u8` - **matches** (returns 27 or 28)
- `Debug`, `Clone`, `PartialEq`, `Eq` - **matches**

## Security Analysis

### Key Generation
**Status: PASS**
- Uses `rand_core::OsRng` which provides cryptographically secure randomness from OS
- No seeding with predictable values

### Signature Validation
**Status: PASS**
- `from_bytes` validates r and s are non-zero (lines 197-199)
- Validates v is 27 or 28 (lines 202-205)
- Validates r,s are valid scalars via `K256Signature::from_scalars` (lines 210-213)
- Strict validation prevents malleability attacks

### Timing Side-Channels
**Status: PASS**
- Uses k256 library which implements constant-time operations
- No manual byte comparisons in critical paths
- `PartialEq` for signatures compares all bytes (standard array comparison)

### Key Material Handling
**Status: PASS**
- `PrivateKey::Debug` redacts key bytes (line 95)
- No logging of secret material

## Logic Correctness

### keccak256
- Uses `sha3::Keccak256` - correct algorithm
- Properly handles empty input (will produce standard empty hash)
- Returns correct H256 type

### keccak256_concat
- Iteratively updates hasher - correct concatenation semantics
- Equivalent to `keccak256(a || b || c || ...)`

### PublicKey::to_address
- `keccak256(public_key)[12..32]` - **correct Ethereum address derivation**
- Takes last 20 bytes of 32-byte hash

### Signature::from_k256_signature
- Correctly extracts r,s to bytes[0..64]
- Correctly converts recovery_id to Ethereum v (adds 27)

### Signature Recovery
- Correctly subtracts 27 from v to get recovery_id (line 255)
- Uses `saturating_sub` to prevent underflow

## Positive Observations

1. **Proper error handling**: All fallible operations return `Result` with appropriate error types
2. **Strict validation**: Signatures validated for non-zero r,s and valid v
3. **Secure randomness**: Uses OS-provided entropy via `OsRng`
4. **Debug safety**: Private key bytes redacted in Debug output
5. **Clean architecture**: Wraps k256 types appropriately
6. **Point validation**: `PublicKey::from_bytes` validates the point is on the curve
7. **Ethereum compatibility**: v values use 27/28 convention
8. **Correct prehash usage**: Sign/verify/recover use prehash variants for proper message handling

## Conclusion

The bach-crypto implementation is complete, correct, and secure. The initial double-hashing issue has been fixed - the code now correctly uses prehash variants (`sign_prehash_recoverable`, `verify_prehash`, `recover_from_prehash`) that accept the pre-computed message hash directly without additional hashing. This ensures Ethereum compatibility and interoperability with standard tooling.

---

**Reviewer**: Reviewer-Logic
**Date**: 2026-02-09
**Verdict**: APPROVED
