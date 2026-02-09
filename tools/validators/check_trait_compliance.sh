#!/bin/bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

usage() {
    cat << EOF
Usage: $(basename "$0") <trait_file> <impl_dir>

Check that all trait methods are implemented in the implementation directory.

Arguments:
    trait_file  Path to the file containing trait definitions
    impl_dir    Directory containing implementation files

Options:
    -h, --help  Show this help message

Exit codes:
    0  All trait methods are implemented
    1  Missing implementations or other errors
EOF
}

# Parse arguments
if [[ $# -lt 1 ]] || [[ "$1" == "-h" ]] || [[ "$1" == "--help" ]]; then
    usage
    exit 0
fi

if [[ $# -lt 2 ]]; then
    echo -e "${RED}Error: Missing required arguments${NC}"
    usage
    exit 1
fi

TRAIT_FILE="$1"
IMPL_DIR="$2"

if [[ ! -f "$TRAIT_FILE" ]]; then
    echo -e "${RED}Error: Trait file not found: $TRAIT_FILE${NC}"
    exit 1
fi

if [[ ! -d "$IMPL_DIR" ]]; then
    echo -e "${RED}Error: Implementation directory not found: $IMPL_DIR${NC}"
    exit 1
fi

echo "Checking trait compliance..."
echo "Trait file: $TRAIT_FILE"
echo "Implementation directory: $IMPL_DIR"
echo ""

ERRORS=0
WARNINGS=0

# Extract trait names and their methods
declare -A TRAIT_METHODS

current_trait=""
in_trait=false
brace_count=0

while IFS= read -r line; do
    # Check for trait definition start
    if [[ "$line" =~ ^[[:space:]]*(pub[[:space:]]+)?trait[[:space:]]+([A-Za-z_][A-Za-z0-9_]*) ]]; then
        current_trait="${BASH_REMATCH[2]}"
        in_trait=true
        brace_count=0
        TRAIT_METHODS["$current_trait"]=""
    fi

    if [[ "$in_trait" == true ]]; then
        # Count braces
        opens=$(echo "$line" | tr -cd '{' | wc -c)
        closes=$(echo "$line" | tr -cd '}' | wc -c)
        brace_count=$((brace_count + opens - closes))

        # Extract function signatures within trait
        if [[ "$line" =~ fn[[:space:]]+([A-Za-z_][A-Za-z0-9_]*)[[:space:]]*\( ]]; then
            method="${BASH_REMATCH[1]}"
            if [[ -n "${TRAIT_METHODS[$current_trait]}" ]]; then
                TRAIT_METHODS["$current_trait"]="${TRAIT_METHODS[$current_trait]} $method"
            else
                TRAIT_METHODS["$current_trait"]="$method"
            fi
        fi

        # Check if trait block ended
        if [[ $brace_count -eq 0 ]] && [[ "$line" =~ \} ]]; then
            in_trait=false
        fi
    fi
done < "$TRAIT_FILE"

# Report found traits
echo "Found traits:"
for trait in "${!TRAIT_METHODS[@]}"; do
    methods="${TRAIT_METHODS[$trait]}"
    method_count=$(echo "$methods" | wc -w | tr -d ' ')
    echo -e "  ${YELLOW}$trait${NC}: $method_count methods"
done
echo ""

# Find all implementation files
IMPL_FILES=$(find "$IMPL_DIR" -name "*.rs" -type f 2>/dev/null || true)

if [[ -z "$IMPL_FILES" ]]; then
    echo -e "${YELLOW}Warning: No Rust files found in $IMPL_DIR${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

# Check each trait for implementations
for trait in "${!TRAIT_METHODS[@]}"; do
    methods="${TRAIT_METHODS[$trait]}"

    if [[ -z "$methods" ]]; then
        echo -e "${YELLOW}Warning: Trait $trait has no methods${NC}"
        WARNINGS=$((WARNINGS + 1))
        continue
    fi

    echo "Checking implementations for trait: $trait"

    # Find impl blocks for this trait
    impl_found=false

    for impl_file in $IMPL_FILES; do
        # Check if this file has an impl for the trait
        if grep -q "impl[[:space:]]*$trait[[:space:]]*for\|impl[[:space:]]*<[^>]*>[[:space:]]*$trait[[:space:]]*for" "$impl_file" 2>/dev/null; then
            impl_found=true
            impl_type=$(grep -oP "impl[[:space:]]*(<[^>]*>)?[[:space:]]*$trait[[:space:]]+for[[:space:]]+\K[A-Za-z_][A-Za-z0-9_<>]*" "$impl_file" 2>/dev/null | head -1 || echo "unknown")
            echo -e "  Found impl in: ${GREEN}$impl_file${NC} (for $impl_type)"

            # Check each method is implemented
            for method in $methods; do
                if grep -q "fn[[:space:]]*$method[[:space:]]*(" "$impl_file" 2>/dev/null; then
                    echo -e "    ${GREEN}✓${NC} $method"
                else
                    echo -e "    ${RED}✗${NC} $method - NOT FOUND"
                    ERRORS=$((ERRORS + 1))
                fi
            done
        fi
    done

    if [[ "$impl_found" == false ]]; then
        echo -e "  ${RED}No implementation found for trait $trait${NC}"
        ERRORS=$((ERRORS + 1))
    fi
    echo ""
done

# Summary
echo "================================"
if [[ $ERRORS -eq 0 ]]; then
    echo -e "${GREEN}✓ Trait compliance check PASSED${NC}"
    if [[ $WARNINGS -gt 0 ]]; then
        echo -e "${YELLOW}  ($WARNINGS warnings)${NC}"
    fi
    exit 0
else
    echo -e "${RED}✗ Trait compliance check FAILED${NC}"
    echo -e "  ${RED}$ERRORS error(s)${NC}, ${YELLOW}$WARNINGS warning(s)${NC}"
    exit 1
fi
