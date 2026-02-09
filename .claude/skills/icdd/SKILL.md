---
name: icdd
description: >
  Interface-Contract-Driven Development workflow for generating production-quality code
  through multi-agent TDD. Includes role isolation (Architect, Tester, Coder, Reviewers,
  Attacker), interface-first design, and adversarial testing. Use when building new
  modules requiring rigorous quality, or for test-driven development with locked interfaces.
license: MIT
compatibility: Requires bash, git, cargo (for Rust projects)
metadata:
  author: bachledger
  version: "1.0.0"
allowed-tools: Bash(cargo:*) Bash(git:*) Bash(./scripts/*) Read Write Edit
---

# ICDD: Interface-Contract-Driven Development

Multi-agent TDD workflow that transforms requirements into production-quality, well-tested code.

## Quick Start

```bash
# 1. Initialize knowledge base (first time only)
./scripts/knowledge/init_kb.sh

# 2. Start with Step 1: Derive requirements
# Load: references/step1-derive-requirements.md
```

## Workflow Pipeline

```
User Input ("Build X with Y")
    │
    ▼
┌─────────────────────────────────────────────┐
│ STEP 1: Derive Requirements (Architect)     │
│ Prompt: references/step1-derive-requirements.md
│ Output: assets/requirements.md (filled)     │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│ STEP 2: Lock Interfaces (Architect)         │
│ Prompt: references/step2-lock-interfaces.md │
│ Output: assets/interface-contract.md        │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│ STEP 3a: Write Tests (Tester) — TDD Red     │
│ Prompt: references/step3-tester.md          │
│ Validator: scripts/validators/check_test_quality.sh
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│ STEP 3b: Implement (Coder) — TDD Green      │
│ Prompt: references/step3-coder.md           │
│ Validators:                                 │
│   - scripts/validators/check_stub_detection.sh
│   - scripts/validators/check_trait_compliance.sh
│   - scripts/validators/check_interface_drift.sh
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│ STEP 3c-e: Review (Parallel)                │
│ Reviewer-Logic: references/step3-reviewer-logic.md
│ Reviewer-Test: references/step3-reviewer-test.md
│ Reviewer-Integration: references/step3-reviewer-integration.md
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│ STEP 4: Attack & Review                     │
│ Attacker: references/step4-attacker.md      │
│ Reviewer: references/step4-reviewer-attack.md
└─────────────────────────────────────────────┘
    │
    ▼
Module Complete → Documenter updates KB
```

## Agent Roles

| Agent | Prompt | Responsibility | Visibility |
|-------|--------|----------------|------------|
| Architect | step1-derive-requirements.md | Requirements derivation | Full |
| Architect | step2-lock-interfaces.md | Interface design & lock | Full |
| Tester | step3-tester.md | Write tests (TDD red) | Interfaces only |
| Coder | step3-coder.md | Implement code (TDD green) | Interfaces + tests |
| Reviewer-Logic | step3-reviewer-logic.md | Audit implementation | Implementation only |
| Reviewer-Test | step3-reviewer-test.md | Audit test coverage | Tests only |
| Reviewer-Integration | step3-reviewer-integration.md | Cross-module check | Full |
| Attacker | step4-attacker.md | Penetration testing | Full + runtime |
| Reviewer-Attack | step4-reviewer-attack.md | Validate attacks | Attack reports |
| Documenter | documenter.md | Knowledge management | Full |

## Isolation Rules (MUST Enforce)

1. **Tester CANNOT see implementation** — Tests from contracts only
2. **Coder CANNOT modify tests** — Code fits tests, not vice versa
3. **Reviewer-Logic CANNOT see tests** — Reviews implementation blind
4. **Reviewer-Test CANNOT see implementation** — Reviews tests blind
5. **Interfaces LOCKED after Step 2** — Any drift triggers rejection

## Validators

| Script | Purpose | When |
|--------|---------|------|
| `check_test_quality.sh` | Detect fake tests | After Tester |
| `check_stub_detection.sh` | Detect stub implementations | After Coder |
| `check_trait_compliance.sh` | Verify trait implementations | After Coder |
| `check_interface_drift.sh` | Detect interface changes | After any code change |
| `check_acceptance.sh` | Verify acceptance criteria | After Tester |

Run validators:
```bash
./scripts/validators/check_test_quality.sh tests/
./scripts/validators/check_stub_detection.sh src/
./scripts/validators/check_trait_compliance.sh src/types/traits.rs src/impl/
./scripts/validators/check_interface_drift.sh src/types/traits.rs
```

## Templates (assets/)

| Template | Purpose |
|----------|---------|
| requirements.md | Requirements graph + risk register |
| interface-contract.md | API/trait/protocol definitions |
| review-checklist.md | Review findings |
| attack-report.md | Penetration test results |
| attack-review.md | Attack quality assessment |
| agent-handoff.md | Agent transition docs |

## Knowledge Base

Initialize and maintain team memory:

```bash
# Initialize
./scripts/knowledge/init_kb.sh

# Trigger documenter
./scripts/knowledge/trigger_documenter.sh "Agent" "module" "summary"

# Check health
./scripts/knowledge/check_kb_health.sh
```

Structure:
```
docs/kb/
├── index.md           # Master index
├── glossary.md        # Term definitions
├── agents/            # Per-agent patterns
├── modules/           # Per-module decisions
├── decisions/         # ADRs
├── issues/            # Open/resolved issues
└── summaries/         # Daily/weekly progress
```

## Parallel Development

```bash
# Create worktrees for parallel module development
./scripts/worktree/create_worktrees.sh . module-a module-b module-c

# Merge when complete
./scripts/worktree/merge_worktrees.sh

# Cleanup
./scripts/worktree/cleanup_worktrees.sh
```

## Final Acceptance Criteria

A module is complete when:

- [ ] All tests pass (`cargo test -p <module>`)
- [ ] `check_stub_detection.sh` passes
- [ ] `check_trait_compliance.sh` passes
- [ ] `check_interface_drift.sh` passes
- [ ] `check_test_quality.sh` passes
- [ ] `check_acceptance.sh` passes
- [ ] Review checklists have no blocking issues
- [ ] Attack report has no unresolved Critical/High vulnerabilities
- [ ] Knowledge base updated

## Claude Code Team Mode

```bash
# Spawn agents with appropriate prompts and visibility constraints
# Team lead coordinates: Tester → Coder → Reviewers → Attacker

# Example: Spawn Tester (cannot see implementation)
# Provide: references/step3-tester.md + interface-contract.md + acceptance criteria

# Example: Spawn Coder (cannot modify tests)
# Provide: references/step3-coder.md + interface-contract.md + test files
```

## Usage Examples

### New Project
1. `./scripts/knowledge/init_kb.sh`
2. Load `references/step1-derive-requirements.md` → fill `assets/requirements.md`
3. Load `references/step2-lock-interfaces.md` → fill `assets/interface-contract.md`
4. For each module, run TDD cycle (3a → 3b → 3c-e → 4)

### Single Module
1. Tester: `references/step3-tester.md` + interface contracts
2. Validate: `./scripts/validators/check_test_quality.sh`
3. Coder: `references/step3-coder.md` + tests
4. Validate: `./scripts/validators/check_stub_detection.sh`
5. Reviews: Load appropriate reviewer prompts
6. Attack: `references/step4-attacker.md`

---

*See individual files in `references/` for detailed agent instructions.*
