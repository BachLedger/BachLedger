#!/bin/bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS] <interface_files...>

Compare locked interface files against git baseline and report any changes.

Arguments:
    interface_files  One or more interface files to check for drift

Options:
    -b, --baseline   Git ref to compare against (default: HEAD)
    -h, --help       Show this help message

Exit codes:
    0  No interface drift detected
    1  Interface drift detected or errors
EOF
}

BASELINE="HEAD"
FILES=()

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            usage
            exit 0
            ;;
        -b|--baseline)
            BASELINE="$2"
            shift 2
            ;;
        -*)
            echo -e "${RED}Error: Unknown option $1${NC}"
            usage
            exit 1
            ;;
        *)
            FILES+=("$1")
            shift
            ;;
    esac
done

if [[ ${#FILES[@]} -eq 0 ]]; then
    echo -e "${RED}Error: No interface files specified${NC}"
    usage
    exit 1
fi

echo "Checking interface drift..."
echo "Baseline: $BASELINE"
echo "Files to check: ${#FILES[@]}"
echo ""

DRIFT_COUNT=0
ERRORS=0

for file in "${FILES[@]}"; do
    echo -e "${BLUE}Checking: $file${NC}"

    if [[ ! -f "$file" ]]; then
        echo -e "  ${YELLOW}Warning: File not found (may be new): $file${NC}"
        DRIFT_COUNT=$((DRIFT_COUNT + 1))
        continue
    fi

    # Check if file exists in baseline
    if ! git show "$BASELINE:$file" &>/dev/null 2>&1; then
        # Try with relative path from repo root
        repo_root=$(git rev-parse --show-toplevel 2>/dev/null || echo "")
        if [[ -n "$repo_root" ]]; then
            rel_path=$(realpath --relative-to="$repo_root" "$file" 2>/dev/null || echo "$file")
            if ! git show "$BASELINE:$rel_path" &>/dev/null 2>&1; then
                echo -e "  ${YELLOW}New file (not in baseline)${NC}"
                DRIFT_COUNT=$((DRIFT_COUNT + 1))
                continue
            fi
            file_ref="$rel_path"
        else
            echo -e "  ${YELLOW}New file (not in baseline)${NC}"
            DRIFT_COUNT=$((DRIFT_COUNT + 1))
            continue
        fi
    else
        file_ref="$file"
    fi

    # Get diff between baseline and current
    diff_output=$(git diff "$BASELINE" -- "$file" 2>/dev/null || true)

    if [[ -z "$diff_output" ]]; then
        echo -e "  ${GREEN}✓ No changes${NC}"
        continue
    fi

    echo -e "  ${RED}✗ Interface drift detected:${NC}"
    DRIFT_COUNT=$((DRIFT_COUNT + 1))

    # Analyze the changes
    added_traits=$(echo "$diff_output" | grep -c "^+.*trait[[:space:]]" || echo "0")
    removed_traits=$(echo "$diff_output" | grep -c "^-.*trait[[:space:]]" || echo "0")
    added_methods=$(echo "$diff_output" | grep -c "^+.*fn[[:space:]]" || echo "0")
    removed_methods=$(echo "$diff_output" | grep -c "^-.*fn[[:space:]]" || echo "0")
    added_structs=$(echo "$diff_output" | grep -c "^+.*struct[[:space:]]" || echo "0")
    removed_structs=$(echo "$diff_output" | grep -c "^-.*struct[[:space:]]" || echo "0")

    if [[ $added_traits -gt 0 ]] || [[ $removed_traits -gt 0 ]]; then
        echo -e "    Traits: ${GREEN}+$added_traits${NC} / ${RED}-$removed_traits${NC}"
    fi
    if [[ $added_methods -gt 0 ]] || [[ $removed_methods -gt 0 ]]; then
        echo -e "    Methods: ${GREEN}+$added_methods${NC} / ${RED}-$removed_methods${NC}"
    fi
    if [[ $added_structs -gt 0 ]] || [[ $removed_structs -gt 0 ]]; then
        echo -e "    Structs: ${GREEN}+$added_structs${NC} / ${RED}-$removed_structs${NC}"
    fi

    # Show specific trait/method changes
    echo ""
    echo "    Changed definitions:"

    # Extract changed trait names
    changed_traits=$(echo "$diff_output" | grep -oP "^[+-].*trait[[:space:]]+\K[A-Za-z_][A-Za-z0-9_]*" | sort -u || true)
    for trait in $changed_traits; do
        if echo "$diff_output" | grep -q "^-.*trait[[:space:]]*$trait"; then
            echo -e "      ${RED}- trait $trait${NC}"
        fi
        if echo "$diff_output" | grep -q "^+.*trait[[:space:]]*$trait"; then
            echo -e "      ${GREEN}+ trait $trait${NC}"
        fi
    done

    # Extract changed method signatures
    echo "$diff_output" | grep -E "^[+-].*fn[[:space:]]" | head -20 | while read -r line; do
        if [[ "$line" == -* ]]; then
            echo -e "      ${RED}$line${NC}"
        else
            echo -e "      ${GREEN}$line${NC}"
        fi
    done

    more_lines=$(echo "$diff_output" | grep -cE "^[+-].*fn[[:space:]]" || echo "0")
    if [[ $more_lines -gt 20 ]]; then
        echo -e "      ${YELLOW}... and $((more_lines - 20)) more method changes${NC}"
    fi

    echo ""
done

# Summary
echo "================================"
if [[ $DRIFT_COUNT -eq 0 ]]; then
    echo -e "${GREEN}✓ No interface drift detected${NC}"
    exit 0
else
    echo -e "${RED}✗ Interface drift detected in $DRIFT_COUNT file(s)${NC}"
    echo ""
    echo "To review all changes:"
    echo "  git diff $BASELINE -- ${FILES[*]}"
    echo ""
    echo "To accept changes as new baseline:"
    echo "  git add ${FILES[*]} && git commit -m 'Update interface definitions'"
    exit 1
fi
