#!/usr/bin/env bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

VERBOSE=false
THRESHOLD=70

usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS] <tests_dir>

Analyze test quality and detect common issues in Rust test files.

Arguments:
    tests_dir  Directory containing test files

Options:
    -t, --threshold   Minimum quality score (0-100, default: 70)
    -v, --verbose     Show detailed analysis
    -h, --help        Show this help message

Detection patterns:
    - Meaningless assertions: assert!(true), assert_eq!(1, 1)
    - Empty test bodies: #[test] fn xxx() { }
    - Tests with only one assert
    - Positive/negative test ratio
    - #[should_panic] tests (negative tests)

Exit codes:
    0  Test quality is sufficient
    1  Test quality issues detected
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
        -t|--threshold)
            THRESHOLD="$2"
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
    echo -e "${RED}Error: Tests directory not specified${NC}"
    usage
    exit 1
fi

TESTS_DIR="$1"

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

FILE_COUNT=$(echo "$TEST_FILES" | wc -l | tr -d ' ')
log_verbose "Scanning $FILE_COUNT file(s)..."
log_verbose ""

TOTAL_TESTS=0
ISSUES=0
POSITIVE_TESTS=0
NEGATIVE_TESTS=0
BOUNDARY_TESTS=0
SINGLE_ASSERT_TESTS=0

declare -a MEANINGLESS_ASSERTIONS
declare -a EMPTY_TESTS
declare -a SINGLE_ASSERT_LIST
declare -a ISSUES_LIST

echo -e "${BLUE}=== Analyzing test files ===${NC}"
echo ""

for file in $TEST_FILES; do
    file_tests=0
    file_issues=0

    log_verbose "Checking: ${YELLOW}$(basename "$file")${NC}"

    # Count test functions
    test_count=$(grep -c '#\[test\]' "$file" 2>/dev/null || echo "0")
    TOTAL_TESTS=$((TOTAL_TESTS + test_count))
    file_tests=$test_count

    # Check for meaningless assertions: assert!(true)
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

    # Check for assert!(false) in non-panic tests
    assert_false=$(grep -n 'assert!\s*(\s*false\s*)' "$file" 2>/dev/null || true)
    if [[ -n "$assert_false" ]]; then
        while IFS= read -r match; do
            if [[ -n "$match" ]]; then
                line_num=$(echo "$match" | cut -d: -f1)
                # Check if this is in a should_panic test
                context=$(sed -n "$((line_num > 10 ? line_num - 10 : 1)),$((line_num))p" "$file" 2>/dev/null || true)
                if ! echo "$context" | grep -q 'should_panic'; then
                    MEANINGLESS_ASSERTIONS+=("$file:$match (assert!(false) outside #[should_panic])")
                    ISSUES=$((ISSUES + 1))
                    file_issues=$((file_issues + 1))
                fi
            fi
        done <<< "$assert_false"
    fi

    # Check for assert_eq!(literal, same_literal)
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

    # Check for assert_ne!(x, x) - always fails
    assert_ne_same=$(grep -nE 'assert_ne!\s*\(\s*([a-z_]+)\s*,\s*\1\s*\)' "$file" 2>/dev/null || true)
    if [[ -n "$assert_ne_same" ]]; then
        while IFS= read -r match; do
            if [[ -n "$match" ]]; then
                MEANINGLESS_ASSERTIONS+=("$file:$match (assert_ne with same value)")
                ISSUES=$((ISSUES + 1))
                file_issues=$((file_issues + 1))
            fi
        done <<< "$assert_ne_same"
    fi

    # Check for empty test bodies using awk
    empty=$(awk '
        /#\[test\]/ { in_test = 1; test_attr_line = NR; next }
        in_test && /fn[[:space:]]+[a-z_]+/ {
            test_line = NR
            test_name = $0
            gsub(/^[[:space:]]+/, "", test_name)
            brace_count = 0
            body_start = 0
            body_content = ""
        }
        in_test && /{/ {
            brace_count += gsub(/{/, "{")
            if (body_start == 0) body_start = NR
        }
        in_test && /}/ { brace_count -= gsub(/}/, "}") }
        in_test && brace_count > 0 && body_start > 0 {
            line = $0
            gsub(/^[[:space:]]+/, "", line)
            gsub(/[[:space:]]+$/, "", line)
            if (line != "" && line != "{" && line != "}") {
                body_content = body_content line
            }
        }
        in_test && brace_count == 0 && /}/ && body_start > 0 {
            gsub(/[[:space:]{}]/, "", body_content)
            if (body_content == "") {
                print test_line ": " test_name
            }
            in_test = 0
            body_content = ""
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

    # Count should_panic tests (negative tests)
    panic_tests=$(grep -c '#\[should_panic' "$file" 2>/dev/null || echo "0")
    NEGATIVE_TESTS=$((NEGATIVE_TESTS + panic_tests))

    # Count error handling tests
    error_tests=$(grep -cE '(is_err\(\)|unwrap_err\(\)|expect_err\(|\.err\(\)|Err\()' "$file" 2>/dev/null || echo "0")
    NEGATIVE_TESTS=$((NEGATIVE_TESTS + error_tests))

    # Positive tests = total - panic tests
    file_positive=$((test_count > panic_tests ? test_count - panic_tests : 0))
    POSITIVE_TESTS=$((POSITIVE_TESTS + file_positive))

    # Check for boundary tests
    boundary=$(grep -cE '(test_.*boundary|test_.*edge|test_.*limit|test_.*max|test_.*min|test_.*zero|test_.*empty|test_.*overflow|test_.*underflow|MAX|MIN|::MAX|::MIN|usize::MAX|u64::MAX|i64::MAX|i64::MIN)' "$file" 2>/dev/null || echo "0")
    BOUNDARY_TESTS=$((BOUNDARY_TESTS + boundary))

    # Check for tests with only one assertion
    single_assert=$(awk '
        /#\[test\]/ { in_test = 1; next }
        in_test && /fn[[:space:]]+[a-z_]+/ {
            test_fn = $0
            test_line = NR
            brace_count = 0
            assert_count = 0
        }
        in_test && /{/ { brace_count += gsub(/{/, "{") }
        in_test && /}/ { brace_count -= gsub(/}/, "}") }
        in_test && brace_count > 0 && /(assert|expect|should)/ {
            assert_count++
        }
        in_test && brace_count == 0 && /}/ {
            if (assert_count == 1) {
                gsub(/^[[:space:]]+/, "", test_fn)
                print test_line ": " test_fn " (only 1 assertion)"
            }
            in_test = 0
        }
    ' "$file" 2>/dev/null || true)

    if [[ -n "$single_assert" ]]; then
        while IFS= read -r match; do
            if [[ -n "$match" ]]; then
                SINGLE_ASSERT_LIST+=("$file:$match")
                SINGLE_ASSERT_TESTS=$((SINGLE_ASSERT_TESTS + 1))
            fi
        done <<< "$single_assert"
    fi

    if [[ "$VERBOSE" == true ]] && [[ $file_tests -gt 0 ]]; then
        echo "  Tests: $file_tests, Issues: $file_issues, Panic tests: $panic_tests"
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

# Report single assertion tests (warning, not error)
if [[ ${#SINGLE_ASSERT_LIST[@]} -gt 0 ]] && [[ "$VERBOSE" == true ]]; then
    echo -e "${YELLOW}Tests with Only One Assertion (${#SINGLE_ASSERT_LIST[@]}):${NC}"
    for test in "${SINGLE_ASSERT_LIST[@]}"; do
        echo "  - $test"
    done
    echo ""
fi

# Calculate and display metrics
echo -e "${BLUE}=== Test Metrics ===${NC}"
echo ""
echo "Total tests found: $TOTAL_TESTS"
echo ""

if [[ $TOTAL_TESTS -gt 0 ]]; then
    # Test type ratio
    echo "Test types:"
    echo -e "  Positive tests (happy path):    ${GREEN}$POSITIVE_TESTS${NC}"
    echo -e "  Negative tests (#[should_panic]): ${YELLOW}$NEGATIVE_TESTS${NC}"
    echo -e "  Tests with single assertion:    ${YELLOW}$SINGLE_ASSERT_TESTS${NC}"

    if [[ $POSITIVE_TESTS -gt 0 ]]; then
        ratio=$((NEGATIVE_TESTS * 100 / POSITIVE_TESTS))
        echo -e "  Negative/Positive ratio:        ${BLUE}$ratio%${NC}"

        if [[ $ratio -lt 20 ]]; then
            echo -e "  ${YELLOW}Warning: Low negative test coverage (recommend >= 20%)${NC}"
            ISSUES_LIST+=("Low negative test ratio: $ratio%")
        fi
    else
        echo -e "  ${YELLOW}Warning: No positive tests found${NC}"
    fi
    echo ""

    # Boundary tests
    echo "Boundary value tests: $BOUNDARY_TESTS"
    boundary_ratio=$((BOUNDARY_TESTS * 100 / TOTAL_TESTS))
    echo -e "  Boundary test ratio: ${BLUE}$boundary_ratio%${NC}"

    if [[ $boundary_ratio -lt 10 ]]; then
        echo -e "  ${YELLOW}Warning: Low boundary test coverage (recommend >= 10%)${NC}"
        ISSUES_LIST+=("Low boundary test coverage: $boundary_ratio%")
    fi
    echo ""
fi

# Calculate quality score
QUALITY_SCORE=100

if [[ $TOTAL_TESTS -gt 0 ]]; then
    # Deduct for meaningless assertions (5 points each)
    meaningless_penalty=$((${#MEANINGLESS_ASSERTIONS[@]} * 5))
    QUALITY_SCORE=$((QUALITY_SCORE - meaningless_penalty))

    # Deduct for empty tests (10 points each)
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

    # Deduct for high single-assertion test ratio (2 points each, max 20)
    single_penalty=$((SINGLE_ASSERT_TESTS * 2))
    if [[ $single_penalty -gt 20 ]]; then
        single_penalty=20
    fi
    QUALITY_SCORE=$((QUALITY_SCORE - single_penalty))

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

# Color based on score
if [[ $QUALITY_SCORE -ge 80 ]]; then
    score_color=$GREEN
elif [[ $QUALITY_SCORE -ge 50 ]]; then
    score_color=$YELLOW
else
    score_color=$RED
fi

echo -e "Quality Score: ${score_color}$QUALITY_SCORE / 100${NC}"
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
        echo ""
    fi

    echo "Recommendations:"
    echo "  1. Remove or fix meaningless assertions (assert!(true), assert_eq!(x, x))"
    echo "  2. Implement empty test bodies or remove unused tests"
    echo "  3. Add more #[should_panic] and error case tests (negative tests)"
    echo "  4. Add boundary value tests (min, max, zero, overflow, empty)"
    echo "  5. Add multiple assertions per test for better coverage"
    exit 1
fi
