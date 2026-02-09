# ICDD Skill: Interface-Contract-Driven Development

> **Skill Name**: `icdd`
> **Version**: 1.0.0
> **Purpose**: Multi-agent TDD workflow for generating production-quality code from requirements

---

## Quick Start

```bash
# Initialize knowledge base (first time only)
./tools/knowledge/init_kb.sh

# Start with Step 1: Derive requirements from a one-sentence description
# Use: tools/prompts/step1-derive-requirements.md
```

---

## Overview

ICDD (Interface-Contract-Driven Development) is a multi-agent workflow that transforms a one-sentence system description into production-quality, well-tested code through:

1. **Rigorous requirement derivation** — Explicit + implicit requirements, risk analysis
2. **Interface-first design** — Lock APIs before implementation
3. **TDD with role isolation** — Tester and Coder cannot see each other's work
4. **Multi-layer review** — Logic, Test, and Integration reviewers
5. **Adversarial testing** — Attacker actively tries to break the system
6. **Knowledge preservation** — Documenter maintains team memory across agent lifecycles

---

## Agent Roles

| Agent | Phase | Responsibility | Visibility |
|-------|-------|----------------|------------|
| **Architect** | 1-2 | Requirements, interface design | Full |
| **Tester** | 3a | Write tests from contracts (TDD red) | Interfaces only, NO implementation |
| **Coder** | 3b | Implement to pass tests (TDD green) | Interfaces + tests, writes implementation |
| **Reviewer-Logic** | 3c | Audit Coder for stubs/fakes | Implementation only, NO tests |
| **Reviewer-Test** | 3d | Audit Tester for coverage gaps | Tests only, NO implementation |
| **Reviewer-Integration** | 3e | Cross-module consistency | Full |
| **Attacker** | 4a | Penetration testing | Full + runtime |
| **Reviewer-Attack** | 4b | Validate attack quality | Attack reports |
| **Documenter** | All | Knowledge management | Full |

---

## Workflow Pipeline

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         ICDD Pipeline                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  [User] "Build a blockchain with parallel execution"                     │
│      │                                                                   │
│      ▼                                                                   │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ STEP 1: Derive Requirements (Architect)                          │   │
│  │ Prompt: tools/prompts/step1-derive-requirements.md               │   │
│  │ Output: tools/templates/requirements.md (filled)                 │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│      │                                                                   │
│      ▼                                                                   │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ STEP 2: Lock Interfaces (Architect)                              │   │
│  │ Prompt: tools/prompts/step2-lock-interfaces.md                   │   │
│  │ Output: tools/templates/interface-contract.md + trait code       │   │
│  │ Validator: tools/validators/check_interface_drift.sh             │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│      │                                                                   │
│      ▼                                                                   │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ STEP 3a: Write Tests (Tester) — TDD Red                          │   │
│  │ Prompt: tools/prompts/step3-tester.md                            │   │
│  │ Input: Interface contracts + acceptance criteria                 │   │
│  │ Output: tests/*.rs (compile but fail)                            │   │
│  │ Validator: tools/validators/check_test_quality.sh                │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│      │                                                                   │
│      ▼                                                                   │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ STEP 3b: Implement (Coder) — TDD Green                           │   │
│  │ Prompt: tools/prompts/step3-coder.md                             │   │
│  │ Input: Interface contracts + Tester's tests                      │   │
│  │ Output: src/*.rs (all tests pass)                                │   │
│  │ Validators:                                                      │   │
│  │   - tools/validators/check_stub_detection.sh                     │   │
│  │   - tools/validators/check_trait_compliance.sh                   │   │
│  │   - tools/validators/check_interface_drift.sh                    │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│      │                                                                   │
│      ▼                                                                   │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ STEP 3c-e: Review (Parallel)                                     │   │
│  │                                                                  │   │
│  │ [Reviewer-Logic]          [Reviewer-Test]                        │   │
│  │ Prompt: step3-reviewer-   Prompt: step3-reviewer-                │   │
│  │         logic.md                  test.md                        │   │
│  │ Sees: Implementation      Sees: Tests only                       │   │
│  │       ─────────────────────────────────────                      │   │
│  │                    │                                             │   │
│  │                    ▼                                             │   │
│  │            [Reviewer-Integration]                                │   │
│  │            Prompt: step3-reviewer-integration.md                 │   │
│  │            Sees: Everything                                      │   │
│  │                                                                  │   │
│  │ Output: tools/templates/review-checklist.md                      │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│      │                                                                   │
│      ▼                                                                   │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ STEP 4a: Attack (Attacker)                                       │   │
│  │ Prompt: tools/prompts/step4-attacker.md                          │   │
│  │ Output: tools/templates/attack-report.md                         │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│      │                                                                   │
│      ▼                                                                   │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ STEP 4b: Review Attack (Reviewer-Attack)                         │   │
│  │ Prompt: tools/prompts/step4-reviewer-attack.md                   │   │
│  │ Output: tools/templates/attack-review.md + fix tasks             │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│      │                                                                   │
│      ▼                                                                   │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ Issues Found? ──Yes──▶ Return to Coder/Tester for fixes          │   │
│  │       │                                                          │   │
│  │      No                                                          │   │
│  │       │                                                          │   │
│  │       ▼                                                          │   │
│  │ [Documenter] Update knowledge base                               │   │
│  │ Prompt: tools/prompts/documenter.md                              │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│      │                                                                   │
│      ▼                                                                   │
│  Module Complete → Next Module or Final Acceptance                       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## File Index

### Prompts (System prompts for each agent)

| File | Agent | Purpose |
|------|-------|---------|
| `prompts/step1-derive-requirements.md` | Architect | Derive requirements from description |
| `prompts/step2-lock-interfaces.md` | Architect | Design and lock interfaces |
| `prompts/step3-tester.md` | Tester | Write tests (TDD first step) |
| `prompts/step3-coder.md` | Coder | Implement code (TDD second step) |
| `prompts/step3-reviewer-logic.md` | Reviewer-Logic | Audit implementation quality |
| `prompts/step3-reviewer-test.md` | Reviewer-Test | Audit test coverage |
| `prompts/step3-reviewer-integration.md` | Reviewer-Integration | Cross-module consistency |
| `prompts/step4-attacker.md` | Attacker | Penetration testing |
| `prompts/step4-reviewer-attack.md` | Reviewer-Attack | Validate attack coverage |
| `prompts/documenter.md` | Documenter | Knowledge management |
| `prompts/agent-handoff.md` | All | Handoff when agent ends |

### Templates (Structured output formats)

| File | Purpose |
|------|---------|
| `templates/requirements.md` | Requirements graph + risk register + acceptance matrix |
| `templates/interface-contract.md` | API/trait/protocol definitions |
| `templates/review-checklist.md` | Review findings (Logic/Test/Integration) |
| `templates/attack-report.md` | Penetration test results |
| `templates/attack-review.md` | Attack quality assessment |
| `templates/agent-handoff.md` | Agent transition documentation |

### Validators (Automated quality checks)

| Script | Purpose | When to Run |
|--------|---------|-------------|
| `validators/check_trait_compliance.sh` | Verify trait implementations | After Coder |
| `validators/check_interface_drift.sh` | Detect interface modifications | After any code change |
| `validators/check_acceptance.sh` | Verify acceptance criteria coverage | After Tester |
| `validators/check_stub_detection.sh` | Detect stubs/fake implementations | After Coder |
| `validators/check_test_quality.sh` | Detect meaningless tests | After Tester |

### Worktree (Parallel development)

| Script | Purpose |
|--------|---------|
| `worktree/create_worktrees.sh` | Create isolated worktrees for parallel module development |
| `worktree/merge_worktrees.sh` | Merge completed worktrees, list conflicts for manual resolution |

### Knowledge (Team memory management)

| Script | Purpose |
|--------|---------|
| `knowledge/init_kb.sh` | Initialize docs/kb/ structure |
| `knowledge/trigger_documenter.sh` | Signal Documenter to update KB |
| `knowledge/broadcast_context.py` | Notify agents of context updates |
| `knowledge/check_kb_health.sh` | Verify KB integrity |

---

## Knowledge Base Structure

```
docs/kb/
├── index.md                 # Master index (Documenter maintains)
├── glossary.md              # Term definitions
├── agents/                  # Per-agent experience & patterns
│   ├── tester.md
│   ├── coder.md
│   ├── attacker.md
│   └── ...
├── modules/                 # Per-module design decisions & issues
├── decisions/               # Architecture Decision Records (ADRs)
├── issues/
│   ├── open/               # Unresolved issues
│   └── resolved/           # Resolved (with solutions)
└── summaries/
    ├── daily/              # Daily progress
    └── weekly/             # Weekly milestones
```

---

## Usage Examples

### Example 1: Start a New Project

```bash
# 1. Initialize knowledge base
./tools/knowledge/init_kb.sh

# 2. Invoke Architect with Step 1 prompt
# Feed tools/prompts/step1-derive-requirements.md as system prompt
# User input: "Build a blockchain with parallel transaction execution"

# 3. Review and confirm requirements.md output

# 4. Invoke Architect with Step 2 prompt
# Lock interfaces in interface-contract.md

# 5. For each module, run the TDD cycle (Step 3a → 3b → 3c-e → 4)
```

### Example 2: Develop a Single Module

```bash
# Assuming interfaces are locked, develop module "bach-primitives"

# Step 3a: Tester writes tests
# Input to Tester: interface-contract.md (primitives section) + acceptance criteria
# Tester CANNOT see any implementation

# Validate test quality
./tools/validators/check_test_quality.sh tests/

# Step 3b: Coder implements
# Input to Coder: interface-contract.md + tests/primitives.rs
# Coder CANNOT modify tests

# Validate implementation
./tools/validators/check_stub_detection.sh src/primitives/
./tools/validators/check_trait_compliance.sh src/types/traits.rs src/primitives/
./tools/validators/check_interface_drift.sh src/types/traits.rs

# Run tests
cargo test -p bach-primitives

# Step 3c-e: Reviews (can run in parallel)
# Reviewer-Logic sees: src/primitives/ (no tests)
# Reviewer-Test sees: tests/primitives.rs (no implementation)
# Reviewer-Integration sees: everything

# Step 4: Attack + Review
# Attacker tries to break the module
# Reviewer-Attack validates attack coverage
```

### Example 3: Parallel Module Development

```bash
# Create worktrees for parallel development
./tools/worktree/create_worktrees.sh . bach-primitives bach-crypto bach-rlp

# Each worktree can have its own Tester → Coder → Review cycle
# When all complete:
./tools/worktree/merge_worktrees.sh
```

### Example 4: Agent Handoff

```bash
# When an agent completes or is being replaced:
# 1. Agent fills out tools/templates/agent-handoff.md
# 2. Trigger Documenter to process handoff
./tools/knowledge/trigger_documenter.sh "Coder" "bach-primitives" "Completed Address and H256 implementation"

# Documenter updates:
# - docs/kb/modules/primitives.md
# - docs/kb/agents/coder.md
# - docs/kb/index.md
```

---

## Validation Checkpoints

Run these validators at key points:

| Checkpoint | Validators to Run |
|------------|-------------------|
| After Tester completes | `check_test_quality.sh`, `check_acceptance.sh` |
| After Coder completes | `check_stub_detection.sh`, `check_trait_compliance.sh`, `check_interface_drift.sh` |
| Before merge | `check_interface_drift.sh` (ensure no interface changes) |
| After Attack | Review `attack-report.md` for Critical/High issues |

---

## Key Constraints

### Isolation Rules (MUST enforce)

1. **Tester CANNOT see implementation code** — Tests are written from contracts only
2. **Coder CANNOT modify tests** — Must make code fit tests, not vice versa
3. **Reviewer-Logic CANNOT see tests** — Reviews implementation blind to test expectations
4. **Reviewer-Test CANNOT see implementation** — Reviews tests blind to how they're satisfied
5. **Interfaces are LOCKED after Step 2** — Any drift triggers rejection

### Anti-Patterns to Detect

| Anti-Pattern | Validator | Detection |
|--------------|-----------|-----------|
| Stub implementations | `check_stub_detection.sh` | `todo!()`, `unimplemented!()`, empty bodies |
| Fake tests | `check_test_quality.sh` | `assert!(true)`, empty test bodies |
| Interface drift | `check_interface_drift.sh` | Git diff on locked files |
| Missing coverage | `check_acceptance.sh` | Acceptance criteria without tests |

---

## Orchestration Notes

When using this skill with Claude Code or other AI tools:

1. **Single module at a time**: Complete the full cycle for one module before starting another
2. **Explicit role switching**: Clearly state which agent role is active
3. **Preserve isolation**: When switching roles, ensure proper information hiding
4. **Run validators**: Execute validation scripts at each checkpoint
5. **Document everything**: Trigger Documenter after each major completion

### Claude Code Team Mode

```bash
# Create a team for module development
# Team lead coordinates: Tester → Coder → Reviewers → Attacker

# Spawn agents with appropriate prompts:
# - Tester gets: step3-tester.md + interface-contract.md + acceptance criteria
# - Coder gets: step3-coder.md + interface-contract.md + test files
# - Each Reviewer gets their specific prompt + allowed files only
```

---

## Final Acceptance Criteria

A module is complete when:

- [ ] All tests pass (`cargo test -p <module>`)
- [ ] `check_stub_detection.sh` passes (no fake implementations)
- [ ] `check_trait_compliance.sh` passes (all traits implemented)
- [ ] `check_interface_drift.sh` passes (interfaces unchanged)
- [ ] `check_test_quality.sh` passes (meaningful tests)
- [ ] `check_acceptance.sh` passes (all criteria covered)
- [ ] Review checklists have no blocking issues
- [ ] Attack report has no unresolved Critical/High vulnerabilities
- [ ] Knowledge base updated by Documenter

---

## Changelog

- **v1.0.0** (2024): Initial release
  - 11 agent prompts
  - 6 output templates
  - 5 validation scripts
  - 4 worktree scripts
  - 4 knowledge management scripts
  - Full docs/kb/ structure
