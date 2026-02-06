#!/bin/bash
# E2E Test Runner for BachLedger
#
# Usage:
#   ./scripts/e2e.sh             # Run all E2E tests
#   ./scripts/e2e.sh --verbose   # Run with verbose output
#   ./scripts/e2e.sh --filter    # Run specific tests (e.g., --filter transfer)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# Change to root directory
cd "$ROOT_DIR"

# Parse arguments
VERBOSE=""
FILTER=""
NOCAPTURE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose|-v)
            VERBOSE="--verbose"
            shift
            ;;
        --filter)
            FILTER="$2"
            shift 2
            ;;
        --nocapture)
            NOCAPTURE="--nocapture"
            shift
            ;;
        --help|-h)
            echo "E2E Test Runner for BachLedger"
            echo ""
            echo "Usage:"
            echo "  ./scripts/e2e.sh             # Run all E2E tests"
            echo "  ./scripts/e2e.sh --verbose   # Run with verbose output"
            echo "  ./scripts/e2e.sh --filter X  # Run tests matching X"
            echo "  ./scripts/e2e.sh --nocapture # Show test output"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${YELLOW}========================================${NC}"
echo -e "${YELLOW}BachLedger E2E Test Suite${NC}"
echo -e "${YELLOW}========================================${NC}"
echo ""

# Build first to catch compilation errors
echo -e "${YELLOW}Building...${NC}"
cargo build -p bach-e2e 2>&1

if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

echo -e "${GREEN}Build successful${NC}"
echo ""

# Run tests
echo -e "${YELLOW}Running E2E tests...${NC}"
echo ""

# Construct test command
CMD="cargo test -p bach-e2e"

if [ -n "$FILTER" ]; then
    CMD="$CMD $FILTER"
fi

if [ -n "$NOCAPTURE" ]; then
    CMD="$CMD -- --nocapture"
fi

# Run with timing
START=$(date +%s)

if $CMD; then
    END=$(date +%s)
    DURATION=$((END - START))
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}All E2E tests passed!${NC}"
    echo -e "${GREEN}Duration: ${DURATION}s${NC}"
    echo -e "${GREEN}========================================${NC}"
    exit 0
else
    END=$(date +%s)
    DURATION=$((END - START))
    echo ""
    echo -e "${RED}========================================${NC}"
    echo -e "${RED}E2E tests failed!${NC}"
    echo -e "${RED}Duration: ${DURATION}s${NC}"
    echo -e "${RED}========================================${NC}"
    exit 1
fi
