#!/bin/bash
# trigger_documenter.sh - Signal documenter agent after work completion
# Usage: ./trigger_documenter.sh <agent_role> <module_name> <summary>

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TRIGGER_DIR="$SCRIPT_DIR/../../.icdd/triggers"
LOG_DIR="$SCRIPT_DIR/../../.icdd/logs"

# Validate arguments
if [ $# -lt 3 ]; then
    echo "Usage: $0 <agent_role> <module_name> <summary>"
    echo "  agent_role: tester|coder|attacker|reviewer-*"
    echo "  module_name: name of the module worked on"
    echo "  summary: brief description of work completed"
    exit 1
fi

AGENT_ROLE="$1"
MODULE_NAME="$2"
SUMMARY="$3"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
TRIGGER_ID="${AGENT_ROLE}_${MODULE_NAME}_${TIMESTAMP}"

# Create directories if needed
mkdir -p "$TRIGGER_DIR"
mkdir -p "$LOG_DIR"

# Create trigger file
TRIGGER_FILE="$TRIGGER_DIR/documenter_${TRIGGER_ID}.json"
cat > "$TRIGGER_FILE" << EOF
{
    "trigger_id": "$TRIGGER_ID",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "source_agent": "$AGENT_ROLE",
    "module": "$MODULE_NAME",
    "summary": "$SUMMARY",
    "status": "pending"
}
EOF

# Log the trigger event
LOG_FILE="$LOG_DIR/triggers.log"
echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) | TRIGGER | documenter | $AGENT_ROLE | $MODULE_NAME | $SUMMARY" >> "$LOG_FILE"

echo "Trigger created: $TRIGGER_FILE"
echo "Documenter will process: $TRIGGER_ID"
