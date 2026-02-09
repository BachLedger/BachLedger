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
Usage: $(basename "$0") [OPTIONS] <tests_dir>

Analyze test quality and detect common issues.

Detects:
    - Meaningless assertions: assert!(true), assert_eq!(1, 1)
    - Empty test bodies
    - Ratio of positive vs negative (error) tests
    - Boundary value test coverage

Arguments:
    tests_dir  Directory containing test files

Options:
    -t, --threshold   Minimum quality score (0-100, default: 70)
    -v, --verbose     Show detailed analysis
    -h, --help        Show this help message

Exit codes:
    0  Test quality is sufficient
    1  Test quality issues detected
EOF
}

THRESHOLD=70
VERBOSE=false
TESTS_DIR=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            usage
            exit 0
            ;;
        -t|--threshold)
            THRESHOLD="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -*)
            echo -e "${RED}Error: Unknown option $1${NC}"
            usage
            exit 1
            ;;
        *)
            TESTS_DIR="$1"
            shift
            ;;
    esac
done

if [[ -z "$TESTS_DIR" ]]; then
    echo -e "${RED}Error: Tests directory not specified${NC}"
    usage
    exit 1
fi

if [[ ! -d "$TESTS_DIR" ]]; then
    echo -e "${RED}Error: Tests directory not found: $TESTS_DIR${NC}"
    exit 1
fi

echo "Analyzing test quality..."
echo "Tests directory: $TESTS_DIR"
echo "Quality threshold: $THRESHOLD%"
echo ""

# Find all test files
TEST_FILES=$(find "$TESTS_DIR" -name "*.rs" -type f 2>/dev/null || true)

if [[ -z "$TEST_FILES" ]]; then
    echo -e "${YELLOW}Warning: No Rust files found in $TESTS_DIR${NC}"
    exit 0
fi

TOTAL_TESTS=0
ISSUES=0
POSITIVE_TESTS=0
NEGATIVE_TESTS=0
BOUNDARY_TESTS=0

declare -a MEANINGLESS_ASSERTIONS
declare -a EMPTY_TESTS
declare -a ISSUES_LIST

echo -e "${BLUE}=== Analyzing test files ===${NC}"
echo ""

for file in $TEST_FILES; do
    file_tests=0
    file_issues=0

    if [[ "$VERBOSE" == true ]]; then
        echo -e "Checking: ${YELLOW}$file${NC}"
    fi

    # Count test functions
    test_count=$(grep -c '#\[test\]' "$file" 2>/dev/null || echo "0")
    TOTAL_TESTS=$((TOTAL_TESTS + test_count))
    file_tests=$test_count

    # Check for meaningless assertions
    # assert!(true)
    assert_true=$(grep -n 'assert!\s*(\s*true\s*)' "$file" 2>/dev/null || true)
    if [[ -n "$assert_true" ]]; then
        while IFS= read -r match; do
            if [[ -n "$match" ]]; then
                MEANINGLESS_ASSERTIONS+=("$file:$match")
                ISSUES=$((ISSUES + 1))
                file_issues=$((file_issues + 1))
            fi
        done <<< "$assert_true"
    fi

    # assert!(false) in non-panic tests
    assert_false=$(grep -n 'assert!\s*(\s*false\s*)' "$file" 2>/dev/null || true)
    if [[ -n "$assert_false" ]]; then
        while IFS= read -r match; do
            if [[ -n "$match" ]]; then
                # Check if this is in a should_panic test
                line_num=$(echo "$match" | cut -d: -f1)
                context=$(sed -n "$((line_num > 10 ? line_num - 10 : 1)),$((line_num))p" "$file" 2>/dev/null || true)
                if ! echo "$context" | grep -q 'should_panic'; then
                    MEANINGLESS_ASSERTIONS+=("$file:$match (assert!(false) outside should_panic)")
                    ISSUES=$((ISSUES + 1))
                    file_issues=$((file_issues + 1))
                fi
            fi
        done <<< "$assert_false"
    fi

    # assert_eq!(same, same) - literals
    assert_eq_same=$(grep -nE 'assert_eq!\s*\(\s*([0-9]+|"[^"]*"|true|false)\s*,\s*\1\s*\)' "$file" 2>/dev/null || true)
    if [[ -n "$assert_eq_same" ]]; then
        while IFS= read -r match; do
            if [[ -n "$match" ]]; then
                MEANINGLESS_ASSERTIONS+=("$file:$match")
                ISSUES=$((ISSUES + 1))
                file_issues=$((file_issues + 1))
            fi
        done <<< "$assert_eq_same"
    fi

    # Check for empty test bodies
    # Look for #[test] followed by fn with empty body
    empty=$(awk '
        /#\[test\]/ { in_test = 1; next }
        in_test && /fn[[:space:]]+test_/ {
            test_line = NR
            test_name = $0
            brace_count = 0
            body_start = 0
            next
        }
        in_test && /{/ {
            brace_count += gsub(/{/, "{")
            if (body_start == 0) body_start = NR
        }
        in_test && /}/ {
            brace_count -= gsub(/}/, "}")
            if (brace_count == 0) {
                if (NR == body_start || NR == body_start + 1) {
                    print test_line ": " test_name " (empty body)"
                }
                in_test = 0
            }
        }
    ' "$file" 2>/dev/null || true)

    if [[ -n "$empty" ]]; then
        while IFS= read -r match; do
            if [[ -n "$match" ]]; then
                EMPTY_TESTS+=("$file:$match")
                ISSUES=$((ISSUES + 1))
                file_issues=$((file_issues + 1))
            fi
        done <<< "$empty"
    fi

    # Count positive tests (no error/panic expected)
    positive=$(grep -c '#\[test\]' "$file" 2>/dev/null || echo "0")
    panic_tests=$(grep -c 'should_panic' "$file" 2>/dev/null || echo "0")
    error_tests=$(grep -c 'is_err\|unwrap_err\|expect_err\|Err(' "$file" 2>/dev/null || echo "0")

    # Estimate: subtract panic tests from total for positive
    file_positive=$((positive > panic_tests ? positive - panic_tests : 0))
    POSITIVE_TESTS=$((POSITIVE_TESTS + file_positive))
    NEGATIVE_TESTS=$((NEGATIVE_TESTS + panic_tests))

    # Check for boundary tests
    # Look for tests with boundary-related names or values
    boundary=$(grep -cE '(test_.*boundary|test_.*edge|test_.*limit|test_.*max|test_.*min|test_.*zero|test_.*empty|test_.*overflow|test_.*underflow|MAX|MIN|0\s*,|u64::MAX|i64::MAX|usize::MAX)' "$file" 2>/dev/null || echo "0")
    BOUNDARY_TESTS=$((BOUNDARY_TESTS + boundary))

    if [[ "$VERBOSE" == true ]] && [[ $file_tests -gt 0 ]]; then
        echo "  Tests: $file_tests, Issues: $file_issues"
    fi
done

echo ""
echo -e "${BLUE}=== Test Quality Report ===${NC}"
echo ""

# Report meaningless assertions
if [[ ${#MEANINGLESS_ASSERTIONS[@]} -gt 0 ]]; then
    echo -e "${RED}Meaningless Assertions (${#MEANINGLESS_ASSERTIONS[@]}):${NC}"
    for assertion in "${MEANINGLESS_ASSERTIONS[@]}"; do
        echo "  - $assertion"
    done
    echo ""
fi

# Report empty tests
if [[ ${#EMPTY_TESTS[@]} -gt 0 ]]; then
    echo -e "${RED}Empty Test Bodies (${#EMPTY_TESTS[@]}):${NC}"
    for test in "${EMPTY_TESTS[@]}"; do
        echo "  - $test"
    done
    echo ""
fi

# Calculate metrics
echo -e "${BLUE}=== Test Metrics ===${NC}"
echo ""
echo "Total tests found: $TOTAL_TESTS"
echo ""

if [[ $TOTAL_TESTS -gt 0 ]]; then
    # Test type ratio
    echo "Test types:"
    echo -e "  Positive tests (happy path): ${GREEN}$POSITIVE_TESTS${NC}"
    echo -e "  Negative tests (error cases): ${YELLOW}$NEGATIVE_TESTS${NC}"

    if [[ $POSITIVE_TESTS -gt 0 ]]; then
        ratio=$((NEGATIVE_TESTS * 100 / POSITIVE_TESTS))
        echo -e "  Negative/Positive ratio: ${BLUE}$ratio%${NC}"

        if [[ $ratio -lt 20 ]]; then
            echo -e "  ${YELLOW}Warning: Low negative test coverage (recommend > 20%)${NC}"
            ISSUES_LIST+=("Low negative test ratio: $ratio%")
        fi
    fi
    echo ""

    # Boundary tests
    echo "Boundary value tests: $BOUNDARY_TESTS"
    boundary_ratio=$((BOUNDARY_TESTS * 100 / TOTAL_TESTS))
    echo -e "  Boundary test ratio: ${BLUE}$boundary_ratio%${NC}"

    if [[ $boundary_ratio -lt 10 ]]; then
        echo -e "  ${YELLOW}Warning: Low boundary test coverage (recommend > 10%)${NC}"
        ISSUES_LIST+=("Low boundary test coverage: $boundary_ratio%")
    fi
    echo ""
fi

# Calculate quality score
QUALITY_SCORE=100

if [[ $TOTAL_TESTS -gt 0 ]]; then
    # Deduct for meaningless assertions
    meaningless_penalty=$((${#MEANINGLESS_ASSERTIONS[@]} * 5))
    QUALITY_SCORE=$((QUALITY_SCORE - meaningless_penalty))

    # Deduct for empty tests
    empty_penalty=$((${#EMPTY_TESTS[@]} * 10))
    QUALITY_SCORE=$((QUALITY_SCORE - empty_penalty))

    # Deduct for low negative test ratio
    if [[ $POSITIVE_TESTS -gt 0 ]]; then
        ratio=$((NEGATIVE_TESTS * 100 / POSITIVE_TESTS))
        if [[ $ratio -lt 20 ]]; then
            QUALITY_SCORE=$((QUALITY_SCORE - 10))
        fi
    fi

    # Deduct for low boundary coverage
    if [[ $boundary_ratio -lt 10 ]]; then
        QUALITY_SCORE=$((QUALITY_SCORE - 10))
    fi

    # Ensure score doesn't go below 0
    if [[ $QUALITY_SCORE -lt 0 ]]; then
        QUALITY_SCORE=0
    fi
else
    QUALITY_SCORE=0
    ISSUES_LIST+=("No tests found")
fi

# Summary
echo "================================"
echo -e "Quality Score: ${BLUE}$QUALITY_SCORE / 100${NC}"
echo ""

if [[ $QUALITY_SCORE -ge $THRESHOLD ]]; then
    echo -e "${GREEN}✓ Test quality check PASSED${NC}"
    if [[ $ISSUES -gt 0 ]]; then
        echo -e "  ${YELLOW}($ISSUES issue(s) found, but within acceptable range)${NC}"
    fi
    exit 0
else
    echo -e "${RED}✗ Test quality check FAILED${NC}"
    echo -e "  Score: $QUALITY_SCORE% (threshold: $THRESHOLD%)"
    echo ""

    if [[ ${#ISSUES_LIST[@]} -gt 0 ]]; then
        echo "Issues to address:"
        for issue in "${ISSUES_LIST[@]}"; do
            echo "  - $issue"
        done
    fi

    echo ""
    echo "Recommendations:"
    echo "  1. Remove or fix meaningless assertions"
    echo "  2. Implement empty test bodies or remove unused tests"
    echo "  3. Add more negative/error case tests"
    echo "  4. Add boundary value tests (min, max, zero, empty)"
    exit 1
fi
