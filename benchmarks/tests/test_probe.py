from __future__ import annotations

import copy
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from typing import Any, Dict, List

from benchmarks.realtree import tree
from benchmarks.runner import run_scenario_set
from benchmarks.schema import load_scenario_set


REPOSITORY = Path(__file__).resolve().parents[2]
SCENARIOS = REPOSITORY / "benchmarks" / "scenarios.json"


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

        self.assertEqual(len(result["trials"]), 8)
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
        scan = committed[1]
        snapshot_load = committed[3]
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
