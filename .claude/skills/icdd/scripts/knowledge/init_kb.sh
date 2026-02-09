#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# init_kb.sh - Initialize the knowledge base directory structure
# =============================================================================

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEFAULT_KB_PATH="$SCRIPT_DIR/../../docs/kb"

# =============================================================================
# Functions
# =============================================================================

print_usage() {
    cat <<EOF
${CYAN}Usage:${NC} $(basename "$0") [options] [kb_path]

${CYAN}Description:${NC}
    Initialize the docs/kb/ knowledge base directory structure for ICDD workflow.

${CYAN}Arguments:${NC}
    kb_path     Path to knowledge base directory (default: docs/kb)

${CYAN}Options:${NC}
    -h, --help  Show this help message
    -f, --force Overwrite existing files

${CYAN}Created Structure:${NC}
    docs/kb/
    ├── index.md           # Main index with navigation
    ├── glossary.md        # Key terms and definitions
    ├── agents/            # Agent role documentation
    │   ├── tester.md
    │   ├── coder.md
    │   ├── attacker.md
    │   ├── reviewer-logic.md
    │   ├── reviewer-test.md
    │   ├── reviewer-integration.md
    │   ├── reviewer-attack.md
    │   └── documenter.md
    ├── modules/           # Per-module documentation
    ├── decisions/         # Architecture Decision Records
    ├── issues/
    │   ├── open/          # Active issues
    │   └── resolved/      # Closed issues
    └── summaries/
        ├── daily/         # Daily progress
        └── weekly/        # Weekly rollups

${CYAN}Examples:${NC}
    $(basename "$0")
        Initialize at default path (docs/kb)

    $(basename "$0") /path/to/custom/kb
        Initialize at custom path

    $(basename "$0") --force
        Reinitialize and overwrite existing files
EOF
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

create_agent_doc() {
    local filepath="$1"
    local agent_name="$2"
    local agent_title="$3"
    local responsibilities="$4"

    cat > "$filepath" << EOF
# ${agent_title} Agent

## Role

The ${agent_title} agent is responsible for ${responsibilities}.

## Responsibilities

- Primary tasks and duties
- Quality standards to maintain
- Deliverables expected

## Inputs

- What this agent receives from others
- Required context and resources

## Outputs

- What this agent produces
- Artifacts and deliverables

## Workflow

1. Receive task assignment
2. Review inputs and context
3. Execute primary responsibilities
4. Validate outputs
5. Handoff to next agent

## Best Practices

- Guidelines for effective work
- Common pitfalls to avoid
- Quality benchmarks

## Experience Log

### Lessons Learned

_Record insights and lessons here as work progresses._

### Patterns

_Document recurring patterns and solutions._

### Issues Encountered

_Track problems and their resolutions._
EOF
}

# =============================================================================
# Main
# =============================================================================

FORCE=false
KB_PATH=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            print_usage
            exit 0
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        -*)
            log_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
        *)
            if [[ -z "$KB_PATH" ]]; then
                KB_PATH="$1"
            else
                log_error "Unexpected argument: $1"
                print_usage
                exit 1
            fi
            shift
            ;;
    esac
done

# Set default path if not provided
if [[ -z "$KB_PATH" ]]; then
    KB_PATH="$DEFAULT_KB_PATH"
fi

# Resolve to absolute path
KB_PATH="$(cd "$(dirname "$KB_PATH")" 2>/dev/null && pwd)/$(basename "$KB_PATH")" || KB_PATH="$1"

echo ""
echo -e "${CYAN}============================================${NC}"
echo -e "${CYAN}Knowledge Base Initialization${NC}"
echo -e "${CYAN}============================================${NC}"
log_info "Path: $KB_PATH"
log_info "Force: $FORCE"
echo ""

# Create directory structure
log_info "Creating directory structure..."

DIRECTORIES=(
    "$KB_PATH"
    "$KB_PATH/agents"
    "$KB_PATH/modules"
    "$KB_PATH/decisions"
    "$KB_PATH/issues/open"
    "$KB_PATH/issues/resolved"
    "$KB_PATH/summaries/daily"
    "$KB_PATH/summaries/weekly"
)

for dir in "${DIRECTORIES[@]}"; do
    mkdir -p "$dir"
    log_success "  $dir"
done

# Create .gitkeep files for empty directories
log_info "Creating .gitkeep files..."
GITKEEP_DIRS=(
    "$KB_PATH/modules"
    "$KB_PATH/decisions"
    "$KB_PATH/issues/open"
    "$KB_PATH/issues/resolved"
    "$KB_PATH/summaries/daily"
    "$KB_PATH/summaries/weekly"
)

for dir in "${GITKEEP_DIRS[@]}"; do
    touch "$dir/.gitkeep"
done
log_success "  .gitkeep files created"

# Create index.md
INDEX_FILE="$KB_PATH/index.md"
if [[ ! -f "$INDEX_FILE" ]] || [[ "$FORCE" == "true" ]]; then
    log_info "Creating index.md..."
    cat > "$INDEX_FILE" << 'INDEXEOF'
# BachLedger Knowledge Base

Central knowledge repository for the ICDD (Interface-Contract Driven Development) workflow.

## Quick Links

- [Glossary](glossary.md) - Key terms and definitions
- [Agent Roles](#agent-roles) - Agent responsibilities and workflows

## Recent Updates

_This section is updated automatically by the Documenter agent._

| Date | Module | Agent | Summary |
|------|--------|-------|---------|
| - | - | - | No updates yet |

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
INDEXEOF
    log_success "  index.md created"
else
    log_warn "  index.md exists (use --force to overwrite)"
fi

# Create glossary.md
GLOSSARY_FILE="$KB_PATH/glossary.md"
if [[ ! -f "$GLOSSARY_FILE" ]] || [[ "$FORCE" == "true" ]]; then
    log_info "Creating glossary.md..."
    cat > "$GLOSSARY_FILE" << 'GLOSSARYEOF'
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
GLOSSARYEOF
    log_success "  glossary.md created"
else
    log_warn "  glossary.md exists (use --force to overwrite)"
fi

# Create agent documentation files
log_info "Creating agent documentation files..."

declare -A AGENTS=(
    ["tester"]="Tester|writing failing tests from interface specifications"
    ["coder"]="Coder|implementing code to pass tests while following contracts"
    ["attacker"]="Attacker|finding edge cases, vulnerabilities, and security issues"
    ["reviewer-logic"]="Reviewer-Logic|verifying implementation correctness and logic"
    ["reviewer-test"]="Reviewer-Test|ensuring test quality and coverage"
    ["reviewer-integration"]="Reviewer-Integration|validating system integration and compatibility"
    ["reviewer-attack"]="Reviewer-Attack|reviewing security findings and attack vectors"
    ["documenter"]="Documenter|maintaining the knowledge base and documentation"
)

for agent in "${!AGENTS[@]}"; do
    IFS='|' read -r title responsibilities <<< "${AGENTS[$agent]}"
    agent_file="$KB_PATH/agents/$agent.md"

    if [[ ! -f "$agent_file" ]] || [[ "$FORCE" == "true" ]]; then
        create_agent_doc "$agent_file" "$agent" "$title" "$responsibilities"
        log_success "  agents/$agent.md created"
    else
        log_warn "  agents/$agent.md exists (use --force to overwrite)"
    fi
done

echo ""
echo -e "${CYAN}============================================${NC}"
echo -e "${CYAN}Summary${NC}"
echo -e "${CYAN}============================================${NC}"
log_success "Knowledge base initialized at: $KB_PATH"
echo ""
log_info "Created structure:"
echo "  - index.md (main navigation)"
echo "  - glossary.md (terminology)"
echo "  - agents/ (8 agent documentation files)"
echo "  - modules/ (empty, for module docs)"
echo "  - decisions/ (empty, for ADRs)"
echo "  - issues/open/ (empty, for active issues)"
echo "  - issues/resolved/ (empty, for closed issues)"
echo "  - summaries/daily/ (empty, for daily reports)"
echo "  - summaries/weekly/ (empty, for weekly reports)"
echo ""

exit 0
