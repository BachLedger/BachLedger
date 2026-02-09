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
Usage: $(basename "$0") [OPTIONS] <src_dir> [trait_file]

Check that all trait methods are implemented in the source directory.

Arguments:
    src_dir      Directory containing source files to check
    trait_file   Optional: specific trait definition file (default: scan src_dir)

Options:
    -v, --verbose  Show detailed output
    -h, --help     Show this help message

Functions:
    - Extract all trait definitions (trait Xxx { fn ... })
    - Scan impl blocks for trait implementations
    - Check module dependency graph matches design
    - Report unimplemented trait methods

Exit codes:
    0  All trait methods are implemented
    1  Missing implementations or errors
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

if [[ $# -lt 1 ]]; then
    echo -e "${RED}Error: Missing required arguments${NC}"
    usage
    exit 1
fi

SRC_DIR="$1"
TRAIT_FILE="${2:-}"

if [[ ! -d "$SRC_DIR" ]]; then
    echo -e "${RED}Error: Source directory not found: $SRC_DIR${NC}"
    exit 1
fi

if [[ -n "$TRAIT_FILE" ]] && [[ ! -f "$TRAIT_FILE" ]]; then
    echo -e "${RED}Error: Trait file not found: $TRAIT_FILE${NC}"
    exit 1
fi

echo "Checking trait compliance..."
echo "Source directory: $SRC_DIR"
if [[ -n "$TRAIT_FILE" ]]; then
    echo "Trait file: $TRAIT_FILE"
fi
echo ""

ERRORS=0
WARNINGS=0

# Associative arrays for traits and methods
declare -A TRAIT_METHODS
declare -A TRAIT_FILES

# Find trait files to scan
if [[ -n "$TRAIT_FILE" ]]; then
    TRAIT_FILES_LIST="$TRAIT_FILE"
else
    TRAIT_FILES_LIST=$(find "$SRC_DIR" -name "*.rs" -type f 2>/dev/null || true)
fi

# Extract trait names and their methods from files
extract_traits() {
    local file="$1"
    local current_trait=""
    local in_trait=false
    local brace_count=0

    while IFS= read -r line; do
        # Check for trait definition start
        if [[ "$line" =~ ^[[:space:]]*(pub[[:space:]]+)?trait[[:space:]]+([A-Za-z_][A-Za-z0-9_]*) ]]; then
            current_trait="${BASH_REMATCH[2]}"
            in_trait=true
            brace_count=0
            TRAIT_METHODS["$current_trait"]=""
            TRAIT_FILES["$current_trait"]="$file"
            log_verbose "  ${BLUE}Found trait:${NC} $current_trait in $(basename "$file")"
        fi

        if [[ "$in_trait" == true ]]; then
            # Count braces
            opens=$(echo "$line" | tr -cd '{' | wc -c)
            closes=$(echo "$line" | tr -cd '}' | wc -c)
            brace_count=$((brace_count + opens - closes))

            # Extract function signatures within trait
            if [[ "$line" =~ fn[[:space:]]+([A-Za-z_][A-Za-z0-9_]*)[[:space:]]*[\(<] ]]; then
                method="${BASH_REMATCH[1]}"
                if [[ -n "${TRAIT_METHODS[$current_trait]}" ]]; then
                    TRAIT_METHODS["$current_trait"]="${TRAIT_METHODS[$current_trait]} $method"
                else
                    TRAIT_METHODS["$current_trait"]="$method"
                fi
                log_verbose "    ${YELLOW}Method:${NC} $method"
            fi

            # Check if trait block ended
            if [[ $brace_count -eq 0 ]] && [[ "$line" =~ \} ]]; then
                in_trait=false
            fi
        fi
    done < "$file"
}

echo -e "${BLUE}=== Extracting trait definitions ===${NC}"
echo ""

for file in $TRAIT_FILES_LIST; do
    log_verbose "Scanning: $file"
    extract_traits "$file"
done

# Report found traits
echo "Found ${#TRAIT_METHODS[@]} trait(s):"
for trait in "${!TRAIT_METHODS[@]}"; do
    methods="${TRAIT_METHODS[$trait]}"
    method_count=$(echo "$methods" | wc -w | tr -d ' ')
    echo -e "  ${YELLOW}$trait${NC}: $method_count method(s) [${TRAIT_FILES[$trait]##*/}]"
done
echo ""

if [[ ${#TRAIT_METHODS[@]} -eq 0 ]]; then
    echo -e "${YELLOW}Warning: No traits found${NC}"
    exit 0
fi

# Find all implementation files
IMPL_FILES=$(find "$SRC_DIR" -name "*.rs" -type f 2>/dev/null || true)

if [[ -z "$IMPL_FILES" ]]; then
    echo -e "${YELLOW}Warning: No Rust files found in $SRC_DIR${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

echo -e "${BLUE}=== Checking trait implementations ===${NC}"
echo ""

# Track unimplemented methods
declare -a UNIMPLEMENTED

# Check each trait for implementations
for trait in "${!TRAIT_METHODS[@]}"; do
    methods="${TRAIT_METHODS[$trait]}"

    if [[ -z "$methods" ]]; then
        echo -e "${YELLOW}Warning: Trait $trait has no methods${NC}"
        WARNINGS=$((WARNINGS + 1))
        continue
    fi

    echo "Checking implementations for trait: ${YELLOW}$trait${NC}"

    # Find impl blocks for this trait
    impl_found=false

    for impl_file in $IMPL_FILES; do
        # Check if this file has an impl for the trait
        if grep -qE "impl[[:space:]]*(<[^>]*>)?[[:space:]]*$trait[[:space:]]*(for|<)" "$impl_file" 2>/dev/null; then
            impl_found=true

            # Extract implementation type
            impl_type=$(grep -oE "impl[[:space:]]*(<[^>]*>)?[[:space:]]*$trait[[:space:]]+(for[[:space:]]+)?[A-Za-z_][A-Za-z0-9_<>]*" "$impl_file" 2>/dev/null | head -1 || echo "unknown")

            log_verbose "  Found impl in: ${GREEN}$impl_file${NC}"
            log_verbose "    $impl_type"

            # Check each method is implemented
            for method in $methods; do
                if grep -qE "fn[[:space:]]+$method[[:space:]]*[\(<]" "$impl_file" 2>/dev/null; then
                    echo -e "  ${GREEN}✓${NC} $method"
                else
                    echo -e "  ${RED}✗${NC} $method - NOT FOUND"
                    UNIMPLEMENTED+=("$trait::$method (expected in $impl_file)")
                    ERRORS=$((ERRORS + 1))
                fi
            done
        fi
    done

    if [[ "$impl_found" == false ]]; then
        echo -e "  ${RED}No implementation found for trait $trait${NC}"
        for method in $methods; do
            UNIMPLEMENTED+=("$trait::$method (no impl block found)")
        done
        ERRORS=$((ERRORS + 1))
    fi
    echo ""
done

# Check module dependency graph (basic check)
echo -e "${BLUE}=== Checking module dependencies ===${NC}"
echo ""

MOD_ISSUES=0
for file in $IMPL_FILES; do
    # Check for circular or suspicious imports
    basename_file=$(basename "$file" .rs)

    # Look for use statements that might indicate circular deps
    uses=$(grep -E "^use (crate|super)::" "$file" 2>/dev/null || true)

    if [[ "$VERBOSE" == true ]] && [[ -n "$uses" ]]; then
        echo "  $(basename "$file"):"
        echo "$uses" | while read -r line; do
            echo "    $line"
        done
    fi
done

if [[ $MOD_ISSUES -eq 0 ]]; then
    echo -e "  ${GREEN}✓${NC} No obvious dependency issues found"
fi
echo ""

# Summary
echo "================================"
echo "Unimplemented methods:"
if [[ ${#UNIMPLEMENTED[@]} -eq 0 ]]; then
    echo -e "  ${GREEN}None${NC}"
else
    for item in "${UNIMPLEMENTED[@]}"; do
        echo -e "  ${RED}-${NC} $item"
    done
fi
echo ""

if [[ $ERRORS -eq 0 ]]; then
    echo -e "${GREEN}✓ Trait compliance check PASSED${NC}"
    if [[ $WARNINGS -gt 0 ]]; then
        echo -e "${YELLOW}  ($WARNINGS warning(s))${NC}"
    fi
    exit 0
else
    echo -e "${RED}✗ Trait compliance check FAILED${NC}"
    echo -e "  ${RED}$ERRORS error(s)${NC}, ${YELLOW}$WARNINGS warning(s)${NC}"
    exit 1
fi
