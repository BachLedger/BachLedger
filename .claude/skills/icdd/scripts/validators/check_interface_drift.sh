#!/usr/bin/env bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

VERBOSE=false
BASELINE="HEAD"

usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS] <interface_path>

Check for interface drift by comparing locked interface files against git baseline.

Arguments:
    interface_path  Directory or file path containing interface definitions

Options:
    -b, --baseline   Git ref to compare against (default: HEAD)
    -v, --verbose    Show detailed diff output
    -h, --help       Show this help message

Functions:
    - Compare locked interface files with git diff
    - Alert on any changes with specific details
    - Report changed traits/methods/formats
    - Return non-zero status if drift detected

Exit codes:
    0  No interface drift detected
    1  Interface drift detected or errors
EOF
}

log_verbose() {
    if [[ "$VERBOSE" == true ]]; then
        echo -e "$1"
    fi
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            usage
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
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
            break
            ;;
    esac
done

if [[ $# -lt 1 ]]; then
    echo -e "${RED}Error: Interface path not specified${NC}"
    usage
    exit 1
fi

INTERFACE_PATH="$1"

# Check if path exists
if [[ ! -e "$INTERFACE_PATH" ]]; then
    echo -e "${RED}Error: Path not found: $INTERFACE_PATH${NC}"
    exit 1
fi

# Get list of files to check
if [[ -d "$INTERFACE_PATH" ]]; then
    FILES=$(find "$INTERFACE_PATH" -name "*.rs" -type f 2>/dev/null || true)
else
    FILES="$INTERFACE_PATH"
fi

if [[ -z "$FILES" ]]; then
    echo -e "${YELLOW}Warning: No Rust files found in $INTERFACE_PATH${NC}"
    exit 0
fi

echo "Checking interface drift..."
echo "Path: $INTERFACE_PATH"
echo "Baseline: $BASELINE"
echo ""

# Check if we're in a git repository
if ! git rev-parse --git-dir &>/dev/null; then
    echo -e "${RED}Error: Not in a git repository${NC}"
    exit 1
fi

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || echo "")

DRIFT_COUNT=0
TOTAL_FILES=0

declare -A DRIFT_DETAILS

echo -e "${BLUE}=== Checking files for drift ===${NC}"
echo ""

for file in $FILES; do
    TOTAL_FILES=$((TOTAL_FILES + 1))

    if [[ ! -f "$file" ]]; then
        log_verbose "  ${YELLOW}Skipping (not a file): $file${NC}"
        continue
    fi

    # Get relative path for git
    if [[ -n "$REPO_ROOT" ]]; then
        rel_path=$(realpath --relative-to="$REPO_ROOT" "$file" 2>/dev/null || echo "$file")
    else
        rel_path="$file"
    fi

    echo -e "Checking: ${BLUE}$(basename "$file")${NC}"

    # Check if file exists in baseline
    if ! git show "$BASELINE:$rel_path" &>/dev/null 2>&1; then
        echo -e "  ${YELLOW}⚠ New file (not in baseline)${NC}"
        DRIFT_COUNT=$((DRIFT_COUNT + 1))
        DRIFT_DETAILS["$file"]="NEW FILE"
        continue
    fi

    # Get diff between baseline and current
    diff_output=$(git diff "$BASELINE" -- "$file" 2>/dev/null || true)

    if [[ -z "$diff_output" ]]; then
        echo -e "  ${GREEN}✓ No changes${NC}"
        continue
    fi

    echo -e "  ${RED}✗ Interface drift detected${NC}"
    DRIFT_COUNT=$((DRIFT_COUNT + 1))

    # Analyze the changes in detail
    added_traits=$(echo "$diff_output" | grep -cE "^\+[^+].*trait[[:space:]]" || echo "0")
    removed_traits=$(echo "$diff_output" | grep -cE "^-[^-].*trait[[:space:]]" || echo "0")
    added_methods=$(echo "$diff_output" | grep -cE "^\+[^+].*fn[[:space:]]" || echo "0")
    removed_methods=$(echo "$diff_output" | grep -cE "^-[^-].*fn[[:space:]]" || echo "0")
    added_structs=$(echo "$diff_output" | grep -cE "^\+[^+].*struct[[:space:]]" || echo "0")
    removed_structs=$(echo "$diff_output" | grep -cE "^-[^-].*struct[[:space:]]" || echo "0")
    added_enums=$(echo "$diff_output" | grep -cE "^\+[^+].*enum[[:space:]]" || echo "0")
    removed_enums=$(echo "$diff_output" | grep -cE "^-[^-].*enum[[:space:]]" || echo "0")

    # Build summary
    summary=""
    if [[ $added_traits -gt 0 ]] || [[ $removed_traits -gt 0 ]]; then
        summary="$summary traits(+$added_traits/-$removed_traits)"
    fi
    if [[ $added_methods -gt 0 ]] || [[ $removed_methods -gt 0 ]]; then
        summary="$summary methods(+$added_methods/-$removed_methods)"
    fi
    if [[ $added_structs -gt 0 ]] || [[ $removed_structs -gt 0 ]]; then
        summary="$summary structs(+$added_structs/-$removed_structs)"
    fi
    if [[ $added_enums -gt 0 ]] || [[ $removed_enums -gt 0 ]]; then
        summary="$summary enums(+$added_enums/-$removed_enums)"
    fi

    DRIFT_DETAILS["$file"]="$summary"

    # Display summary
    echo -e "    Changes: $summary"

    # Extract and display specific changes
    echo ""
    echo "    Changed definitions:"

    # Extract changed trait names
    changed_traits=$(echo "$diff_output" | grep -oE "^[+-][^+-].*trait[[:space:]]+[A-Za-z_][A-Za-z0-9_]*" 2>/dev/null || true)
    if [[ -n "$changed_traits" ]]; then
        echo "$changed_traits" | while read -r line; do
            if [[ "$line" == -* ]]; then
                trait_name=$(echo "$line" | grep -oE "trait[[:space:]]+[A-Za-z_][A-Za-z0-9_]*" | awk '{print $2}')
                echo -e "      ${RED}- trait $trait_name${NC}"
            elif [[ "$line" == +* ]]; then
                trait_name=$(echo "$line" | grep -oE "trait[[:space:]]+[A-Za-z_][A-Za-z0-9_]*" | awk '{print $2}')
                echo -e "      ${GREEN}+ trait $trait_name${NC}"
            fi
        done
    fi

    # Extract changed method signatures (limit output)
    changed_methods=$(echo "$diff_output" | grep -E "^[+-][^+-].*fn[[:space:]]" | head -20 || true)
    if [[ -n "$changed_methods" ]]; then
        echo "$changed_methods" | while read -r line; do
            if [[ "$line" == -* ]]; then
                echo -e "      ${RED}$line${NC}"
            else
                echo -e "      ${GREEN}$line${NC}"
            fi
        done

        more_lines=$(echo "$diff_output" | grep -cE "^[+-][^+-].*fn[[:space:]]" || echo "0")
        if [[ $more_lines -gt 20 ]]; then
            echo -e "      ${YELLOW}... and $((more_lines - 20)) more method changes${NC}"
        fi
    fi

    # Show full diff in verbose mode
    if [[ "$VERBOSE" == true ]]; then
        echo ""
        echo "    Full diff:"
        echo "$diff_output" | head -50 | sed 's/^/      /'
        lines=$(echo "$diff_output" | wc -l)
        if [[ $lines -gt 50 ]]; then
            echo -e "      ${YELLOW}... ($((lines - 50)) more lines)${NC}"
        fi
    fi

    echo ""
done

# Summary
echo "================================"
echo "Drift Summary:"
echo "  Files checked: $TOTAL_FILES"
echo "  Files with drift: $DRIFT_COUNT"
echo ""

if [[ $DRIFT_COUNT -eq 0 ]]; then
    echo -e "${GREEN}✓ No interface drift detected${NC}"
    exit 0
else
    echo -e "${RED}✗ Interface drift detected in $DRIFT_COUNT file(s)${NC}"
    echo ""
    echo "Drifted files:"
    for file in "${!DRIFT_DETAILS[@]}"; do
        echo -e "  ${RED}-${NC} $(basename "$file"): ${DRIFT_DETAILS[$file]}"
    done
    echo ""
    echo "To review all changes:"
    echo "  git diff $BASELINE -- $INTERFACE_PATH"
    echo ""
    echo "To accept changes as new baseline:"
    echo "  git add $INTERFACE_PATH && git commit -m 'Update interface definitions'"
    exit 1
fi
