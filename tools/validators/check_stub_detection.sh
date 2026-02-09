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
Usage: $(basename "$0") [OPTIONS] <src_dir>

Detect stub implementations and incomplete code in Rust source files.

Detects:
    - todo!(), unimplemented!(), panic!("not impl...")
    - Empty implementations: { }, { Default::default() }
    - #[allow(unused)] hiding code
    - Hardcoded return values bypassing logic

Arguments:
    src_dir  Directory containing source files to check

Options:
    -e, --exclude   Pattern to exclude (can be used multiple times)
    -h, --help      Show this help message

Exit codes:
    0  No stubs or incomplete implementations found
    1  Stubs or incomplete implementations detected
EOF
}

EXCLUDE_PATTERNS=()
SRC_DIR=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            usage
            exit 0
            ;;
        -e|--exclude)
            EXCLUDE_PATTERNS+=("$2")
            shift 2
            ;;
        -*)
            echo -e "${RED}Error: Unknown option $1${NC}"
            usage
            exit 1
            ;;
        *)
            SRC_DIR="$1"
            shift
            ;;
    esac
done

if [[ -z "$SRC_DIR" ]]; then
    echo -e "${RED}Error: Source directory not specified${NC}"
    usage
    exit 1
fi

if [[ ! -d "$SRC_DIR" ]]; then
    echo -e "${RED}Error: Source directory not found: $SRC_DIR${NC}"
    exit 1
fi

echo "Detecting stubs and incomplete implementations..."
echo "Source directory: $SRC_DIR"
echo ""

TOTAL_ISSUES=0
declare -A ISSUES_BY_TYPE

# Build exclude pattern for grep
EXCLUDE_ARGS=""
for pattern in "${EXCLUDE_PATTERNS[@]:-}"; do
    if [[ -n "$pattern" ]]; then
        EXCLUDE_ARGS="$EXCLUDE_ARGS --exclude=$pattern"
    fi
done

# Find all Rust source files (excluding tests by default for some checks)
SRC_FILES=$(find "$SRC_DIR" -name "*.rs" -type f 2>/dev/null || true)

if [[ -z "$SRC_FILES" ]]; then
    echo -e "${YELLOW}Warning: No Rust files found in $SRC_DIR${NC}"
    exit 0
fi

check_pattern() {
    local pattern="$1"
    local description="$2"
    local issue_type="$3"
    local results=""

    for file in $SRC_FILES; do
        # Skip test files for certain checks
        if [[ "$issue_type" == "hardcoded" ]] && [[ "$file" =~ test ]]; then
            continue
        fi

        matches=$(grep -nE "$pattern" "$file" 2>/dev/null || true)
        if [[ -n "$matches" ]]; then
            while IFS= read -r match; do
                if [[ -n "$match" ]]; then
                    line_num=$(echo "$match" | cut -d: -f1)
                    line_content=$(echo "$match" | cut -d: -f2-)
                    results="$results\n  $file:$line_num: $line_content"
                fi
            done <<< "$matches"
        fi
    done

    if [[ -n "$results" ]]; then
        count=$(echo -e "$results" | grep -c "^  " || echo "0")
        echo -e "${RED}$description ($count found):${NC}"
        echo -e "$results"
        echo ""
        TOTAL_ISSUES=$((TOTAL_ISSUES + count))
        ISSUES_BY_TYPE["$issue_type"]=$((${ISSUES_BY_TYPE["$issue_type"]:-0} + count))
        return 1
    fi
    return 0
}

echo -e "${BLUE}=== Checking for explicit stubs ===${NC}"
echo ""

# Check for todo!()
check_pattern 'todo!\s*\(' "todo!() macros" "todo" || true

# Check for unimplemented!()
check_pattern 'unimplemented!\s*\(' "unimplemented!() macros" "unimplemented" || true

# Check for panic!("not implemented") variants
check_pattern 'panic!\s*\(\s*"[^"]*not[[:space:]]*impl' "panic!(\"not implemented\") patterns" "panic_notimpl" || true

echo -e "${BLUE}=== Checking for empty implementations ===${NC}"
echo ""

# Check for empty function bodies (accounting for various formatting)
# This is tricky - look for fn ... { } or fn ... {\n}
for file in $SRC_FILES; do
    # Skip test files
    if [[ "$file" =~ test ]]; then
        continue
    fi

    # Use awk to find functions with empty or near-empty bodies
    empty_fns=$(awk '
        /fn[[:space:]]+[a-zA-Z_]/ {
            fn_line = NR;
            fn_text = $0;
            in_fn = 1;
            brace_count = 0;
            body_lines = 0;
            has_content = 0;
        }
        in_fn && /{/ { brace_count += gsub(/{/, "{") }
        in_fn && /}/ { brace_count -= gsub(/}/, "}") }
        in_fn && brace_count > 0 && !/^[[:space:]]*$/ && !/^[[:space:]]*[{}][[:space:]]*$/ {
            body_lines++
            if (!/^[[:space:]]*(\/\/|\/\*|\*)/) {
                has_content = 1
            }
        }
        in_fn && brace_count == 0 && /}/ {
            if (body_lines <= 1 && !has_content) {
                print fn_line ": " fn_text
            }
            in_fn = 0
        }
    ' "$file" 2>/dev/null || true)

    if [[ -n "$empty_fns" ]]; then
        while IFS= read -r match; do
            if [[ -n "$match" ]]; then
                echo -e "  ${RED}$file:$match${NC}"
                TOTAL_ISSUES=$((TOTAL_ISSUES + 1))
                ISSUES_BY_TYPE["empty_fn"]=$((${ISSUES_BY_TYPE["empty_fn"]:-0} + 1))
            fi
        done <<< "$empty_fns"
    fi
done

# Check for Default::default() only returns
check_pattern 'fn[^{]*\{[[:space:]]*Default::default\(\)[[:space:]]*\}' "Functions returning only Default::default()" "default_only" || true

echo -e "${BLUE}=== Checking for suppressed warnings ===${NC}"
echo ""

# Check for #[allow(unused)] that might hide incomplete code
check_pattern '#\[allow\(unused' "#[allow(unused)] attributes" "allow_unused" || true

# Check for #[allow(dead_code)]
check_pattern '#\[allow\(dead_code' "#[allow(dead_code)] attributes" "allow_dead" || true

echo -e "${BLUE}=== Checking for hardcoded return values ===${NC}"
echo ""

# Check for suspicious hardcoded returns (common stub patterns)
# Look for functions that just return literals
for file in $SRC_FILES; do
    # Skip test files
    if [[ "$file" =~ test ]]; then
        continue
    fi

    # Check for functions returning hardcoded true/false
    hardcoded=$(grep -nE 'fn[[:space:]]+[a-z_]+.*->.*bool[^{]*\{[[:space:]]*(true|false)[[:space:]]*\}' "$file" 2>/dev/null || true)
    if [[ -n "$hardcoded" ]]; then
        while IFS= read -r match; do
            if [[ -n "$match" ]]; then
                echo -e "  ${YELLOW}$file:$match${NC}"
                TOTAL_ISSUES=$((TOTAL_ISSUES + 1))
                ISSUES_BY_TYPE["hardcoded"]=$((${ISSUES_BY_TYPE["hardcoded"]:-0} + 1))
            fi
        done <<< "$hardcoded"
    fi

    # Check for functions returning 0 or empty values
    hardcoded_zero=$(grep -nE 'fn[[:space:]]+[a-z_]+.*->[^{]*\{[[:space:]]*(0|""|vec!\[\]|\[\])[[:space:]]*\}' "$file" 2>/dev/null || true)
    if [[ -n "$hardcoded_zero" ]]; then
        while IFS= read -r match; do
            if [[ -n "$match" ]]; then
                echo -e "  ${YELLOW}$file:$match${NC}"
                TOTAL_ISSUES=$((TOTAL_ISSUES + 1))
                ISSUES_BY_TYPE["hardcoded"]=$((${ISSUES_BY_TYPE["hardcoded"]:-0} + 1))
            fi
        done <<< "$hardcoded_zero"
    fi
done
echo ""

# Summary
echo "================================"
echo "Stub Detection Summary:"
echo ""

if [[ ${#ISSUES_BY_TYPE[@]} -gt 0 ]]; then
    for issue_type in "${!ISSUES_BY_TYPE[@]}"; do
        count="${ISSUES_BY_TYPE[$issue_type]}"
        case "$issue_type" in
            todo) desc="todo!() macros" ;;
            unimplemented) desc="unimplemented!() macros" ;;
            panic_notimpl) desc="panic!(not impl) patterns" ;;
            empty_fn) desc="Empty function bodies" ;;
            default_only) desc="Default::default() only" ;;
            allow_unused) desc="#[allow(unused)]" ;;
            allow_dead) desc="#[allow(dead_code)]" ;;
            hardcoded) desc="Hardcoded returns" ;;
            *) desc="$issue_type" ;;
        esac
        echo -e "  ${YELLOW}$desc:${NC} $count"
    done
    echo ""
fi

if [[ $TOTAL_ISSUES -eq 0 ]]; then
    echo -e "${GREEN}✓ No stubs or incomplete implementations detected${NC}"
    exit 0
else
    echo -e "${RED}✗ Found $TOTAL_ISSUES stub(s) or incomplete implementation(s)${NC}"
    echo ""
    echo "Recommendations:"
    echo "  1. Replace todo!() and unimplemented!() with actual implementations"
    echo "  2. Remove or implement empty function bodies"
    echo "  3. Review #[allow(unused)] attributes - they may hide incomplete code"
    echo "  4. Replace hardcoded return values with proper logic"
    exit 1
fi
