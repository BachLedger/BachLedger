# BachLedger Knowledge Base

Central knowledge repository for the ICDD (Interface-Contract Driven Development) workflow.

## Quick Links

- [Glossary](glossary.md) - Key terms and definitions
- [Agent Roles](#agent-roles) - Agent responsibilities and workflows

## Sections

### [Agents](agents/)
Documentation for each agent role in the ICDD workflow:
- [Tester](agents/tester.md) - Test-first development agent
- [Coder](agents/coder.md) - Implementation agent
- [Attacker](agents/attacker.md) - Security testing agent
- [Reviewer: Logic](agents/reviewer-logic.md) - Logic review agent
- [Reviewer: Test](agents/reviewer-test.md) - Test quality review agent
- [Reviewer: Integration](agents/reviewer-integration.md) - Integration review agent
- [Reviewer: Attack](agents/reviewer-attack.md) - Security review agent
- [Documenter](agents/documenter.md) - Documentation agent

### [Modules](modules/)
Per-module documentation including:
- Interface definitions
- Implementation notes
- Test coverage
- Known issues

### [Decisions](decisions/)
Architecture Decision Records (ADRs) tracking significant design choices.

### [Issues](issues/)
Issue tracking:
- [Open Issues](issues/open/) - Active issues
- [Resolved Issues](issues/resolved/) - Closed issues with resolutions

### [Summaries](summaries/)
Progress reports:
- [Daily Summaries](summaries/daily/) - Daily progress
- [Weekly Summaries](summaries/weekly/) - Weekly rollups

## Agent Roles

| Role | Primary Responsibility | Inputs | Outputs |
|------|----------------------|--------|---------|
| Tester | Write failing tests from specs | Interface spec | Test files |
| Coder | Implement to pass tests | Failing tests | Implementation |
| Attacker | Find edge cases and vulnerabilities | Implementation | Attack tests |
| Reviewer-Logic | Verify correctness | Code + tests | Review report |
| Reviewer-Test | Verify test quality | Tests | Review report |
| Reviewer-Integration | Verify system integration | All code | Review report |
| Reviewer-Attack | Verify security | Attack results | Security report |
| Documenter | Update knowledge base | All outputs | Documentation |

## Workflow

```
Spec -> Tester -> Coder -> Attacker -> Reviewers -> Documenter
          ^                                |
          +--------------------------------+
                    (iteration)
```

## Getting Started

1. Review the [Glossary](glossary.md) for terminology
2. Read your [agent role documentation](agents/)
3. Check [modules/](modules/) for current work
4. Review [open issues](issues/open/) for known problems
