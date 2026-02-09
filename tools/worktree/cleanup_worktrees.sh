#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# cleanup_worktrees.sh - Clean up git worktrees (merged and/or all)
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

# Options
DRY_RUN=false
FORCE=false
MAIN_BRANCH="main"

# =============================================================================
# Functions
# =============================================================================

print_usage() {
    cat <<EOF
${CYAN}Usage:${NC} $(basename "$0") [options]

${CYAN}Description:${NC}
    Clean up git worktrees that are no longer needed.
    By default, only removes worktrees whose branches have been merged to main.

${CYAN}Options:${NC}
    -h, --help      Show this help message
    -n, --dry-run   Show what would be deleted without actually deleting
    -f, --force     Force delete all worktrees (including unmerged)
    -m, --main      Main branch name to check merges against (default: main)

${CYAN}Examples:${NC}
    $(basename "$0")
        Remove only merged worktrees

    $(basename "$0") --dry-run
        Show what would be removed without deleting

    $(basename "$0") --force
        Remove all worktrees (including unmerged ones)

    $(basename "$0") --main master --dry-run
        Check against 'master' branch in dry-run mode

${CYAN}Safety:${NC}
    - Unmerged worktrees are kept by default with a warning
    - Use --dry-run first to see what will be deleted
    - Use --force to override and delete everything
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

log_dry() {
    echo -e "${CYAN}[DRY-RUN]${NC} $*"
}

is_branch_merged() {
    local branch="$1"
    # Check if branch is merged into main
    git branch --merged "$MAIN_BRANCH" 2>/dev/null | grep -qE "^\s*$branch$"
}

get_worktree_branch() {
    local worktree_path="$1"
    # Get the branch name for a worktree
    git worktree list --porcelain | grep -A2 "worktree $worktree_path$" | grep "^branch" | sed 's/branch refs\/heads\///'
}

remove_worktree() {
    local worktree_path="$1"
    local branch="$2"
    local remove_branch="${3:-false}"

    if [[ "$DRY_RUN" == "true" ]]; then
        log_dry "Would remove worktree: $worktree_path"
        if [[ "$remove_branch" == "true" ]]; then
            log_dry "Would delete branch: $branch"
        fi
        return 0
    fi

    # Remove the worktree
    if git worktree remove "$worktree_path" --force 2>/dev/null; then
        log_success "Removed worktree: $worktree_path"
    else
        log_error "Failed to remove worktree: $worktree_path"
        return 1
    fi

    # Optionally delete the branch
    if [[ "$remove_branch" == "true" ]] && [[ -n "$branch" ]]; then
        if git branch -d "$branch" 2>/dev/null; then
            log_success "Deleted branch: $branch"
        elif [[ "$FORCE" == "true" ]] && git branch -D "$branch" 2>/dev/null; then
            log_warn "Force deleted branch: $branch"
        else
            log_warn "Could not delete branch: $branch"
        fi
    fi

    return 0
}

# =============================================================================
# Main
# =============================================================================

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            print_usage
            exit 0
            ;;
        -n|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        -m|--main)
            MAIN_BRANCH="$2"
            shift 2
            ;;
        -*)
            log_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
        *)
            log_error "Unexpected argument: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Ensure we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    log_error "Not in a git repository"
    exit 1
fi

# Get the main worktree (the original repo)
MAIN_WORKTREE="$(git rev-parse --show-toplevel)"

echo ""
echo -e "${CYAN}============================================${NC}"
echo -e "${CYAN}Git Worktree Cleanup${NC}"
echo -e "${CYAN}============================================${NC}"

if [[ "$DRY_RUN" == "true" ]]; then
    log_info "Mode: ${YELLOW}DRY-RUN${NC} (no changes will be made)"
else
    log_info "Mode: ${RED}LIVE${NC} (changes will be applied)"
fi

if [[ "$FORCE" == "true" ]]; then
    log_warn "Force mode enabled - will delete ALL worktrees"
fi

log_info "Main branch: $MAIN_BRANCH"
log_info "Main worktree: $MAIN_WORKTREE"

echo ""
log_info "Scanning worktrees..."
echo ""

# Get list of worktrees (excluding the main one)
WORKTREES=()
while IFS= read -r line; do
    worktree_path=$(echo "$line" | awk '{print $1}')
    if [[ "$worktree_path" != "$MAIN_WORKTREE" ]] && [[ -n "$worktree_path" ]]; then
        WORKTREES+=("$worktree_path")
    fi
done < <(git worktree list | tail -n +1)

if [[ ${#WORKTREES[@]} -eq 0 ]]; then
    log_info "No additional worktrees found."
    exit 0
fi

log_info "Found ${#WORKTREES[@]} worktree(s):"
echo ""

MERGED=()
UNMERGED=()
REMOVED=0
KEPT=0

for worktree in "${WORKTREES[@]}"; do
    branch=$(get_worktree_branch "$worktree")
    branch_display="${branch:-<detached>}"

    echo -e "  ${BLUE}Worktree:${NC} $worktree"
    echo -e "  ${BLUE}Branch:${NC}   $branch_display"

    if [[ -z "$branch" ]]; then
        # Detached HEAD
        if [[ "$FORCE" == "true" ]]; then
            echo -e "  ${YELLOW}Status:${NC}   Detached HEAD - will remove (--force)"
            if remove_worktree "$worktree" "" false; then
                ((REMOVED++))
            fi
        else
            echo -e "  ${YELLOW}Status:${NC}   Detached HEAD - keeping (use --force to remove)"
            ((KEPT++))
            UNMERGED+=("$worktree")
        fi
    elif is_branch_merged "$branch"; then
        echo -e "  ${GREEN}Status:${NC}   Merged into $MAIN_BRANCH - removing"
        MERGED+=("$worktree")
        if remove_worktree "$worktree" "$branch" true; then
            ((REMOVED++))
        fi
    else
        if [[ "$FORCE" == "true" ]]; then
            echo -e "  ${YELLOW}Status:${NC}   NOT merged - removing anyway (--force)"
            if remove_worktree "$worktree" "$branch" true; then
                ((REMOVED++))
            fi
        else
            echo -e "  ${RED}Status:${NC}   NOT merged into $MAIN_BRANCH - keeping"
            ((KEPT++))
            UNMERGED+=("$worktree")
        fi
    fi
    echo ""
done

# Prune stale worktree entries
if [[ "$DRY_RUN" != "true" ]]; then
    log_info "Pruning stale worktree entries..."
    git worktree prune
fi

echo ""
echo -e "${CYAN}============================================${NC}"
echo -e "${CYAN}Summary${NC}"
echo -e "${CYAN}============================================${NC}"

if [[ "$DRY_RUN" == "true" ]]; then
    echo -e "  ${CYAN}Would remove:${NC} $REMOVED worktree(s)"
    echo -e "  ${CYAN}Would keep:${NC}   $KEPT worktree(s)"
else
    echo -e "  ${GREEN}Removed:${NC} $REMOVED worktree(s)"
    echo -e "  ${YELLOW}Kept:${NC}    $KEPT worktree(s)"
fi

if [[ ${#UNMERGED[@]} -gt 0 ]] && [[ "$FORCE" != "true" ]]; then
    echo ""
    log_warn "Unmerged worktrees were kept:"
    for wt in "${UNMERGED[@]}"; do
        echo -e "  ${YELLOW}-${NC} $wt"
    done
    echo ""
    log_info "Use ${CYAN}--force${NC} to remove all worktrees regardless of merge status"
fi

echo ""

if [[ ${#UNMERGED[@]} -gt 0 ]] && [[ "$FORCE" != "true" ]]; then
    exit 1
fi

exit 0
