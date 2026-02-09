# Code Review Checklist

## Review Information

| Field | Value |
|-------|-------|
| Module/PR | [MODULE_NAME or PR_NUMBER] |
| Author | [AUTHOR] |
| Reviewer | [REVIEWER] |
| Review Date | [DATE] |
| Review Status | In Progress / Approved / Changes Requested |

---

## Review Summary

| Section | Status | Critical Issues | Notes |
|---------|--------|-----------------|-------|
| Reviewer-Logic | Pass / Fail / Partial | [COUNT] | [NOTES] |
| Reviewer-Test | Pass / Fail / Partial | [COUNT] | [NOTES] |
| Reviewer-Integration | Pass / Fail / Partial | [COUNT] | [NOTES] |

**Overall Verdict:** Approved / Approved with Comments / Changes Required / Rejected

---

## 1. Reviewer-Logic (代码逻辑审查)

### 1.1 Stub & Fake Implementation Detection

| Check | Status | Location | Notes |
|-------|--------|----------|-------|
| No `todo!()` macros in production code | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| No `unimplemented!()` in production code | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| No `panic!()` for expected error conditions | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| No placeholder return values | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| No hardcoded test data in production | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| No commented-out code blocks | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| No `#[allow(dead_code)]` without justification | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |

### 1.2 Unwrap/Expect Abuse Detection

| Check | Status | Location | Notes |
|-------|--------|----------|-------|
| No `.unwrap()` on fallible operations | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| No `.expect()` without clear message | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Array indexing uses `.get()` or bounds check | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| No unchecked arithmetic operations | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Proper `Result`/`Option` propagation | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |

**Allowed Exceptions:**
- `unwrap()` in tests is acceptable
- `expect()` for invariants that indicate bugs (document rationale)
- Const/static initialization where panic is acceptable

### 1.3 Error Handling Quality

| Check | Status | Location | Notes |
|-------|--------|----------|-------|
| Errors include context | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Error types are descriptive | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| No silent error swallowing | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Errors are recoverable where appropriate | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Error messages are actionable | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |

### 1.4 Logic Correctness

| Check | Status | Location | Notes |
|-------|--------|----------|-------|
| Business logic matches requirements | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Edge cases handled | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| No off-by-one errors | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Correct operator usage (&&, ||, ==, etc.) | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Loop termination guaranteed | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Recursion has base case | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |

### 1.5 Resource Management

| Check | Status | Location | Notes |
|-------|--------|----------|-------|
| No resource leaks (files, connections) | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Proper cleanup on error paths | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Bounded memory usage | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| No unbounded growth | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |

---

## 2. Reviewer-Test (测试审查)

### 2.1 Test Coverage Analysis

| Metric | Required | Actual | Status |
|--------|----------|--------|--------|
| Line coverage | [REQUIRED]% | [ACTUAL]% | Pass / Fail |
| Branch coverage | [REQUIRED]% | [ACTUAL]% | Pass / Fail |
| Function coverage | [REQUIRED]% | [ACTUAL]% | Pass / Fail |

**Uncovered Critical Paths:**
- [ ] [PATH_1]: [REASON]
- [ ] [PATH_2]: [REASON]

### 2.2 Negative Test Cases (负向测试)

| Scenario | Test Exists | Test Name | Status |
|----------|-------------|-----------|--------|
| Invalid input format | Yes / No | [TEST_NAME] | Pass / Fail / N/A |
| Null/None values | Yes / No | [TEST_NAME] | Pass / Fail / N/A |
| Empty collections | Yes / No | [TEST_NAME] | Pass / Fail / N/A |
| Malformed data | Yes / No | [TEST_NAME] | Pass / Fail / N/A |
| Permission denied | Yes / No | [TEST_NAME] | Pass / Fail / N/A |
| Network failure | Yes / No | [TEST_NAME] | Pass / Fail / N/A |
| Timeout conditions | Yes / No | [TEST_NAME] | Pass / Fail / N/A |
| Resource exhaustion | Yes / No | [TEST_NAME] | Pass / Fail / N/A |

### 2.3 Boundary Test Cases (边界测试)

| Boundary | Test Exists | Test Name | Status |
|----------|-------------|-----------|--------|
| Minimum valid value | Yes / No | [TEST_NAME] | Pass / Fail / N/A |
| Maximum valid value | Yes / No | [TEST_NAME] | Pass / Fail / N/A |
| Value at min - 1 | Yes / No | [TEST_NAME] | Pass / Fail / N/A |
| Value at max + 1 | Yes / No | [TEST_NAME] | Pass / Fail / N/A |
| Empty string / zero length | Yes / No | [TEST_NAME] | Pass / Fail / N/A |
| Maximum length | Yes / No | [TEST_NAME] | Pass / Fail / N/A |
| Integer overflow | Yes / No | [TEST_NAME] | Pass / Fail / N/A |
| Integer underflow | Yes / No | [TEST_NAME] | Pass / Fail / N/A |

### 2.4 Test Quality

| Check | Status | Location | Notes |
|-------|--------|----------|-------|
| Tests are deterministic | Pass / Fail / N/A | [TEST_FILE] | [NOTES] |
| Tests are independent | Pass / Fail / N/A | [TEST_FILE] | [NOTES] |
| Tests have clear assertions | Pass / Fail / N/A | [TEST_FILE] | [NOTES] |
| Test names describe behavior | Pass / Fail / N/A | [TEST_FILE] | [NOTES] |
| No test interdependencies | Pass / Fail / N/A | [TEST_FILE] | [NOTES] |
| Proper test fixtures/setup | Pass / Fail / N/A | [TEST_FILE] | [NOTES] |
| Tests clean up after themselves | Pass / Fail / N/A | [TEST_FILE] | [NOTES] |

### 2.5 Missing Test Identification

| Missing Test | Priority | Rationale |
|--------------|----------|-----------|
| [DESCRIPTION] | High / Medium / Low | [WHY_NEEDED] |
| [DESCRIPTION] | High / Medium / Low | [WHY_NEEDED] |

---

## 3. Reviewer-Integration (集成审查)

### 3.1 Interface Consistency

| Check | Status | Location | Notes |
|-------|--------|----------|-------|
| API matches interface contract | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Return types are consistent | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Error types match specification | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Method signatures stable | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| No undocumented breaking changes | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |

### 3.2 Dependency Analysis

| Check | Status | Location | Notes |
|-------|--------|----------|-------|
| No circular dependencies | Pass / Fail / N/A | [MODULES] | [NOTES] |
| Dependency versions pinned | Pass / Fail / N/A | [CARGO.TOML] | [NOTES] |
| No unnecessary dependencies | Pass / Fail / N/A | [CARGO.TOML] | [NOTES] |
| Compatible with downstream consumers | Pass / Fail / N/A | [MODULES] | [NOTES] |
| Feature flags documented | Pass / Fail / N/A | [CARGO.TOML] | [NOTES] |

### 3.3 Serialization Compatibility

| Check | Status | Location | Notes |
|-------|--------|----------|-------|
| Wire format unchanged for existing fields | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| New fields have defaults | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Serde attributes correct | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Backward compatible | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Forward compatible | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Serialization tests exist | Pass / Fail / N/A | [TEST_FILE] | [NOTES] |

### 3.4 Cross-Module Integration

| Check | Status | Location | Notes |
|-------|--------|----------|-------|
| Data flows correctly between modules | Pass / Fail / N/A | [MODULES] | [NOTES] |
| Event handling correct | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Async boundaries respected | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Transaction boundaries correct | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |

### 3.5 Concurrency & Thread Safety

| Check | Status | Location | Notes |
|-------|--------|----------|-------|
| Proper synchronization | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| No data races | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Deadlock-free design | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Send/Sync bounds correct | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |

---

## 4. Security Review (Optional but Recommended)

| Check | Status | Location | Notes |
|-------|--------|----------|-------|
| Input validation | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| No injection vulnerabilities | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Proper authentication checks | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Authorization enforced | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Sensitive data protected | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |
| Cryptography used correctly | Pass / Fail / N/A | [FILE:LINE] | [NOTES] |

---

## 5. Issues Found

### Critical Issues (Must Fix)

| ID | Location | Description | Recommendation |
|----|----------|-------------|----------------|
| C-001 | [FILE:LINE] | [DESCRIPTION] | [RECOMMENDATION] |
| C-002 | [FILE:LINE] | [DESCRIPTION] | [RECOMMENDATION] |

### Major Issues (Should Fix)

| ID | Location | Description | Recommendation |
|----|----------|-------------|----------------|
| M-001 | [FILE:LINE] | [DESCRIPTION] | [RECOMMENDATION] |
| M-002 | [FILE:LINE] | [DESCRIPTION] | [RECOMMENDATION] |

### Minor Issues (Consider Fixing)

| ID | Location | Description | Recommendation |
|----|----------|-------------|----------------|
| m-001 | [FILE:LINE] | [DESCRIPTION] | [RECOMMENDATION] |
| m-002 | [FILE:LINE] | [DESCRIPTION] | [RECOMMENDATION] |

### Suggestions (Optional Improvements)

| ID | Location | Description | Benefit |
|----|----------|-------------|---------|
| S-001 | [FILE:LINE] | [DESCRIPTION] | [BENEFIT] |
| S-002 | [FILE:LINE] | [DESCRIPTION] | [BENEFIT] |

---

## 6. Follow-up Actions

| Action | Owner | Due Date | Status |
|--------|-------|----------|--------|
| [ACTION_1] | [OWNER] | [DATE] | Open / Done |
| [ACTION_2] | [OWNER] | [DATE] | Open / Done |

---

## Sign-off

| Role | Name | Date | Verdict |
|------|------|------|---------|
| Primary Reviewer | [NAME] | [DATE] | Approved / Changes Required |
| Secondary Reviewer | [NAME] | [DATE] | Approved / Changes Required |

---

## Revision History

| Version | Date | Reviewer | Changes |
|---------|------|----------|---------|
| 1 | [DATE] | [REVIEWER] | Initial review |
| [VERSION] | [DATE] | [REVIEWER] | [CHANGES] |
