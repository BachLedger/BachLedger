# Logic Review: bach-crypto

## Summary

| Category | Status | Issues |
|----------|--------|--------|
| Stub Detection | PASS | 0 |
| Unused Code | PASS | 0 |
| Hardcoded Returns | PASS | 0 |
| Unwrap Abuse | MINOR | 5 |
| Interface Drift | PASS | 0 |
| Logic Correctness | MAJOR | 1 |
| Security | PASS | 0 |

**Overall**: NEEDS_REVISION

## Issues

### Issue #1: Incorrect Message Hashing in Sign/Verify/Recover
- **File**: `/Users/moonshot/dev/working/bachledger/rust/bach-crypto/src/lib.rs`
- **Lines**: 87, 243, 261
- **Severity**: MAJOR
- **Description**: The code applies an extra Keccak256 hash to the message during sign/verify/recover operations. The interface contract states that `message` is already a "32-byte message hash (NOT the raw message)", but the implementation uses `Keccak256::new_with_prefix(message.as_bytes())` which hashes the already-hashed message again.

  This results in:
  - Signing: `sign(H(data))` actually signs `H(H(data))`
  - Verifying: `verify(H(data))` verifies against `H(H(data))`
  - Recovery: `recover(H(data))` recovers from `H(H(data))`

  While internally consistent (sign/verify/recover all apply the same double-hash), this deviates from the standard Ethereum signing behavior where the message hash is signed directly without additional hashing.

- **Fix**: Use the k256 `prehash` variants that accept the raw 32-byte hash directly without additional hashing:
  ```rust
  // For signing:
  let (sig, recovery_id) = self.inner.sign_prehash_recoverable(message.as_bytes())
      .expect("signing should not fail with valid key");

  // For verifying:
  use k256::ecdsa::signature::hazmat::PrehashVerifier;
  verifying_key.verify_prehash(message.as_bytes(), &k256_sig).is_ok()

  // For recovery:
  VerifyingKey::recover_from_prehash(message.as_bytes(), &k256_sig, recovery_id)
  ```

### Issue #2: Unwrap in `Signature::from_bytes` (safe)
- **File**: line 211-212
- **Severity**: MINOR
- **Description**: `try_into().unwrap()` for array conversion from slice
- **Justification**: These unwraps are safe - the slices are exactly 32 bytes by construction
- **Recommendation**: Consider using array indexing pattern or explicit match for documentation

### Issue #3: Unwrap in `Signature::verify` (safe)
- **File**: lines 233-234
- **Severity**: MINOR
- **Description**: Same `try_into().unwrap()` pattern
- **Justification**: Safe - extracting from fixed-size internal array

### Issue #4: Unwrap in `Signature::recover` (safe)
- **File**: lines 249-250
- **Severity**: MINOR
- **Description**: Same pattern
- **Justification**: Safe for same reason

### Issue #5: Unwrap in `Signature::r()` and `Signature::s()`
- **File**: lines 270, 275
- **Severity**: MINOR
- **Description**: `try_into().unwrap()` for returning references
- **Justification**: Safe - internal array is always 65 bytes

### Issue #6: Expect in `PrivateKey::sign`
- **File**: line 88-89
- **Severity**: MINOR
- **Description**: Uses `.expect("signing should not fail with valid key")`
- **Justification**: This is acceptable - if the signing key is valid (enforced by construction), signing cannot fail. The message provides clear context.

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
- `sign(&self, message: &H256) -> Signature` - **matches signature, logic issue noted**
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
- `from_bytes` validates r and s are non-zero (line 200-202)
- Validates v is 27 or 28 (lines 206-208)
- Validates r,s are valid scalars via `K256Signature::from_scalars` (lines 213-216)
- Strict validation prevents malleability attacks

### Timing Side-Channels
**Status: PASS**
- Uses k256 library which implements constant-time operations
- No manual byte comparisons in critical paths
- `PartialEq` for signatures compares all bytes (standard array comparison)

### Key Material Handling
**Status: PASS**
- `PrivateKey::Debug` redacts key bytes (line 98)
- No logging of secret material
- Note: Does not implement `Zeroize` trait (mentioned in contract but not required)

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
- Correctly subtracts 27 from v to get recovery_id (line 258)
- Uses `saturating_sub` to prevent underflow

## Positive Observations

1. **Proper error handling**: All fallible operations return `Result` with appropriate error types
2. **Strict validation**: Signatures validated for non-zero r,s and valid v
3. **Secure randomness**: Uses OS-provided entropy via `OsRng`
4. **Debug safety**: Private key bytes redacted in Debug output
5. **Clean architecture**: Wraps k256 types appropriately
6. **Point validation**: `PublicKey::from_bytes` validates the point is on the curve
7. **Ethereum compatibility**: v values use 27/28 convention

## Conclusion

The bach-crypto implementation is well-structured with proper error handling and security practices. However, there is one **MAJOR issue** with the message hashing - the code applies an extra Keccak256 hash during sign/verify/recover operations. While internally consistent (all operations apply the same extra hash), this deviates from standard Ethereum signing behavior.

**Recommendation**: Fix the message hashing to use prehash variants that accept the 32-byte hash directly without additional hashing. This is required for interoperability with Ethereum tooling and wallets.

---

**Reviewer**: Reviewer-Logic
**Date**: 2026-02-09
**Verdict**: NEEDS_REVISION (1 MAJOR issue)
