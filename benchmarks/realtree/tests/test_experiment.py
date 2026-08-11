"""Tests for the experiment artifact contract and the ledger built from it.

The ledger is the part of this work that outlives the session, so the things worth
pinning are the ones that would silently corrupt the record: reading the control and
candidate the wrong way round, writing YAML that parses back as a different type, and
letting an artifact that no longer matches its contract contribute a row anyway.
"""

from __future__ import annotations

import argparse
import io
import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from benchmarks.realtree import experiment as experiment_model
from benchmarks.realtree import record, summary

_REVISION = "1" * 40
_SOURCE_DIGEST = "2" * 64
_RUN_DIGEST = "3" * 64
_SCHEDULE_DIGEST = "4" * 64


def _provenance() -> dict:
    return {
        "engine_revision": _REVISION,
        "harness_revision": _REVISION,
        "harness_sha256": _SOURCE_DIGEST,
        "target": "aarch64-apple-darwin",
        "build_profile": "release",
        "features": [],
        "build_command": "cargo build --release --no-default-features --example perf_probe",
    }


def _metric(median: float) -> dict:
    return {
        "count": 12,
        "min": median,
        "max": median,
        "median": median,
        "mean": median,
        "p90": median,
        "stdev": 0.0,
        "mad": 0.0,
    }


def _run_document() -> dict:
    """A minimal but realistic measurement run, with sorted variant keys."""
    return {
        "schema": "fdu-realtree-run-v2",
        "started_utc": "2026-08-10T12:00:00Z",
        "note": "",
        "host": {
            "cpu_model": "Test CPU",
            "cpu_count": 8,
            "system": "Darwin",
            "release": "25.5.0",
            "filesystem": "apfs",
            "toolchain": "rustc 1.97.1 (test)",
        },
        "conditions": {
            "os_cache": "warm-steady",
            "trials": 12,
            "warmups": 3,
            "interleaved": True,
            "schedule": "round-robin-by-ordinal-v1",
            "schedule_sha256": _SCHEDULE_DIGEST,
            "schedule_seed": None,
        },
        "tree": {
            "label": "fixture",
            "root_id": "a" * 64,
            "counts": {"total": 100, "directories": 10, "files": 88, "symlinks": 2, "other": 0},
            "sizes": {"apparent_bytes": 4096, "allocated_bytes": 8192},
            "max_depth": 4,
            "engine_digest": "5" * 64,
        },
        "tree_mutated_during_run": [],
        # Deliberately alphabetical, which is the order that would invert the
        # comparison if declaration order were not recorded separately.
        "variant_order": ["control", "candidate"],
        "variants": {
            "candidate": {"kind": "fdu-probe", "name": "candidate", "notes": "", "sha256": "b" * 64, "size_bytes": 1, "args": [], **_provenance()},
            "control": {"kind": "fdu-probe", "name": "control", "notes": "", "sha256": "c" * 64, "size_bytes": 1, "args": [], **_provenance()},
        },
        "jobs": {
            "cold-scan-index": {
                "argv": [],
                "description": "d",
                "start_state": "cold",
            }
        },
        "statistics": {
            "cold-scan-index": {
                "variants": {
                    "control": {"samples": 12, "invalid": 0, "metrics": {"wall_ns": _metric(1000.0)}},
                    "candidate": {"samples": 12, "invalid": 0, "metrics": {"wall_ns": _metric(700.0)}},
                },
                "comparisons": {
                    "candidate_vs_control": {
                        "control": "control",
                        "candidate": "candidate",
                        "metrics": {
                            "wall_ns": {
                                "pairs": 12,
                                "median_delta": -300.0,
                                "median_change_pct": -30.0,
                                "ci95_change_pct": [-33.0, -27.0],
                                "improved": True,
                                "significant": True,
                            }
                        },
                    }
                },
            }
        },
        "reference_tools": {
            "dust": {
                "argv": ["{binary}", "-d", "1", "{root}"],
                "identity": {},
                "wall_ns": _metric(900.0),
                "cpu_ns": _metric(1800.0),
            }
        },
    }


class FromRunTests(unittest.TestCase):
    def _payload(self, **overrides):
        arguments = {
            "experiment_id": "exp-042",
            "title": "Test experiment",
            "hypotheses": ["H1"],
            "control": "before",
            "candidate": "after",
            "complexity": {"lines_changed": 10},
            "verdict": {
                "decision": "accepted",
                "primary_job": "cold-scan-index",
                "reason": "faster",
            },
            "run_artifact": "docs/project/experiments/evidence/exp-042-run.json",
            "run_artifact_sha256": _RUN_DIGEST,
            "evidence_grade": "claim-grade",
        }
        arguments.update(overrides)
        return experiment_model.from_run(_run_document(), **arguments)

    def test_declaration_order_decides_the_control(self) -> None:
        payload = self._payload()
        metrics = payload["results"][0]["metrics"]["wall_ns"]
        self.assertEqual(metrics["control_median"], 1000.0)
        self.assertEqual(metrics["candidate_median"], 700.0)
        self.assertEqual(metrics["change_pct"], -30.0)

    def test_alphabetical_fallback_does_not_silently_invert(self) -> None:
        # Without variant_order the mapping order is alphabetical, so `candidate`
        # would come first. The comparison key then does not exist, and the metrics
        # must come out empty rather than backwards.
        document = _run_document()
        del document["variant_order"]
        payload = experiment_model.from_run(
            document,
            experiment_id="exp-042",
            title="t",
            hypotheses=[],
            control="a",
            candidate="b",
            complexity={"lines_changed": 0},
            verdict={"decision": "rejected", "primary_job": "cold-scan-index", "reason": "r"},
        )
        metrics = payload["results"][0]["metrics"]["wall_ns"]
        self.assertEqual(metrics["change_pct"], 0.0)
        self.assertEqual(metrics["pairs"], 0)

    def test_measured_facts_are_read_from_the_run_not_the_caller(self) -> None:
        payload = self._payload()
        self.assertEqual(payload["subject"]["tree_entries"], 100)
        self.assertEqual(payload["subject"]["tree_root_id"], "a" * 64)
        self.assertEqual(payload["method"]["trials"], 12)
        self.assertEqual(payload["date"], "2026-08-10")

    def test_reference_tools_are_carried_through(self) -> None:
        payload = self._payload()
        self.assertEqual(payload["reference_tools"][0]["name"], "dust")
        self.assertEqual(payload["reference_tools"][0]["wall_ns_median"], 900.0)

    def test_no_filesystem_path_from_the_tree_is_recorded(self) -> None:
        payload = self._payload()
        # Nothing in the payload may be an absolute path or a path-looking fragment
        # from the measured tree. The argv template's `{root}` placeholder is a
        # literal, not a path, so it is checked separately below.
        def walk(value):
            if isinstance(value, dict):
                for item in value.values():
                    yield from walk(item)
            elif isinstance(value, list):
                for item in value:
                    yield from walk(item)
            elif isinstance(value, str):
                yield value

        for text in walk(payload):
            self.assertFalse(
                text.startswith("/") or text.startswith("~"),
                f"payload carries a filesystem path: {text!r}",
            )
        self.assertIn("{root}", payload["reference_tools"][0]["argv"])


class EvidenceContractTests(unittest.TestCase):
    def _payload(self, *, decision: str = "rejected") -> dict:
        return experiment_model.from_run(
            _run_document(),
            experiment_id="exp-042",
            title="Evidence contract",
            hypotheses=["H1"],
            control="before",
            candidate="after",
            complexity={"lines_changed": 10},
            verdict={
                "decision": decision,
                "primary_job": "cold-scan-index",
                "reason": "measured",
            },
            run_artifact="docs/project/experiments/evidence/exp-042-run.json",
            run_artifact_sha256=_RUN_DIGEST,
            evidence_grade="claim-grade",
        )

    def test_claim_grade_requires_explicit_feature_provenance(self) -> None:
        payload = self._payload()
        del payload["method"]["control_binary"]["features"]
        with self.assertRaisesRegex(ValueError, "control_binary.features"):
            experiment_model.Experiment.model_validate(payload)

    def test_v2_claim_grade_remains_backward_compatible(self) -> None:
        payload = self._payload()
        self.assertEqual(payload["method"]["run_schema"], "fdu-realtree-run-v2")
        experiment_model.Experiment.model_validate(payload)

    def test_v3_claim_grade_requires_an_environment_identity(self) -> None:
        payload = self._payload()
        payload["method"]["run_schema"] = "fdu-realtree-run-v3"
        with self.assertRaisesRegex(ValueError, "subject.environment_cell"):
            experiment_model.Experiment.model_validate(payload)

    def test_v3_environment_identity_is_promoted_from_the_raw_run(self) -> None:
        document = _run_document()
        document["schema"] = "fdu-realtree-run-v3"
        document["environment"] = {
            "cell_id": "github-ubuntu-24.04-x64",
            "fingerprint_sha256": "6" * 64,
            "runner_class": "github-hosted-shared",
        }
        document["workload"] = {
            "kind": "generated-corpus",
            "portable_identity_sha256": "7" * 64,
        }
        payload = experiment_model.from_run(
            document,
            experiment_id="exp-042",
            title="Cross environment",
            hypotheses=["H1"],
            control="before",
            candidate="after",
            complexity={"lines_changed": 10},
            verdict={
                "decision": "rejected",
                "primary_job": "cold-scan-index",
                "reason": "measured",
            },
            run_artifact="docs/project/experiments/evidence/exp-042-run.json",
            run_artifact_sha256=_RUN_DIGEST,
            evidence_grade="claim-grade",
        )
        validated = experiment_model.Experiment.model_validate(payload)
        self.assertEqual(validated.subject.environment_cell, "github-ubuntu-24.04-x64")
        self.assertEqual(
            validated.subject.portable_workload_identity, "7" * 64
        )

    def test_accepted_requires_latency_and_resource_gates(self) -> None:
        payload = self._payload(decision="accepted")
        with self.assertRaisesRegex(ValueError, "decision gates"):
            experiment_model.Experiment.model_validate(payload)

        payload["verdict"].update(
            {
                "latency_gate_passed": True,
                "resource_guardrails": [
                    {
                        "metric": "cpu_ns",
                        "maximum_regression_pct": 10.0,
                        "status": "passed",
                        "reason": "inside limit",
                    },
                    {
                        "metric": "peak_rss_bytes",
                        "maximum_regression_pct": 10.0,
                        "status": "passed",
                        "reason": "inside limit",
                    },
                ],
            }
        )
        experiment_model.Experiment.model_validate(payload)


class RecordHeadlineTests(unittest.TestCase):
    @staticmethod
    def _arguments(**overrides):
        values = {
            "primary_job": "cold-scan-index",
            "primary_metric": "wall_ns",
            "control_variant": None,
            "candidate_variant": None,
            "decision": "rejected",
            "max_cpu_regression_pct": 10.0,
            "max_rss_regression_pct": 10.0,
            "resource_waiver": None,
        }
        values.update(overrides)
        return argparse.Namespace(**values)

    def test_multi_variant_run_requires_an_explicit_pair(self) -> None:
        document = _run_document()
        comparisons = document["statistics"]["cold-scan-index"]["comparisons"]
        comparisons["third_vs_control"] = comparisons["candidate_vs_control"]
        with self.assertRaisesRegex(ValueError, "requires --control-variant"):
            record._headline(document, self._arguments())

    def test_reversed_explicit_pair_selects_only_that_comparison(self) -> None:
        document = _run_document()
        comparisons = document["statistics"]["cold-scan-index"]["comparisons"]
        reverse = json.loads(json.dumps(comparisons["candidate_vs_control"]))
        reverse["control"] = "candidate"
        reverse["candidate"] = "control"
        reverse["metrics"]["wall_ns"]["median_change_pct"] = 42.0
        comparisons["control_vs_candidate"] = reverse

        headline = record._headline(
            document,
            self._arguments(
                control_variant="candidate", candidate_variant="control"
            ),
        )
        self.assertEqual(headline, 42.0)

    def test_unknown_explicit_pair_fails_instead_of_falling_back(self) -> None:
        with self.assertRaisesRegex(ValueError, "has no comparison"):
            record._headline(
                _run_document(),
                self._arguments(
                    control_variant="candidate", candidate_variant="missing"
                ),
            )

    def test_non_baseline_missing_comparison_reports_validation_error(self) -> None:
        document = _run_document()
        document["statistics"]["cold-scan-index"]["comparisons"] = {}

        with self.assertRaisesRegex(ValueError, "no comparison.*cold-scan-index"):
            record._verdict_evidence(document, self._arguments(), None)

    def test_constant_runwide_rss_is_unmeasured_when_recording(self) -> None:
        document = _run_document()
        comparison = document["statistics"]["cold-scan-index"]["comparisons"][
            "candidate_vs_control"
        ]["metrics"]
        comparison["cpu_ns"] = {
            "median_change_pct": 0.0,
            "ci95_change_pct": [0.0, 0.0],
        }
        comparison["peak_rss_bytes"] = {
            "median_change_pct": 0.0,
            "ci95_change_pct": [0.0, 0.0],
        }
        constant_rss_bytes: int = 1_000
        document["samples"] = [
            {
                "variant": variant,
                "warmup": False,
                "valid": True,
                "metrics": {"peak_rss_bytes": constant_rss_bytes},
            }
            for _ordinal in range(2)
            for variant in ("control", "candidate")
        ]
        arguments = self._arguments(decision="accepted")
        headline = record._headline(document, arguments)

        with self.assertRaisesRegex(ValueError, "resource guardrails"):
            record._verdict_evidence(document, arguments, headline)

        arguments.decision = "rejected"
        evidence = record._verdict_evidence(document, arguments, headline)
        rss = next(
            guardrail
            for guardrail in evidence["resource_guardrails"]
            if guardrail["metric"] == "peak_rss_bytes"
        )
        self.assertEqual(rss["status"], "not-measured")
        self.assertIn("process-launcher floor", rss["reason"])

    def test_accepted_latency_win_requires_resource_guardrails_or_a_waiver(self) -> None:
        document = _run_document()
        statistics = document["statistics"]["cold-scan-index"]
        statistics["variants"]["control"]["metrics"].update(
            {"cpu_ns": _metric(100.0), "peak_rss_bytes": _metric(100.0)}
        )
        statistics["variants"]["candidate"]["metrics"].update(
            {"cpu_ns": _metric(220.0), "peak_rss_bytes": _metric(105.0)}
        )
        comparison = statistics["comparisons"]["candidate_vs_control"]["metrics"]
        comparison["cpu_ns"] = {
            "median_change_pct": 120.0,
            "ci95_change_pct": [110.0, 130.0],
        }
        comparison["peak_rss_bytes"] = {
            "median_change_pct": 5.0,
            "ci95_change_pct": [4.0, 6.0],
        }
        arguments = self._arguments(decision="accepted")
        headline = record._headline(document, arguments)

        with self.assertRaisesRegex(ValueError, "resource guardrails"):
            record._verdict_evidence(document, arguments, headline)

        arguments.resource_waiver = "latency is the product priority"
        evidence = record._verdict_evidence(document, arguments, headline)
        self.assertTrue(evidence["latency_gate_passed"])
        self.assertEqual(
            [guardrail["status"] for guardrail in evidence["resource_guardrails"]],
            ["waived", "passed"],
        )


class YamlScalarTests(unittest.TestCase):
    """A string has to come back as a string, or the schema rejects it downstream."""

    def test_digit_strings_are_quoted(self) -> None:
        self.assertEqual(record._scalar("1"), '"1"')
        self.assertEqual(record._scalar("2026"), '"2026"')
        self.assertEqual(record._scalar("1.5"), '"1.5"')

    def test_iso_dates_are_quoted_for_yaml_portability(self) -> None:
        self.assertEqual(record._scalar("2026-08-11"), '"2026-08-11"')

    def test_missing_validator_fails_closed(self) -> None:
        error = io.StringIO()
        with (
            mock.patch.object(record.subprocess, "run", side_effect=FileNotFoundError("missing")),
            mock.patch.object(record.sys, "stderr", error),
        ):
            self.assertEqual(record._validate(Path("missing.md")), 1)
        self.assertIn("validation failed", error.getvalue())

    def test_boolean_and_null_lookalikes_are_quoted(self) -> None:
        for text in ("true", "False", "yes", "no", "null", "~", "on", "off"):
            self.assertTrue(
                record._scalar(text).startswith('"'), f"{text!r} was left unquoted"
            )

    def test_leading_dash_is_quoted(self) -> None:
        self.assertEqual(record._scalar("-d"), '"-d"')

    def test_actual_numbers_are_not_quoted(self) -> None:
        self.assertEqual(record._scalar(1), "1")
        self.assertEqual(record._scalar(True), "true")
        self.assertEqual(record._scalar(None), "null")

    def test_ordinary_prose_stays_plain(self) -> None:
        self.assertEqual(record._scalar("parallel producer"), "parallel producer")

    def test_text_with_a_colon_is_quoted(self) -> None:
        self.assertTrue(record._scalar("exp-001: faster").startswith('"'))


class RenderRoundTripTests(unittest.TestCase):
    """The artifact must validate against its own compiled contract."""

    def setUp(self) -> None:
        if shutil.which("uvx") is None:
            self.skipTest("uvx is not available")
        self.scratch = Path(tempfile.mkdtemp(prefix="fdu-exp-test-"))
        self.addCleanup(shutil.rmtree, self.scratch, ignore_errors=True)

    def test_a_written_artifact_satisfies_the_contract(self) -> None:
        payload = experiment_model.from_run(
            _run_document(),
            experiment_id="exp-042",
            title="Round trip",
            hypotheses=["H1"],
            control="before",
            candidate="after",
            complexity={"lines_changed": 10, "new_dependencies": [], "notes": ""},
            verdict={
                "decision": "accepted",
                "primary_job": "cold-scan-index",
                "reason": "faster",
            },
            run_artifact="docs/project/experiments/evidence/exp-042-run.json",
            run_artifact_sha256=_RUN_DIGEST,
            evidence_grade="claim-grade",
        )
        schema = Path("docs/project/experiments/experiment.schema.yaml").resolve()
        if not schema.is_file():
            self.skipTest("compiled schema is not present")
        shutil.copy(schema, self.scratch / record.SCHEMA_NAME)
        destination = self.scratch / "exp-042-round-trip.md"
        destination.write_text(record._render(payload, "# body\n"), encoding="utf-8")

        completed = subprocess.run(
            ["softschema", "validate", str(destination)],
            capture_output=True,
            timeout=600,
        )
        document = json.loads(completed.stdout.decode("utf-8", errors="replace"))
        self.assertEqual(
            document["outcome"],
            "valid",
            document.get("structural", {}).get("errors"),
        )
        self.assertEqual(document["values"]["id"], "exp-042")
        self.assertEqual(document["values"]["reference_tools"][0]["argv"][2], "1")


class SummaryRenderTests(unittest.TestCase):
    def _experiment(self, **overrides):
        payload = experiment_model.from_run(
            _run_document(),
            experiment_id="exp-042",
            title="Test experiment",
            hypotheses=["H1"],
            control="before",
            candidate="after",
            complexity={"lines_changed": 10, "new_dependencies": [], "notes": ""},
            verdict={
                "decision": "accepted",
                "primary_job": "cold-scan-index",
                "primary_metric": "wall_ns",
                "change_pct": -30.0,
                "reason": "faster",
            },
            run_artifact="docs/project/experiments/evidence/exp-042-run.json",
            run_artifact_sha256=_RUN_DIGEST,
            evidence_grade="claim-grade",
        )
        payload["_path"] = "docs/project/experiments/exp-042-test.md"
        payload.update(overrides)
        return payload

    def test_the_report_states_the_decision_and_the_number(self) -> None:
        text = summary.render([self._experiment()])
        self.assertIn("accepted", text)
        self.assertIn("-30.00%", text)
        self.assertIn("Test experiment", text)

    def test_a_baseline_gets_a_single_measured_column(self) -> None:
        baseline = self._experiment()
        baseline["verdict"] = dict(baseline["verdict"], decision="baseline", change_pct=None)
        text = summary.render([baseline])
        self.assertIn("| metric | value |", text)
        self.assertNotIn("+0.00%", text)

    def test_a_mutated_tree_is_called_out_loudly(self) -> None:
        broken = self._experiment()
        broken["subject"] = dict(broken["subject"], tree_mutated_during_run=True)
        text = " ".join(summary.render([broken]).split())
        self.assertIn("those numbers are not comparable", text)

    def test_oracle_rejections_disqualify_a_job(self) -> None:
        broken = self._experiment()
        broken["results"][0]["invalid_samples"] = 3
        text = summary.render([broken])
        self.assertIn("prove nothing", text)

    def test_no_dependency_is_stated_positively(self) -> None:
        text = summary.render([self._experiment()])
        self.assertIn("no new dependencies", text)

    def test_significant_resource_regression_is_not_rendered_as_noise(self) -> None:
        experiment = self._experiment()
        experiment["results"][0]["metrics"]["cpu_ns"] = {
            "control_median": 100.0,
            "candidate_median": 204.0,
            "change_pct": 104.0,
            "ci95_low_pct": 100.9,
            "ci95_high_pct": 108.6,
            "direction": "regression",
            "ci_excludes_zero": True,
            "significant_improvement": False,
            "significant": True,
            "pairs": 12,
        }
        text = summary.render([experiment])
        self.assertIn("+104.00% (significant regression)", text)

    def test_latency_and_resource_decisions_are_rendered_separately(self) -> None:
        experiment = self._experiment()
        experiment["verdict"].update(
            {
                "latency_gate_passed": True,
                "resource_guardrails": [
                    {
                        "metric": "cpu_ns",
                        "maximum_regression_pct": 10.0,
                        "observed_change_pct": 104.0,
                        "status": "waived",
                        "reason": "waived: interactive latency is the product priority",
                    }
                ],
            }
        )
        text = summary.render([experiment])
        self.assertIn("Latency gate: **passed**", text)
        self.assertIn("`cpu_ns`: **waived**", text)


if __name__ == "__main__":
    unittest.main()
