#!/usr/bin/env bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

VERBOSE=false

usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS] <src_dir>

Detect stub implementations and incomplete code in Rust source files.

Arguments:
    src_dir  Directory containing source files to check

Options:
    -e, --exclude   Pattern to exclude (can be used multiple times)
    -v, --verbose   Show detailed output
    -h, --help      Show this help message

Detection patterns:
    - todo!() / unimplemented!() / panic!("not impl...")
    - Empty function bodies: { } or { Default::default() }
    - #[allow(unused)] hiding incomplete code
    - Hardcoded return values: return true; return 0; return vec![];

Exit codes:
    0  No stubs or incomplete implementations found
    1  Stubs or incomplete implementations detected
EOF
}

log_verbose() {
    if [[ "$VERBOSE" == true ]]; then
        echo -e "$1"
    fi
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
        -v|--verbose)
            VERBOSE=true
            shift
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
if [[ ${#EXCLUDE_PATTERNS[@]} -gt 0 ]]; then
    echo "Exclude patterns: ${EXCLUDE_PATTERNS[*]}"
fi
echo ""

TOTAL_ISSUES=0
declare -A ISSUES_BY_TYPE
declare -a ISSUES_LIST

# Build find exclude arguments
FIND_EXCLUDES=""
for pattern in "${EXCLUDE_PATTERNS[@]:-}"; do
    if [[ -n "$pattern" ]]; then
        FIND_EXCLUDES="$FIND_EXCLUDES ! -path '*$pattern*'"
    fi
done

# Find all Rust source files
SRC_FILES=$(eval "find '$SRC_DIR' -name '*.rs' -type f $FIND_EXCLUDES 2>/dev/null" || true)

if [[ -z "$SRC_FILES" ]]; then
    echo -e "${YELLOW}Warning: No Rust files found in $SRC_DIR${NC}"
    exit 0
fi

FILE_COUNT=$(echo "$SRC_FILES" | wc -l | tr -d ' ')
log_verbose "Scanning $FILE_COUNT file(s)..."
log_verbose ""

check_pattern() {
    local pattern="$1"
    local description="$2"
    local issue_type="$3"
    local skip_tests="${4:-false}"
    local count=0

    for file in $SRC_FILES; do
        # Skip test files for certain checks
        if [[ "$skip_tests" == "true" ]] && [[ "$file" =~ (test|tests|_test\.rs) ]]; then
            continue
        fi

        matches=$(grep -nE "$pattern" "$file" 2>/dev/null || true)
        if [[ -n "$matches" ]]; then
            while IFS= read -r match; do
                if [[ -n "$match" ]]; then
                    line_num=$(echo "$match" | cut -d: -f1)
                    line_content=$(echo "$match" | cut -d: -f2- | sed 's/^[[:space:]]*//')
                    ISSUES_LIST+=("$file:$line_num: $line_content")
                    count=$((count + 1))
                fi
            done <<< "$matches"
        fi
    done

    if [[ $count -gt 0 ]]; then
        echo -e "${RED}$description ($count found):${NC}"
        # Show recent issues for this pattern
        for item in "${ISSUES_LIST[@]: -$count}"; do
            echo "  $item"
        done
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
check_pattern 'panic!\s*\(\s*"[^"]*[Nn]ot[[:space:]]*[Ii]mpl' "panic!(\"not implemented\") patterns" "panic_notimpl" || true

echo -e "${BLUE}=== Checking for empty implementations ===${NC}"
echo ""

# Check for empty function bodies
EMPTY_FN_COUNT=0
for file in $SRC_FILES; do
    # Skip test files
    if [[ "$file" =~ (test|tests|_test\.rs) ]]; then
        continue
    fi

    # Use awk to find functions with empty or near-empty bodies
    empty_fns=$(awk '
        /fn[[:space:]]+[a-zA-Z_]/ {
            fn_line = NR;
            fn_text = $0;
            in_fn = 1;
            brace_count = 0;
            body_start = 0;
            body_content = "";
        }
        in_fn && /{/ {
            brace_count += gsub(/{/, "{")
            if (body_start == 0) body_start = NR
        }
        in_fn && /}/ { brace_count -= gsub(/}/, "}") }
        in_fn && brace_count > 0 {
            # Collect body content (skip whitespace-only lines)
            if (!/^[[:space:]]*$/ && !/^[[:space:]]*[{}][[:space:]]*$/) {
                gsub(/^[[:space:]]+/, "", $0)
                body_content = body_content " " $0
            }
        }
        in_fn && brace_count == 0 && /}/ {
            # Check if body is empty or trivial
            gsub(/[[:space:]]+/, "", body_content)
            if (body_content == "" || body_content == "{}" || body_content ~ /^[\{\}[:space:]]*$/) {
                print fn_line ": " fn_text
            }
            in_fn = 0
            body_content = ""
        }
    ' "$file" 2>/dev/null || true)

    if [[ -n "$empty_fns" ]]; then
        while IFS= read -r match; do
            if [[ -n "$match" ]]; then
                echo -e "  ${RED}$file:$match${NC}"
                ISSUES_LIST+=("$file:$match [empty body]")
                EMPTY_FN_COUNT=$((EMPTY_FN_COUNT + 1))
            fi
        done <<< "$empty_fns"
    fi
done

if [[ $EMPTY_FN_COUNT -gt 0 ]]; then
    echo -e "${RED}Empty function bodies ($EMPTY_FN_COUNT found)${NC}"
    TOTAL_ISSUES=$((TOTAL_ISSUES + EMPTY_FN_COUNT))
    ISSUES_BY_TYPE["empty_fn"]=$((${ISSUES_BY_TYPE["empty_fn"]:-0} + EMPTY_FN_COUNT))
    echo ""
fi

# Check for Default::default() only returns
check_pattern '\{[[:space:]]*Default::default\(\)[[:space:]]*\}' "Functions returning only Default::default()" "default_only" true || true

echo -e "${BLUE}=== Checking for suppressed warnings ===${NC}"
echo ""

# Check for #[allow(unused)] that might hide incomplete code
check_pattern '#\[allow\(unused' "#[allow(unused)] attributes" "allow_unused" true || true

# Check for #[allow(dead_code)]
check_pattern '#\[allow\(dead_code' "#[allow(dead_code)] attributes" "allow_dead" true || true

echo -e "${BLUE}=== Checking for hardcoded return values ===${NC}"
echo ""

HARDCODED_COUNT=0
for file in $SRC_FILES; do
    # Skip test files
    if [[ "$file" =~ (test|tests|_test\.rs) ]]; then
        continue
    fi

    # Check for functions returning hardcoded true/false (single-line)
    hardcoded_bool=$(grep -nE 'fn[[:space:]]+[a-z_]+.*->[[:space:]]*(bool|Bool)[^{]*\{[[:space:]]*(true|false)[[:space:]]*\}' "$file" 2>/dev/null || true)
    if [[ -n "$hardcoded_bool" ]]; then
        while IFS= read -r match; do
            if [[ -n "$match" ]]; then
                echo -e "  ${YELLOW}$file:$match${NC}"
                ISSUES_LIST+=("$file:$match [hardcoded bool]")
                HARDCODED_COUNT=$((HARDCODED_COUNT + 1))
            fi
        done <<< "$hardcoded_bool"
    fi

    # Check for functions returning 0 or empty collections
    hardcoded_zero=$(grep -nE 'fn[[:space:]]+[a-z_]+.*->[^{]*\{[[:space:]]*(0|""|vec!\[\]|\[\]|None)[[:space:]]*\}' "$file" 2>/dev/null || true)
    if [[ -n "$hardcoded_zero" ]]; then
        while IFS= read -r match; do
            if [[ -n "$match" ]]; then
                echo -e "  ${YELLOW}$file:$match${NC}"
                ISSUES_LIST+=("$file:$match [hardcoded value]")
                HARDCODED_COUNT=$((HARDCODED_COUNT + 1))
            fi
        done <<< "$hardcoded_zero"
    fi

    # Check for return true; return false; return 0; at start of function
    simple_return=$(grep -nE '^\s*return\s+(true|false|0|vec!\[\]|\[\]);?\s*$' "$file" 2>/dev/null || true)
    if [[ -n "$simple_return" ]]; then
        while IFS= read -r match; do
            if [[ -n "$match" ]]; then
                line_num=$(echo "$match" | cut -d: -f1)
                # Check context - is this the only statement in a function?
                context_before=$(sed -n "$((line_num > 5 ? line_num - 5 : 1)),$((line_num - 1))p" "$file" 2>/dev/null | grep -c "fn " || echo "0")
                context_after=$(sed -n "$((line_num + 1)),$((line_num + 2))p" "$file" 2>/dev/null | grep -c "}" || echo "0")
                if [[ $context_before -gt 0 ]] && [[ $context_after -gt 0 ]]; then
                    echo -e "  ${YELLOW}$file:$match${NC}"
                    ISSUES_LIST+=("$file:$match [simple return]")
                    HARDCODED_COUNT=$((HARDCODED_COUNT + 1))
                fi
            fi
        done <<< "$simple_return"
    fi
done

if [[ $HARDCODED_COUNT -gt 0 ]]; then
    echo -e "${RED}Hardcoded return values ($HARDCODED_COUNT found)${NC}"
    TOTAL_ISSUES=$((TOTAL_ISSUES + HARDCODED_COUNT))
    ISSUES_BY_TYPE["hardcoded"]=$((${ISSUES_BY_TYPE["hardcoded"]:-0} + HARDCODED_COUNT))
fi
echo ""

# Summary
echo "================================"
echo -e "${BLUE}Stub Detection Summary${NC}"
echo ""

if [[ ${#ISSUES_BY_TYPE[@]} -gt 0 ]]; then
    echo "Issues by type:"
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
