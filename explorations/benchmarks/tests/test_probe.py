from __future__ import annotations

import copy
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any, Dict, List

from benchmarks.realtree import tree
from benchmarks.runner import run_scenario_set
from benchmarks.schema import load_scenario_set


REPOSITORY = Path(__file__).resolve().parents[3]
SCENARIOS = REPOSITORY / "explorations" / "benchmarks" / "scenarios.json"

# Incremental component-allocation budgets for the fixed 128 -> 256 entry fixture below.
# The allocation count budgets deliberately leave less than one event per added entry
# above each measured platform slope: restoring one path clone per entry must fail even
# when fixed process and tree costs do not move. Windows' filesystem backend has a
# higher slope, independently reproduced by the directory-shaped Rust invariant.
# Requested-byte and reallocation limits keep less obvious growth visible too.
_ALLOCATION_SLOPE_BUDGETS = {
    "scan-index": {
        # Linux/macOS fit below 9.5. Windows measured 14.16 here and 14.33 in
        # the directory-shaped invariant, whose Windows ceiling is also 15.0.
        "allocs": 15.0 if os.name == "nt" else 9.5,
        "reallocs": 0.05,
        "bytes_allocated": 1_500.0,
    },
    "opened-discovery": {
        # This flat-tree probe measured 24.16 on Linux. The directory-shaped invariant
        # measured 34.43 on Windows; these ceilings leave less than one allocation per
        # entry of slack on each platform and independently reject a restored clone.
        "allocs": 35.0 if os.name == "nt" else 24.5,
        "reallocs": 0.25,
        "bytes_allocated": 2_500.0,
    },
}

_DETACHED_STREAMING_COUNTERS = (
    "ancestry_overlay_inserts",
    "effect_paths",
    "effect_path_bytes",
    "impact_candidates",
    "impact_ancestor_visits",
    "impact_retained_dirty_paths",
    "impact_all_dirty",
    "journal_retained_commits",
    "journal_cloned_commits",
    "journal_oversized_commits",
    "journal_dropped_commits",
)


def _assert_allocation_slope(
    mode: str,
    smaller: Dict[str, int],
    larger: Dict[str, int],
    added_entries: int,
) -> None:
    """Keep per-entry growth below the recorded ownership budget."""
    breaches = []
    for metric, per_entry_budget in _ALLOCATION_SLOPE_BUDGETS[mode].items():
        growth = larger[metric] - smaller[metric]
        allowed = per_entry_budget * added_entries
        if growth > allowed:
            breaches.append(
                f"{mode} {metric} grew by {growth} for {added_entries} entries; "
                f"budget is {allowed:.2f} ({per_entry_budget:.2f} per entry)"
            )
    if breaches:
        raise AssertionError("; ".join(breaches))


def _assert_detached_streaming_work_is_zero(counters: Dict[str, int]) -> None:
    restored = {
        name: counters[name]
        for name in _DETACHED_STREAMING_COUNTERS
        if counters[name] != 0
    }
    if restored:
        raise AssertionError(f"detached construction performed streaming work: {restored}")


def _probe_path() -> Path:
    target = Path(os.environ.get("CARGO_TARGET_DIR", REPOSITORY / "target"))
    if not target.is_absolute():
        target = REPOSITORY / target
    suffix = ".exe" if os.name == "nt" else ""
    return (target / "debug" / "examples" / f"perf_probe{suffix}").resolve()


class FduProbeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.probe = _probe_path()
        if not cls.probe.is_file():
            raise AssertionError(
                f"performance probe is missing at {cls.probe}; "
                "run `make performance-probe`"
            )

    def test_every_committed_probe_job_passes_end_to_end(self) -> None:
        scenarios = load_scenario_set(SCENARIOS)
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            result = run_scenario_set(
                scenarios,
                executables={"fdu-probe": [str(self.probe)]},
                work_directory=root / "work",
                output_directory=root / "results",
                order_seed="probe-smoke-v1",
            )

        expected_samples = sum(
            scenario["method"]["trials"] + scenario["method"]["warmups"]
            for scenario in scenarios["scenarios"]
        )
        self.assertEqual(len(result["trials"]), expected_samples)
        self.assertTrue(
            all(trial["validation"]["valid"] for trial in result["trials"])
        )
        self.assertTrue(
            all("component_ns" in trial["probe_metrics"] for trial in result["trials"])
        )
        self.assertTrue(
            all(
                trial["probe_metrics"]["component_ns"]
                <= trial["timing"]["wall_ns"]
                for trial in result["trials"]
            )
        )
        if os.name == "posix" and hasattr(os, "wait4"):
            self.assertTrue(
                all(
                    trial["resources"]["collector"] == "posix-wait4-rusage-v1"
                    for trial in result["trials"]
                )
            )

    def test_scan_summaries_match_an_independent_real_tree_oracle(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory) / "tree"
            (root / "nested").mkdir(parents=True)
            older = root / "older.txt"
            newer = root / "nested" / "newer.txt"
            older.write_bytes(b"older")
            newer.write_bytes(b"newer")
            os.utime(older, ns=(10_000_000_000, 10_000_000_000))
            os.utime(newer, ns=(30_000_000_000, 30_000_000_000))
            oracle = tree.fingerprint(root, label="probe-integration")

            for mode in ("scan-index", "scan-producer"):
                completed = subprocess.run(
                    [str(self.probe), mode, "--root", str(root)],
                    check=True,
                    capture_output=True,
                    text=True,
                    timeout=30,
                )
                summary = json.loads(completed.stdout)["summary"]
                disagreement = tree.probe_agrees(oracle, summary)
                if disagreement is not None:
                    self.fail(self._oracle_forensics(root, mode, disagreement, oracle, summary))

    def test_scan_diagnostics_are_versioned_and_run_scoped(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory) / "tree"
            (root / "nested").mkdir(parents=True)
            (root / "nested" / "entry.txt").write_text("evidence", encoding="utf-8")

            completed = subprocess.run(
                [
                    str(self.probe),
                    "scan-index",
                    "--root",
                    str(root),
                    "--threads",
                    "2",
                    "--diagnostics",
                ],
                check=True,
                capture_output=True,
                text=True,
                timeout=30,
            )
            document = json.loads(completed.stdout)

        diagnostics = document["scan_diagnostics"]
        self.assertEqual(diagnostics["schema"], "fdu-scan-diagnostics-v1")
        policy = diagnostics["worker_policy"]
        self.assertEqual(policy["outcome"], "fixed")
        self.assertFalse(policy["events_truncated"])
        self.assertEqual(policy["ready_directories_at_finish"], 0)
        self.assertEqual(policy["in_flight_directories_at_finish"], 0)
        self.assertEqual(policy["handoff_backlog_at_finish"], 0)
        self.assertGreaterEqual(policy["handoff_backlog_high_water"], 1)
        backend = diagnostics["backend"]
        if sys.platform == "darwin":
            self.assertEqual(
                backend["macos_bulk_attempts"],
                backend["macos_bulk_successes"] + backend["macos_bulk_fallbacks"],
            )
            self.assertIsNone(backend["unavailable_reason"])
        else:
            self.assertIsNone(backend["macos_bulk_attempts"])
            self.assertTrue(backend["unavailable_reason"])

    def test_streaming_campaign_modes_report_exact_commit_oracles(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory) / "tree"
            (root / "nested").mkdir(parents=True)
            (root / "a.txt").write_text("a", encoding="utf-8")
            (root / "nested" / "b.txt").write_text("bb", encoding="utf-8")
            filesystem_oracle = tree.fingerprint(root, label="opened-probe")

            def run(*arguments: str, counters: bool = False) -> Dict[str, Any]:
                environment = os.environ.copy()
                if counters:
                    environment["FDU_COUNTERS"] = "1"
                completed = subprocess.run(
                    [str(self.probe), *arguments, "--root", str(root)],
                    check=True,
                    capture_output=True,
                    text=True,
                    timeout=30,
                    env=environment,
                )
                return json.loads(completed.stdout)

            large = run("delta-apply-large", "--operations", "600")
            batched = run(
                "delta-apply-batched",
                "--operations",
                "600",
                "--batch-size",
                "300",
            )
            opened = run("opened-discovery", "--batch-size", "2")
            counted = run(
                "delta-apply-large",
                "--operations",
                "600",
                counters=True,
            )

        self.assertEqual(large["summary"]["engine_digest"], batched["summary"]["engine_digest"])
        self.assertEqual(large["summary"]["apply"]["inserted"], 601)
        self.assertEqual(large["summary"]["commit"]["commits"], 1)
        self.assertEqual(large["summary"]["commit"]["changes"], 601)
        self.assertEqual(large["summary"]["commit"]["all_dirty_commits"], 1)
        self.assertEqual(batched["summary"]["commit"]["commits"], 3)
        self.assertEqual(batched["summary"]["commit"]["changes"], 601)
        self.assertEqual(batched["summary"]["commit"]["all_dirty_commits"], 2)
        self.assertEqual(batched["summary"]["commit"]["dirty_paths"], 3)
        self.assertEqual(len(large["summary"]["commit"]["digest"]), 64)
        self.assertEqual(len(batched["summary"]["commit"]["digest"]), 64)
        self.assertNotEqual(
            large["summary"]["commit"]["digest"],
            batched["summary"]["commit"]["digest"],
        )
        self.assertIsNone(
            tree.synthetic_delta_probe_agrees(
                {},
                large["summary"],
                operations=600,
                batch_size=None,
            )
        )
        self.assertIsNone(
            tree.synthetic_delta_probe_agrees(
                {},
                batched["summary"],
                operations=600,
                batch_size=300,
            )
        )

        disagreement = tree.probe_agrees(filesystem_oracle, opened["summary"])
        self.assertIsNone(disagreement)
        self.assertGreater(opened["summary"]["commit"]["commits"], 0)
        self.assertGreaterEqual(opened["summary"]["commit"]["changes"], 3)
        self.assertGreater(opened["summary"]["commit"]["state_transitions"], 0)
        self.assertEqual(opened["summary"]["commit"]["first_clock"], 1)
        self.assertEqual(len(opened["summary"]["commit"]["digest"]), 64)

        counters = counted["summary"]["counters"]
        self.assertEqual(counters["public_batches"], 1)
        self.assertEqual(counters["public_accepted_ops"], 601)
        self.assertEqual(counters["ancestry_overlay_inserts"], 601)
        self.assertEqual(counters["ancestry_path_comparisons"], 600)
        self.assertEqual(counters["ancestry_parent_proofs"], 601)
        self.assertEqual(counters["effect_paths"], 601)
        self.assertEqual(counters["impact_candidates"], 601)
        self.assertEqual(counters["impact_all_dirty"], 1)
        self.assertEqual(counters["journal_cloned_commits"], 1)
        self.assertEqual(counters["journal_retained_commits"], 1)
        self.assertGreater(counters["allocs"], 0)
        self.assertGreater(counters["bytes_allocated"], 0)

    def test_streaming_allocation_slopes_and_detached_zero_work_are_bounded(self) -> None:
        measurements: Dict[str, Dict[int, Dict[str, int]]] = {
            mode: {} for mode in _ALLOCATION_SLOPE_BUDGETS
        }
        with tempfile.TemporaryDirectory() as temporary_directory:
            base = Path(temporary_directory)
            for entries in (128, 256):
                root = base / f"tree-{entries}"
                root.mkdir()
                for index in range(entries):
                    (root / f"entry-{index:04}.dat").write_bytes(b"x")

                for mode in _ALLOCATION_SLOPE_BUDGETS:
                    environment = {**os.environ, "FDU_COUNTERS": "1"}
                    completed = subprocess.run(
                        [
                            str(self.probe),
                            mode,
                            "--root",
                            str(root),
                            "--threads",
                            "1",
                            "--batch-size",
                            "64",
                        ],
                        check=True,
                        capture_output=True,
                        text=True,
                        timeout=30,
                        env=environment,
                    )
                    counters = json.loads(completed.stdout)["summary"]["counters"]
                    measurements[mode][entries] = counters
                    if mode == "scan-index":
                        _assert_detached_streaming_work_is_zero(counters)

        for mode, by_size in measurements.items():
            with self.subTest(mode=mode):
                _assert_allocation_slope(mode, by_size[128], by_size[256], 128)

    def test_allocation_slope_guard_rejects_one_restored_allocation_per_entry(self) -> None:
        entries = 128
        for mode, budgets in _ALLOCATION_SLOPE_BUDGETS.items():
            smaller = {metric: 10_000 for metric in budgets}
            at_budget = {
                metric: smaller[metric] + int(per_entry * entries)
                for metric, per_entry in budgets.items()
            }
            _assert_allocation_slope(mode, smaller, at_budget, entries)

            restored = dict(at_budget)
            restored["allocs"] += entries
            with self.assertRaisesRegex(AssertionError, rf"{mode} allocs grew"):
                _assert_allocation_slope(mode, smaller, restored, entries)

    def test_detached_zero_work_guard_rejects_one_restored_effect_path(self) -> None:
        counters = {name: 0 for name in _DETACHED_STREAMING_COUNTERS}
        _assert_detached_streaming_work_is_zero(counters)

        counters["effect_paths"] = 1
        with self.assertRaisesRegex(AssertionError, "effect_paths"):
            _assert_detached_streaming_work_is_zero(counters)

    def _oracle_forensics(
        self,
        root: Path,
        mode: str,
        disagreement: str,
        oracle: Dict[str, Any],
        summary: Dict[str, Any],
    ) -> str:
        """Explain a digest mismatch instead of reporting two opaque hashes.

        A multiset digest cannot name the record that differed, but the fixture is
        five entries, so a raw stat dump plus one immediate re-fingerprint separates
        the two failure classes: a tree that changed between the oracle walk and the
        probe run (the re-fingerprint follows the probe), and an engine that reads
        the same tree differently (the re-fingerprint follows the first walk).
        """
        lines = [f"{mode} summary disagreed with the independent tree oracle: {disagreement}"]
        recheck = tree.fingerprint(root, label="probe-integration-recheck")
        probe_digest = summary.get("engine_digest")
        for label, digest in (
            ("oracle", oracle.get("engine_digest")),
            ("probe", probe_digest),
            ("recheck", recheck.get("engine_digest")),
        ):
            lines.append(f"{label} digest: {digest}")
        if recheck.get("engine_digest") == probe_digest:
            lines.append("recheck matches the probe: the tree changed after the oracle walk")
        elif recheck.get("engine_digest") == oracle.get("engine_digest"):
            lines.append("recheck matches the oracle: the engine read this stable tree differently")
        else:
            lines.append("recheck matches neither side: the tree is still changing")
        lines.append(f"oracle components: {oracle.get('engine_digest_components')}")
        lines.append(f"recheck components: {recheck.get('engine_digest_components')}")
        for path in sorted(root.rglob("*")):
            status = os.lstat(path)
            lines.append(
                f"{path.relative_to(root)}: mode={status.st_mode:#o} size={status.st_size} "
                f"mtime_ns={status.st_mtime_ns} ctime_ns={status.st_ctime_ns} "
                f"nlink={status.st_nlink}"
            )
        return "\n".join(lines)

    def test_wrong_probe_evidence_and_snapshot_states_are_rejected(self) -> None:
        committed = load_scenario_set(SCENARIOS)["scenarios"]
        scan = next(scenario for scenario in committed if scenario["job"] == "scan-index")
        snapshot_load = next(
            scenario for scenario in committed if scenario["job"] == "snapshot-load"
        )
        cases: List[Dict[str, Any]] = []

        wrong_digest = copy.deepcopy(scan)
        wrong_digest["id"] = "negative/wrong-digest"
        wrong_digest["validation"]["stdout_json"]["matches_manifest"][
            "summary.engine_digest"
        ] = "semantic_digest"
        cases.append(wrong_digest)

        wrong_count = copy.deepcopy(scan)
        wrong_count["id"] = "negative/wrong-count"
        wrong_count["validation"]["stdout_json"]["matches_manifest"][
            "summary.entries"
        ] = "counts.files"
        cases.append(wrong_count)

        wrong_source = copy.deepcopy(scan)
        wrong_source["id"] = "negative/wrong-source"
        wrong_source["validation"]["stdout_json"]["equals"]["source"] = "snapshot"
        cases.append(wrong_source)

        wrong_snapshot = copy.deepcopy(scan)
        wrong_snapshot["id"] = "negative/wrong-snapshot-postcondition"
        wrong_snapshot["validation"]["snapshot"] = "exists"
        cases.append(wrong_snapshot)

        corrupt_snapshot = copy.deepcopy(snapshot_load)
        corrupt_snapshot["id"] = "negative/corrupt-snapshot"
        corrupt_snapshot["snapshot_state"] = "corrupt"
        corrupt_snapshot["preparation"]["snapshot_argv"] = None
        cases.append(corrupt_snapshot)

        missing_snapshot = copy.deepcopy(snapshot_load)
        missing_snapshot["id"] = "negative/missing-snapshot"
        missing_snapshot["snapshot_state"] = "absent"
        missing_snapshot["preparation"]["snapshot_argv"] = None
        missing_snapshot["validation"]["snapshot"] = "absent"
        cases.append(missing_snapshot)

        for scenario in cases:
            scenario["method"]["order_group"] = "negative-probe-contract"
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            result = run_scenario_set(
                {"scenarios": cases, "schema": "fdu-performance-scenarios-v1"},
                executables={"fdu-probe": [str(self.probe)]},
                work_directory=root / "work",
                output_directory=root / "results",
                order_seed="negative-probe-v1",
            )

        self.assertEqual(len(result["trials"]), len(cases))
        reasons_by_id = {
            trial["scenario_id"]: "\n".join(trial["validation"]["reasons"])
            for trial in result["trials"]
        }
        self.assertIn("manifest mismatch", reasons_by_id["negative/wrong-digest"])
        self.assertIn("manifest mismatch", reasons_by_id["negative/wrong-count"])
        self.assertIn("value mismatch", reasons_by_id["negative/wrong-source"])
        self.assertIn(
            "snapshot postcondition", reasons_by_id["negative/wrong-snapshot-postcondition"]
        )
        self.assertIn("unexpected exit code", reasons_by_id["negative/corrupt-snapshot"])
        self.assertIn("unexpected exit code", reasons_by_id["negative/missing-snapshot"])


if __name__ == "__main__":
    unittest.main()
