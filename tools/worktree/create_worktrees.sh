#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# create_worktrees.sh - Create independent git worktrees for parallel development
# =============================================================================

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Default values
INTERFACE_DIR="$PROJECT_ROOT/rust/interfaces"
WORKTREE_BASE="$PROJECT_ROOT/.."

# =============================================================================
# Functions
# =============================================================================

print_usage() {
    cat <<EOF
${CYAN}Usage:${NC} $(basename "$0") <module1> [module2] ...

${CYAN}Description:${NC}
    Create independent git worktrees for parallel module development.
    Each module gets its own worktree at ../bach-<module> on branch feat/<module>.

${CYAN}Arguments:${NC}
    module1...    Module names to create worktrees for (e.g., primitives crypto rlp)

${CYAN}Options:${NC}
    -h, --help    Show this help message
    -b, --base    Base directory for worktrees (default: parent of project root)
    -i, --interfaces  Copy interface files from this directory (default: rust/interfaces)
    --no-interfaces   Don't copy interface files

${CYAN}Examples:${NC}
    $(basename "$0") primitives crypto rlp
        Creates worktrees:
          ../bach-primitives (feat/primitives)
          ../bach-crypto (feat/crypto)
          ../bach-rlp (feat/rlp)

    $(basename "$0") -b /tmp/worktrees evm scheduler
        Creates worktrees in /tmp/worktrees/

${CYAN}Output:${NC}
    Lists all created worktrees with their paths and branches.
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

copy_interfaces() {
    local worktree_path="$1"
    local module="$2"

    if [[ ! -d "$INTERFACE_DIR" ]]; then
        log_warn "Interface directory not found: $INTERFACE_DIR"
        return 0
    fi

    local interface_dest="$worktree_path/interfaces"
    mkdir -p "$interface_dest"

    # Copy all interface files (make them read-only)
    if [[ -n "$(ls -A "$INTERFACE_DIR" 2>/dev/null)" ]]; then
        cp -r "$INTERFACE_DIR/"* "$interface_dest/" 2>/dev/null || true
        # Make interface files read-only
        find "$interface_dest" -type f -exec chmod 444 {} \; 2>/dev/null || true
        log_info "  Copied interface files to $interface_dest (read-only)"
    fi
}

create_worktree() {
    local module="$1"
    local worktree_path="$WORKTREE_BASE/bach-$module"
    local branch_name="feat/$module"

    echo ""
    log_info "Creating worktree for module: ${CYAN}$module${NC}"
    log_info "  Path: $worktree_path"
    log_info "  Branch: $branch_name"

    # Check if worktree already exists
    if [[ -d "$worktree_path" ]]; then
        log_warn "  Worktree already exists at $worktree_path"
        return 1
    fi

    # Try to create worktree with new branch
    if git worktree add "$worktree_path" -b "$branch_name" 2>/dev/null; then
        log_success "  Created worktree with new branch"
    elif git worktree add "$worktree_path" "$branch_name" 2>/dev/null; then
        # Branch already exists, just add worktree
        log_success "  Created worktree (branch already existed)"
    else
        log_error "  Failed to create worktree"
        return 1
    fi

    # Copy interface files if enabled
    if [[ "$COPY_INTERFACES" == "true" ]]; then
        copy_interfaces "$worktree_path" "$module"
    fi

    return 0
}

# =============================================================================
# Main
# =============================================================================

COPY_INTERFACES="true"
MODULES=()

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            print_usage
            exit 0
            ;;
        -b|--base)
            WORKTREE_BASE="$2"
            shift 2
            ;;
        -i|--interfaces)
            INTERFACE_DIR="$2"
            shift 2
            ;;
        --no-interfaces)
            COPY_INTERFACES="false"
            shift
            ;;
        -*)
            log_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
        *)
            MODULES+=("$1")
            shift
            ;;
    esac
done

# Validate arguments
if [[ ${#MODULES[@]} -eq 0 ]]; then
    log_error "At least one module name is required"
    echo ""
    print_usage
    exit 1
fi

# Ensure we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    log_error "Not in a git repository"
    exit 1
fi

# Create base directory if needed
mkdir -p "$WORKTREE_BASE"

echo ""
echo -e "${CYAN}============================================${NC}"
echo -e "${CYAN}Creating Git Worktrees${NC}"
echo -e "${CYAN}============================================${NC}"
log_info "Base directory: $WORKTREE_BASE"
log_info "Modules: ${MODULES[*]}"

CREATED=()
FAILED=()

for module in "${MODULES[@]}"; do
    if create_worktree "$module"; then
        CREATED+=("$module")
    else
        FAILED+=("$module")
    fi
done

echo ""
echo -e "${CYAN}============================================${NC}"
echo -e "${CYAN}Summary${NC}"
echo -e "${CYAN}============================================${NC}"

if [[ ${#CREATED[@]} -gt 0 ]]; then
    echo ""
    log_success "Created worktrees (${#CREATED[@]}):"
    for module in "${CREATED[@]}"; do
        echo -e "  ${GREEN}-${NC} $WORKTREE_BASE/bach-$module ${BLUE}(feat/$module)${NC}"
    done
fi

if [[ ${#FAILED[@]} -gt 0 ]]; then
    echo ""
    log_warn "Failed modules (${#FAILED[@]}):"
    for module in "${FAILED[@]}"; do
        echo -e "  ${RED}-${NC} $module"
    done
fi

echo ""
log_info "List all worktrees with: ${CYAN}git worktree list${NC}"

if [[ ${#FAILED[@]} -gt 0 ]]; then
    exit 1
fi

exit 0
