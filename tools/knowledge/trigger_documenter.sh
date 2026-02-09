#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# trigger_documenter.sh - Trigger Documenter agent after work completion
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
KB_PATH="$PROJECT_ROOT/docs/kb"
ICDD_DIR="$PROJECT_ROOT/.icdd"
TRIGGER_DIR="$ICDD_DIR/triggers"
LOG_DIR="$ICDD_DIR/logs"

# Valid event types
VALID_EVENTS=("completed" "issue" "decision")
VALID_AGENTS=("tester" "coder" "attacker" "reviewer-logic" "reviewer-test" "reviewer-integration" "reviewer-attack" "documenter" "orchestrator")

# =============================================================================
# Functions
# =============================================================================

print_usage() {
    cat <<EOF
${CYAN}Usage:${NC} $(basename "$0") <agent_name> <module_name> <event_type> [summary]

${CYAN}Description:${NC}
    Trigger the Documenter agent after work completion or significant events.
    Records the trigger event, scans recent changes, and updates the knowledge base.

${CYAN}Arguments:${NC}
    agent_name    Source agent (tester|coder|attacker|reviewer-*|orchestrator)
    module_name   Name of the module worked on
    event_type    Type of event (completed|issue|decision)
    summary       Optional summary of the work (prompted if not provided)

${CYAN}Options:${NC}
    -h, --help    Show this help message

${CYAN}Event Types:${NC}
    completed     Agent finished assigned work successfully
    issue         Agent encountered a problem requiring attention
    decision      A design decision was made that should be documented

${CYAN}Examples:${NC}
    $(basename "$0") tester primitives completed "Added 15 unit tests for Address"
    $(basename "$0") coder evm issue "Blocked by missing interface"
    $(basename "$0") reviewer-logic crypto decision "Use k256 over secp256k1"

${CYAN}Output:${NC}
    - Creates trigger file in .icdd/triggers/
    - Logs event to .icdd/logs/triggers.log
    - Scans recent document changes
    - Updates docs/kb/index.md "Recent Updates" section
EOF
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

validate_agent() {
    local agent="$1"
    for valid in "${VALID_AGENTS[@]}"; do
        if [[ "$agent" == "$valid" ]]; then
            return 0
        fi
    done
    return 1
}

validate_event() {
    local event="$1"
    for valid in "${VALID_EVENTS[@]}"; do
        if [[ "$event" == "$valid" ]]; then
            return 0
        fi
    done
    return 1
}

scan_recent_changes() {
    local module="$1"
    local since="${2:-1 hour ago}"

    log_info "Scanning recent changes..."

    # Find recently modified files in docs/kb
    local changed_files=()
    while IFS= read -r -d '' file; do
        changed_files+=("${file#$KB_PATH/}")
    done < <(find "$KB_PATH" -name "*.md" -mmin -60 -print0 2>/dev/null || true)

    if [[ ${#changed_files[@]} -gt 0 ]]; then
        log_info "  Found ${#changed_files[@]} recently changed file(s):"
        for f in "${changed_files[@]}"; do
            echo "    - $f"
        done
    else
        log_info "  No recent changes detected in knowledge base"
    fi

    # Return files as newline-separated string
    printf '%s\n' "${changed_files[@]}"
}

update_recent_updates() {
    local agent="$1"
    local module="$2"
    local summary="$3"
    local date
    date=$(date +%Y-%m-%d)

    local index_file="$KB_PATH/index.md"

    if [[ ! -f "$index_file" ]]; then
        log_warn "index.md not found, skipping update"
        return 0
    fi

    log_info "Updating Recent Updates in index.md..."

    # Create a new entry line
    local new_entry="| $date | $module | $agent | $summary |"

    # Check if the table exists and has the placeholder
    if grep -q "| - | - | - | No updates yet |" "$index_file"; then
        # Replace placeholder with actual entry
        sed -i.bak "s/| - | - | - | No updates yet |/$new_entry/" "$index_file"
        rm -f "$index_file.bak"
        log_success "  Added first entry to Recent Updates"
    elif grep -q "## Recent Updates" "$index_file"; then
        # Add new entry after the table header (skip header row and separator)
        # Find the line number of the table header row
        local header_line
        header_line=$(grep -n "| Date | Module | Agent | Summary |" "$index_file" | head -1 | cut -d: -f1)

        if [[ -n "$header_line" ]]; then
            # Insert after header + separator (2 lines after header)
            local insert_line=$((header_line + 2))
            sed -i.bak "${insert_line}i\\
$new_entry
" "$index_file"
            rm -f "$index_file.bak"
            log_success "  Added entry to Recent Updates"
        else
            log_warn "  Could not find table header, skipping update"
        fi
    else
        log_warn "  Recent Updates section not found"
    fi
}

create_trigger_file() {
    local agent="$1"
    local module="$2"
    local event="$3"
    local summary="$4"
    local timestamp
    timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    local trigger_id="${agent}_${module}_$(date +%Y%m%d_%H%M%S)"

    mkdir -p "$TRIGGER_DIR"

    local trigger_file="$TRIGGER_DIR/documenter_${trigger_id}.json"

    cat > "$trigger_file" << EOF
{
    "trigger_id": "$trigger_id",
    "timestamp": "$timestamp",
    "source_agent": "$agent",
    "module": "$module",
    "event_type": "$event",
    "summary": "$summary",
    "status": "pending"
}
EOF

    echo "$trigger_file"
}

log_trigger_event() {
    local agent="$1"
    local module="$2"
    local event="$3"
    local summary="$4"

    mkdir -p "$LOG_DIR"

    local log_file="$LOG_DIR/triggers.log"
    local timestamp
    timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)

    echo "$timestamp | TRIGGER | documenter | $agent | $module | $event | $summary" >> "$log_file"
}

print_documenter_prompt() {
    local agent="$1"
    local module="$2"
    local event="$3"
    local summary="$4"

    echo ""
    echo -e "${CYAN}============================================${NC}"
    echo -e "${CYAN}Documenter Activation Prompt${NC}"
    echo -e "${CYAN}============================================${NC}"
    echo ""
    echo "The following event requires documentation:"
    echo ""
    echo "  Source Agent: $agent"
    echo "  Module:       $module"
    echo "  Event Type:   $event"
    echo "  Summary:      $summary"
    echo ""
    echo "Documenter should:"
    echo "  1. Review the changes made by $agent"
    echo "  2. Update docs/kb/modules/$module.md (create if needed)"
    if [[ "$event" == "issue" ]]; then
        echo "  3. Create issue in docs/kb/issues/open/"
    elif [[ "$event" == "decision" ]]; then
        echo "  3. Create ADR in docs/kb/decisions/"
    fi
    echo "  4. Update agent experience log: docs/kb/agents/$agent.md"
    echo "  5. Update index.md if needed"
    echo ""
}

# =============================================================================
# Main
# =============================================================================

# Parse arguments
if [[ "${1:-}" == "-h" ]] || [[ "${1:-}" == "--help" ]]; then
    print_usage
    exit 0
fi

if [[ $# -lt 3 ]]; then
    log_error "Missing required arguments"
    echo ""
    print_usage
    exit 1
fi

AGENT_NAME="$1"
MODULE_NAME="$2"
EVENT_TYPE="$3"
SUMMARY="${4:-}"

# Validate agent name
if ! validate_agent "$AGENT_NAME"; then
    log_error "Invalid agent name: $AGENT_NAME"
    echo "Valid agents: ${VALID_AGENTS[*]}"
    exit 1
fi

# Validate event type
if ! validate_event "$EVENT_TYPE"; then
    log_error "Invalid event type: $EVENT_TYPE"
    echo "Valid events: ${VALID_EVENTS[*]}"
    exit 1
fi

# Prompt for summary if not provided
if [[ -z "$SUMMARY" ]]; then
    echo -n "Enter summary: "
    read -r SUMMARY
    if [[ -z "$SUMMARY" ]]; then
        log_error "Summary is required"
        exit 1
    fi
fi

echo ""
echo -e "${CYAN}============================================${NC}"
echo -e "${CYAN}Triggering Documenter${NC}"
echo -e "${CYAN}============================================${NC}"
log_info "Agent:  $AGENT_NAME"
log_info "Module: $MODULE_NAME"
log_info "Event:  $EVENT_TYPE"
log_info "Summary: $SUMMARY"
echo ""

# Create trigger file
TRIGGER_FILE=$(create_trigger_file "$AGENT_NAME" "$MODULE_NAME" "$EVENT_TYPE" "$SUMMARY")
log_success "Created trigger: $(basename "$TRIGGER_FILE")"

# Log the event
log_trigger_event "$AGENT_NAME" "$MODULE_NAME" "$EVENT_TYPE" "$SUMMARY"
log_success "Logged to triggers.log"

# Scan recent changes
scan_recent_changes "$MODULE_NAME"

# Update Recent Updates section in index.md
update_recent_updates "$AGENT_NAME" "$MODULE_NAME" "$SUMMARY"

# Print documenter prompt
print_documenter_prompt "$AGENT_NAME" "$MODULE_NAME" "$EVENT_TYPE" "$SUMMARY"

exit 0
