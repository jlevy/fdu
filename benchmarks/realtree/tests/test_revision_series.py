"""Tests for deterministic Git revision selection in the replay runner."""

from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path

from benchmarks.realtree import revision_series


class RevisionResolutionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.repository = Path(self.temporary.name)
        self._git("init", "--quiet")
        self._git("config", "user.name", "Revision Test")
        self._git("config", "user.email", "revision@example.invalid")
        self.first = self._commit("crates/fdu/value.txt", "one\n", "first engine")
        self._commit("docs/note.md", "docs\n", "documentation only")
        self.third = self._commit("crates/fdu/value.txt", "three\n", "third engine")

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def _git(self, *arguments: str) -> str:
        completed = subprocess.run(
            ["git", "-C", str(self.repository), *arguments],
            check=True,
            capture_output=True,
            text=True,
        )
        return completed.stdout.strip()

    def _commit(self, relative: str, contents: str, subject: str) -> str:
        path = self.repository / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(contents, encoding="utf-8")
        self._git("add", relative)
        self._git("commit", "--quiet", "-m", subject)
        return self._git("rev-parse", "HEAD")

    def test_a_path_filtered_range_keeps_its_absolute_anchor(self) -> None:
        revisions = revision_series.resolve_revisions(
            self.repository,
            revision_range=f"{self.first}..{self.third}",
            paths=["crates/fdu"],
        )

        self.assertEqual([item.commit for item in revisions], [self.first, self.third])
        self.assertEqual(revisions[0].subject, "first engine")
        self.assertEqual(revisions[1].subject, "third engine")

    def test_explicit_revisions_are_resolved_and_deduplicated_in_order(self) -> None:
        revisions = revision_series.resolve_revisions(
            self.repository,
            requested=[self.first[:10], self.third, self.first],
        )

        self.assertEqual([item.commit for item in revisions], [self.first, self.third])

    def test_revision_sources_are_mutually_exclusive(self) -> None:
        with self.assertRaises(revision_series.RevisionSeriesError):
            revision_series.resolve_revisions(
                self.repository,
                requested=[self.first, self.third],
                revision_range=f"{self.first}..{self.third}",
            )


if __name__ == "__main__":
    unittest.main()
