#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# check_kb_health.sh - Verify knowledge base integrity
# =============================================================================

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script directory and paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DEFAULT_KB_PATH="$PROJECT_ROOT/docs/kb"
ICDD_DIR="$PROJECT_ROOT/.icdd"

# Counters
ERRORS=0
WARNINGS=0
CHECKS_PASSED=0

# =============================================================================
# Functions
# =============================================================================

print_usage() {
    cat <<EOF
${CYAN}Usage:${NC} $(basename "$0") [options] [kb_path]

${CYAN}Description:${NC}
    Check knowledge base integrity and report issues.

${CYAN}Arguments:${NC}
    kb_path     Path to knowledge base (default: docs/kb)

${CYAN}Options:${NC}
    -h, --help      Show this help message
    -v, --verbose   Show all checks (including passed)
    -q, --quiet     Only show errors and warnings
    --fix           Attempt to fix simple issues (create missing dirs)

${CYAN}Checks Performed:${NC}
    1. Required files exist (index.md, glossary.md, agent docs)
    2. Required directories exist
    3. Internal links are valid
    4. No orphan documents (unlinked from index)
    5. Agent files have required sections
    6. Open issues are not stale (>30 days old)
    7. Document format validation

${CYAN}Exit Codes:${NC}
    0 - Healthy (no errors)
    1 - Unhealthy (has errors)
    2 - Degraded (has warnings but no errors)

${CYAN}Examples:${NC}
    $(basename "$0")
        Check default knowledge base

    $(basename "$0") --verbose
        Show all checks including passed

    $(basename "$0") --fix /path/to/kb
        Check and fix simple issues
EOF
}

log_info() {
    if [[ "$VERBOSE" == "true" ]]; then
        echo -e "${BLUE}[INFO]${NC} $*"
    fi
}

log_pass() {
    ((CHECKS_PASSED++))
    if [[ "$VERBOSE" == "true" ]]; then
        echo -e "  ${GREEN}[OK]${NC} $*"
    fi
}

log_warn() {
    ((WARNINGS++))
    if [[ "$QUIET" != "true" ]]; then
        echo -e "  ${YELLOW}[WARN]${NC} $*"
    fi
}

log_error() {
    ((ERRORS++))
    echo -e "  ${RED}[ERROR]${NC} $*"
}

log_section() {
    if [[ "$QUIET" != "true" ]]; then
        echo ""
        echo -e "${CYAN}$*${NC}"
    fi
}

check_required_files() {
    log_section "Checking required files..."

    local REQUIRED_FILES=(
        "index.md"
        "glossary.md"
        "agents/tester.md"
        "agents/coder.md"
        "agents/attacker.md"
        "agents/reviewer-logic.md"
        "agents/reviewer-test.md"
        "agents/reviewer-integration.md"
        "agents/reviewer-attack.md"
        "agents/documenter.md"
    )

    for file in "${REQUIRED_FILES[@]}"; do
        if [[ -f "$KB_PATH/$file" ]]; then
            log_pass "$file"
        else
            log_error "Missing: $file"
        fi
    done
}

check_required_directories() {
    log_section "Checking required directories..."

    local REQUIRED_DIRS=(
        "agents"
        "modules"
        "decisions"
        "issues/open"
        "issues/resolved"
        "summaries/daily"
        "summaries/weekly"
    )

    for dir in "${REQUIRED_DIRS[@]}"; do
        if [[ -d "$KB_PATH/$dir" ]]; then
            log_pass "$dir/"
        else
            if [[ "$FIX" == "true" ]]; then
                mkdir -p "$KB_PATH/$dir"
                touch "$KB_PATH/$dir/.gitkeep"
                log_warn "Created missing directory: $dir/"
            else
                log_error "Missing directory: $dir/"
            fi
        fi
    done
}

check_internal_links() {
    log_section "Checking internal links..."

    local broken_count=0

    while IFS= read -r -d '' mdfile; do
        local file_dir
        file_dir=$(dirname "$mdfile")
        local relfile="${mdfile#$KB_PATH/}"

        # Extract markdown links [text](path)
        while IFS= read -r link; do
            # Skip external links, anchors, and empty
            if [[ "$link" =~ ^https?:// ]] || [[ "$link" =~ ^# ]] || [[ -z "$link" ]]; then
                continue
            fi

            # Remove anchor from link
            local link_path="${link%%#*}"

            if [[ -n "$link_path" ]]; then
                # Resolve relative path
                local resolved_path="$file_dir/$link_path"

                if [[ ! -e "$resolved_path" ]]; then
                    log_warn "Broken link in $relfile: $link"
                    ((broken_count++))
                fi
            fi
        done < <(grep -oE '\]\([^)]+\)' "$mdfile" 2>/dev/null | sed 's/\](//' | sed 's/)$//' || true)
    done < <(find "$KB_PATH" -name "*.md" -print0 2>/dev/null)

    if [[ $broken_count -eq 0 ]]; then
        log_pass "No broken internal links found"
    fi
}

check_orphan_files() {
    log_section "Checking for orphan files..."

    local orphan_count=0
    local index_file="$KB_PATH/index.md"

    if [[ ! -f "$index_file" ]]; then
        log_warn "Cannot check orphans: index.md not found"
        return
    fi

    while IFS= read -r -d '' mdfile; do
        local relpath="${mdfile#$KB_PATH/}"

        # Skip index.md itself and .gitkeep
        if [[ "$relpath" == "index.md" ]]; then
            continue
        fi

        # Check if file is referenced in index.md
        if ! grep -q "$relpath" "$index_file" 2>/dev/null; then
            # Check if parent directory is referenced
            local parent_dir
            parent_dir=$(dirname "$relpath")
            if [[ "$parent_dir" != "." ]] && grep -q "$parent_dir" "$index_file" 2>/dev/null; then
                continue
            fi
            log_warn "Potentially orphan file: $relpath"
            ((orphan_count++))
        fi
    done < <(find "$KB_PATH" -name "*.md" -not -name "index.md" -print0 2>/dev/null)

    if [[ $orphan_count -eq 0 ]]; then
        log_pass "No orphan files detected"
    fi
}

check_agent_structure() {
    log_section "Checking agent file structure..."

    local agent_files
    agent_files=$(find "$KB_PATH/agents" -name "*.md" 2>/dev/null || true)

    for agent_file in $agent_files; do
        local agent_name
        agent_name=$(basename "$agent_file" .md)
        local missing_sections=""

        # Check for key sections
        if ! grep -qE "^## Role|^# .* Agent" "$agent_file" 2>/dev/null; then
            missing_sections="$missing_sections Role/Title"
        fi
        if ! grep -qE "^## Responsibilities|^## What I Do" "$agent_file" 2>/dev/null; then
            missing_sections="$missing_sections Responsibilities"
        fi
        if ! grep -qE "^## Inputs|^## Outputs" "$agent_file" 2>/dev/null; then
            missing_sections="$missing_sections Inputs/Outputs"
        fi

        if [[ -n "$missing_sections" ]]; then
            log_warn "$agent_name.md may be missing sections:$missing_sections"
        else
            log_pass "$agent_name.md has expected structure"
        fi
    done
}

check_stale_issues() {
    log_section "Checking for stale open issues..."

    local open_issues_dir="$KB_PATH/issues/open"
    local stale_count=0
    local stale_days=30

    if [[ ! -d "$open_issues_dir" ]]; then
        log_pass "No open issues directory"
        return
    fi

    while IFS= read -r -d '' issue_file; do
        local relpath="${issue_file#$KB_PATH/}"
        local mtime
        mtime=$(stat -f %m "$issue_file" 2>/dev/null || stat -c %Y "$issue_file" 2>/dev/null)
        local now
        now=$(date +%s)
        local age_days=$(( (now - mtime) / 86400 ))

        if [[ $age_days -gt $stale_days ]]; then
            log_warn "Stale issue ($age_days days old): $relpath"
            ((stale_count++))
        fi
    done < <(find "$open_issues_dir" -name "*.md" -print0 2>/dev/null)

    if [[ $stale_count -eq 0 ]]; then
        log_pass "No stale open issues (>$stale_days days)"
    fi
}

check_document_format() {
    log_section "Checking document format..."

    local format_issues=0

    while IFS= read -r -d '' mdfile; do
        local relpath="${mdfile#$KB_PATH/}"

        # Check for required H1 heading
        if ! head -5 "$mdfile" | grep -qE "^# "; then
            log_warn "Missing H1 heading: $relpath"
            ((format_issues++))
        fi

        # Check for empty files (excluding .gitkeep equivalent)
        local line_count
        line_count=$(wc -l < "$mdfile" | tr -d ' ')
        if [[ $line_count -lt 3 ]]; then
            log_warn "Nearly empty file (<3 lines): $relpath"
            ((format_issues++))
        fi
    done < <(find "$KB_PATH" -name "*.md" -print0 2>/dev/null)

    if [[ $format_issues -eq 0 ]]; then
        log_pass "All documents have valid format"
    fi
}

check_icdd_status() {
    log_section "Checking ICDD workflow status..."

    if [[ -d "$ICDD_DIR/triggers" ]]; then
        local pending
        pending=$(find "$ICDD_DIR/triggers" -name "*.json" 2>/dev/null | wc -l | tr -d ' ')
        if [[ $pending -gt 0 ]]; then
            echo -e "  ${BLUE}[INFO]${NC} Pending triggers: $pending"
        else
            log_pass "No pending triggers"
        fi
    fi

    if [[ -d "$ICDD_DIR/notifications" ]]; then
        local unread
        unread=$(grep -l '"status": "unread"' "$ICDD_DIR/notifications"/*.json 2>/dev/null | wc -l | tr -d ' ')
        if [[ $unread -gt 0 ]]; then
            echo -e "  ${BLUE}[INFO]${NC} Unread notifications: $unread"
        else
            log_pass "No unread notifications"
        fi
    fi
}

# =============================================================================
# Main
# =============================================================================

VERBOSE=false
QUIET=false
FIX=false
KB_PATH=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            print_usage
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -q|--quiet)
            QUIET=true
            shift
            ;;
        --fix)
            FIX=true
            shift
            ;;
        -*)
            echo -e "${RED}[ERROR]${NC} Unknown option: $1" >&2
            print_usage
            exit 1
            ;;
        *)
            if [[ -z "$KB_PATH" ]]; then
                KB_PATH="$1"
            else
                echo -e "${RED}[ERROR]${NC} Unexpected argument: $1" >&2
                print_usage
                exit 1
            fi
            shift
            ;;
    esac
done

# Set default path if not provided
if [[ -z "$KB_PATH" ]]; then
    KB_PATH="$DEFAULT_KB_PATH"
fi

# Resolve to absolute path
if [[ -d "$KB_PATH" ]]; then
    KB_PATH="$(cd "$KB_PATH" && pwd)"
fi

echo ""
echo -e "${CYAN}==========================================${NC}"
echo -e "${CYAN}Knowledge Base Health Check${NC}"
echo -e "${CYAN}==========================================${NC}"
echo -e "${BLUE}Path:${NC} $KB_PATH"
if [[ "$FIX" == "true" ]]; then
    echo -e "${YELLOW}Mode: FIX (will attempt repairs)${NC}"
fi

# Check if KB exists
if [[ ! -d "$KB_PATH" ]]; then
    echo ""
    log_error "Knowledge base directory does not exist: $KB_PATH"
    echo ""
    echo "Run init_kb.sh to create the knowledge base structure."
    exit 1
fi

# Run all checks
check_required_files
check_required_directories
check_internal_links
check_orphan_files
check_agent_structure
check_stale_issues
check_document_format
check_icdd_status

# Summary
echo ""
echo -e "${CYAN}==========================================${NC}"
echo -e "${CYAN}Health Check Summary${NC}"
echo -e "${CYAN}==========================================${NC}"
echo -e "  ${GREEN}Passed:${NC}   $CHECKS_PASSED"
echo -e "  ${YELLOW}Warnings:${NC} $WARNINGS"
echo -e "  ${RED}Errors:${NC}   $ERRORS"

echo ""
if [[ $ERRORS -gt 0 ]]; then
    echo -e "${RED}Status: UNHEALTHY${NC} - Fix errors before proceeding"
    exit 1
elif [[ $WARNINGS -gt 0 ]]; then
    echo -e "${YELLOW}Status: DEGRADED${NC} - Review warnings"
    exit 2
else
    echo -e "${GREEN}Status: HEALTHY${NC}"
    exit 0
fi
