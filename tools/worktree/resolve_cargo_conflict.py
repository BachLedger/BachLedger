#!/usr/bin/env python3
"""
Auto-resolve Cargo.toml workspace members conflicts.

Extracts all `members = [...]` entries from conflicted file,
merges into single sorted unique list, preserves other content.
"""

import argparse
import re
import sys
from pathlib import Path


def parse_args():
    parser = argparse.ArgumentParser(
        description="Auto-resolve Cargo.toml workspace members conflicts"
    )
    parser.add_argument(
        "cargo_toml_path",
        nargs="?",
        default="Cargo.toml",
        help="Path to conflicted Cargo.toml (default: Cargo.toml)",
    )
    return parser.parse_args()


def extract_members_from_block(block: str) -> set[str]:
    """Extract member paths from a members = [...] block."""
    members = set()

    # Match members = [ ... ] with potential multiline
    pattern = r'members\s*=\s*\[(.*?)\]'
    match = re.search(pattern, block, re.DOTALL)

    if match:
        content = match.group(1)
        # Extract quoted strings
        for m in re.finditer(r'"([^"]+)"', content):
            members.add(m.group(1))

    return members


def resolve_cargo_conflict(file_path: str) -> bool:
    """
    Resolve Cargo.toml workspace members conflicts.

    Returns True if successful, False otherwise.
    """
    path = Path(file_path)

    if not path.exists():
        print(f"Error: File not found: {file_path}", file=sys.stderr)
        return False

    content = path.read_text()

    # Check if file has conflict markers
    if "<<<<<<< " not in content:
        print(f"No conflict markers found in {file_path}")
        return True

    # Collect all members from all versions
    all_members = set()

    # Split by conflict markers and extract members from each section
    # Pattern for conflict blocks
    conflict_pattern = r'<<<<<<<[^\n]*\n(.*?)(?:=======\n(.*?))?>>>>>>>[^\n]*\n?'

    for match in re.finditer(conflict_pattern, content, re.DOTALL):
        ours = match.group(1) or ""
        theirs = match.group(2) or ""

        all_members.update(extract_members_from_block(ours))
        all_members.update(extract_members_from_block(theirs))

    # Also extract members from non-conflicted parts
    # Remove conflict markers temporarily
    clean_content = re.sub(conflict_pattern, "", content, flags=re.DOTALL)
    all_members.update(extract_members_from_block(clean_content))

    if not all_members:
        print(f"Warning: No workspace members found in {file_path}", file=sys.stderr)
        # Still try to resolve by taking HEAD version

    # Sort members
    sorted_members = sorted(all_members)

    # Build the new members block
    if sorted_members:
        members_str = ",\n    ".join(f'"{m}"' for m in sorted_members)
        new_members_block = f"members = [\n    {members_str},\n]"
    else:
        new_members_block = "members = []"

    # Remove conflict markers and rebuild content
    # First, remove the conflicted members sections
    resolved = content

    # Handle conflicts around members = [...]
    # This regex matches conflict blocks containing members
    members_conflict_pattern = (
        r'<<<<<<<[^\n]*\n'
        r'(?:.*?members\s*=\s*\[.*?\].*?)?'
        r'=======\n'
        r'(?:.*?members\s*=\s*\[.*?\].*?)?'
        r'>>>>>>>[^\n]*\n?'
    )

    # Replace members-related conflicts
    def replace_members_conflict(match):
        return ""  # Remove the conflict, we'll add merged members later

    resolved = re.sub(members_conflict_pattern, replace_members_conflict, resolved, flags=re.DOTALL)

    # If there are still generic conflicts, try to resolve them
    # by keeping the HEAD (ours) version
    def resolve_generic_conflict(match):
        ours = match.group(1) or ""
        return ours

    resolved = re.sub(
        r'<<<<<<<[^\n]*\n(.*?)=======\n.*?>>>>>>>[^\n]*\n?',
        resolve_generic_conflict,
        resolved,
        flags=re.DOTALL
    )

    # Find existing members block and replace it, or add if missing
    if re.search(r'members\s*=\s*\[', resolved):
        resolved = re.sub(
            r'members\s*=\s*\[.*?\]',
            new_members_block,
            resolved,
            flags=re.DOTALL
        )
    elif "[workspace]" in resolved and sorted_members:
        # Add members after [workspace]
        resolved = re.sub(
            r'(\[workspace\])',
            f'\\1\n{new_members_block}',
            resolved
        )

    # Clean up multiple blank lines
    resolved = re.sub(r'\n{3,}', '\n\n', resolved)

    # Write back
    path.write_text(resolved)

    print(f"Resolved {file_path}")
    if sorted_members:
        print(f"  Merged {len(sorted_members)} workspace members:")
        for m in sorted_members:
            print(f"    - {m}")

    return True


def main():
    args = parse_args()

    try:
        success = resolve_cargo_conflict(args.cargo_toml_path)
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
