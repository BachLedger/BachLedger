#!/bin/bash
# check_kb_health.sh - Verify knowledge base integrity
# Usage: ./check_kb_health.sh [kb_path]

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
KB_PATH="${1:-$SCRIPT_DIR/../../docs/kb}"
KB_PATH=$(cd "$(dirname "$KB_PATH")" 2>/dev/null && pwd)/$(basename "$KB_PATH") || KB_PATH="$1"

ERRORS=0
WARNINGS=0

echo "=========================================="
echo "Knowledge Base Health Check"
echo "Path: $KB_PATH"
echo "=========================================="
echo ""

# Check if KB exists
if [ ! -d "$KB_PATH" ]; then
    echo "[ERROR] Knowledge base directory does not exist: $KB_PATH"
    exit 1
fi

# Check required files
echo "Checking required files..."
REQUIRED_FILES=(
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
    if [ -f "$KB_PATH/$file" ]; then
        echo "  [OK] $file"
    else
        echo "  [ERROR] Missing: $file"
        ((ERRORS++))
    fi
done

# Check required directories
echo ""
echo "Checking required directories..."
REQUIRED_DIRS=(
    "agents"
    "modules"
    "decisions"
    "issues/open"
    "issues/resolved"
    "summaries/daily"
    "summaries/weekly"
)

for dir in "${REQUIRED_DIRS[@]}"; do
    if [ -d "$KB_PATH/$dir" ]; then
        echo "  [OK] $dir/"
    else
        echo "  [ERROR] Missing directory: $dir/"
        ((ERRORS++))
    fi
done

# Check for broken internal links in markdown files
echo ""
echo "Checking internal links..."
BROKEN_LINKS=0

while IFS= read -r -d '' mdfile; do
    # Extract markdown links [text](path)
    while IFS= read -r link; do
        # Skip external links and anchors
        if [[ "$link" =~ ^https?:// ]] || [[ "$link" =~ ^# ]] || [[ -z "$link" ]]; then
            continue
        fi

        # Remove anchor from link
        link_path="${link%%#*}"

        if [ -n "$link_path" ]; then
            # Resolve relative path
            file_dir=$(dirname "$mdfile")
            resolved_path="$file_dir/$link_path"

            if [ ! -e "$resolved_path" ]; then
                echo "  [WARNING] Broken link in $(basename "$mdfile"): $link"
                ((WARNINGS++))
                ((BROKEN_LINKS++))
            fi
        fi
    done < <(grep -oE '\]\([^)]+\)' "$mdfile" 2>/dev/null | sed 's/\](//' | sed 's/)$//' || true)
done < <(find "$KB_PATH" -name "*.md" -print0 2>/dev/null)

if [ $BROKEN_LINKS -eq 0 ]; then
    echo "  [OK] No broken internal links found"
fi

# Check for orphan files (files not linked from index.md)
echo ""
echo "Checking for orphan files..."
ORPHANS=0

if [ -f "$KB_PATH/index.md" ]; then
    while IFS= read -r -d '' mdfile; do
        relpath="${mdfile#$KB_PATH/}"
        # Skip index.md itself
        if [ "$relpath" = "index.md" ]; then
            continue
        fi

        # Check if file is referenced in index.md
        if ! grep -q "$relpath" "$KB_PATH/index.md" 2>/dev/null; then
            # Check if parent directory is referenced
            parent_dir=$(dirname "$relpath")
            if ! grep -q "$parent_dir" "$KB_PATH/index.md" 2>/dev/null; then
                echo "  [WARNING] Potentially orphan file: $relpath"
                ((WARNINGS++))
                ((ORPHANS++))
            fi
        fi
    done < <(find "$KB_PATH" -name "*.md" -not -path "$KB_PATH/index.md" -print0 2>/dev/null)
fi

if [ $ORPHANS -eq 0 ]; then
    echo "  [OK] No orphan files detected"
fi

# Check agent files have required sections
echo ""
echo "Checking agent file structure..."
AGENT_FILES=$(find "$KB_PATH/agents" -name "*.md" 2>/dev/null)

for agent_file in $AGENT_FILES; do
    agent_name=$(basename "$agent_file" .md)
    missing_sections=""

    # Check for key sections
    if ! grep -q "## Role" "$agent_file" 2>/dev/null && ! grep -q "# .*Agent" "$agent_file" 2>/dev/null; then
        missing_sections="$missing_sections Role/Title"
    fi
    if ! grep -q "## Responsibilities\|## What I Do" "$agent_file" 2>/dev/null; then
        missing_sections="$missing_sections Responsibilities"
    fi

    if [ -n "$missing_sections" ]; then
        echo "  [WARNING] $agent_name.md may be missing sections:$missing_sections"
        ((WARNINGS++))
    else
        echo "  [OK] $agent_name.md has expected structure"
    fi
done

# Check trigger/notification directories
echo ""
echo "Checking ICDD workflow directories..."
ICDD_DIR="$SCRIPT_DIR/../../.icdd"

if [ -d "$ICDD_DIR/triggers" ]; then
    pending=$(find "$ICDD_DIR/triggers" -name "*.json" -newer "$ICDD_DIR/triggers" 2>/dev/null | wc -l | tr -d ' ')
    echo "  [INFO] Pending triggers: $pending"
fi

if [ -d "$ICDD_DIR/notifications" ]; then
    unread=$(grep -l '"status": "unread"' "$ICDD_DIR/notifications"/*.json 2>/dev/null | wc -l | tr -d ' ')
    echo "  [INFO] Unread notifications: $unread"
fi

# Summary
echo ""
echo "=========================================="
echo "Health Check Summary"
echo "=========================================="
echo "Errors:   $ERRORS"
echo "Warnings: $WARNINGS"

if [ $ERRORS -gt 0 ]; then
    echo ""
    echo "Status: UNHEALTHY - Fix errors before proceeding"
    exit 1
elif [ $WARNINGS -gt 0 ]; then
    echo ""
    echo "Status: DEGRADED - Review warnings"
    exit 0
else
    echo ""
    echo "Status: HEALTHY"
    exit 0
fi
