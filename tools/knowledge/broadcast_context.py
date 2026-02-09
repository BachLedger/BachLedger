#!/usr/bin/env python3
"""
broadcast_context.py - Broadcast context updates to active agents

Usage:
    ./broadcast_context.py --changed "file1,file2" --summary "..." --for "Coder,Attacker"
    ./broadcast_context.py --json '{"changed_files": [...], "summary": "...", "relevant_for": [...]}'
"""

import argparse
import json
import os
import sys
from datetime import datetime
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent.resolve()
NOTIFICATIONS_DIR = SCRIPT_DIR / "../../.icdd/notifications"
LOG_DIR = SCRIPT_DIR / "../../.icdd/logs"

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


def normalize_agent_name(name: str) -> str:
    """Normalize agent name to lowercase with hyphens."""
    return name.lower().strip().replace("_", "-")


def validate_agents(agents: list[str]) -> list[str]:
    """Validate and normalize agent names."""
    normalized = []
    for agent in agents:
        norm = normalize_agent_name(agent)
        if norm not in VALID_AGENTS:
            print(f"Warning: Unknown agent '{agent}' (normalized: '{norm}')", file=sys.stderr)
        normalized.append(norm)
    return normalized


def create_notification(agent: str, changed_files: list[str], summary: str, broadcast_id: str) -> Path:
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


def log_broadcast(broadcast_id: str, agents: list[str], changed_files: list[str], summary: str):
    """Log the broadcast event."""
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    log_file = LOG_DIR / "broadcasts.log"

    timestamp = datetime.utcnow().isoformat() + "Z"
    agents_str = ",".join(agents)
    files_str = ",".join(changed_files[:5])  # Limit to first 5 files in log
    if len(changed_files) > 5:
        files_str += f"...+{len(changed_files)-5}"

    log_entry = f"{timestamp} | BROADCAST | {broadcast_id} | {agents_str} | {files_str} | {summary}\n"

    with open(log_file, "a") as f:
        f.write(log_entry)


def main():
    parser = argparse.ArgumentParser(
        description="Broadcast context updates to active agents"
    )
    parser.add_argument(
        "--changed",
        type=str,
        help="Comma-separated list of changed files"
    )
    parser.add_argument(
        "--summary",
        type=str,
        help="Summary of the changes"
    )
    parser.add_argument(
        "--for",
        dest="relevant_for",
        type=str,
        help="Comma-separated list of agents to notify"
    )
    parser.add_argument(
        "--json",
        type=str,
        help="JSON input with changed_files, summary, relevant_for"
    )

    args = parser.parse_args()

    # Parse input
    if args.json:
        try:
            data = json.loads(args.json)
            changed_files = data.get("changed_files", [])
            summary = data.get("summary", "")
            relevant_for = data.get("relevant_for", [])
        except json.JSONDecodeError as e:
            print(f"Error parsing JSON: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        if not args.changed or not args.summary or not args.relevant_for:
            print("Error: --changed, --summary, and --for are required (or use --json)", file=sys.stderr)
            sys.exit(1)

        changed_files = [f.strip() for f in args.changed.split(",") if f.strip()]
        summary = args.summary
        relevant_for = [a.strip() for a in args.relevant_for.split(",") if a.strip()]

    if not changed_files:
        print("Error: No changed files specified", file=sys.stderr)
        sys.exit(1)

    if not relevant_for:
        print("Error: No target agents specified", file=sys.stderr)
        sys.exit(1)

    # Validate and normalize agents
    agents = validate_agents(relevant_for)

    # Generate broadcast ID
    broadcast_id = datetime.utcnow().strftime("%Y%m%d%H%M%S") + f"_{os.getpid()}"

    # Create notifications
    created = []
    for agent in agents:
        filepath = create_notification(agent, changed_files, summary, broadcast_id)
        created.append((agent, filepath))
        print(f"Notification created for {agent}: {filepath.name}")

    # Log the broadcast
    log_broadcast(broadcast_id, agents, changed_files, summary)

    print(f"\nBroadcast {broadcast_id} sent to {len(agents)} agent(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
