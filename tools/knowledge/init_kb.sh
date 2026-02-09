#!/bin/bash
# init_kb.sh - Initialize the knowledge base directory structure
# Usage: ./init_kb.sh [base_path]

set -e

BASE_PATH="${1:-$(dirname "$0")/../../docs/kb}"
BASE_PATH=$(cd "$(dirname "$BASE_PATH")" && pwd)/$(basename "$BASE_PATH")

echo "Initializing knowledge base at: $BASE_PATH"

# Create directory structure
mkdir -p "$BASE_PATH"/{agents,modules,decisions,issues/{open,resolved},summaries/{daily,weekly}}

# Create .gitkeep files for empty directories
touch "$BASE_PATH/modules/.gitkeep"
touch "$BASE_PATH/decisions/.gitkeep"
touch "$BASE_PATH/issues/open/.gitkeep"
touch "$BASE_PATH/issues/resolved/.gitkeep"
touch "$BASE_PATH/summaries/daily/.gitkeep"
touch "$BASE_PATH/summaries/weekly/.gitkeep"

# Create index.md
cat > "$BASE_PATH/index.md" << 'EOF'
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
EOF

# Create glossary.md
cat > "$BASE_PATH/glossary.md" << 'EOF'
# Glossary

Key terms used in the BachLedger ICDD workflow.

## Development Methodology

### ICDD (Interface-Contract Driven Development)
Development approach where interfaces and their contracts (preconditions, postconditions, invariants) are defined before implementation. Tests are derived from these contracts.

### TDD (Test-Driven Development)
Development cycle: write failing test -> implement minimal code to pass -> refactor. Red-Green-Refactor.

### BDD (Behavior-Driven Development)
Extension of TDD focusing on behavior specifications using Given-When-Then format.

### Design by Contract
Software design approach where interfaces specify obligations (preconditions), guarantees (postconditions), and invariants.

## Rust Concepts

### Trait
Rust's mechanism for defining shared behavior. Similar to interfaces in other languages.

### Interface Contract
The specification of a trait including:
- Method signatures
- Preconditions (what must be true before calling)
- Postconditions (what will be true after calling)
- Invariants (what remains true throughout)

### Acceptance Criteria
Specific, testable conditions that must be met for a feature to be considered complete.

## Blockchain Concepts

### EVM (Ethereum Virtual Machine)
Stack-based virtual machine that executes smart contract bytecode.

### Transaction
Signed message that changes blockchain state.

### Block
Collection of transactions with a header containing metadata.

### State Trie
Merkle Patricia Trie storing account states.

### Gas
Unit measuring computational work in EVM execution.

### Nonce
Counter preventing transaction replay attacks.

## Testing Terms

### Unit Test
Tests a single function or method in isolation.

### Integration Test
Tests multiple components working together.

### Property-Based Test
Tests properties that should hold for all inputs.

### Fuzz Test
Tests with random/malformed inputs to find edge cases.

### Attack Vector
Potential method of exploiting a vulnerability.

### Edge Case
Unusual or extreme input that may cause unexpected behavior.

## Agent Workflow Terms

### Handoff
Transfer of work from one agent to another with context.

### Context Broadcast
Notification to agents about relevant changes.

### Work Unit
Discrete piece of work completed by an agent.

### Trigger
Signal that activates an agent or workflow step.
EOF

echo "Knowledge base initialized at $BASE_PATH"
echo "Created: index.md, glossary.md, and directory structure"
