"""Tests for the stability pass's QA helpers: the pty probe's reading of frames, and the
two deliberately broken fdu wrappers the correctness passes must fail against."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]
BREAKS = ROOT / "tests" / "correctness"
ERASE = b"\r\x1b[2K"

# A stand-in for the real fdu: it prints its arguments as a JSON report and exits with
# the status FAKE_STATUS names.
FAKE_FDU = """#!/bin/sh
printf '{"argv": "%s", "provenance": {"source": "cold_scan", "freshness": "fresh"}}' "$*"
exit "${FAKE_STATUS:-0}"
"""


def frame(text: str, color: bool = False) -> bytes:
    body = text.encode()
    return ERASE + (b"\x1b[36m" + body + b"\x1b[0m" if color else body)


@unittest.skipIf(sys.platform == "win32", "the pty probe is Unix only")
class FrameReadingTests(unittest.TestCase):
    """What the probe counts as a drawn frame, and as one erased before the report."""

    def setUp(self) -> None:
        from scripts.qa import pty_probe

        self.probe = pty_probe

    def test_frames_are_read_in_order_without_color(self) -> None:
        out = frame("⠋ Scanning 10 files", color=True) + frame("⠙ Scanning 20 files") + ERASE
        self.assertEqual(
            self.probe.frames(out + b"report\n"), ["⠋ Scanning 10 files", "⠙ Scanning 20 files"]
        )

    def test_a_resize_marker_ends_the_frame_it_lands_in(self) -> None:
        out = frame("⠋ Scanning 10 files") + self.probe.RESIZED + b" tail" + frame("⠙ Scanning")
        self.assertEqual(self.probe.frames(out), ["⠋ Scanning 10 files", "⠙ Scanning"])

    def test_the_last_frame_must_be_erased_before_the_report(self) -> None:
        erased = frame("⠋ Scanning") + ERASE + b"report\n"
        self.assertTrue(self.probe.after_last_frame_is_erased(erased))
        left_behind = frame("⠋ Scanning") + b"\nreport\n"
        self.assertFalse(self.probe.after_last_frame_is_erased(left_behind))
        self.assertFalse(self.probe.after_last_frame_is_erased(b"report\n"))

    def test_below_twenty_columns_only_the_spinner_and_phase_remain(self) -> None:
        self.assertTrue(self.probe.minimal("⠋ Scanning"))
        self.assertFalse(self.probe.minimal("⠋ Scanning 10 files"))
        self.assertFalse(self.probe.minimal("Scanning"))

    def test_width_counts_terminal_columns(self) -> None:
        self.assertEqual(self.probe.width("abc"), 3)
        self.assertEqual(self.probe.width("日本"), 4)
        self.assertEqual(self.probe.width("é"), 1)

    def test_missing_trees_are_a_usage_error_not_a_failed_check(self) -> None:
        environment = {k: v for k, v in os.environ.items() if not k.startswith(("FDU_QA_", "FDU"))}
        result = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "qa" / "pty_probe.py")],
            capture_output=True,
            text=True,
            env=environment,
            check=False,
        )
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn("--tree, --small, --analyze-tree not given", result.stderr)

    def test_the_slow_trees_fall_back_on_the_medium_tree_as_the_playbook_says(self) -> None:
        environment = {"FDU_QA_SMALL": "/s", "FDU_QA_MEDIUM": "/m"}
        with mock.patch.dict(os.environ, environment, clear=True):
            args = self.probe.parser().parse_args([])
            self.assertEqual((args.tree, args.small, args.analyze_tree), ("/m", "/s", "/m"))
            os.environ["FDU_QA_MEDIUM_ANALYZE"] = "/m/docs"
            self.assertEqual(self.probe.parser().parse_args([]).analyze_tree, "/m/docs")
            os.environ.update(FDU_QA_PROGRESS_TREE="/slow", FDU_QA_PROGRESS_ANALYZE="/deep")
            args = self.probe.parser().parse_args([])
            self.assertEqual((args.tree, args.analyze_tree), ("/slow", "/deep"))

    def test_a_child_that_cannot_set_itself_up_exits_and_runs_none_of_the_parents_code(
        self,
    ) -> None:
        # The child inherits this patch across the fork, and only the child calls setsid.
        # Unwinding instead of exiting, it would go on to run this test's own assertions.
        cache = tempfile.mkdtemp(prefix="fdu-pty-test-")
        self.addCleanup(lambda: os.path.isdir(cache) and os.rmdir(cache))
        probe = self.probe.Probe("/bin/sh", cache)
        with mock.patch.object(os, "setsid", side_effect=OSError("no session")):
            run = probe.run(["-c", "exit 0"], probe.env(), timeout=30)
        self.assertEqual(self.probe.exit_code(run.status), 127)
        self.assertTrue(os.path.isdir(cache))


@unittest.skipIf(sys.platform == "win32", "the wrappers stand in for a Unix fdu")
class BreakWrapperTests(unittest.TestCase):
    """Each wrapper breaks exactly what its pass watches, and passes the rest through."""

    def setUp(self) -> None:
        self.scratch = tempfile.TemporaryDirectory()
        self.fake = Path(self.scratch.name) / "fdu"
        self.fake.write_text(FAKE_FDU, encoding="utf-8")
        self.fake.chmod(0o755)

    def tearDown(self) -> None:
        self.scratch.cleanup()

    def wrapped(
        self, wrapper: str, *args: str, status: int = 0
    ) -> subprocess.CompletedProcess[str]:
        environment = dict(os.environ, FDU_REAL=str(self.fake), FAKE_STATUS=str(status))
        return subprocess.run(
            [sys.executable, str(BREAKS / wrapper), *args],
            capture_output=True,
            text=True,
            env=environment,
            check=False,
        )

    def test_the_wrappers_run_directly_as_fdu_bin(self) -> None:
        # warm_cold.py and cross_warm.py execute FDU_BIN itself, not through Python.
        for name in ("break_no_snapshot.py", "break_partial_stored.py"):
            with self.subTest(name=name):
                path = BREAKS / name
                self.assertTrue(os.access(path, os.X_OK))
                self.assertTrue(path.read_text(encoding="utf-8").startswith("#!/usr/bin/env "))

    def test_no_snapshot_turns_every_store_off(self) -> None:
        result = self.wrapped(
            "break_no_snapshot.py", "tree", "--cache", "on", "--cache=on", "--format", "json"
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        argv = json.loads(result.stdout)["argv"]
        self.assertEqual(argv, "tree --cache off --cache=off --format json")

    def test_no_snapshot_leaves_other_requests_alone(self) -> None:
        result = self.wrapped("break_no_snapshot.py", "on", "--cache", "auto", status=2)
        self.assertEqual(result.returncode, 2)
        self.assertEqual(json.loads(result.stdout)["argv"], "on --cache auto")

    def test_partial_stored_answers_stale_ok_with_a_relabeled_cold_scan(self) -> None:
        result = self.wrapped("break_partial_stored.py", "tree", "--stale-ok", status=2)
        self.assertEqual(result.returncode, 2)
        document = json.loads(result.stdout)
        self.assertEqual(document["argv"], "tree --cache off")
        self.assertEqual(document["provenance"], {"source": "cache_only", "freshness": "stale"})

    def test_partial_stored_passes_every_other_request_through(self) -> None:
        result = self.wrapped("break_partial_stored.py", "tree", "--cache", "on", status=1)
        self.assertEqual(result.returncode, 1)
        document = json.loads(result.stdout)
        self.assertEqual(document["provenance"]["source"], "cold_scan")

    def test_without_the_real_binary_they_refuse_to_run(self) -> None:
        for name in ("break_no_snapshot.py", "break_partial_stored.py"):
            with self.subTest(name=name):
                environment = {k: v for k, v in os.environ.items() if k != "FDU_REAL"}
                result = subprocess.run(
                    [sys.executable, str(BREAKS / name)],
                    capture_output=True,
                    text=True,
                    env=environment,
                    check=False,
                )
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("FDU_REAL", result.stderr)


if __name__ == "__main__":
    unittest.main()
