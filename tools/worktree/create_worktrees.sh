#!/bin/bash
set -euo pipefail

# Create independent git worktrees for parallel module development

usage() {
    cat <<EOF
Usage: $(basename "$0") <base_dir> <module1> [module2] ...

Create independent git worktrees for parallel module development.

Arguments:
    base_dir    Base directory where worktrees will be created
    module1...  Module names to create worktrees for

Each module gets a worktree at <base_dir>/<module> on branch feat/<module>

Example:
    $(basename "$0") ../worktrees auth payments users
EOF
    exit "${1:-0}"
}

# Parse arguments
if [[ "${1:-}" == "-h" ]] || [[ "${1:-}" == "--help" ]]; then
    usage 0
fi

if [[ $# -lt 2 ]]; then
    echo "Error: At least base_dir and one module required" >&2
    usage 1
fi

BASE_DIR="$1"
shift
MODULES=("$@")

# Ensure we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "Error: Not in a git repository" >&2
    exit 1
fi

# Create base directory if it doesn't exist
mkdir -p "$BASE_DIR"

echo "Creating worktrees in: $BASE_DIR"
echo "Modules: ${MODULES[*]}"
echo ""

CREATED=()
FAILED=()

for module in "${MODULES[@]}"; do
    worktree_path="$BASE_DIR/$module"
    branch_name="feat/$module"

    echo "Creating worktree for module: $module"
    echo "  Path: $worktree_path"
    echo "  Branch: $branch_name"

    if git worktree add "$worktree_path" -b "$branch_name" 2>/dev/null; then
        echo "  Status: SUCCESS"
        CREATED+=("$worktree_path")
    elif git worktree add "$worktree_path" "$branch_name" 2>/dev/null; then
        # Branch already exists, just add worktree
        echo "  Status: SUCCESS (branch existed)"
        CREATED+=("$worktree_path")
    else
        echo "  Status: FAILED"
        FAILED+=("$module")
    fi
    echo ""
done

echo "============================================"
echo "Summary"
echo "============================================"

if [[ ${#CREATED[@]} -gt 0 ]]; then
    echo ""
    echo "Created worktrees:"
    for path in "${CREATED[@]}"; do
        echo "  - $path"
    done
fi

if [[ ${#FAILED[@]} -gt 0 ]]; then
    echo ""
    echo "Failed modules:"
    for module in "${FAILED[@]}"; do
        echo "  - $module"
    done
    exit 1
fi

echo ""
echo "All worktrees created successfully!"
echo ""
echo "List all worktrees with: git worktree list"
