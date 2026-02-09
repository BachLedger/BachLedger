# Agent Handoff Document

## Basic Information (基本信息)

| Field | Value |
|-------|-------|
| Agent Role | [ROLE_NAME] |
| Agent ID | [AGENT_ID] |
| Target Module | [MODULE_NAME] |
| Handoff Date | [DATE] |
| Time Period | [START_DATE] - [END_DATE] |
| Session Duration | [DURATION] |
| Handoff Reason | Task Complete / Context Limit / Shift Change / Emergency |

---

## 1. Session Summary (会话摘要)

### 1.1 Original Assignment

**Task Description:**
```
[ORIGINAL_TASK_DESCRIPTION]
```

**Success Criteria:**
- [ ] [CRITERION_1]
- [ ] [CRITERION_2]
- [ ] [CRITERION_3]

**Constraints:**
- [CONSTRAINT_1]
- [CONSTRAINT_2]

### 1.2 Overall Progress

| Metric | Value |
|--------|-------|
| Completion Percentage | [PERCENT]% |
| Tasks Completed | [COUNT] / [TOTAL] |
| Tests Written | [COUNT] |
| Tests Passing | [COUNT] / [TOTAL] |
| Files Modified | [COUNT] |
| Lines Changed | +[ADDED] / -[REMOVED] |

---

## 2. Completed Work (已完成工作)

### 2.1 Completed Tasks Checklist

| Task ID | Description | Status | Verification |
|---------|-------------|--------|--------------|
| [ID] | [DESCRIPTION] | Done | [HOW_VERIFIED] |
| [ID] | [DESCRIPTION] | Done | [HOW_VERIFIED] |
| [ID] | [DESCRIPTION] | Done | [HOW_VERIFIED] |

### 2.2 Code Changes

#### Files Created

| File Path | Purpose | Lines |
|-----------|---------|-------|
| `[PATH]` | [PURPOSE] | [LINES] |
| `[PATH]` | [PURPOSE] | [LINES] |

#### Files Modified

| File Path | Changes | Lines Changed |
|-----------|---------|---------------|
| `[PATH]` | [DESCRIPTION] | +[N] / -[N] |
| `[PATH]` | [DESCRIPTION] | +[N] / -[N] |

#### Files Deleted

| File Path | Reason |
|-----------|--------|
| `[PATH]` | [REASON] |

### 2.3 Tests Written

| Test File | Test Name | Status | Coverage |
|-----------|-----------|--------|----------|
| `[PATH]` | `[TEST_NAME]` | Passing / Failing | [WHAT_IT_COVERS] |
| `[PATH]` | `[TEST_NAME]` | Passing / Failing | [WHAT_IT_COVERS] |

### 2.4 Documentation Updated

| Document | Changes |
|----------|---------|
| `[PATH]` | [DESCRIPTION] |
| `[PATH]` | [DESCRIPTION] |

---

## 3. Incomplete Work (未完成工作)

### 3.1 In-Progress Tasks

| Task ID | Description | Progress | Remaining Work |
|---------|-------------|----------|----------------|
| [ID] | [DESCRIPTION] | [PERCENT]% | [REMAINING] |
| [ID] | [DESCRIPTION] | [PERCENT]% | [REMAINING] |

### 3.2 Detailed Progress on Incomplete Tasks

#### Task [ID]: [TITLE]

**Current State:**
```
[DESCRIPTION_OF_CURRENT_STATE]
```

**What's Done:**
- [x] [COMPLETED_STEP_1]
- [x] [COMPLETED_STEP_2]

**What's Left:**
- [ ] [REMAINING_STEP_1]
- [ ] [REMAINING_STEP_2]

**Estimated Remaining Effort:** [ESTIMATE]

**Current Code State:**
```rust
// Location: [FILE_PATH]:[LINE]
// Status: [PARTIAL/STUB/BROKEN]
[CURRENT_CODE_SNIPPET]
```

**Next Steps:**
1. [NEXT_STEP_1]
2. [NEXT_STEP_2]

### 3.3 Not Started Tasks

| Task ID | Description | Priority | Dependencies |
|---------|-------------|----------|--------------|
| [ID] | [DESCRIPTION] | High / Medium / Low | [DEPS] |
| [ID] | [DESCRIPTION] | High / Medium / Low | [DEPS] |

---

## 4. Important Decisions (重要决策)

### 4.1 Design Decisions Made

| Decision | Options Considered | Choice | Rationale |
|----------|-------------------|--------|-----------|
| [DECISION] | [OPTIONS] | [CHOICE] | [WHY] |
| [DECISION] | [OPTIONS] | [CHOICE] | [WHY] |

### 4.2 Decision Details

#### Decision: [TITLE]

**Context:**
```
[WHY_THIS_DECISION_WAS_NEEDED]
```

**Options Considered:**

| Option | Pros | Cons |
|--------|------|------|
| [OPTION_A] | [PROS] | [CONS] |
| [OPTION_B] | [PROS] | [CONS] |

**Final Decision:** [CHOICE]

**Rationale:**
```
[DETAILED_REASONING]
```

**Impact:**
- [IMPACT_1]
- [IMPACT_2]

### 4.3 Deferred Decisions

| Decision | Why Deferred | Recommended Action |
|----------|--------------|-------------------|
| [DECISION] | [REASON] | [RECOMMENDATION] |

---

## 5. Problems Encountered (遇到的问题)

### 5.1 Resolved Problems

| Problem | Solution | Time Spent |
|---------|----------|------------|
| [PROBLEM] | [SOLUTION] | [TIME] |
| [PROBLEM] | [SOLUTION] | [TIME] |

### 5.2 Problem Resolution Details

#### Problem: [TITLE]

**Description:**
```
[DETAILED_PROBLEM_DESCRIPTION]
```

**Root Cause:**
```
[ROOT_CAUSE_ANALYSIS]
```

**Solution Applied:**
```rust
// [SOLUTION_CODE_OR_DESCRIPTION]
```

**Lessons Learned:**
- [LESSON_1]
- [LESSON_2]

### 5.3 Unresolved Problems

| Problem | Attempted Solutions | Blocker | Suggested Approach |
|---------|--------------------|---------|--------------------|
| [PROBLEM] | [ATTEMPTS] | [BLOCKER] | [SUGGESTION] |
| [PROBLEM] | [ATTEMPTS] | [BLOCKER] | [SUGGESTION] |

### 5.4 Known Issues

| Issue | Severity | Workaround | Permanent Fix Needed |
|-------|----------|------------|---------------------|
| [ISSUE] | Critical / High / Medium / Low | [WORKAROUND] | [FIX_DESCRIPTION] |

---

## 6. Advice for Next Agent (给下一个Agent的建议)

### 6.1 Priority Actions

1. **[HIGHEST_PRIORITY]**: [DESCRIPTION]
   - Why: [REASON]
   - How: [APPROACH]

2. **[SECOND_PRIORITY]**: [DESCRIPTION]
   - Why: [REASON]
   - How: [APPROACH]

3. **[THIRD_PRIORITY]**: [DESCRIPTION]
   - Why: [REASON]
   - How: [APPROACH]

### 6.2 Gotchas and Warnings

| Warning | Details | How to Avoid |
|---------|---------|--------------|
| [WARNING_1] | [DETAILS] | [AVOIDANCE] |
| [WARNING_2] | [DETAILS] | [AVOIDANCE] |

### 6.3 Useful Commands

```bash
# [DESCRIPTION_1]
[COMMAND_1]

# [DESCRIPTION_2]
[COMMAND_2]

# [DESCRIPTION_3]
[COMMAND_3]
```

### 6.4 Key Files to Review

| File | Why Important |
|------|---------------|
| `[PATH]` | [REASON] |
| `[PATH]` | [REASON] |
| `[PATH]` | [REASON] |

### 6.5 Dependencies and External Resources

| Resource | Purpose | Access Notes |
|----------|---------|--------------|
| [RESOURCE] | [PURPOSE] | [NOTES] |
| [RESOURCE] | [PURPOSE] | [NOTES] |

### 6.6 Recommended Approach

```
[RECOMMENDED_APPROACH_FOR_CONTINUING_WORK]
```

---

## 7. Updated Documents List (已更新文档列表)

### 7.1 Project Documents Updated

| Document | Path | Changes Made |
|----------|------|--------------|
| Requirements | `[PATH]` | [CHANGES] |
| Interface Contract | `[PATH]` | [CHANGES] |
| Design Doc | `[PATH]` | [CHANGES] |

### 7.2 Test Documents Updated

| Document | Path | Changes Made |
|----------|------|--------------|
| Test Plan | `[PATH]` | [CHANGES] |
| Test Cases | `[PATH]` | [CHANGES] |

### 7.3 Documents Needing Update

| Document | Path | What Needs Update |
|----------|------|-------------------|
| [DOC] | `[PATH]` | [WHAT_NEEDS_UPDATE] |
| [DOC] | `[PATH]` | [WHAT_NEEDS_UPDATE] |

---

## 8. Context Snapshot (上下文快照)

### 8.1 Repository State

```
Branch: [BRANCH_NAME]
Last Commit: [COMMIT_HASH]
Commit Message: [MESSAGE]
Uncommitted Changes: Yes / No
```

### 8.2 Build Status

| Check | Status |
|-------|--------|
| `cargo build` | Passing / Failing |
| `cargo test` | Passing / Failing |
| `cargo clippy` | Passing / Warnings / Failing |
| `cargo fmt --check` | Passing / Failing |

### 8.3 Test Status Summary

```
Test Results:
  Total: [TOTAL]
  Passed: [PASSED]
  Failed: [FAILED]
  Ignored: [IGNORED]

Failing Tests:
  - [TEST_1]: [REASON]
  - [TEST_2]: [REASON]
```

### 8.4 Environment Notes

| Aspect | Value |
|--------|-------|
| Rust Version | [VERSION] |
| Key Dependencies | [DEPS] |
| Environment Variables | [VARS] |
| Special Configuration | [CONFIG] |

---

## 9. Communication Log (沟通记录)

### 9.1 Messages Sent

| To | Subject | Summary |
|----|---------|---------|
| [RECIPIENT] | [SUBJECT] | [SUMMARY] |
| [RECIPIENT] | [SUBJECT] | [SUMMARY] |

### 9.2 Messages Received

| From | Subject | Summary | Action Taken |
|------|---------|---------|--------------|
| [SENDER] | [SUBJECT] | [SUMMARY] | [ACTION] |
| [SENDER] | [SUBJECT] | [SUMMARY] | [ACTION] |

### 9.3 Pending Communications

| With | Topic | Status | Next Action |
|------|-------|--------|-------------|
| [PERSON] | [TOPIC] | Awaiting Response | [ACTION] |
| [PERSON] | [TOPIC] | Needs Follow-up | [ACTION] |

---

## Sign-off

| Role | Agent ID | Timestamp |
|------|----------|-----------|
| Outgoing Agent | [AGENT_ID] | [TIMESTAMP] |
| Incoming Agent | [AGENT_ID] | [TIMESTAMP] (upon receipt) |

---

## Revision History

| Version | Date | Agent | Changes |
|---------|------|-------|---------|
| 1.0 | [DATE] | [AGENT_ID] | Initial handoff document |

---

## Quick Reference Card

### Commands to Run First
```bash
# Verify build
cargo build --all

# Run tests
cargo test --all

# Check current status
git status
git log -5 --oneline
```

### Key Locations
- Main code: `[PATH]`
- Tests: `[PATH]`
- Config: `[PATH]`
- Docs: `[PATH]`

### Contacts
- Team Lead: [CONTACT]
- Module Owner: [CONTACT]
- Security: [CONTACT]
