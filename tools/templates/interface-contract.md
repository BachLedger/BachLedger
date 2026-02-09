# Interface Contract Document

## Contract Information

| Field | Value |
|-------|-------|
| Module Name | [MODULE_NAME] |
| Contract Version | [VERSION] |
| Author | [AUTHOR] |
| Date | [DATE] |
| Status | Draft / Locked / Deprecated |

---

## Version Locking Mechanism

### Contract Version Rules

- **Major Version (X.0.0)**: Breaking changes to interface signatures or behavior
- **Minor Version (0.X.0)**: Backward-compatible additions
- **Patch Version (0.0.X)**: Bug fixes with no interface changes

### Compatibility Matrix

| Provider Version | Consumer Min Version | Notes |
|------------------|---------------------|-------|
| [VERSION] | [MIN_VERSION] | [NOTES] |

### Breaking Change Policy

1. Breaking changes require major version bump
2. Deprecated interfaces must be supported for at least one major version
3. All consumers must be notified before breaking changes are merged

---

## 1. 网络协议接口 (Network Protocol Interfaces)

### 1.1 HTTP/RPC Endpoints

#### Endpoint: [ENDPOINT_NAME]

| Field | Value |
|-------|-------|
| Method | GET / POST / PUT / DELETE |
| Path | `/api/v[VERSION]/[PATH]` |
| Authentication | Required / Optional / None |
| Rate Limit | [REQUESTS]/[TIME_WINDOW] |

**Request Schema:**
```json
{
  "field1": "[TYPE] - [DESCRIPTION]",
  "field2": "[TYPE] - [DESCRIPTION]",
  "optional_field?": "[TYPE] - [DESCRIPTION]"
}
```

**Response Schema (Success - 200):**
```json
{
  "result": "[TYPE] - [DESCRIPTION]",
  "metadata": {
    "timestamp": "u64 - Unix timestamp",
    "version": "string - API version"
  }
}
```

**Response Schema (Error - 4xx/5xx):**
```json
{
  "error": {
    "code": "string - Error code",
    "message": "string - Human-readable message",
    "details": "object? - Additional error details"
  }
}
```

**Error Codes:**

| Code | HTTP Status | Description | Resolution |
|------|-------------|-------------|------------|
| [ERROR_CODE] | [STATUS] | [DESCRIPTION] | [RESOLUTION] |

---

### 1.2 P2P Messages

#### Message: [MESSAGE_TYPE]

| Field | Value |
|-------|-------|
| Message ID | `0x[HEX_ID]` |
| Direction | Broadcast / Direct / Request-Response |
| Priority | High / Medium / Low |
| Max Size | [SIZE] bytes |

**Message Schema:**
```rust
struct [MessageType] {
    /// [DESCRIPTION]
    pub header: MessageHeader,
    /// [DESCRIPTION]
    pub payload: [PayloadType],
    /// [DESCRIPTION]
    pub signature: Signature,
}
```

**Validation Rules:**
- [ ] [RULE_1]
- [ ] [RULE_2]
- [ ] [RULE_3]

**Handling Behavior:**

| Condition | Action |
|-----------|--------|
| Valid message | [ACTION] |
| Invalid signature | [ACTION] |
| Unknown sender | [ACTION] |
| Duplicate message | [ACTION] |

---

### 1.3 Consensus Messages

#### Message: [CONSENSUS_MESSAGE_TYPE]

| Field | Value |
|-------|-------|
| Phase | Propose / Prevote / Precommit / Commit |
| Broadcast Scope | Validators / All Nodes |
| Timeout | [DURATION] |

**Message Schema:**
```rust
struct [ConsensusMessage] {
    /// Block height
    pub height: u64,
    /// Consensus round
    pub round: u32,
    /// [DESCRIPTION]
    pub [FIELD]: [TYPE],
    /// Validator signature
    pub signature: ValidatorSignature,
}
```

**State Transitions:**
```
[STATE_1] ──[MESSAGE]──► [STATE_2]
    │                        │
    ▼                        ▼
[STATE_3] ◄──[MESSAGE]── [STATE_4]
```

**Timing Constraints:**

| Constraint | Value | Consequence of Violation |
|------------|-------|-------------------------|
| Message timeout | [DURATION] | [CONSEQUENCE] |
| Round timeout | [DURATION] | [CONSEQUENCE] |

---

## 2. 函数调用接口 (Function Call Interfaces)

### 2.1 Rust Traits

#### Trait: [TRAIT_NAME]

```rust
/// [TRAIT_DESCRIPTION]
///
/// # Contract
/// - [INVARIANT_1]
/// - [INVARIANT_2]
///
/// # Thread Safety
/// [THREAD_SAFETY_GUARANTEE]
pub trait [TraitName]: [BOUNDS] {
    /// Associated type for [PURPOSE]
    type [AssocType]: [BOUNDS];

    /// [METHOD_DESCRIPTION]
    ///
    /// # Arguments
    /// * `[ARG]` - [DESCRIPTION]
    ///
    /// # Returns
    /// [RETURN_DESCRIPTION]
    ///
    /// # Errors
    /// Returns `[ErrorType]` when:
    /// - [ERROR_CONDITION_1]
    /// - [ERROR_CONDITION_2]
    ///
    /// # Panics
    /// [PANIC_CONDITIONS or "This function does not panic"]
    ///
    /// # Contract
    /// - Pre: [PRECONDITION]
    /// - Post: [POSTCONDITION]
    fn [method_name](&self, [args]: [Type]) -> Result<[ReturnType], [ErrorType]>;

    /// [METHOD_DESCRIPTION]
    fn [method_name_2](&mut self, [args]: [Type]) -> [ReturnType];
}
```

**Implementation Requirements:**

| Requirement | Description | Verification |
|-------------|-------------|--------------|
| [REQ_1] | [DESCRIPTION] | [HOW_TO_VERIFY] |
| [REQ_2] | [DESCRIPTION] | [HOW_TO_VERIFY] |

---

### 2.2 SDK API

#### Function: [FUNCTION_NAME]

```rust
/// [FUNCTION_DESCRIPTION]
///
/// # Examples
/// ```rust
/// use [crate]::[module]::[function_name];
///
/// let result = [function_name]([example_args]);
/// assert_eq!(result, [expected]);
/// ```
pub fn [function_name]([args]: [Type]) -> Result<[ReturnType], [ErrorType]> {
    // Implementation
}
```

**API Stability:**

| Aspect | Guarantee |
|--------|-----------|
| Function signature | Stable since v[VERSION] |
| Return type | Stable since v[VERSION] |
| Error variants | May add new variants |
| Behavior | [BEHAVIOR_GUARANTEE] |

---

### 2.3 CLI Commands

#### Command: `[COMMAND_NAME]`

```
USAGE:
    [BINARY] [COMMAND] [OPTIONS] [ARGS]

DESCRIPTION:
    [DESCRIPTION]

OPTIONS:
    -h, --help              Print help information
    -v, --verbose           Enable verbose output
    --[OPTION]=[VALUE]      [OPTION_DESCRIPTION]

ARGS:
    <[ARG_NAME]>            [ARG_DESCRIPTION]

EXAMPLES:
    # [EXAMPLE_DESCRIPTION]
    $ [BINARY] [COMMAND] [EXAMPLE_ARGS]

    # [EXAMPLE_DESCRIPTION_2]
    $ [BINARY] [COMMAND] [EXAMPLE_ARGS_2]

EXIT CODES:
    0    Success
    1    General error
    [CODE]    [DESCRIPTION]
```

**Environment Variables:**

| Variable | Description | Default |
|----------|-------------|---------|
| `[VAR_NAME]` | [DESCRIPTION] | [DEFAULT] |

---

## 3. 数据格式接口 (Data Format Interfaces)

### 3.1 Serialization Formats

#### Format: [FORMAT_NAME]

| Field | Value |
|-------|-------|
| Encoding | JSON / Bincode / Protobuf / SSZ |
| Schema Version | [VERSION] |
| Max Size | [SIZE] bytes |

**Schema Definition:**
```rust
#[derive(Serialize, Deserialize)]
pub struct [StructName] {
    /// [FIELD_DESCRIPTION]
    #[serde(rename = "[WIRE_NAME]")]
    pub [field_name]: [Type],

    /// [FIELD_DESCRIPTION]
    #[serde(default)]
    pub [optional_field]: Option<[Type]>,

    /// [FIELD_DESCRIPTION]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub [list_field]: Vec<[Type]>,
}
```

**Wire Format Example:**
```json
{
  "[WIRE_NAME]": [VALUE],
  "[FIELD_2]": [VALUE]
}
```

**Compatibility Rules:**
- [ ] Unknown fields are ignored (forward compatibility)
- [ ] Missing optional fields use defaults
- [ ] [ADDITIONAL_RULE]

---

### 3.2 Configuration Formats

#### Config: [CONFIG_NAME]

**File Location:** `[PATH]/[FILENAME].[EXT]`

**Schema:**
```toml
# [SECTION_DESCRIPTION]
[section]
# [FIELD_DESCRIPTION]
# Type: [TYPE]
# Default: [DEFAULT]
# Required: Yes/No
[field_name] = [EXAMPLE_VALUE]

# [SUBSECTION_DESCRIPTION]
[section.subsection]
[field] = [VALUE]
```

**Validation Rules:**

| Field | Constraint | Error Message |
|-------|------------|---------------|
| [FIELD] | [CONSTRAINT] | [MESSAGE] |

**Migration:**

| From Version | To Version | Migration Steps |
|--------------|------------|-----------------|
| [OLD] | [NEW] | [STEPS] |

---

### 3.3 File Formats

#### Format: [FILE_FORMAT_NAME]

| Field | Value |
|-------|-------|
| Extension | `.[EXT]` |
| Magic Bytes | `[HEX_BYTES]` |
| Endianness | Little / Big |
| Version | [VERSION] |

**File Structure:**
```
┌─────────────────────────────────────┐
│ Header (32 bytes)                   │
│   ├─ Magic (4 bytes): [MAGIC]       │
│   ├─ Version (4 bytes): u32         │
│   ├─ Flags (4 bytes): u32           │
│   └─ Reserved (20 bytes)            │
├─────────────────────────────────────┤
│ Index Section ([SIZE] bytes)        │
│   └─ [DESCRIPTION]                  │
├─────────────────────────────────────┤
│ Data Section (variable)             │
│   └─ [DESCRIPTION]                  │
├─────────────────────────────────────┤
│ Checksum (32 bytes): SHA256         │
└─────────────────────────────────────┘
```

**Integrity Checks:**
- [ ] Magic bytes match expected value
- [ ] Version is supported
- [ ] Checksum validates
- [ ] [ADDITIONAL_CHECK]

---

## 4. Interface Dependencies

### 4.1 Dependency Graph

```
[INTERFACE_1]
      │
      ▼
[INTERFACE_2] ──► [INTERFACE_3]
      │                │
      ▼                ▼
[INTERFACE_4] ◄── [INTERFACE_5]
```

### 4.2 Dependency Table

| Interface | Depends On | Depended By | Version Constraint |
|-----------|------------|-------------|-------------------|
| [INTERFACE_1] | - | [INTERFACES] | - |
| [INTERFACE_2] | [INTERFACE_1] | [INTERFACES] | >= [VERSION] |

---

## 5. Contract Verification

### 5.1 Automated Checks

| Check | Tool | Command | Required |
|-------|------|---------|----------|
| Type compatibility | `cargo check` | `cargo check --all-features` | Yes |
| API documentation | `cargo doc` | `cargo doc --no-deps` | Yes |
| Breaking changes | `cargo-semver-checks` | `cargo semver-checks` | Yes |

### 5.2 Manual Review Checklist

- [ ] All public APIs documented
- [ ] Error conditions specified
- [ ] Thread safety documented
- [ ] Performance characteristics noted
- [ ] Breaking changes flagged

---

## 6. Sign-off

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Contract Author | [NAME] | [DATE] | [ ] |
| API Consumer Rep | [NAME] | [DATE] | [ ] |
| Security Review | [NAME] | [DATE] | [ ] |

---

## Revision History

| Version | Date | Author | Changes | Breaking |
|---------|------|--------|---------|----------|
| 1.0.0 | [DATE] | [AUTHOR] | Initial contract | - |
| [VERSION] | [DATE] | [AUTHOR] | [CHANGES] | Yes/No |
