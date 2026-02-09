#!/usr/bin/env python3
"""
Auto-resolve lib.rs `pub mod` declaration conflicts.

Collects all `pub mod xxx;` lines from conflict markers,
merges into sorted unique list, preserves other content.
"""

import argparse
import re
import sys
from pathlib import Path


def parse_args():
    parser = argparse.ArgumentParser(
        description="Auto-resolve lib.rs pub mod declaration conflicts"
    )
    parser.add_argument(
        "lib_rs_path",
        nargs="?",
        default="src/lib.rs",
        help="Path to conflicted lib.rs (default: src/lib.rs)",
    )
    return parser.parse_args()


def extract_pub_mods(block: str) -> set[str]:
    """Extract pub mod declarations from a code block."""
    mods = set()

    # Match pub mod xxx; or pub(crate) mod xxx; etc.
    pattern = r'^(pub(?:\([^)]+\))?\s+mod\s+\w+\s*;)'

    for line in block.split('\n'):
        line = line.strip()
        match = re.match(pattern, line)
        if match:
            mods.add(match.group(1))

    return mods


def extract_mod_name(declaration: str) -> str:
    """Extract module name from a pub mod declaration."""
    match = re.search(r'mod\s+(\w+)', declaration)
    return match.group(1) if match else declaration


def resolve_librs_conflict(file_path: str) -> bool:
    """
    Resolve lib.rs pub mod declaration conflicts.

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

    lines = content.split('\n')
    resolved_lines = []
    all_pub_mods = set()
    in_conflict = False
    conflict_section = ""
    skip_until_end = False
    pub_mods_insertion_point = None

    i = 0
    while i < len(lines):
        line = lines[i]

        if line.startswith("<<<<<<< "):
            in_conflict = True
            conflict_section = ""
            i += 1
            continue

        if line.startswith("======="):
            # End of "ours" section, start of "theirs"
            # Extract pub mods from "ours"
            all_pub_mods.update(extract_pub_mods(conflict_section))
            conflict_section = ""
            i += 1
            continue

        if line.startswith(">>>>>>> "):
            # End of conflict
            # Extract pub mods from "theirs"
            all_pub_mods.update(extract_pub_mods(conflict_section))
            in_conflict = False
            conflict_section = ""

            # Mark where to insert merged pub mods
            if pub_mods_insertion_point is None:
                pub_mods_insertion_point = len(resolved_lines)

            i += 1
            continue

        if in_conflict:
            conflict_section += line + "\n"
            i += 1
            continue

        # Outside conflict - keep line unless it's a pub mod we're collecting
        stripped = line.strip()
        if re.match(r'^pub(?:\([^)]+\))?\s+mod\s+\w+\s*;', stripped):
            all_pub_mods.add(stripped)
            if pub_mods_insertion_point is None:
                pub_mods_insertion_point = len(resolved_lines)
        else:
            resolved_lines.append(line)

        i += 1

    # Sort pub mods by module name
    sorted_mods = sorted(all_pub_mods, key=extract_mod_name)

    # Insert merged pub mods at the insertion point
    if sorted_mods and pub_mods_insertion_point is not None:
        # Find a good insertion point (after use statements, before other code)
        insert_at = pub_mods_insertion_point

        # Insert the sorted pub mod declarations
        for j, mod_decl in enumerate(sorted_mods):
            resolved_lines.insert(insert_at + j, mod_decl)
    elif sorted_mods:
        # No insertion point found, add at end of use statements or beginning
        insert_at = 0
        for idx, line in enumerate(resolved_lines):
            if line.strip().startswith("use "):
                insert_at = idx + 1
            elif line.strip() and not line.strip().startswith("//") and not line.strip().startswith("#"):
                break

        for j, mod_decl in enumerate(sorted_mods):
            resolved_lines.insert(insert_at + j, mod_decl)

    # Join and clean up
    resolved = '\n'.join(resolved_lines)

    # Clean up multiple blank lines
    resolved = re.sub(r'\n{3,}', '\n\n', resolved)

    # Ensure file ends with newline
    if not resolved.endswith('\n'):
        resolved += '\n'

    # Write back
    path.write_text(resolved)

    print(f"Resolved {file_path}")
    if sorted_mods:
        print(f"  Merged {len(sorted_mods)} pub mod declarations:")
        for m in sorted_mods:
            print(f"    - {extract_mod_name(m)}")

    return True


def main():
    args = parse_args()

    try:
        success = resolve_librs_conflict(args.lib_rs_path)
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
