"""Tests for the experiment artifact contract and the ledger built from it.

The ledger is the part of this work that outlives the session, so the things worth
pinning are the ones that would silently corrupt the record: reading the control and
candidate the wrong way round, writing YAML that parses back as a different type, and
letting an artifact that no longer matches its contract contribute a row anyway.
"""

from __future__ import annotations

import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

from benchmarks.realtree import experiment as experiment_model
from benchmarks.realtree import record, summary


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
        "schema": "fdu-realtree-run-v1",
        "started_utc": "2026-08-10T12:00:00Z",
        "note": "",
        "host": {
            "cpu_model": "Test CPU",
            "cpu_count": 8,
            "system": "Darwin",
            "release": "25.5.0",
            "filesystem": "apfs",
        },
        "conditions": {
            "os_cache": "warm-steady",
            "trials": 12,
            "warmups": 3,
            "interleaved": True,
            "schedule": "round-robin-by-ordinal-v1",
        },
        "tree": {
            "label": "fixture",
            "root_id": "a" * 64,
            "counts": {"total": 100, "directories": 10, "files": 88, "symlinks": 2, "other": 0},
            "sizes": {"apparent_bytes": 4096, "allocated_bytes": 8192},
            "max_depth": 4,
        },
        "tree_mutated_during_run": [],
        # Deliberately alphabetical, which is the order that would invert the
        # comparison if declaration order were not recorded separately.
        "variant_order": ["control", "candidate"],
        "variants": {
            "candidate": {"kind": "fdu-probe", "name": "candidate", "notes": "", "sha256": "b" * 64, "size_bytes": 1, "args": []},
            "control": {"kind": "fdu-probe", "name": "control", "notes": "", "sha256": "c" * 64, "size_bytes": 1, "args": []},
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


class YamlScalarTests(unittest.TestCase):
    """A string has to come back as a string, or the schema rejects it downstream."""

    def test_digit_strings_are_quoted(self) -> None:
        self.assertEqual(record._scalar("1"), '"1"')
        self.assertEqual(record._scalar("2026"), '"2026"')
        self.assertEqual(record._scalar("1.5"), '"1.5"')

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
        )
        schema = Path("docs/project/experiments/experiment.schema.yaml").resolve()
        if not schema.is_file():
            self.skipTest("compiled schema is not present")
        shutil.copy(schema, self.scratch / record.SCHEMA_NAME)
        destination = self.scratch / "exp-042-round-trip.md"
        destination.write_text(record._render(payload, "# body\n"), encoding="utf-8")

        completed = subprocess.run(
            ["uvx", "softschema@latest", "validate", str(destination)],
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


if __name__ == "__main__":
    unittest.main()
