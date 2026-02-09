#!/bin/bash
set -euo pipefail

# Merge worktree branches back to main with conflict resolution

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
    cat <<EOF
Usage: $(basename "$0") [-h] [-n] [-k]

Merge all worktree feature branches back to main.

Options:
    -h, --help      Show this help message
    -n, --dry-run   Show what would be done without making changes
    -k, --keep      Keep worktrees after successful merge

Process:
    1. List all worktrees and their branches
    2. For each non-main branch:
       - Check if interface files were modified (REJECT if so)
       - Attempt merge to main
       - On conflict: call conflict resolution scripts
    3. Clean up worktrees after merge (unless -k specified)

Interface files that trigger rejection:
    - **/interfaces/**
    - **/traits.rs
    - **/api.rs
EOF
    exit "${1:-0}"
}

DRY_RUN=false
KEEP_WORKTREES=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            usage 0
            ;;
        -n|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -k|--keep)
            KEEP_WORKTREES=true
            shift
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage 1
            ;;
    esac
done

# Ensure we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "Error: Not in a git repository" >&2
    exit 1
fi

# Get the main worktree (bare repo or main checkout)
MAIN_WORKTREE="$(git worktree list --porcelain | grep -A2 "^worktree " | head -3 | grep "^worktree " | cut -d' ' -f2)"
CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"

echo "Main worktree: $MAIN_WORKTREE"
echo "Current branch: $CURRENT_BRANCH"
echo ""

# List all worktrees
echo "============================================"
echo "Worktrees"
echo "============================================"
git worktree list
echo ""

# Check for interface file modifications
check_interface_modifications() {
    local branch="$1"
    local base_branch="${2:-main}"

    # Get files changed between base and feature branch
    local changed_files
    changed_files=$(git diff --name-only "$base_branch"..."$branch" 2>/dev/null || echo "")

    if [[ -z "$changed_files" ]]; then
        return 0
    fi

    # Check for interface files
    local interface_files
    interface_files=$(echo "$changed_files" | grep -E "(interfaces/|traits\.rs$|api\.rs$)" || true)

    if [[ -n "$interface_files" ]]; then
        echo "REJECTED: Interface files modified:"
        echo "$interface_files" | sed 's/^/  - /'
        return 1
    fi

    return 0
}

# List conflicted files for manual resolution
resolve_conflicts() {
    local conflicted_files
    conflicted_files=$(git diff --name-only --diff-filter=U)

    if [[ -z "$conflicted_files" ]]; then
        return 0
    fi

    echo "Conflicts detected - manual resolution required:"
    echo "$conflicted_files" | sed 's/^/  - /'
    echo ""
    echo "Please resolve conflicts using git commands:"
    echo "  git diff --name-only --diff-filter=U  # List conflicted files"
    echo "  git checkout --ours <file>            # Keep our version"
    echo "  git checkout --theirs <file>          # Keep their version"
    echo "  # Or manually edit the file to merge changes"
    echo "  git add <file>                        # Mark as resolved"

    return 1
}

# Process each worktree
MERGED=()
FAILED=()
SKIPPED=()

echo "============================================"
echo "Processing worktrees"
echo "============================================"
echo ""

while IFS= read -r line; do
    # Parse worktree list output
    worktree_path=$(echo "$line" | awk '{print $1}')
    branch=$(echo "$line" | awk '{print $3}' | tr -d '[]')

    # Skip main worktree and main/master branches
    if [[ "$branch" == "main" ]] || [[ "$branch" == "master" ]] || [[ "$branch" == "(bare)" ]]; then
        continue
    fi

    # Skip detached HEAD
    if [[ "$branch" == *"detached"* ]]; then
        continue
    fi

    echo "Processing: $branch"
    echo "  Worktree: $worktree_path"

    # Check for interface modifications
    if ! check_interface_modifications "$branch"; then
        FAILED+=("$branch (interface modification)")
        echo ""
        continue
    fi

    if $DRY_RUN; then
        echo "  [DRY-RUN] Would merge $branch into main"
        SKIPPED+=("$branch")
        echo ""
        continue
    fi

    # Attempt merge
    echo "  Merging into main..."

    # Save current state
    original_branch="$CURRENT_BRANCH"

    # Checkout main if not already there
    if [[ "$CURRENT_BRANCH" != "main" ]]; then
        git checkout main 2>/dev/null || {
            echo "  Error: Could not checkout main"
            FAILED+=("$branch (checkout failed)")
            echo ""
            continue
        }
    fi

    # Try to merge
    if git merge --no-ff "$branch" -m "Merge branch '$branch' into main" 2>/dev/null; then
        echo "  Merged successfully!"
        MERGED+=("$branch")

        # Remove worktree if requested
        if ! $KEEP_WORKTREES; then
            echo "  Removing worktree..."
            git worktree remove "$worktree_path" --force 2>/dev/null || true
            git branch -d "$branch" 2>/dev/null || true
        fi
    else
        echo "  Merge conflict detected"

        # Try to resolve conflicts
        if resolve_conflicts; then
            # Complete the merge
            git commit --no-edit 2>/dev/null || true
            echo "  Merged with resolved conflicts!"
            MERGED+=("$branch")

            if ! $KEEP_WORKTREES; then
                echo "  Removing worktree..."
                git worktree remove "$worktree_path" --force 2>/dev/null || true
                git branch -d "$branch" 2>/dev/null || true
            fi
        else
            echo "  Could not resolve all conflicts"
            git merge --abort 2>/dev/null || true
            FAILED+=("$branch (unresolved conflicts)")
        fi
    fi

    # Return to original branch if different
    if [[ "$original_branch" != "main" ]] && [[ "$original_branch" != "$branch" ]]; then
        git checkout "$original_branch" 2>/dev/null || true
    fi

    echo ""
done < <(git worktree list)

echo "============================================"
echo "Summary"
echo "============================================"

if [[ ${#MERGED[@]} -gt 0 ]]; then
    echo ""
    echo "Successfully merged:"
    printf '  - %s\n' "${MERGED[@]}"
fi

if [[ ${#SKIPPED[@]} -gt 0 ]]; then
    echo ""
    echo "Skipped (dry-run):"
    printf '  - %s\n' "${SKIPPED[@]}"
fi

if [[ ${#FAILED[@]} -gt 0 ]]; then
    echo ""
    echo "Failed:"
    printf '  - %s\n' "${FAILED[@]}"
    exit 1
fi

echo ""
echo "Done!"
