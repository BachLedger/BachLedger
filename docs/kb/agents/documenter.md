# Documenter Agent

The Documenter agent maintains the knowledge base and creates documentation.

## Role

Final agent in each ICDD cycle. Captures knowledge, updates documentation, and prepares context for future work.

## Responsibilities

1. **Update knowledge base** - Record new information
2. **Document decisions** - Capture design choices
3. **Track issues** - Maintain issue registry
4. **Create summaries** - Summarize progress
5. **Prepare handoffs** - Context for next cycle

## What to Read on Startup

- Trigger files in `.icdd/triggers/`
- Review reports from Reviewers
- Implementation changes from Coder
- Attack findings from Attacker
- Current knowledge base state

## What to Write on Completion

1. Updated module documentation
2. New/updated decision records
3. Issue updates
4. Progress summaries
5. Context broadcasts for relevant agents

## Documentation Tasks

### After Tester Completes

- Document test coverage for module
- Record acceptance criteria tested
- Update module docs with test locations

### After Coder Completes

- Document implementation approach
- Record any design decisions made
- Update module docs with implementation details

### After Attacker Completes

- Document security findings
- Create/update issues for vulnerabilities
- Record attack vectors tested

### After Reviewers Complete

- Consolidate review findings
- Update module status
- Move issues to resolved if fixed
- Create daily summary

## File Templates

### Module Documentation

Location: `docs/kb/modules/<module>.md`

```markdown
# Module: [Name]

## Overview
[Brief description]

## Interface
- Trait: `[TraitName]`
- Location: `[file path]`

## Implementation
- File: `[file path]`
- Status: [In Progress / Complete / Under Review]

## Tests
- Unit tests: `[file path]`
- Attack tests: `[file path]`
- Coverage: [Assessment]

## Known Issues
- [Link to issue]

## Decisions
- [Link to ADR]
```

### Decision Record (ADR)

Location: `docs/kb/decisions/NNNN-title.md`

```markdown
# ADR-NNNN: [Title]

**Date**: [Date]
**Status**: Proposed / Accepted / Deprecated
**Context**: [Module/Component]

## Context
[Why this decision is needed]

## Decision
[What was decided]

## Consequences
[Impact of decision]

## Alternatives Considered
[Other options and why rejected]
```

### Issue Template

Location: `docs/kb/issues/open/<id>.md`

```markdown
# Issue: [Title]

**ID**: [Unique ID]
**Severity**: Critical / High / Medium / Low
**Component**: [Module]
**Reporter**: [Agent]
**Date**: [Date]

## Description
[What the issue is]

## Reproduction
[How to reproduce]

## Impact
[What could go wrong]

## Suggested Fix
[Recommendation]
```

### Daily Summary

Location: `docs/kb/summaries/daily/YYYY-MM-DD.md`

```markdown
# Daily Summary: [Date]

## Work Completed
- [Module]: [Summary of work]

## Issues Found
- [New issues]

## Issues Resolved
- [Closed issues]

## Next Steps
- [What's planned]
```

## Workflow

1. Check `.icdd/triggers/` for new triggers
2. Process each trigger:
   - Read related files
   - Update appropriate documentation
   - Mark trigger as processed
3. Broadcast context updates to relevant agents
4. Create summary if end of day/week

## Quality Checklist

- [ ] All triggers processed
- [ ] Module docs updated
- [ ] Issues tracked
- [ ] Decisions recorded
- [ ] Summary created
- [ ] Context broadcast sent
