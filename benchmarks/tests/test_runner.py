from __future__ import annotations

import json
import os
import sys
import tempfile
import time
import unittest
from unittest import mock
from pathlib import Path
from typing import Any, Dict

from benchmarks.runner import RunnerError, run_scenario_set
from benchmarks.schema import (
    SchemaError,
    load_scenario_set,
    result_document_hash,
    validate_result_document,
)


FIXTURES = Path(__file__).resolve().parent / "fixtures"


def _scenario_set(mode: str = "success") -> Dict[str, Any]:
    document = load_scenario_set(FIXTURES / "schema" / "scenario-valid.json")
    scenario = document["scenarios"][0]
    argv = scenario["command"]["argv"]
    argv[argv.index("success")] = mode
    scenario["validation"]["stdout_json"]["equals"]["sentinel"] = None
    return document


class ScenarioRunnerTests(unittest.TestCase):
    def test_successful_run_reestablishes_state_and_records_immutable_trials(self) -> None:
        document = _scenario_set()
        executable = [sys.executable, str(FIXTURES / "fake_benchmark.py")]
        previous = os.environ.get("FDU_SENTINEL")
        os.environ["FDU_SENTINEL"] = "must-not-leak"
        try:
            with tempfile.TemporaryDirectory() as temporary_directory:
                base = Path(temporary_directory)
                result = run_scenario_set(
                    document,
                    executables={"fixture": executable},
                    work_directory=base / "work",
                    output_directory=base / "results",
                    order_seed="stable-order",
                )
                result_paths = list((base / "results").glob("run-*.json"))
                persisted = json.loads(result_paths[0].read_text(encoding="utf-8"))
        finally:
            if previous is None:
                os.environ.pop("FDU_SENTINEL", None)
            else:
                os.environ["FDU_SENTINEL"] = previous

        self.assertEqual(len(result_paths), 1)
        self.assertEqual(len(result["trials"]), 3)
        self.assertEqual(
            [trial["sample_kind"] for trial in result["trials"]],
            ["warmup", "timed", "timed"],
        )
        self.assertTrue(all(trial["validation"]["valid"] for trial in result["trials"]))
        self.assertTrue(all(trial["output"]["stdout_bytes"] > 0 for trial in result["trials"]))
        self.assertTrue(
            all(trial["timing"]["first_output_ns"] is not None for trial in result["trials"])
        )
        self.assertTrue(all(trial["output"]["artifact"] is None for trial in result["trials"]))
        self.assertTrue(
            all("/private/" not in " ".join(trial["argv"]) for trial in result["trials"])
        )
        self.assertEqual(persisted, result)
        validate_result_document(persisted)

        persisted["unexpected"] = True
        persisted["result_hash"] = result_document_hash(persisted)
        with self.assertRaisesRegex(SchemaError, "unknown fields"):
            validate_result_document(persisted)

    def test_result_validation_rejects_internal_evidence_tampering(self) -> None:
        document = _scenario_set()
        document["scenarios"][0]["method"].update(
            {"trials": 1, "warmups": 0}
        )
        with tempfile.TemporaryDirectory() as temporary_directory:
            base = Path(temporary_directory)
            result = run_scenario_set(
                document,
                executables={
                    "fixture": [sys.executable, str(FIXTURES / "fake_benchmark.py")]
                },
                work_directory=base / "work",
                output_directory=base / "results",
                order_seed="tampering",
            )

        wrong_hash = json.loads(json.dumps(result))
        wrong_hash["result_hash"] = "0" * 64
        with self.assertRaisesRegex(SchemaError, "does not match"):
            validate_result_document(wrong_hash)

        inconsistent = json.loads(json.dumps(result))
        inconsistent["trials"][0]["validation"]["valid"] = False
        inconsistent["result_hash"] = result_document_hash(inconsistent)
        with self.assertRaisesRegex(SchemaError, "disagrees"):
            validate_result_document(inconsistent)

        reordered = json.loads(json.dumps(result))
        reordered["invocation_order"][0]["sample_kind"] = "warmup"
        reordered["trials"][0]["sample_kind"] = "warmup"
        reordered["result_hash"] = result_document_hash(reordered)
        with self.assertRaisesRegex(SchemaError, "declared method"):
            validate_result_document(reordered)

        extra_count = json.loads(json.dumps(result))
        extra_count["trials"][0]["corpus"]["counts"]["unexpected"] = 1
        extra_count["result_hash"] = result_document_hash(extra_count)
        with self.assertRaisesRegex(SchemaError, "unknown fields"):
            validate_result_document(extra_count)

    def test_wrong_output_and_nonzero_exit_are_recorded_as_invalid(self) -> None:
        executable = [sys.executable, str(FIXTURES / "fake_benchmark.py")]
        for mode, expected_reason in (
            ("wrong-digest", "manifest mismatch"),
            ("nonzero", "exit code"),
            ("invalid-json", "valid JSON"),
        ):
            with self.subTest(mode=mode):
                document = _scenario_set(mode)
                document["scenarios"][0]["method"].update(
                    {"trials": 1, "warmups": 0}
                )
                with tempfile.TemporaryDirectory() as temporary_directory:
                    base = Path(temporary_directory)
                    result = run_scenario_set(
                        document,
                        executables={"fixture": executable},
                        work_directory=base / "work",
                        output_directory=base / "results",
                        order_seed="invalid-cases",
                    )
                trial = result["trials"][0]
                self.assertFalse(trial["validation"]["valid"])
                self.assertTrue(
                    any(expected_reason in reason for reason in trial["validation"]["reasons"])
                )

    def test_non_json_human_output_uses_byte_and_digest_contracts(self) -> None:
        document = _scenario_set("invalid-json")
        scenario = document["scenarios"][0]
        scenario["job"] = "cli-human"
        scenario["validation"]["stdout_json"] = None
        scenario["method"].update({"trials": 1, "warmups": 0})
        with tempfile.TemporaryDirectory() as temporary_directory:
            base = Path(temporary_directory)
            result = run_scenario_set(
                document,
                executables={
                    "fixture": [sys.executable, str(FIXTURES / "fake_benchmark.py")]
                },
                work_directory=base / "work",
                output_directory=base / "results",
                order_seed="human-output",
            )

        self.assertTrue(result["trials"][0]["validation"]["valid"])

    def test_digest_json_capture_is_bounded_while_the_pipe_is_drained(self) -> None:
        document = _scenario_set("raw-output")
        scenario = document["scenarios"][0]
        scenario["command"]["argv"].extend(["--output-bytes", "128"])
        scenario["method"].update({"trials": 1, "warmups": 0})
        with tempfile.TemporaryDirectory() as temporary_directory:
            base = Path(temporary_directory)
            with mock.patch(
                "benchmarks.runner.MAX_DIGEST_JSON_CAPTURE_BYTES", 16
            ):
                result = run_scenario_set(
                    document,
                    executables={
                        "fixture": [
                            sys.executable,
                            str(FIXTURES / "fake_benchmark.py"),
                        ]
                    },
                    work_directory=base / "work",
                    output_directory=base / "results",
                    order_seed="bounded-output",
                )

        trial = result["trials"][0]
        self.assertEqual(trial["output"]["stdout_bytes"], 128)
        self.assertFalse(trial["validation"]["valid"])
        self.assertTrue(
            any(
                "bounded JSON validation capture" in reason
                for reason in trial["validation"]["reasons"]
            )
        )

    def test_timeout_kills_the_process_group_and_remains_an_invalid_trial(self) -> None:
        if os.name != "posix":
            self.skipTest("POSIX process-group assertion")
        document = _scenario_set("child-sleep")
        document["scenarios"][0]["method"].update(
            {"timeout_seconds": 0.2, "trials": 1, "warmups": 0}
        )
        document["scenarios"][0]["output_sink"] = "output-file"
        with tempfile.TemporaryDirectory() as temporary_directory:
            base = Path(temporary_directory)
            result = run_scenario_set(
                document,
                executables={
                    "fixture": [sys.executable, str(FIXTURES / "fake_benchmark.py")]
                },
                work_directory=base / "work",
                output_directory=base / "results",
                order_seed="timeout",
            )

            trial = result["trials"][0]
            artifact = (
                base
                / "results"
                / f"artifacts-{result['identity']['run_id']}"
                / trial["output"]["artifact"]
            )
            child_pid = int(artifact.read_text(encoding="utf-8"))
            for _attempt in range(100):
                try:
                    os.kill(child_pid, 0)
                except ProcessLookupError:
                    break
                time.sleep(0.01)
            else:
                self.fail("timed-out grandchild process survived group cleanup")

        self.assertTrue(trial["process"]["timed_out"])
        self.assertFalse(trial["validation"]["valid"])
        self.assertTrue(any("timed out" in reason for reason in trial["validation"]["reasons"]))

    def test_postcondition_detects_corpus_mutation_and_next_trial_starts_clean(self) -> None:
        document = _scenario_set("mutate-corpus")
        scenario = document["scenarios"][0]
        scenario["command"]["argv"].extend(["--corpus", "{corpus_root}"])
        scenario["method"].update({"trials": 2, "warmups": 0})
        with tempfile.TemporaryDirectory() as temporary_directory:
            base = Path(temporary_directory)
            result = run_scenario_set(
                document,
                executables={
                    "fixture": [sys.executable, str(FIXTURES / "fake_benchmark.py")]
                },
                work_directory=base / "work",
                output_directory=base / "results",
                order_seed="mutation",
            )

        self.assertEqual(len(result["trials"]), 2)
        self.assertTrue(
            all(trial["validation"]["precondition"] for trial in result["trials"])
        )
        self.assertTrue(
            all(not trial["validation"]["postcondition"] for trial in result["trials"])
        )

    def test_invocation_order_is_stable_for_the_same_seed(self) -> None:
        document = _scenario_set()
        second = json.loads(json.dumps(document["scenarios"][0]))
        second["id"] = "fixture/contract/second"
        document["scenarios"].append(second)
        for scenario in document["scenarios"]:
            scenario["method"].update({"trials": 3, "warmups": 0})
        orders = []
        for _index in range(2):
            with tempfile.TemporaryDirectory() as temporary_directory:
                base = Path(temporary_directory)
                result = run_scenario_set(
                    document,
                    executables={
                        "fixture": [
                            sys.executable,
                            str(FIXTURES / "fake_benchmark.py"),
                        ]
                    },
                    work_directory=base / "work",
                    output_directory=base / "results",
                    order_seed="repeatable",
                )
                orders.append(result["invocation_order"])

        self.assertEqual(orders[0], orders[1])

    def test_snapshot_and_warm_cache_state_are_prepared_for_every_trial(self) -> None:
        executable = [sys.executable, str(FIXTURES / "fake_benchmark.py")]
        for corpus_state, snapshot_state, snapshot_matches in (
            ("baseline", "compatible-unchanged", True),
            ("one-change", "compatible-changed", False),
        ):
            with self.subTest(snapshot_state=snapshot_state):
                document = _scenario_set()
                scenario = document["scenarios"][0]
                if corpus_state != "baseline":
                    scenario["corpus"].update(
                        {"recipe_id": "churn-local", "target_entries": 100}
                    )
                scenario["corpus"]["state"] = corpus_state
                scenario["snapshot_state"] = snapshot_state
                scenario["filesystem_cache_state"] = "verified-warm"
                scenario["method"].update({"trials": 2, "warmups": 0})
                scenario["preparation"]["snapshot_argv"] = [
                    "{executable}",
                    "--mode",
                    "prepare-snapshot",
                    "--manifest",
                    "{manifest_path}",
                    "--snapshot",
                    "{snapshot_path}",
                ]
                scenario["preparation"]["filesystem_cache_argv"] = [
                    "{executable}",
                    "--mode",
                    "prepare-cache",
                    "--manifest",
                    "{manifest_path}",
                    "--cache-marker",
                    "{run_root}/cache-ready",
                ]
                scenario["command"]["argv"].extend(
                    [
                        "--snapshot",
                        "{snapshot_path}",
                        "--require-file",
                        "{snapshot_path}",
                        "--require-file",
                        "{run_root}/cache-ready",
                    ]
                )
                scenario["validation"]["snapshot"] = "exists"
                scenario["validation"]["stdout_json"]["equals"].update(
                    {
                        "required_files": 2,
                        "snapshot_matches_manifest": snapshot_matches,
                    }
                )
                with tempfile.TemporaryDirectory() as temporary_directory:
                    base = Path(temporary_directory)
                    result = run_scenario_set(
                        document,
                        executables={"fixture": executable},
                        work_directory=base / "work",
                        output_directory=base / "results",
                        order_seed="prepared-states",
                    )

                self.assertTrue(
                    all(trial["validation"]["valid"] for trial in result["trials"])
                )

    def test_portable_runner_rejects_an_unsubstantiated_cold_cache_label(self) -> None:
        document = _scenario_set()
        scenario = document["scenarios"][0]
        scenario["filesystem_cache_state"] = "controlled-cold"
        scenario["preparation"]["filesystem_cache_argv"] = [
            "{executable}",
            "--mode",
            "success",
            "--manifest",
            "{manifest_path}",
        ]
        with tempfile.TemporaryDirectory() as temporary_directory:
            base = Path(temporary_directory)
            with self.assertRaisesRegex(RunnerError, "cannot substantiate"):
                run_scenario_set(
                    document,
                    executables={
                        "fixture": [
                            sys.executable,
                            str(FIXTURES / "fake_benchmark.py"),
                        ]
                    },
                    work_directory=base / "work",
                    output_directory=base / "results",
                    order_seed="false-cold-label",
                )


if __name__ == "__main__":
    unittest.main()
