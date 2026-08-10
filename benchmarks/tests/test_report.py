from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path
from typing import Any, Dict, List, Optional

from benchmarks.report import (
    baseline_compatibility_reasons,
    executable_identity_differences,
    render_report,
    scenario_statistics,
)
from benchmarks.schema import result_document_hash, validate_result_document


FIXTURES = Path(__file__).resolve().parent / "fixtures"


def _identity_hash(components: List[Dict[str, str]]) -> str:
    encoded = json.dumps(
        {"components": components},
        ensure_ascii=True,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def _mapping_hash(value: Dict[str, Any]) -> str:
    encoded = json.dumps(
        value, ensure_ascii=True, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def _synthetic_result(
    walls_ms: List[int], *, invalid_indexes: Optional[List[int]] = None
) -> Dict[str, Any]:
    invalid = set(invalid_indexes or [])
    scenario = json.loads(
        (FIXTURES / "schema" / "scenario-valid.json").read_text(encoding="utf-8")
    )["scenarios"][0]
    scenario["method"].update({"trials": len(walls_ms), "warmups": 0})
    components = [{"name": "fdu", "sha256": "1" * 64}]
    harness_base = {
        "components": [
            {"name": "corpus.py", "sha256": "a" * 64},
            {"name": "runner.py", "sha256": "b" * 64},
            {"name": "schema.py", "sha256": "c" * 64},
        ],
        "python_implementation": "cpython",
        "python_version": "3.13.5",
    }
    invocation_order = []
    trials = []
    for index, wall_ms in enumerate(walls_ms):
        valid = index not in invalid
        invocation = {
            "ordinal": index,
            "sample_index": index,
            "sample_kind": "timed",
            "scenario_id": scenario["id"],
        }
        invocation_order.append(invocation)
        trials.append(
            {
                "argv": scenario["command"]["argv"],
                "corpus": {
                    "counts": {
                        "directories": 11,
                        "files": 90,
                        "hardlink_groups": 0,
                        "other": 0,
                        "symlinks": 0,
                        "total": 101,
                    },
                    "manifest_hash": "2" * 64,
                    "recipe_hash": "3" * 64,
                    "recipe_id": "contract",
                    "semantic_digest": "4" * 64,
                    "state": {"id": "baseline", "transition_index": 0},
                },
                "environment": {
                    "explicitly_unset": ["FDU_SENTINEL"],
                    "set": {
                        "FDU_PERF_MODE": "fixture",
                        "LANG": "C",
                        "LC_ALL": "C",
                        "TZ": "UTC",
                    },
                },
                "filesystem_cache_state": "uncontrolled",
                "ordinal": index,
                "output": {
                    "artifact": None,
                    "stderr_bytes": 0,
                    "stderr_sha256": "5" * 64,
                    "stdout_bytes": 100,
                    "stdout_sha256": "6" * 64,
                },
                "process": {
                    "exit_code": 0,
                    "signal": None,
                    "spawn_error": None,
                    "timed_out": False,
                },
                "resources": {
                    "major_faults": None,
                    "peak_rss_bytes": None,
                    "reason": "synthetic fixture",
                    "system_cpu_ns": None,
                    "user_cpu_ns": None,
                },
                "sample_index": index,
                "sample_kind": "timed",
                "scenario_id": scenario["id"],
                "snapshot_state": "absent",
                "timing": {
                    "first_output_ns": wall_ms * 500_000,
                    "wall_ns": wall_ms * 1_000_000,
                },
                "validation": {
                    "postcondition": True,
                    "precondition": True,
                    "reasons": [] if valid else ["synthetic invalid sample"],
                    "valid": valid,
                },
            }
        )
    result: Dict[str, Any] = {
        "executables": {
            "fixture": {
                "components": components,
                "identity_hash": _identity_hash(components),
            }
        },
        "host_class": {
            "architecture": "aarch64",
            "cpu_logical": 8,
            "filesystem": "apfs",
            "filesystem_reason": None,
            "host_class": "fixture-host-v1",
            "kernel": "fixture-kernel",
            "memory_bytes": 16_000_000_000,
            "memory_reason": None,
            "operating_system": "FixtureOS",
        },
        "harness": {
            **harness_base,
            "identity_hash": _mapping_hash(harness_base),
        },
        "identity": {
            "finished_at_utc": "2026-08-09T12:00:01.000000Z",
            "run_id": "7" * 32,
            "started_at_utc": "2026-08-09T12:00:00.000000Z",
        },
        "invocation_order": invocation_order,
        "method": {
            "order_algorithm": "sha256-round-robin-v1",
            "order_seed": "synthetic-order",
        },
        "scenario_definitions": [scenario],
        "schema": "fdu-performance-result-v1",
        "source": {"dirty": False, "reason": None, "revision": "8" * 40},
        "trials": trials,
    }
    result["result_hash"] = result_document_hash(result)
    validate_result_document(result)
    return result


class ReportStatisticsTests(unittest.TestCase):
    def test_invalid_samples_are_excluded_and_small_sets_omit_p95(self) -> None:
        result = _synthetic_result([10, 20, 30, 1000], invalid_indexes=[3])

        statistics = scenario_statistics(result)[0]

        self.assertEqual(statistics["valid_trials"], 3)
        self.assertEqual(statistics["invalid_trials"], 1)
        self.assertEqual(statistics["median_wall_ns"], 20_000_000)
        self.assertEqual(statistics["mad_wall_ns"], 10_000_000)
        self.assertIsNone(statistics["p95_wall_ns"])
        self.assertEqual(statistics["p95_reason"], "requires at least 20 valid trials")
        self.assertAlmostEqual(statistics["entries_per_second"], 5_050.0)

    def test_p95_uses_nearest_rank_only_when_twenty_samples_exist(self) -> None:
        result = _synthetic_result(list(range(1, 21)))

        statistics = scenario_statistics(result)[0]

        self.assertEqual(statistics["p95_wall_ns"], 19_000_000)
        self.assertIsNone(statistics["p95_reason"])

    def test_baseline_compatibility_names_every_material_mismatch(self) -> None:
        baseline = _synthetic_result([10, 11, 12])
        current = json.loads(json.dumps(baseline))
        self.assertEqual(baseline_compatibility_reasons(current, baseline), [])

        current["host_class"]["host_class"] = "other-host"
        current["scenario_definitions"][0]["snapshot_state"] = "corrupt"
        current["executables"]["fixture"]["components"][0]["name"] = "other"
        current["trials"][0]["corpus"]["semantic_digest"] = "9" * 64
        reasons = baseline_compatibility_reasons(current, baseline)

        self.assertTrue(any("host class" in reason for reason in reasons))
        self.assertTrue(any("snapshot_state" in reason for reason in reasons))
        self.assertTrue(any("command shape" in reason for reason in reasons))
        self.assertTrue(any("corpus" in reason for reason in reasons))

        changed_binary = json.loads(json.dumps(baseline))
        changed_binary["executables"]["fixture"]["identity_hash"] = "9" * 64
        self.assertEqual(baseline_compatibility_reasons(changed_binary, baseline), [])
        self.assertTrue(executable_identity_differences(changed_binary, baseline))

    def test_report_is_deterministic_and_contains_every_raw_trial(self) -> None:
        result = _synthetic_result([10, 20, 30, 1000], invalid_indexes=[3])

        first = render_report(result)
        second = render_report(json.loads(json.dumps(result)))

        self.assertEqual(first, second)
        expected = (
            Path(__file__).resolve().parents[1] / "expected" / "report-v1.md"
        ).read_text(encoding="utf-8")
        self.assertEqual(first, expected)
        self.assertIn("No performance claim", first)
        self.assertIn("fixture/contract/baseline", first)
        self.assertIn("synthetic invalid sample", first)
        self.assertEqual(first.count("| timed | fixture/contract/baseline |"), 4)


if __name__ == "__main__":
    unittest.main()
