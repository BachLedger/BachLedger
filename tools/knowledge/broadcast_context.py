#!/usr/bin/env python3
"""
broadcast_context.py - Broadcast context updates to active agents

Reads recent changes from docs/kb/, generates JSON context updates,
and writes broadcast files for agents to consume.

Usage:
    ./broadcast_context.py --changed "file1,file2" --summary "..." --for "Coder,Attacker"
    ./broadcast_context.py --json '{"changed_files": [...], "summary": "...", "relevant_for": [...]}'
    ./broadcast_context.py --scan  # Auto-detect recent changes
"""

import argparse
import json
import os
import sys
from datetime import datetime, timedelta
from pathlib import Path
from typing import Optional

# Colors for terminal output
class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    CYAN = '\033[0;36m'
    NC = '\033[0m'  # No Color

SCRIPT_DIR = Path(__file__).parent.resolve()
PROJECT_ROOT = SCRIPT_DIR / "../.."
KB_PATH = PROJECT_ROOT / "docs/kb"
NOTIFICATIONS_DIR = PROJECT_ROOT / ".icdd/notifications"
BROADCAST_FILE = PROJECT_ROOT / ".icdd/broadcast.json"
LOG_DIR = PROJECT_ROOT / ".icdd/logs"

VALID_AGENTS = [
    "tester",
    "coder",
    "attacker",
    "reviewer-logic",
    "reviewer-test",
    "reviewer-integration",
    "reviewer-attack",
    "documenter",
    "orchestrator"
]

# Mapping of file patterns to relevant agents
RELEVANCE_MAP = {
    "agents/tester": ["tester"],
    "agents/coder": ["coder"],
    "agents/attacker": ["attacker"],
    "agents/reviewer": ["reviewer-logic", "reviewer-test", "reviewer-integration", "reviewer-attack"],
    "agents/documenter": ["documenter"],
    "modules/": ["coder", "tester", "attacker"],
    "decisions/": ["coder", "reviewer-logic", "orchestrator"],
    "issues/open": ["coder", "tester", "orchestrator"],
    "issues/resolved": ["documenter"],
    "glossary": ["tester", "coder", "documenter"],
}


def log_info(msg: str) -> None:
    print(f"{Colors.BLUE}[INFO]{Colors.NC} {msg}")


def log_success(msg: str) -> None:
    print(f"{Colors.GREEN}[OK]{Colors.NC} {msg}")


def log_warn(msg: str) -> None:
    print(f"{Colors.YELLOW}[WARN]{Colors.NC} {msg}")


def log_error(msg: str) -> None:
    print(f"{Colors.RED}[ERROR]{Colors.NC} {msg}", file=sys.stderr)


def normalize_agent_name(name: str) -> str:
    """Normalize agent name to lowercase with hyphens."""
    return name.lower().strip().replace("_", "-").replace(" ", "-")


def validate_agents(agents: list[str]) -> list[str]:
    """Validate and normalize agent names."""
    normalized = []
    for agent in agents:
        norm = normalize_agent_name(agent)
        if norm not in VALID_AGENTS:
            log_warn(f"Unknown agent '{agent}' (normalized: '{norm}')")
        normalized.append(norm)
    return list(set(normalized))  # Remove duplicates


def infer_relevant_agents(changed_files: list[str]) -> list[str]:
    """Infer which agents should be notified based on changed files."""
    relevant = set()

    for filepath in changed_files:
        filepath_lower = filepath.lower()
        for pattern, agents in RELEVANCE_MAP.items():
            if pattern in filepath_lower:
                relevant.update(agents)

    # If no specific relevance found, notify key agents
    if not relevant:
        relevant = {"coder", "tester", "documenter"}

    return list(relevant)


def scan_recent_changes(minutes: int = 60) -> list[str]:
    """Scan KB directory for recently modified files."""
    if not KB_PATH.exists():
        log_warn(f"KB path does not exist: {KB_PATH}")
        return []

    cutoff = datetime.now() - timedelta(minutes=minutes)
    changed = []

    for md_file in KB_PATH.rglob("*.md"):
        try:
            mtime = datetime.fromtimestamp(md_file.stat().st_mtime)
            if mtime > cutoff:
                rel_path = md_file.relative_to(KB_PATH)
                changed.append(str(rel_path))
        except OSError:
            continue

    return changed


def create_notification(
    agent: str,
    changed_files: list[str],
    summary: str,
    broadcast_id: str
) -> Path:
    """Create a notification file for an agent."""
    NOTIFICATIONS_DIR.mkdir(parents=True, exist_ok=True)

    timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
    filename = f"{agent}_{timestamp}_{broadcast_id[:8]}.json"
    filepath = NOTIFICATIONS_DIR / filename

    notification = {
        "broadcast_id": broadcast_id,
        "timestamp": datetime.utcnow().isoformat() + "Z",
        "target_agent": agent,
        "changed_files": changed_files,
        "summary": summary,
        "status": "unread"
    }

    with open(filepath, "w") as f:
        json.dump(notification, f, indent=2)

    return filepath


def write_broadcast_file(
    changed_files: list[str],
    summary: str,
    relevant_for: list[str],
    broadcast_id: str
) -> Path:
    """Write the main broadcast JSON file for agents to read."""
    BROADCAST_FILE.parent.mkdir(parents=True, exist_ok=True)

    broadcast = {
        "type": "context_updated",
        "broadcast_id": broadcast_id,
        "timestamp": datetime.utcnow().isoformat() + "Z",
        "changed_files": changed_files,
        "summary": summary,
        "relevant_for": relevant_for
    }

    with open(BROADCAST_FILE, "w") as f:
        json.dump(broadcast, f, indent=2)

    return BROADCAST_FILE


def log_broadcast(
    broadcast_id: str,
    agents: list[str],
    changed_files: list[str],
    summary: str
) -> None:
    """Log the broadcast event."""
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    log_file = LOG_DIR / "broadcasts.log"

    timestamp = datetime.utcnow().isoformat() + "Z"
    agents_str = ",".join(agents)
    files_str = ",".join(changed_files[:5])
    if len(changed_files) > 5:
        files_str += f"...+{len(changed_files)-5}"

    log_entry = f"{timestamp} | BROADCAST | {broadcast_id} | {agents_str} | {files_str} | {summary}\n"

    with open(log_file, "a") as f:
        f.write(log_entry)


def print_usage() -> None:
    """Print detailed usage information."""
    print(f"""
{Colors.CYAN}Usage:{Colors.NC} broadcast_context.py [options]

{Colors.CYAN}Description:{Colors.NC}
    Broadcast context updates to active agents.
    Reads recent changes from docs/kb/, generates JSON context updates,
    and writes broadcast files for agents to consume.

{Colors.CYAN}Options:{Colors.NC}
    -h, --help          Show this help message
    --changed FILES     Comma-separated list of changed files
    --summary TEXT      Summary of the changes
    --for AGENTS        Comma-separated list of agents to notify
    --json JSON_STRING  JSON input with changed_files, summary, relevant_for
    --scan              Auto-detect recent changes (last 60 minutes)
    --scan-minutes N    Auto-detect changes from last N minutes

{Colors.CYAN}Output Format:{Colors.NC}
    {{
      "type": "context_updated",
      "broadcast_id": "20240101120000_12345",
      "timestamp": "2024-01-01T12:00:00Z",
      "changed_files": ["docs/kb/modules/evm.md"],
      "summary": "EVM module tests completed",
      "relevant_for": ["Coder", "Attacker"]
    }}

{Colors.CYAN}Examples:{Colors.NC}
    # Explicit files and agents
    ./broadcast_context.py --changed "modules/evm.md,modules/rlp.md" \\
                          --summary "Updated EVM and RLP docs" \\
                          --for "Coder,Attacker"

    # JSON input
    ./broadcast_context.py --json '{{"changed_files": ["modules/evm.md"],
                                    "summary": "EVM updated",
                                    "relevant_for": ["Coder"]}}'

    # Auto-scan for changes
    ./broadcast_context.py --scan --summary "Recent KB updates"

    # Scan with custom time window
    ./broadcast_context.py --scan-minutes 120 --summary "Last 2 hours of changes"
""")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Broadcast context updates to active agents",
        add_help=False
    )
    parser.add_argument("-h", "--help", action="store_true", help="Show help")
    parser.add_argument("--changed", type=str, help="Comma-separated list of changed files")
    parser.add_argument("--summary", type=str, help="Summary of the changes")
    parser.add_argument("--for", dest="relevant_for", type=str, help="Comma-separated list of agents")
    parser.add_argument("--json", type=str, help="JSON input")
    parser.add_argument("--scan", action="store_true", help="Auto-detect recent changes")
    parser.add_argument("--scan-minutes", type=int, default=60, help="Minutes to scan back")

    args = parser.parse_args()

    if args.help:
        print_usage()
        return 0

    changed_files: list[str] = []
    summary: str = ""
    relevant_for: list[str] = []

    # Parse input
    if args.json:
        try:
            data = json.loads(args.json)
            changed_files = data.get("changed_files", [])
            summary = data.get("summary", "")
            relevant_for = data.get("relevant_for", [])
        except json.JSONDecodeError as e:
            log_error(f"Error parsing JSON: {e}")
            return 1
    elif args.scan:
        log_info(f"Scanning for changes in last {args.scan_minutes} minutes...")
        changed_files = scan_recent_changes(args.scan_minutes)
        summary = args.summary or "Knowledge base updates detected"
        if not changed_files:
            log_info("No recent changes found")
            return 0
    else:
        if not args.changed or not args.summary:
            log_error("Either --json, --scan, or both --changed and --summary required")
            print_usage()
            return 1

        changed_files = [f.strip() for f in args.changed.split(",") if f.strip()]
        summary = args.summary

    if args.relevant_for:
        relevant_for = [a.strip() for a in args.relevant_for.split(",") if a.strip()]

    if not changed_files:
        log_error("No changed files specified")
        return 1

    # Infer relevant agents if not specified
    if not relevant_for:
        relevant_for = infer_relevant_agents(changed_files)
        log_info(f"Auto-detected relevant agents: {', '.join(relevant_for)}")

    # Validate and normalize agents
    agents = validate_agents(relevant_for)

    # Generate broadcast ID
    broadcast_id = datetime.utcnow().strftime("%Y%m%d%H%M%S") + f"_{os.getpid()}"

    print()
    print(f"{Colors.CYAN}============================================{Colors.NC}")
    print(f"{Colors.CYAN}Broadcasting Context Update{Colors.NC}")
    print(f"{Colors.CYAN}============================================{Colors.NC}")
    log_info(f"Broadcast ID: {broadcast_id}")
    log_info(f"Changed files: {len(changed_files)}")
    for f in changed_files[:5]:
        print(f"    - {f}")
    if len(changed_files) > 5:
        print(f"    ... and {len(changed_files) - 5} more")
    log_info(f"Summary: {summary}")
    log_info(f"Target agents: {', '.join(agents)}")
    print()

    # Write main broadcast file
    broadcast_path = write_broadcast_file(changed_files, summary, agents, broadcast_id)
    log_success(f"Broadcast file: {broadcast_path.relative_to(PROJECT_ROOT)}")

    # Create individual notifications
    log_info("Creating agent notifications...")
    for agent in agents:
        filepath = create_notification(agent, changed_files, summary, broadcast_id)
        log_success(f"  {agent}: {filepath.name}")

    # Log the broadcast
    log_broadcast(broadcast_id, agents, changed_files, summary)
    log_success("Logged to broadcasts.log")

    print()
    print(f"{Colors.CYAN}============================================{Colors.NC}")
    print(f"{Colors.CYAN}Summary{Colors.NC}")
    print(f"{Colors.CYAN}============================================{Colors.NC}")
    log_success(f"Broadcast {broadcast_id} sent to {len(agents)} agent(s)")
    print()

    # Print the broadcast JSON for reference
    print("Broadcast content:")
    print(json.dumps({
        "type": "context_updated",
        "changed_files": changed_files,
        "summary": summary,
        "relevant_for": agents
    }, indent=2))
    print()

    return 0


if __name__ == "__main__":
    sys.exit(main())
