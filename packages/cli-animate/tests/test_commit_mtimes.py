"""Demo preparation dates Git paths literally and stays inside the selected tree."""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HELPER = Path(__file__).resolve().parents[1] / "examples/fdu/commit-mtimes.py"
OLD = 1577836800
NEW = 1578096000
SENTINEL = 1700000000


class CommitMtimesTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory(prefix="commit-mtimes-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.repo = self.root / "source"
        self.repo.mkdir()
        # Git hook environments must not redirect fixtures to the real repository.
        self.env = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        self.env.update(
            GIT_CONFIG_GLOBAL=os.devnull,
            GIT_CONFIG_NOSYSTEM="1",
            GIT_AUTHOR_NAME="Timestamp fixture",
            GIT_AUTHOR_EMAIL="fixture@example.invalid",
            GIT_COMMITTER_NAME="Timestamp fixture",
            GIT_COMMITTER_EMAIL="fixture@example.invalid",
        )
        self.git("init", "-q", "--initial-branch=main")

    def git(self, *args: str, stamp: int = OLD) -> bytes:
        env = self.env | {
            "GIT_AUTHOR_DATE": f"@{stamp} +0000",
            "GIT_COMMITTER_DATE": f"@{stamp} +0000",
        }
        return subprocess.run(
            ["git", "-c", f"core.hooksPath={os.devnull}", *args],
            cwd=self.repo,
            env=env,
            capture_output=True,
            check=True,
        ).stdout

    def commit(self, stamp: int = OLD) -> None:
        self.git("add", "--all")
        self.git("commit", "-qm", "fixture", stamp=stamp)

    def run_helper(self, tree: Path, git_dir: Path | None = None) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, "-B", "-I", str(HELPER), str(git_dir or self.repo / ".git"), "HEAD", str(tree)],
            cwd=self.root,
            env=self.env,
            capture_output=True,
            text=True,
            timeout=20,
        )

    def test_newline_names_keep_independent_commit_dates(self) -> None:
        (self.repo / "b").write_text("old", encoding="utf-8")
        self.commit()
        names = ["a\nb", "\nleading-newline", "tab\tname", "café", "-leading", "$(touch OWNED)"]
        for name in names:
            (self.repo / name).write_text("new", encoding="utf-8")
        self.commit(NEW)

        result = self.run_helper(self.repo)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(int((self.repo / "b").stat().st_mtime), OLD)
        for name in names:
            self.assertEqual(int((self.repo / name).stat().st_mtime), NEW, repr(name))
        self.assertFalse((self.root / "OWNED").exists())
        self.assertEqual(result.stdout, "")

    def test_marker_like_names_are_paths_not_commit_headers(self) -> None:
        names = ["\x011234567890", "\n\x011234567890", "1234567890"]
        for name in names:
            (self.repo / name).write_text("tracked", encoding="utf-8")
        self.commit()

        result = self.run_helper(self.repo)
        self.assertEqual(result.returncode, 0, result.stderr)
        for name in names:
            self.assertEqual(int((self.repo / name).stat().st_mtime), OLD, repr(name))

    def test_leaf_symlinks_date_the_links_and_leave_targets_untouched(self) -> None:
        outside = self.root / "outside"
        outside.write_text("sentinel", encoding="utf-8")
        (self.repo / "leaf").symlink_to(outside)
        (self.repo / "broken").symlink_to(self.root / "absent")
        (self.repo / "d").mkdir()
        (self.repo / "d/file").write_text("old", encoding="utf-8")
        self.commit()
        (self.repo / "d/new").write_text("new", encoding="utf-8")
        self.commit(NEW)
        os.utime(outside, (SENTINEL, SENTINEL))

        result = self.run_helper(self.repo)
        self.assertEqual(result.returncode, 0, result.stderr)
        for name in ("leaf", "broken", "d/file"):
            self.assertEqual(int((self.repo / name).lstat().st_mtime), OLD)
        self.assertEqual(int((self.repo / "d").stat().st_mtime), NEW)
        self.assertEqual(int(outside.stat().st_mtime), SENTINEL)
        self.assertEqual(outside.read_text(encoding="utf-8"), "sentinel")
        self.assertIn("dated 4 paths and 1 directories", result.stderr)

    def test_directory_symlink_mismatch_fails_without_dating_outside(self) -> None:
        (self.repo / "d").mkdir()
        (self.repo / "d/file").write_text("tracked", encoding="utf-8")
        (self.repo / "safe").write_text("tracked", encoding="utf-8")
        self.commit()
        outside = self.root / "outside"
        outside.mkdir()
        (outside / "file").write_text("sentinel", encoding="utf-8")
        tree = self.root / "different-tree"
        tree.mkdir()
        (tree / "d").symlink_to(outside, target_is_directory=True)
        (tree / "safe").write_text("tracked", encoding="utf-8")
        for path in (outside, outside / "file", tree / "d"):
            os.utime(path, (SENTINEL, SENTINEL), follow_symlinks=False)

        result = self.run_helper(tree)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("symlink", result.stderr)
        self.assertIn("dated 1 paths and 0 directories", result.stderr)
        self.assertIn("skipped 1 paths and 1 directories", result.stderr)
        for path in (outside, outside / "file", tree / "d"):
            self.assertEqual(int(path.lstat().st_mtime), SENTINEL)
        self.assertEqual(int((tree / "safe").stat().st_mtime), OLD)
        self.assertEqual((outside / "file").read_text(encoding="utf-8"), "sentinel")

    def test_missing_or_nondirectory_root_fails_before_git_history(self) -> None:
        (self.repo / "tracked").write_text("tracked", encoding="utf-8")
        self.commit()
        plain_file = self.root / "file"
        plain_file.write_text("sentinel", encoding="utf-8")
        for tree in (self.root / "absent", plain_file):
            result = self.run_helper(tree)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("TREE must be an existing directory", result.stderr)
            self.assertNotIn("dated", result.stderr)
            self.assertNotIn("not a git repository", result.stderr)
        result = self.run_helper(self.root / "absent", git_dir=self.root / "absent.git")
        self.assertIn("TREE must be an existing directory", result.stderr)

    def test_partial_tree_reports_actual_updates_and_returns_failure(self) -> None:
        (self.repo / "present").write_text("tracked", encoding="utf-8")
        (self.repo / "missing").write_text("tracked", encoding="utf-8")
        self.commit()
        tree = self.root / "partial"
        tree.mkdir()
        (tree / "present").write_text("tracked", encoding="utf-8")

        result = self.run_helper(tree)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("dated 1 paths and 0 directories", result.stderr)
        self.assertIn("skipped 1 paths and 0 directories", result.stderr)
        self.assertEqual(int((tree / "present").stat().st_mtime), OLD)

    def test_first_parent_merge_dates_changed_paths_at_the_merge(self) -> None:
        (self.repo / "unchanged").write_text("base", encoding="utf-8")
        self.commit()
        self.git("checkout", "-qb", "side")
        (self.repo / "merged").write_text("side", encoding="utf-8")
        self.commit(OLD + 86400)
        self.git("checkout", "-q", "main")
        self.git("merge", "--no-ff", "-qm", "merge", "side", stamp=NEW)

        result = self.run_helper(self.repo)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(int((self.repo / "unchanged").stat().st_mtime), OLD)
        self.assertEqual(int((self.repo / "merged").stat().st_mtime), NEW)


if __name__ == "__main__":
    unittest.main()
