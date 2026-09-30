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
)

# The synced body is the description, then, when there are notes, a `## Notes` heading
# and the notes. tbd reads it back by splitting at the first such heading (its
# `parseMarkdownWithFrontmatter`, 0.9.0), so notes that themselves open with `## Notes`
# survive the round trip, and this splits exactly as tbd does. tbd has no other body
# section: `design` and `acceptance_criteria` are not fields of its schema.
NOTES_HEADING = re.compile(r"(^|\n)## Notes\n", re.IGNORECASE)


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

    The frontmatter ends at the first line that is exactly `---`, as tbd's reader has it;
    a `---` inside an indented block scalar is content, not the delimiter.
    """
    text = text.replace("\r\n", "\n")
    match = re.match(r"---\n(.*?)^---[ \t]*(?:\n|\Z)(.*)", text, flags=re.S | re.M)
    if match is None:
        raise VerifyError("synced bead has no YAML frontmatter")
    frontmatter, body = match.groups()
    fields = parse_yaml(frontmatter)
    if not isinstance(fields, dict):
        raise VerifyError("synced bead frontmatter is not a mapping")
    return fields, body


# YAML parsing, restricted to the block style tbd writes (the `yaml` package's stringify,
# line width 0): block mappings and sequences, sequences of flat mappings, flow `[]` and
# `{}`, plain, single- and double-quoted scalars, and literal block scalars. Scalars
# resolve by the YAML 1.2 core schema, which is the one tbd's writer uses, so `null` is
# null, `1` is an integer, and `"C:\\"` is one backslash. Anything outside that subset
# raises `VerifyError` rather than being compared as raw text: a verifier that silently
# misreads its input reports drift that is not there, or hides drift that is.
# Standard library only, so verifying a sync needs no dependency.

_KEY = re.compile(r"(?P<key>[A-Za-z_][\w.-]*)[ \t]*:(?:[ \t]+(?P<rest>.*))?$")
_DOUBLE_QUOTED = re.compile(r'"(?P<body>(?:[^"\\]|\\.)*)"')
_SINGLE_QUOTED = re.compile(r"'(?P<body>(?:[^']|'')*)'")
_BLOCK_HEADER = re.compile(
    r"\|(?:(?P<chomp1>[+-])?(?P<indent1>[1-9])?|(?P<indent2>[1-9])(?P<chomp2>[+-]))$"
)
_ESCAPES = {
    "0": "\0",
    "a": "\a",
    "b": "\b",
    "t": "\t",
    "\t": "\t",
    "n": "\n",
    "v": "\v",
    "f": "\f",
    "r": "\r",
    "e": "\x1b",
    " ": " ",
    '"': '"',
    "/": "/",
    "\\": "\\",
    "N": "\x85",
    "_": "\xa0",
    "L": "\u2028",
    "P": "\u2029",
}
_HEX_ESCAPES = {"x": 2, "u": 4, "U": 8}
_NULL = {"", "~", "null", "Null", "NULL"}
_BOOLEANS = {
    "true": True,
    "True": True,
    "TRUE": True,
    "false": False,
    "False": False,
    "FALSE": False,
}
_FLOAT = re.compile(r"[-+]?(?:\.[0-9]+|[0-9]+(?:\.[0-9]*)?)(?:[eE][-+]?[0-9]+)?")
_SPECIAL_FLOATS = {
    **{
        f"{sign}{spelling}": float(f"{sign}inf")
        for sign in ("", "+", "-")
        for spelling in (".inf", ".Inf", ".INF")
    },
    **{spelling: float("nan") for spelling in (".nan", ".NaN", ".NAN")},
}


def parse_yaml(text: str) -> Any:
    """Parse one YAML document in the subset tbd writes."""
    # A final newline ends the last line; it does not begin an empty one.
    lines = text.removesuffix("\n").split("\n")
    position = _skip_blank(lines, 0)
    if position == len(lines):
        return None
    value, position = _parse_node(lines, position, _indent_of(lines[position]))
    position = _skip_blank(lines, position)
    if position != len(lines):
        raise VerifyError(f"unexpected YAML at line {position + 1}: {lines[position].strip()}")
    return value


def _indent_of(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def _skip_blank(lines: list[str], position: int) -> int:
    while position < len(lines) and (
        not lines[position].strip() or lines[position].lstrip().startswith("#")
    ):
        position += 1
    return position


def _parse_node(lines: list[str], position: int, indent: int) -> tuple[Any, int]:
    """Parse the block collection whose first line, at `position`, is at `indent`."""
    text = lines[position][indent:]
    if text == "-" or text.startswith("- "):
        return _parse_sequence(lines, position, indent)
    return _parse_mapping(lines, position, indent)


def _parse_sequence(lines: list[str], position: int, indent: int) -> tuple[list[Any], int]:
    items: list[Any] = []
    while True:
        position = _skip_blank(lines, position)
        if position == len(lines) or _indent_of(lines[position]) < indent:
            return items, position
        line = lines[position]
        text = line[indent:]
        if _indent_of(line) != indent or not (text == "-" or text.startswith("- ")):
            raise VerifyError(f"unexpected YAML at line {position + 1}: {line.strip()}")
        rest = text[1:].strip()
        if rest and _KEY.match(rest):
            # `- key: value` opens a mapping whose later keys align with this first one.
            lines[position] = " " * (indent + 2) + rest
            value, position = _parse_mapping(lines, position, indent + 2)
        else:
            value, position = _parse_value(lines, position, indent, rest)
        items.append(value)


def _parse_mapping(lines: list[str], position: int, indent: int) -> tuple[dict[str, Any], int]:
    mapping: dict[str, Any] = {}
    while True:
        position = _skip_blank(lines, position)
        if position == len(lines) or _indent_of(lines[position]) < indent:
            return mapping, position
        line = lines[position]
        match = _KEY.match(line[indent:])
        if _indent_of(line) != indent or match is None:
            raise VerifyError(f"unexpected YAML at line {position + 1}: {line.strip()}")
        key = match["key"]
        if key in mapping:
            raise VerifyError(f"duplicate YAML key {key!r} at line {position + 1}")
        mapping[key], position = _parse_value(
            lines, position, indent, (match["rest"] or "").strip()
        )


def _parse_value(lines: list[str], position: int, indent: int, rest: str) -> tuple[Any, int]:
    """Parse the value that follows a key or a sequence dash on line `position`."""
    if rest.startswith("|"):
        return _parse_literal(lines, position, indent, rest)
    if rest.startswith(">"):
        raise VerifyError(f"folded block scalar at line {position + 1}; tbd writes literal ones")
    if rest and not rest.startswith("#"):
        return parse_scalar(rest), position + 1
    # Nothing on this line: the value is the nested block below it, or null. A sequence
    # may sit at its key's own indentation.
    following = _skip_blank(lines, position + 1)
    if following < len(lines):
        child = _indent_of(lines[following])
        text = lines[following][child:]
        if child > indent or (child == indent and (text == "-" or text.startswith("- "))):
            return _parse_node(lines, following, child)
    return None, position + 1


def _parse_literal(lines: list[str], position: int, indent: int, header: str) -> tuple[str, int]:
    match = _BLOCK_HEADER.match(header.split(" #", 1)[0].strip())
    if match is None:
        raise VerifyError(f"unreadable block scalar header at line {position + 1}: {header}")
    chomp = match["chomp1"] or match["chomp2"] or ""
    explicit = match["indent1"] or match["indent2"]
    content: list[str] = []
    block_indent = indent + int(explicit) if explicit else None
    position += 1
    while position < len(lines):
        line = lines[position]
        if line.strip():
            if block_indent is None:
                block_indent = _indent_of(line)
            if _indent_of(line) < block_indent or block_indent <= indent:
                break
            content.append(line[block_indent:])
        else:
            content.append(line[block_indent:] if block_indent is not None else "")
        position += 1
    trailing = 0
    while content and not content[-1].strip():
        content.pop()
        trailing += 1
    text = "\n".join(content)
    if chomp == "-":
        return text, position
    if chomp == "+":
        return text + "\n" * (trailing + (1 if text else 0)), position
    return (text + "\n" if text else ""), position


def parse_scalar(text: str) -> Any:
    """Interpret one single-line YAML scalar by the core schema."""
    text = text.strip()
    if text.startswith('"'):
        match = _DOUBLE_QUOTED.match(text)
        _require_only_comment(text, match)
        return _unescape_double(match["body"])
    if text.startswith("'"):
        match = _SINGLE_QUOTED.match(text)
        _require_only_comment(text, match)
        return match["body"].replace("''", "'")
    text = re.split(r"[ \t]+#", text, maxsplit=1)[0].rstrip()
    if text in ("[]", "{}"):
        return [] if text == "[]" else {}
    if text[:1] in "[{&*!|>%@`" and text:
        raise VerifyError(f"YAML construct outside the subset tbd writes: {text}")
    if text in _NULL:
        return None
    if text in _BOOLEANS:
        return _BOOLEANS[text]
    if re.fullmatch(r"[-+]?[0-9]+", text):
        return int(text)
    if re.fullmatch(r"0o[0-7]+", text):
        return int(text[2:], 8)
    if re.fullmatch(r"0x[0-9a-fA-F]+", text):
        return int(text[2:], 16)
    if _FLOAT.fullmatch(text):
        return float(text)
    if text in _SPECIAL_FLOATS:
        return _SPECIAL_FLOATS[text]
    return text


def _require_only_comment(text: str, match: re.Match[str] | None) -> None:
    if match is None:
        raise VerifyError(f"unterminated quoted YAML scalar: {text}")
    after = text[match.end() :]
    if after.strip() and not re.match(r"[ \t]+#", after):
        raise VerifyError(f"unexpected text after a quoted YAML scalar: {text}")


def _unescape_double(body: str) -> str:
    """Decode a double-quoted scalar's escapes, all of YAML 1.2's and no others."""
    decoded: list[str] = []
    position = 0
    while position < len(body):
        character = body[position]
        if character != "\\":
            decoded.append(character)
            position += 1
            continue
        code = body[position + 1]
        if code in _ESCAPES:
            decoded.append(_ESCAPES[code])
            position += 2
        elif code in _HEX_ESCAPES:
            width = _HEX_ESCAPES[code]
            digits = body[position + 2 : position + 2 + width]
            if not re.fullmatch(rf"[0-9a-fA-F]{{{width}}}", digits):
                raise VerifyError(f"malformed \\{code} escape in a YAML scalar: \\{code}{digits}")
            decoded.append(chr(int(digits, 16)))
            position += 2 + width
        else:
            raise VerifyError(f"unknown escape \\{code} in a YAML scalar")
    return "".join(decoded)


def split_body(body: str) -> dict[str, str]:
    """Split a synced body into description and notes, as tbd's own reader does."""
    body = body.strip()
    match = NOTES_HEADING.search(body)
    if match is None:
        return {"description": body, "notes": ""}
    return {"description": body[: match.start()].strip(), "notes": body[match.end() :].strip()}


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
