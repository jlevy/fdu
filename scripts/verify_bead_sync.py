#!/usr/bin/env python3
"""Verify that synced beads match their local database state.

`tbd sync` pushes each bead to a git branch as Markdown with YAML frontmatter. This
compares that published form against what the local database holds, field by field, so a
sync can be checked rather than assumed.

    python3 scripts/verify_bead_sync.py                 # every local bead
    python3 scripts/verify_bead_sync.py fdu-tt2j fdu-a1  # only these
    python3 scripts/verify_bead_sync.py --ref tbd-sync   # against a different branch

Exits 0 when every bead matches, 1 on any mismatch, 2 if the comparison could not be
made at all. Read-only: it runs no command that writes to the database or to git.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass, field
from typing import Any

SYNC_REF = "origin/tbd-sync"
SYNC_DIR = ".tbd/data-sync/issues"

# Compared because each is independently editable and so can independently drift. The
# timestamps and `version` are deliberately absent: sync rewrites them by design, so
# they would report a difference on every run and teach the reader to ignore output.
COMPARED_FIELDS = (
    "title",
    "kind",
    "status",
    "priority",
    "spec_path",
    "labels",
    "dependencies",
    "description",
    "notes",
    "design",
    "acceptance_criteria",
)

# The synced body is the description followed by the long-form fields as Markdown
# sections. Splitting them back apart is what lets notes -- the running commentary a
# bead accumulates -- be verified as its own field rather than smuggled into the
# description comparison.
BODY_SECTIONS = {
    "Notes": "notes",
    "Design": "design",
    "Acceptance Criteria": "acceptance_criteria",
}


class VerifyError(Exception):
    """The comparison could not be performed, as distinct from finding a difference."""


@dataclass
class BeadResult:
    """One bead's comparison outcome."""

    display_id: str
    internal_id: str
    mismatches: dict[str, tuple[Any, Any]] = field(default_factory=dict)
    missing_remote: bool = False

    @property
    def ok(self) -> bool:
        return not self.mismatches and not self.missing_remote


def run(command: list[str]) -> str:
    """Run a command and return stdout, raising `VerifyError` with its stderr."""
    try:
        done = subprocess.run(command, capture_output=True, text=True, check=False)
    except FileNotFoundError as error:
        raise VerifyError(f"{command[0]} not found on PATH") from error
    if done.returncode != 0:
        detail = done.stderr.strip() or done.stdout.strip() or f"exit {done.returncode}"
        raise VerifyError(f"{' '.join(command)}: {detail}")
    return done.stdout


def parse_frontmatter(text: str) -> tuple[dict[str, Any], str]:
    """Split a synced bead into its frontmatter mapping and its body.

    Deliberately a small YAML subset -- scalars, scalar lists, and the list-of-mappings
    shape `dependencies` uses -- so that verifying a sync needs no dependency beyond the
    standard library. Anything outside that subset is left as a raw string and compared
    as one.
    """
    if not text.startswith("---\n"):
        raise VerifyError("synced bead has no YAML frontmatter")
    _, frontmatter, body = text.split("---\n", 2)

    fields: dict[str, Any] = {}
    key: str | None = None
    for line in frontmatter.splitlines():
        if not line.strip():
            continue
        if not line.startswith(" ") and ":" in line:
            key, _, value = line.partition(":")
            key = key.strip()
            fields[key] = parse_scalar(value.strip()) if value.strip() else []
        elif key is not None and line.lstrip().startswith("- "):
            item = line.lstrip()[2:].strip()
            existing = fields.get(key)
            if not isinstance(existing, list):
                existing = []
            existing.append(parse_scalar(item))
            fields[key] = existing
        elif key is not None and isinstance(fields.get(key), list) and fields[key]:
            # A continuation line of the current list item's mapping.
            sub_key, _, sub_value = line.strip().partition(":")
            last = fields[key][-1]
            if isinstance(last, dict):
                last[sub_key.strip()] = parse_scalar(sub_value.strip())
    return fields, body


def split_body(body: str) -> dict[str, str]:
    """Split a synced body into the local fields it was assembled from."""
    headings = "|".join(re.escape(name) for name in BODY_SECTIONS)
    parts = re.split(rf"^## ({headings})\s*$", body, flags=re.M)

    fields = {"description": parts[0]}
    for heading, text in zip(parts[1::2], parts[2::2]):
        fields[BODY_SECTIONS[heading]] = text
    return fields


def parse_scalar(value: str) -> Any:
    """Interpret one YAML scalar, or the opening of an inline mapping."""
    if ": " in value and not value.startswith('"'):
        key, _, rest = value.partition(":")
        return {key.strip(): parse_scalar(rest.strip())}
    if value.startswith('"') and value.endswith('"') and len(value) >= 2:
        return value[1:-1]
    if value == "[]":
        return []
    if re.fullmatch(r"-?\d+", value):
        return int(value)
    return value


def normalize(value: Any) -> Any:
    """Reduce a value to the form worth comparing across the two representations.

    Local JSON and synced YAML disagree on incidentals -- absent versus empty, trailing
    whitespace, member ordering within a dependency list -- none of which mean the sync
    lost anything.
    """
    if value is None or value == [] or value == "":
        # An absent field, an empty string, and an empty list all say the same thing:
        # nothing was recorded. Only a difference in content is worth reporting.
        return ""
    if isinstance(value, str):
        return value.strip()
    if isinstance(value, list):
        return sorted((json.dumps(normalize(item), sort_keys=True) for item in value))
    if isinstance(value, dict):
        return {key: normalize(item) for key, item in value.items()}
    return value


def load_local_beads(bead_ids: list[str]) -> list[dict[str, Any]]:
    """Read the named beads, or every bead, from the local database.

    Always via `tbd show`, even when enumerating: `tbd list` returns a summary record
    that omits `dependencies` and `spec_path`, and comparing those absent fields against
    a fully-populated synced file reports drift that does not exist.
    """
    if not bead_ids:
        listed = json.loads(run(["tbd", "list", "--all", "--json"]))
        if isinstance(listed, dict):
            listed = listed.get("issues", [])
        bead_ids = [bead.get("displayId") or bead["id"] for bead in listed]

    beads = []
    for bead_id in bead_ids:
        parsed = json.loads(run(["tbd", "show", bead_id, "--json"]))
        beads.append(parsed[0] if isinstance(parsed, list) else parsed)
    return beads


def compare_bead(bead: dict[str, Any], ref: str) -> BeadResult:
    """Compare one local bead against its synced counterpart."""
    internal_id = bead.get("internalId") or bead["id"]
    display_id = bead.get("displayId") or bead["id"]
    result = BeadResult(display_id=display_id, internal_id=internal_id)

    try:
        remote_text = run(["git", "show", f"{ref}:{SYNC_DIR}/{internal_id}.md"])
    except VerifyError:
        # Distinguished from a field difference: the bead was never published at all,
        # which usually means the sync has not run since it was created.
        result.missing_remote = True
        return result

    remote_fields, remote_body = parse_frontmatter(remote_text)
    remote_fields.update(split_body(remote_body))

    for name in COMPARED_FIELDS:
        local_value = normalize(bead.get(name))
        remote_value = normalize(remote_fields.get(name))
        if local_value != remote_value:
            result.mismatches[name] = (local_value, remote_value)
    return result


def summarize(value: Any, width: int = 60) -> str:
    """Render a value for a single terminal line."""
    text = value if isinstance(value, str) else json.dumps(value)
    text = " ".join(text.split())
    return text if len(text) <= width else f"{text[: width - 1]}…"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Verify synced beads match the local database.",
        epilog="With no bead ids, every bead in the local database is checked.",
    )
    parser.add_argument("bead_ids", nargs="*", metavar="BEAD_ID", help="beads to check")
    parser.add_argument(
        "--ref", default=SYNC_REF, help=f"git ref holding the sync branch (default: {SYNC_REF})"
    )
    parser.add_argument("--quiet", action="store_true", help="report only mismatches")
    args = parser.parse_args()

    try:
        beads = load_local_beads(args.bead_ids)
        results = [compare_bead(bead, args.ref) for bead in beads]
    except VerifyError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    except (KeyError, ValueError) as error:
        print(f"error: could not read bead data: {error}", file=sys.stderr)
        return 2

    for result in results:
        if result.ok:
            if not args.quiet:
                print(f"ok       {result.display_id}")
            continue
        if result.missing_remote:
            print(f"MISSING  {result.display_id} ({result.internal_id}) not on {args.ref}")
            continue
        print(f"DIFFERS  {result.display_id}")
        for name, (local_value, remote_value) in result.mismatches.items():
            print(f"           {name}:")
            print(f"             local  {summarize(local_value)}")
            print(f"             synced {summarize(remote_value)}")

    failed = [result for result in results if not result.ok]
    print(f"\n{len(results) - len(failed)}/{len(results)} beads match {args.ref}")
    if failed:
        print("A mismatch usually means `tbd sync` has not run since the last edit.")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
