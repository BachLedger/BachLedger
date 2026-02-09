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
Usage: $(basename "$0") [OPTIONS] <requirements_file> <tests_dir>

Parse requirements file to extract acceptance criteria and verify test coverage.

Arguments:
    requirements_file  Markdown file containing acceptance criteria (requirements.md)
    tests_dir          Directory containing test files

Options:
    -v, --verbose  Show detailed matching information
    -h, --help     Show this help message

Functions:
    - Parse acceptance criteria from requirements.md
    - Scan test files for matching keywords
    - Report uncovered acceptance items
    - Generate coverage report

Acceptance criteria formats recognized:
    - Lines starting with "- [ ]" or "- [x]"
    - Lines under "## Acceptance Criteria" section
    - Lines containing "MUST", "SHALL", "SHOULD" keywords

Exit codes:
    0  All acceptance criteria have test coverage
    1  Missing test coverage or errors
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

if [[ $# -lt 2 ]]; then
    echo -e "${RED}Error: Missing required arguments${NC}"
    usage
    exit 1
fi

REQUIREMENTS_FILE="$1"
TESTS_DIR="$2"

if [[ ! -f "$REQUIREMENTS_FILE" ]]; then
    echo -e "${RED}Error: Requirements file not found: $REQUIREMENTS_FILE${NC}"
    exit 1
fi

if [[ ! -d "$TESTS_DIR" ]]; then
    echo -e "${RED}Error: Tests directory not found: $TESTS_DIR${NC}"
    exit 1
fi

echo "Checking acceptance criteria coverage..."
echo "Requirements: $REQUIREMENTS_FILE"
echo "Tests directory: $TESTS_DIR"
echo ""

# Extract acceptance criteria from requirements file
declare -a CRITERIA
declare -a CRITERIA_IDS
declare -a CRITERIA_KEYWORDS

extract_criteria() {
    local in_acceptance_section=false
    local criterion_id=0

    while IFS= read -r line; do
        # Check for acceptance criteria section
        if [[ "$line" =~ ^##[[:space:]]*(Acceptance|Requirements|Criteria|验收) ]]; then
            in_acceptance_section=true
            log_verbose "  ${BLUE}Found acceptance section${NC}"
            continue
        fi

        # Check for next section (exit acceptance section)
        if [[ "$line" =~ ^##[[:space:]] ]] && [[ "$in_acceptance_section" == true ]]; then
            in_acceptance_section=false
        fi

        # Extract checkbox items
        if [[ "$line" =~ ^[[:space:]]*-[[:space:]]*\[[[:space:]x]\][[:space:]]*(.+)$ ]]; then
            criterion="${BASH_REMATCH[1]}"
            criterion_id=$((criterion_id + 1))
            CRITERIA+=("$criterion")
            CRITERIA_IDS+=("AC-$criterion_id")
            log_verbose "  ${YELLOW}AC-$criterion_id:${NC} $criterion"
            continue
        fi

        # Extract MUST/SHALL/SHOULD statements
        if [[ "$line" =~ (MUST|SHALL|SHOULD)[[:space:]] ]]; then
            # Clean up the line
            criterion=$(echo "$line" | sed 's/^[[:space:]]*[-*][[:space:]]*//' | sed 's/^[[:space:]]*//')
            if [[ -n "$criterion" ]] && [[ ${#criterion} -gt 10 ]]; then
                criterion_id=$((criterion_id + 1))
                CRITERIA+=("$criterion")
                CRITERIA_IDS+=("AC-$criterion_id")
                log_verbose "  ${YELLOW}AC-$criterion_id:${NC} $criterion"
            fi
            continue
        fi

        # In acceptance section, treat list items as criteria
        if [[ "$in_acceptance_section" == true ]] && [[ "$line" =~ ^[[:space:]]*[-*][[:space:]]+(.+)$ ]]; then
            criterion="${BASH_REMATCH[1]}"
            if [[ ! "$criterion" =~ ^\[.?\] ]]; then  # Skip if already processed as checkbox
                criterion_id=$((criterion_id + 1))
                CRITERIA+=("$criterion")
                CRITERIA_IDS+=("AC-$criterion_id")
                log_verbose "  ${YELLOW}AC-$criterion_id:${NC} $criterion"
            fi
        fi
    done < "$REQUIREMENTS_FILE"
}

echo -e "${BLUE}=== Extracting acceptance criteria ===${NC}"
echo ""

extract_criteria

echo "Found ${#CRITERIA[@]} acceptance criteria"
echo ""

if [[ ${#CRITERIA[@]} -eq 0 ]]; then
    echo -e "${YELLOW}Warning: No acceptance criteria found in $REQUIREMENTS_FILE${NC}"
    echo "Expected formats:"
    echo "  - [ ] Criterion description"
    echo "  - The system MUST do something"
    echo "  - ## Acceptance Criteria section with list items"
    exit 0
fi

# Get all test files and their content
TEST_FILES=$(find "$TESTS_DIR" -name "*.rs" -type f 2>/dev/null || true)
TEST_CONTENT=""

if [[ -n "$TEST_FILES" ]]; then
    TEST_CONTENT=$(cat $TEST_FILES 2>/dev/null || true)
fi

# Extract test names
declare -a TEST_NAMES
while IFS= read -r line; do
    if [[ "$line" =~ fn[[:space:]]+(test_[a-z_0-9]+) ]]; then
        TEST_NAMES+=("${BASH_REMATCH[1]}")
    fi
done <<< "$TEST_CONTENT"

log_verbose "Found ${#TEST_NAMES[@]} test functions"
log_verbose ""

COVERED=0
UNCOVERED=0
declare -a UNCOVERED_CRITERIA
declare -a COVERED_CRITERIA

echo -e "${BLUE}=== Checking coverage ===${NC}"
echo ""

# Check each criterion for coverage
for i in "${!CRITERIA[@]}"; do
    criterion="${CRITERIA[$i]}"
    criterion_id="${CRITERIA_IDS[$i]}"

    # Generate search terms from criterion
    # Extract key words (nouns, verbs) - simplified approach
    search_terms=$(echo "$criterion" | tr '[:upper:]' '[:lower:]' | \
        sed 's/[^a-z0-9 ]/ /g' | \
        tr ' ' '\n' | \
        grep -E '^[a-z]{4,}$' | \
        grep -vE '^(must|shall|should|will|would|could|when|then|that|this|with|from|have|been|being|also|each|able|into|such)$' | \
        sort -u || true)

    found=false
    matching_tests=""
    matching_terms=""

    for term in $search_terms; do
        if [[ -z "$term" ]]; then continue; fi

        # Search in test content
        if echo "$TEST_CONTENT" | grep -qi "$term" 2>/dev/null; then
            found=true
            matching_terms="$matching_terms $term"

            if [[ "$VERBOSE" == true ]]; then
                # Find which test files contain this term
                for tf in $TEST_FILES; do
                    if grep -qi "$term" "$tf" 2>/dev/null; then
                        tf_base=$(basename "$tf")
                        if [[ ! "$matching_tests" =~ "$tf_base" ]]; then
                            matching_tests="$matching_tests $tf_base"
                        fi
                    fi
                done
            fi
        fi
    done

    # Also check for criterion ID in test comments
    if echo "$TEST_CONTENT" | grep -q "$criterion_id" 2>/dev/null; then
        found=true
        matching_terms="$matching_terms [ID match]"
    fi

    if [[ "$found" == true ]]; then
        echo -e "${GREEN}✓${NC} [$criterion_id] $criterion"
        if [[ "$VERBOSE" == true ]]; then
            echo -e "  ${YELLOW}Matched terms:${NC}$matching_terms"
            if [[ -n "$matching_tests" ]]; then
                echo -e "  ${YELLOW}Found in:${NC}$matching_tests"
            fi
        fi
        COVERED=$((COVERED + 1))
        COVERED_CRITERIA+=("$criterion_id")
    else
        echo -e "${RED}✗${NC} [$criterion_id] $criterion"
        log_verbose "  ${YELLOW}Search terms:${NC} $(echo $search_terms | tr '\n' ' ')"
        UNCOVERED=$((UNCOVERED + 1))
        UNCOVERED_CRITERIA+=("$criterion_id: $criterion")
    fi
done

echo ""

# Generate coverage report
echo "================================"
echo -e "${BLUE}Coverage Report${NC}"
echo ""
echo "Summary:"
echo -e "  Total criteria:  ${#CRITERIA[@]}"
echo -e "  Covered:         ${GREEN}$COVERED${NC}"
echo -e "  Uncovered:       ${RED}$UNCOVERED${NC}"

if [[ ${#CRITERIA[@]} -gt 0 ]]; then
    coverage_pct=$((COVERED * 100 / ${#CRITERIA[@]}))

    # Color based on coverage level
    if [[ $coverage_pct -ge 80 ]]; then
        color=$GREEN
    elif [[ $coverage_pct -ge 50 ]]; then
        color=$YELLOW
    else
        color=$RED
    fi

    echo -e "  Coverage:        ${color}$coverage_pct%${NC}"
fi

echo ""

if [[ $UNCOVERED -eq 0 ]]; then
    echo -e "${GREEN}✓ All acceptance criteria have test coverage${NC}"
    exit 0
else
    echo -e "${RED}✗ Missing test coverage for $UNCOVERED criteria:${NC}"
    echo ""
    for criterion in "${UNCOVERED_CRITERIA[@]}"; do
        echo -e "  ${RED}-${NC} $criterion"
    done
    echo ""
    echo "Recommendations:"
    echo "  1. Add tests that cover the uncovered criteria"
    echo "  2. Add criterion IDs (e.g., // AC-1) in test comments"
    echo "  3. Use descriptive test names matching the criteria keywords"
    exit 1
fi
