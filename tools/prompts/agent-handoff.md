# Agent Handoff Prompt

## Purpose

This prompt is triggered when any ICDD agent finishes its work or is being replaced. It ensures continuity by capturing everything the next agent (or returning agent) needs to know.

## When to Use

- Agent completes its assigned phase
- Agent is being replaced mid-task
- Agent encounters a blocker requiring escalation
- Agent session is ending (timeout, error, etc.)

## Handoff Template

Every agent MUST complete this template before ending:

```markdown
# Agent Handoff Document

## Identification

- **Agent Role**: [Architect/Tester/Coder/Reviewer-Logic/Reviewer-Test/Reviewer-Integration/Attacker/Reviewer-Attack/Documenter]
- **Phase**: [Step 1/2/3/4]
- **Timestamp**: [ISO 8601 timestamp]
- **Session ID**: [if available]

## Work Summary

### Completed Work
[List all work that was fully completed]

1. [Completed item 1]
   - Output: [file/artifact]
   - Verified: [YES/NO]

2. [Completed item 2]
   - Output: [file/artifact]
   - Verified: [YES/NO]

### Incomplete Work
[List all work that was started but not finished]

1. [Incomplete item 1]
   - Progress: [percentage or description]
   - Blocking issue: [what's preventing completion]
   - Files affected: [list]

2. [Incomplete item 2]
   - Progress: [percentage or description]
   - Blocking issue: [what's preventing completion]
   - Files affected: [list]

### Not Started
[List work that was planned but not begun]

1. [Not started item 1]
   - Reason: [why not started]
   - Prerequisite: [what's needed first]

## Decisions Made

### Decision 1: [Title]
- **Context**: [Why this decision was needed]
- **Options Considered**: [List alternatives]
- **Decision**: [What was decided]
- **Rationale**: [Why this option]
- **Impact**: [What this affects]

### Decision 2: [Title]
[Same format...]

## Problems Encountered

### Problem 1: [Title]
- **Description**: [What went wrong]
- **Attempted Solutions**: [What was tried]
- **Resolution**: [RESOLVED/UNRESOLVED]
- **If Resolved**: [How it was fixed]
- **If Unresolved**: [Current state, suggestions]

### Problem 2: [Title]
[Same format...]

## Artifacts Produced

| Artifact | Path | Status | Notes |
|----------|------|--------|-------|
| [name] | [path] | COMPLETE/PARTIAL | [notes] |
| [name] | [path] | COMPLETE/PARTIAL | [notes] |

## State Information

### Environment State
- **Working Directory**: [path]
- **Branch**: [git branch]
- **Last Commit**: [hash and message]
- **Uncommitted Changes**: [list files]

### Application State
[Any runtime state that matters]

### Test State
- **Tests Passing**: [N]/[total]
- **Tests Failing**: [list if any]
- **Last Test Run**: [timestamp]

## For Next Agent

### Critical Context
[Most important things the next agent must know]

1. [Critical point 1]
2. [Critical point 2]
3. [Critical point 3]

### Warnings
[Things to watch out for]

1. [Warning 1]
2. [Warning 2]

### Recommended Next Steps
[What should be done next, in order]

1. [Step 1]
2. [Step 2]
3. [Step 3]

### Resources
[Helpful references]

- [Document/file with description]
- [Document/file with description]

## Verification Checklist

Before handoff, verify:

- [ ] All completed work is tested/verified
- [ ] All artifacts are saved and committed (if appropriate)
- [ ] All decisions are documented with rationale
- [ ] All blockers are clearly described
- [ ] Next steps are actionable
- [ ] No secrets or credentials in handoff document
```

## Handoff Best Practices

### DO

1. **Be specific**: "Fixed validation in `parse_input()` at line 42" not "Fixed bug"
2. **Include paths**: Always use absolute paths for files
3. **Explain why**: Decisions without rationale are hard to evaluate later
4. **List blockers clearly**: What's needed, who can provide it
5. **Prioritize**: Put most critical info first

### DON'T

1. **Assume context**: Next agent may have no prior knowledge
2. **Leave loose ends**: Every incomplete item needs a clear state
3. **Skip verification**: Claiming work is done without testing
4. **Forget state**: Application/test/environment state matters
5. **Be vague**: "Some tests fail" is not useful

## Emergency Handoff

If the agent must stop immediately (error, timeout):

```markdown
# EMERGENCY HANDOFF

**Agent**: [role]
**Reason**: [error/timeout/other]
**Timestamp**: [time]

## Last Known State
- Working on: [what]
- Progress: [how far]
- Files being edited: [list]

## Critical Warning
[Most important thing next agent must know]

## Recovery Steps
1. [How to recover/continue]
```

## Handoff Verification

The receiving agent should:

1. **Acknowledge receipt**: Confirm handoff was received
2. **Review artifacts**: Verify all listed files exist
3. **Check state**: Verify environment matches description
4. **Run tests**: Confirm test state matches claims
5. **Ask clarifying questions**: If anything is unclear

```markdown
# Handoff Acknowledgment

**Receiving Agent**: [role]
**Timestamp**: [time]

## Verification Results
- Artifacts verified: [YES/NO]
- State verified: [YES/NO]
- Tests verified: [YES/NO]

## Clarifications Needed
[List any questions]

## Proceeding With
[What the receiving agent will do first]
```

## Integration with Documenter

The Documenter agent collects all handoffs and:

1. Archives handoff documents
2. Extracts decisions for ADRs
3. Updates progress tracking
4. Maintains handoff log
5. Identifies patterns/issues across handoffs
