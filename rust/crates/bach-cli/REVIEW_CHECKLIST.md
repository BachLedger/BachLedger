# BachLedger CLI Review Checklist

CLI-specific review criteria for `bach-cli` crate.

See also: `../bach-sdk/REVIEW_CHECKLIST.md` for general Rust review criteria.

---

## Command Structure

### Subcommands
- [ ] Logical grouping of related commands
- [ ] Consistent naming (verb-noun or noun-verb, but consistent)
- [ ] No overly deep nesting (max 2-3 levels)

Expected structure:
```
bach-cli
  wallet
    create      - Create new wallet
    import      - Import from private key/mnemonic
    export      - Export wallet (with confirmation)
    list        - List wallets
    balance     - Show balance
  tx
    send        - Send transaction
    status      - Check transaction status
    decode      - Decode raw transaction
  contract
    deploy      - Deploy contract
    call        - Call contract method (read)
    send        - Send contract transaction (write)
  account
    nonce       - Get account nonce
    code        - Get contract code
  block
    get         - Get block by number/hash
    latest      - Get latest block
```

---

## Argument & Flag Review

### Required vs Optional
- [ ] Required args are positional or clearly marked
- [ ] Optional args have sensible defaults
- [ ] Defaults documented in help text

### Common Flags
- [ ] `--rpc-url` / `-r` - RPC endpoint
- [ ] `--chain-id` / `-c` - Chain ID
- [ ] `--keyfile` / `-k` - Path to keyfile
- [ ] `--json` / `-j` - JSON output
- [ ] `--quiet` / `-q` - Suppress output
- [ ] `--verbose` / `-v` - Verbose output
- [ ] `--help` / `-h` - Help (auto from clap)
- [ ] `--version` / `-V` - Version (auto from clap)

### Consistency
- [ ] Same flag means same thing across commands
- [ ] Short flags used consistently
- [ ] No conflicting short flags within a command

---

## Input Handling

### Addresses
- [ ] Accept with or without `0x` prefix
- [ ] Validate checksum if provided (warn if invalid)
- [ ] Case-insensitive (normalize to lowercase internally)

### Amounts
- [ ] Support unit suffixes: `wei`, `gwei`, `ether`
- [ ] Default unit documented (wei recommended)
- [ ] Handle decimal input for ether (e.g., `1.5 ether`)
- [ ] Validate no negative amounts

### Private Keys
- [ ] NEVER accept via command line argument
- [ ] Accept via:
  - [ ] Environment variable (`BACH_PRIVATE_KEY`)
  - [ ] Keyfile path (`--keyfile`)
  - [ ] Interactive prompt (masked input)
- [ ] Clear from memory after use

### Hex Data
- [ ] Accept with or without `0x` prefix
- [ ] Validate even length
- [ ] Validate valid hex characters

---

## Output Formatting

### Human-Readable (Default)
- [ ] Clear labels for each field
- [ ] Proper alignment
- [ ] Units shown (wei, gwei, block number)
- [ ] Addresses checksummed
- [ ] Hashes full length (no truncation by default)
- [ ] Timestamps human-readable

Example:
```
Transaction sent successfully!
  Hash:       0x1234...abcd
  From:       0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d
  To:         0xdAC17F958D2ee523a2206206994597C13D831ec7
  Value:      1.5 ETH
  Gas Limit:  21000
  Gas Price:  20 Gwei
  Nonce:      42
```

### JSON Output (`--json`)
- [ ] Valid JSON
- [ ] Consistent field naming (snake_case)
- [ ] Numbers as strings for large values (avoid JS precision loss)
- [ ] All values present (no omitted fields)
- [ ] No extra logging mixed in
- [ ] Exit code 0 on success, non-zero on failure

Example:
```json
{
  "success": true,
  "transaction_hash": "0x1234...abcd",
  "from": "0x742d35cc6634c0532925a3b844bc9e7595f0ab3d",
  "to": "0xdac17f958d2ee523a2206206994597c13d831ec7",
  "value": "1500000000000000000",
  "gas_limit": "21000",
  "gas_price": "20000000000",
  "nonce": "42"
}
```

---

## Error Handling

### User-Friendly Errors
- [ ] No stack traces in release mode
- [ ] Clear explanation of what went wrong
- [ ] Suggestion for how to fix (when possible)
- [ ] Reference to `--help` when appropriate

### Error Categories
- [ ] Input validation errors (bad address, invalid amount)
- [ ] Configuration errors (missing RPC URL, no keyfile)
- [ ] Network errors (connection refused, timeout)
- [ ] Chain errors (insufficient funds, nonce too low)

### Exit Codes
- [ ] `0` - Success
- [ ] `1` - General error
- [ ] `2` - Invalid arguments
- [ ] Consider specific codes for common errors

---

## Security Review

### Sensitive Data
- [ ] Private keys NEVER echoed to terminal
- [ ] Private keys NEVER in command history
- [ ] Passwords/passphrases use secure prompt (no echo)
- [ ] Keyfile permissions checked (warn if too open)

### Dangerous Operations
- [ ] Confirmation prompt for:
  - [ ] Sending transactions
  - [ ] Deploying contracts
  - [ ] Exporting private keys
- [ ] `--yes` / `-y` flag to skip confirmation (for scripting)
- [ ] Show transaction details before confirmation

### Network Security
- [ ] Warn if using HTTP instead of HTTPS
- [ ] Validate RPC URL format
- [ ] Timeout on network operations

---

## Configuration

### Config File
- [ ] Support `~/.config/bach-cli/config.toml` (or platform-appropriate)
- [ ] Support project-local `.bach-cli.toml`
- [ ] Document config file format

Example config:
```toml
[default]
rpc_url = "https://rpc.bachledger.io"
chain_id = 1

[testnet]
rpc_url = "https://testnet-rpc.bachledger.io"
chain_id = 5
```

### Environment Variables
- [ ] `BACH_RPC_URL` - RPC endpoint
- [ ] `BACH_CHAIN_ID` - Chain ID
- [ ] `BACH_PRIVATE_KEY` - Private key (for scripting)
- [ ] `BACH_KEYFILE` - Path to keyfile
- [ ] All env vars documented in help

### Precedence
1. CLI flags (highest)
2. Environment variables
3. Config file
4. Built-in defaults (lowest)

---

## Testing Checklist

### Unit Tests
- [ ] Argument parsing
- [ ] Input validation
- [ ] Output formatting
- [ ] Error messages

### Integration Tests
- [ ] Each command with valid input
- [ ] Each command with invalid input
- [ ] Config file loading
- [ ] Environment variable handling

### Manual Testing
- [ ] Run each command manually
- [ ] Test `--help` at each level
- [ ] Test JSON output parsing with `jq`
- [ ] Test in CI/scripting context

---

## Documentation

### Help Text
- [ ] Brief description for each command
- [ ] All flags documented
- [ ] Examples in command help
- [ ] Clear usage line

### README
- [ ] Installation instructions
- [ ] Quick start examples
- [ ] Configuration guide
- [ ] Troubleshooting section

---

## Dependencies

### Recommended Crates
- [ ] `clap` - Argument parsing (with derive)
- [ ] `tokio` - Async runtime
- [ ] `serde` / `serde_json` - JSON output
- [ ] `toml` - Config file parsing
- [ ] `colored` - Terminal colors (optional output)
- [ ] `dialoguer` - Interactive prompts
- [ ] `indicatif` - Progress bars (optional)

### Avoid
- [ ] Heavy dependencies for simple tasks
- [ ] Crates with security advisories
- [ ] Unmaintained crates
