"""The bead-sync verifier: reads tbd's synced files as tbd writes and reads them (fdu-f7cf)."""

from __future__ import annotations

import unittest
from typing import Any
from unittest import mock

from scripts import verify_bead_sync
from scripts.verify_bead_sync import (
    VerifyError,
    compare_bead,
    parse_frontmatter,
    parse_scalar,
    parse_yaml,
    split_body,
)

# The shape tbd 0.9.0 writes: the `yaml` package's stringify at line width 0, then the
# description, then `## Notes` and the notes.
SYNCED = """---
type: is
id: is-01m32tww1snf1n9w7dnqkv98s5
title: "PR #98 review R2: fdu C:\\\\ drops pagefile and hiberfil"
kind: bug
status: closed
priority: 1
version: 4
spec_path: null
labels:
  - output
  - external
dependencies:
  - type: blocks
    target: is-01m0rw7bvxtw87tgde30emgs56
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/98
created_at: 2026-09-21T20:35:38.681Z
close_reason: |
  Fixed. A block scalar may hold what looks like frontmatter:
  ---
  title: not a key
  - not an item

  and blank lines.
resolution: null
---
The description, which may mention ## Design without being split there.

## Design

Still the description.

## Notes

## Notes

Notes that open with their own heading, as fdu-a0cf's do.
"""


def local_bead(**overrides: Any) -> dict[str, Any]:
    """The same bead as `tbd show --json` reports it."""
    bead: dict[str, Any] = {
        "id": "is-01m32tww1snf1n9w7dnqkv98s5",
        "displayId": "fdu-zrki",
        "title": "PR #98 review R2: fdu C:\\ drops pagefile and hiberfil",
        "kind": "bug",
        "status": "closed",
        "priority": 1,
        "spec_path": None,
        "labels": ["output", "external"],
        "dependencies": [{"type": "blocks", "target": "is-01m0rw7bvxtw87tgde30emgs56"}],
        "description": (
            "The description, which may mention ## Design without being split there.\n\n"
            "## Design\n\nStill the description."
        ),
        "notes": "## Notes\n\nNotes that open with their own heading, as fdu-a0cf's do.",
    }
    bead.update(overrides)
    return bead


def compare(bead: dict[str, Any], synced: str = SYNCED) -> dict[str, tuple[Any, Any]]:
    with mock.patch.object(verify_bead_sync, "run", return_value=synced):
        result = compare_bead(bead, "origin/tbd-sync")
    assert not result.missing_remote
    return result.mismatches


class FrontmatterTests(unittest.TestCase):
    def test_reads_the_scalars_and_collections_tbd_writes(self) -> None:
        fields, _ = parse_frontmatter(SYNCED)
        self.assertEqual(fields["title"], "PR #98 review R2: fdu C:\\ drops pagefile and hiberfil")
        self.assertIsNone(fields["spec_path"])
        self.assertEqual(fields["priority"], 1)
        self.assertEqual(fields["labels"], ["output", "external"])
        self.assertEqual(
            fields["dependencies"],
            [{"type": "blocks", "target": "is-01m0rw7bvxtw87tgde30emgs56"}],
        )
        self.assertEqual(fields["refs"][0]["url"], "https://github.com/jlevy/fdu/pull/98")
        self.assertEqual(fields["created_at"], "2026-09-21T20:35:38.681Z")
        self.assertIsNone(fields["resolution"])

    def test_a_block_scalar_is_content_to_the_end_of_its_indentation(self) -> None:
        fields, body = parse_frontmatter(SYNCED)
        self.assertEqual(
            fields["close_reason"],
            "Fixed. A block scalar may hold what looks like frontmatter:\n---\n"
            "title: not a key\n- not an item\n\nand blank lines.\n",
        )
        self.assertTrue(body.startswith("The description"))
        self.assertEqual(parse_yaml("a: |-\n  x\n\n"), {"a": "x"})
        self.assertEqual(parse_yaml("a: |+\n  x\n\n"), {"a": "x\n\n"})
        self.assertEqual(parse_yaml("a: |\nb: 1"), {"a": "", "b": 1})

    def test_double_quoted_escapes_decode_as_yaml_defines_them(self) -> None:
        self.assertEqual(parse_scalar('"C:\\\\"'), "C:\\")
        self.assertEqual(parse_scalar('"say \\"hi\\"\\tthen\\nstop"'), 'say "hi"\tthen\nstop')
        self.assertEqual(
            parse_scalar('"\\x41\\u00e9\\U0001F600\\N\\L"'), "A\u00e9\U0001f600\x85\u2028"
        )
        self.assertEqual(parse_scalar("'it''s \\ plain'"), "it's \\ plain")
        self.assertEqual(parse_scalar('"quoted" # comment'), "quoted")

    def test_plain_scalars_resolve_by_the_core_schema(self) -> None:
        for text, value in [
            ("null", None),
            ("~", None),
            ("", None),
            ("true", True),
            ("False", False),
            ("12", 12),
            ("-3", -3),
            ("0x1f", 31),
            ("1.5", 1.5),
            # YAML 1.2, as tbd writes it: these are strings, not 1.1 booleans or times.
            ("on", "on"),
            ("yes", "yes"),
            ("12:30:00", "12:30:00"),
            ("docs/project/a.md", "docs/project/a.md"),
            ("plain # trailing comment", "plain"),
            ("[]", []),
            ("{}", {}),
        ]:
            with self.subTest(text=text):
                self.assertEqual(parse_scalar(text), value)

    def test_constructs_tbd_never_writes_fail_instead_of_being_compared_raw(self) -> None:
        for text in [
            'title: "unterminated',
            'title: "bad \\q escape"',
            'title: "short \\u00e"',
            "labels: [a, b]",
            "title: &anchor x",
            "close_reason: >\n  folded\n",
            "title: a\n  nested: b\n",
            "title: a\ntitle: b\n",
        ]:
            with self.subTest(text=text), self.assertRaises(VerifyError):
                parse_yaml(text)
        with self.assertRaises(VerifyError):
            parse_frontmatter("no frontmatter\n")


class BodyTests(unittest.TestCase):
    def test_splits_at_the_first_notes_heading_as_tbd_does(self) -> None:
        self.assertEqual(
            split_body("\nDesc\n\n## Notes\n\n## Notes\n\nKept.\n"),
            {"description": "Desc", "notes": "## Notes\n\nKept."},
        )
        self.assertEqual(split_body("Desc only\n"), {"description": "Desc only", "notes": ""})
        self.assertEqual(
            split_body("## notes\nLower case is tbd's heading too.\n"),
            {"description": "", "notes": "Lower case is tbd's heading too."},
        )


class CompareTests(unittest.TestCase):
    def test_the_four_reported_false_mismatches_now_match(self) -> None:
        # fdu-zrki (escaped backslash), fdu-wfvx and fdu-cggg (null against empty or
        # absent spec_path), fdu-a0cf (notes opening with `## Notes`).
        self.assertEqual(compare(local_bead()), {})
        self.assertEqual(compare(local_bead(spec_path="")), {})
        bead = local_bead()
        del bead["spec_path"]
        self.assertEqual(compare(bead), {})

    def test_real_differences_are_still_reported(self) -> None:
        self.assertIn("title", compare(local_bead(title="PR #98 review R2: fdu C:\\\\ drops")))
        self.assertIn("spec_path", compare(local_bead(spec_path="docs/plan.md")))
        self.assertIn("labels", compare(local_bead(labels=["output"])))
        # Losing the notes' own heading is a loss, and must not be normalized away.
        mismatches = compare(local_bead(notes="Notes that open with their own heading."))
        self.assertIn("notes", mismatches)
        self.assertIn("description", compare(local_bead(description="Different.")))


if __name__ == "__main__":
    unittest.main()
