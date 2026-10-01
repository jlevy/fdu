"""Tests for how the peer-agreement script judges top-level rows against GNU du -l."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from typing import Any

from scripts import qa_peer_agreement as peers

BLOCK = 4096


def blocks(path: Path) -> tuple[int, int]:
    """A path's own apparent and allocated size, as the tools read it."""
    info = path.lstat()
    return info.st_size, info.st_blocks * 512


def reading(tool: str, metric: str, total: int, children: dict[str, int]) -> dict[str, Any]:
    return {
        "tool": tool,
        "metric": metric,
        "total": total,
        "seconds": 0.0,
        "command": tool,
        "children": children,
        "errors": {},
        "failed": [],
        "gave_up": [],
        "skipped": 0,
        "unmeasured": False,
        "exit": 0,
        "attempts": 1,
    }


def record(
    fdu_children: dict[str, int],
    du_children: dict[str, int],
    top_level: dict[str, dict[str, dict[str, int]]] | None,
) -> dict[str, Any]:
    """An ext4-shaped record: fdu, GNU du -l allocated, fdu, with the tree's facts.

    The root and every top-level directory occupy one block each, and GNU du -l's total
    is fdu's plus those blocks, so only the top-level rows are in question."""
    fdu_total = sum(fdu_children.values())
    directories = 1 + len(fdu_children)
    facts: dict[str, Any] = {
        "hard_links": {},
        "symlinks": {},
        "dirs": {
            "count": directories,
            "apparent": directories * BLOCK,
            "allocated": directories * BLOCK,
            "root_apparent": BLOCK,
            "root_allocated": BLOCK,
        },
        "unlisted": {},
        "unlisted_paths": [],
        "unstatted": {"entries": 0},
        "unstatted_paths": [],
    }
    if top_level is not None:
        facts["top_level"] = top_level
    fdu = [
        reading("fdu", "allocated", fdu_total, fdu_children),
        reading("fdu", "apparent", fdu_total, fdu_children),
    ]
    du = reading("GNU du -l", "allocated", fdu_total + directories * BLOCK, du_children)
    return {"root": "/tree", "facts": facts, "readings": [*fdu, du, *fdu]}


def own_blocks(dirs: int = 1, links: int = 0) -> dict[str, dict[str, int]]:
    """One top-level subtree's facts: its directories and symbolic links, one block each."""
    return {
        "dirs": {"count": dirs, "apparent": dirs * BLOCK, "allocated": dirs * BLOCK},
        "symlinks": {"count": links, "apparent": links * 20, "allocated": links * BLOCK},
    }


class TopLevelFactsTests(unittest.TestCase):
    """The walk measures what GNU du -l adds to each top-level row, per subtree."""

    def test_each_subtree_counts_its_own_directories_and_links(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "a" / "inner").mkdir(parents=True)
            (root / "a" / "file").write_bytes(b"x" * 10_000)
            (root / "a" / "inner" / "link").symlink_to("y" * 300)
            (root / "b").mkdir()
            # A link directly in the root is no top-level row: du lists only directories.
            (root / "root link").symlink_to("a")
            facts = peers.tree_facts(root)
            top = facts["top_level"]
            assert isinstance(top, dict)
            self.assertEqual(sorted(top), ["a", "b"])
            a_apparent, a_allocated = blocks(root / "a")
            inner_apparent, inner_allocated = blocks(root / "a" / "inner")
            link_apparent, link_allocated = blocks(root / "a" / "inner" / "link")
            self.assertEqual(
                top["a"]["dirs"],
                {
                    "count": 2,
                    "apparent": a_apparent + inner_apparent,
                    "allocated": a_allocated + inner_allocated,
                },
            )
            self.assertEqual(
                top["a"]["symlinks"],
                {"count": 1, "apparent": link_apparent, "allocated": link_allocated},
            )
            b_apparent, b_allocated = blocks(root / "b")
            self.assertEqual(
                top["b"]["dirs"], {"count": 1, "apparent": b_apparent, "allocated": b_allocated}
            )
            self.assertEqual(top["b"]["symlinks"], {})

    def test_an_unlistable_subtree_still_counts_its_own_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "locked").mkdir()
            (root / "locked").chmod(0)
            try:
                facts = peers.tree_facts(root)
            finally:
                (root / "locked").chmod(0o755)
            top = facts["top_level"]
            assert isinstance(top, dict)
            self.assertEqual(top["locked"]["dirs"]["allocated"], blocks(root / "locked")[1])


class TopLevelJudgementTests(unittest.TestCase):
    """On ext4 each GNU du -l row adds its subtree's own blocks; nothing else may differ."""

    def test_rows_that_differ_only_by_their_own_blocks_agree(self) -> None:
        fdu = {"a": 40 * BLOCK, "b": 0}
        du = {"a": 43 * BLOCK, "b": BLOCK}
        top = {"a": own_blocks(dirs=2, links=1), "b": own_blocks()}
        text, failures, _ = peers.judge(record(fdu, du, top))
        self.assertEqual(failures, 0, text)
        self.assertIn("agree with GNU du -l exactly", text)
        self.assertIn(": 0 of 2.", text)
        self.assertIn("directory and symbolic-link blocks", text)
        self.assertIn("16.0 KiB", text)

    def test_any_other_difference_still_fails_its_row(self) -> None:
        fdu = {"a": 40 * BLOCK, "b": 0}
        du = {"a": 44 * BLOCK, "b": BLOCK}  # one block more than a's own
        top = {"a": own_blocks(dirs=2, links=1), "b": own_blocks()}
        text, failures, _ = peers.judge(record(fdu, du, top))
        self.assertEqual(failures, 1, text)
        self.assertIn(": 1 of 2.", text)
        row = "| `a` | 160.0 KiB to 160.0 KiB | 176.0 KiB | +16.0 KiB | +12.0 KiB | +4,096 B |"
        self.assertIn(row, text)
        self.assertNotIn("| `b` |", text)

    def test_a_row_short_of_its_own_blocks_fails_too(self) -> None:
        fdu = {"a": 40 * BLOCK}
        du = {"a": 40 * BLOCK}  # du counted the files but not a's directory
        text, failures, _ = peers.judge(record(fdu, du, {"a": own_blocks()}))
        self.assertEqual(failures, 1, text)

    def test_a_directory_on_one_side_only_fails(self) -> None:
        fdu = {"a": 40 * BLOCK}
        du = {"a": 41 * BLOCK, "gone": BLOCK}
        text, failures, _ = peers.judge(record(fdu, du, {"a": own_blocks()}))
        self.assertEqual(failures, 1, text)
        self.assertIn("| `gone` | absent to absent | 4.0 KiB | — | — | — |", text)

    def test_where_directories_occupy_no_blocks_rows_must_match_exactly(self) -> None:
        # APFS: nothing to add, so the rows are compared as they were read.
        fdu = {"a": 40 * BLOCK}
        empty = {"a": {"dirs": {"count": 1, "apparent": 64, "allocated": 0}, "symlinks": {}}}
        text, failures, _ = peers.judge(record(fdu, {"a": 40 * BLOCK}, empty))
        self.assertEqual(failures, 0, text)
        self.assertNotIn("directory and symbolic-link blocks", text)

    def test_a_record_saved_without_the_facts_is_judged_as_before(self) -> None:
        # A record from before the walk measured top-level subtrees has nothing to add.
        fdu = {"a": 40 * BLOCK}
        self.assertEqual(peers.judge(record(fdu, {"a": 40 * BLOCK}, None))[1], 0)
        self.assertEqual(peers.judge(record(fdu, {"a": 41 * BLOCK}, None))[1], 1)


class SelfTestToolsTests(unittest.TestCase):
    """The self-test requires every tool the platform runs, and no other."""

    def test_bsd_du_is_required_only_on_macos(self) -> None:
        self.assertIn(("BSD du", "allocated"), peers.required_readings("darwin"))
        linux = peers.required_readings("linux")
        self.assertNotIn(("BSD du", "allocated"), linux)
        self.assertEqual(set(linux), set(peers.MODELS) - {("BSD du", "allocated")})


if __name__ == "__main__":
    unittest.main()
