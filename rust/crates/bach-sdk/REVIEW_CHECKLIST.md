# BachLedger SDK Review Checklist

This checklist ensures consistent, thorough code reviews for the SDK and CLI crates.

---

## 1. Code Quality

### Error Handling
- [ ] No `unwrap()` in library code (except tests)
- [ ] No `expect()` in library code (except tests with descriptive messages)
- [ ] All errors use custom error types (not `String` or `Box<dyn Error>`)
- [ ] Error types implement `std::error::Error`
- [ ] Error messages are actionable and user-friendly
- [ ] `?` operator used consistently for error propagation
- [ ] No silent error swallowing (empty `catch` blocks)

### Documentation
- [ ] All public types have doc comments (`///`)
- [ ] All public functions have doc comments with:
  - [ ] Brief description
  - [ ] `# Arguments` section (if applicable)
  - [ ] `# Returns` section
  - [ ] `# Errors` section (for fallible functions)
  - [ ] `# Examples` section (for key APIs)
- [ ] Module-level documentation (`//!`) explaining purpose
- [ ] `#![warn(missing_docs)]` enabled in lib.rs

### Naming & Style
- [ ] Types use PascalCase
- [ ] Functions and variables use snake_case
- [ ] Constants use SCREAMING_SNAKE_CASE
- [ ] No single-letter variable names (except iterators: `i`, `j`, `k`)
- [ ] Names are descriptive and domain-appropriate
- [ ] Acronyms follow Rust conventions (e.g., `HttpClient`, not `HTTPClient`)

### Code Organization
- [ ] One concept per module
- [ ] Public API is minimal and well-defined
- [ ] Implementation details are private
- [ ] Re-exports in lib.rs for public API
- [ ] Logical file/module structure

### Rust Idioms
- [ ] Use `impl Trait` where appropriate
- [ ] Prefer `&str` over `&String` in function parameters
- [ ] Prefer `&[T]` over `&Vec<T>` in function parameters
- [ ] Use `Option` for optional values (not sentinel values)
- [ ] Use `Result` for fallible operations
- [ ] Implement standard traits where appropriate (`Clone`, `Debug`, `Default`, etc.)
- [ ] Use `#[derive]` macros appropriately
- [ ] No unnecessary `clone()` calls

---

## 2. Security

### Private Key Handling
- [ ] Private keys NEVER appear in:
  - [ ] Log messages
  - [ ] Error messages
  - [ ] Debug output (`Debug` impl)
  - [ ] Display output (`Display` impl)
- [ ] Private keys are zeroized on drop (use `zeroize` crate)
- [ ] Private key types do not implement `Clone` (prevent accidental copies)
- [ ] Private keys loaded from secure sources only (env vars, files with proper permissions)

### Signing Security
- [ ] Signatures use constant-time comparison
- [ ] No timing side-channels in cryptographic operations
- [ ] Proper nonce generation (if applicable)
- [ ] Chain ID included in signing to prevent replay attacks

### Input Validation
- [ ] All public API inputs validated before use
- [ ] Address format validated (length, checksum if applicable)
- [ ] Transaction parameters validated (gas limits, values)
- [ ] Hex string parsing handles malformed input gracefully
- [ ] Integer overflow checked for arithmetic operations
- [ ] No panics on invalid user input

### Network Security
- [ ] TLS used for RPC connections (configurable)
- [ ] No credentials in URLs or logs
- [ ] Request timeouts configured
- [ ] Response size limits enforced

---

## 3. API Usability

### Builder Pattern
- [ ] Complex types use builder pattern
- [ ] Builders have sensible defaults
- [ ] Required fields enforced at compile time (if possible)
- [ ] Builder methods are chainable (return `&mut Self` or `Self`)

### Defaults
- [ ] `Default` implemented where appropriate
- [ ] Default values are documented
- [ ] Default gas limit is reasonable (21000 for transfers)
- [ ] Default chain ID is documented (or required)

### Error Experience
- [ ] Errors are specific (not generic "operation failed")
- [ ] Errors include context (what was attempted)
- [ ] Errors are recoverable where possible
- [ ] Error variants cover all failure modes

### Async API
- [ ] Async functions clearly marked
- [ ] Sync alternatives provided where useful
- [ ] Proper cancellation support
- [ ] No blocking in async context

### Type Safety
- [ ] Distinct types for distinct concepts (Address vs H160)
- [ ] Units clear from types (Wei vs Gwei vs Ether)
- [ ] Newtypes used to prevent mixing similar values

---

## 4. Performance

### Memory
- [ ] No unnecessary allocations in hot paths
- [ ] Large data passed by reference
- [ ] `Cow<'_, str>` used where ownership is conditional
- [ ] Pre-allocated collections where size is known

### Serialization
- [ ] Efficient serialization format (RLP for transactions)
- [ ] No redundant serialization/deserialization
- [ ] Cached computed values where appropriate (tx hash)

### Concurrency
- [ ] Thread-safe types where needed (`Send + Sync`)
- [ ] No unnecessary locking
- [ ] Consider connection pooling for RPC client

---

## 5. Testing (for test review)

### Coverage
- [ ] Unit tests for all public functions
- [ ] Edge cases tested (empty input, max values, zero values)
- [ ] Error paths tested
- [ ] Integration tests for end-to-end flows

### Test Quality
- [ ] Tests have descriptive names
- [ ] Tests are independent (no shared mutable state)
- [ ] Tests use fixtures/constants for test data
- [ ] No network calls in unit tests (mock RPC)

### Security Tests
- [ ] Private key not leaked in any output
- [ ] Invalid inputs rejected gracefully
- [ ] Signature verification tested with known vectors

---

## 6. CLI-Specific (for bach-cli)

### User Experience
- [ ] Clear help messages (`--help`)
- [ ] Sensible subcommand structure
- [ ] Consistent flag naming across commands
- [ ] Progress indicators for long operations
- [ ] Colored output (with `--no-color` option)

### Configuration
- [ ] Config file support (TOML/YAML)
- [ ] Environment variable overrides
- [ ] CLI flags override config
- [ ] Clear precedence documented

### Output Formats
- [ ] Human-readable default output
- [ ] JSON output option (`--json`)
- [ ] Quiet mode (`--quiet`)
- [ ] Verbose mode (`--verbose`)

### Security
- [ ] Private keys via env var or file (not CLI arg)
- [ ] Sensitive data masked in output
- [ ] Confirmation prompt for dangerous operations

---

## Review Process

### Before Approving
1. All critical items addressed
2. All major items addressed or documented as known issues
3. Minor items can be deferred with tracking issues
4. Tests pass
5. No new warnings from `cargo clippy`
6. Documentation builds without warnings

### Severity Levels
- **Critical**: Security issues, data loss risk, crashes - MUST fix before merge
- **Major**: Broken functionality, poor error handling - SHOULD fix before merge
- **Minor**: Style issues, missing docs, minor improvements - CAN defer

---

## Quick Commands

```bash
# Run all checks before review
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo doc --no-deps

# Check for unwrap/expect in lib code (excluding tests)
rg "\.unwrap\(\)" --type rust -g '!*test*' -g '!tests/*'
rg "\.expect\(" --type rust -g '!*test*' -g '!tests/*'

# Check for missing docs
cargo doc --no-deps 2>&1 | grep "warning: missing documentation"
```
