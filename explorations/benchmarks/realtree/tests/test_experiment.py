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

from benchmarks.realtree.summary import _validator
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


class HeadlineSelectionTests(unittest.TestCase):
    """The recorded change must belong to the pair the experiment claims to be about.

    Every case here used to return quietly: a wrong number, or a null that reads as
    "this experiment was not comparative". The artifact is the permanent record the
    ledger is regenerated from, so a wrong number here outlives the run that made it.
    """

    @staticmethod
    def _arguments(**overrides):
        import argparse

        values = {
            "primary_job": "cold-scan-index",
            "primary_metric": "wall_ns",
            "control_variant": None,
            "candidate_variant": None,
        }
        values.update(overrides)
        return argparse.Namespace(**values)

    @staticmethod
    def _sweep_run() -> dict:
        """A run holding two comparisons, as a thread or batch sweep produces."""
        run = _run_document()
        comparisons = run["statistics"]["cold-scan-index"]["comparisons"]
        comparisons["four_threads_vs_control"] = {
            "control": "control",
            "candidate": "four_threads",
            "metrics": {
                "wall_ns": {
                    "pairs": 12,
                    "median_delta": -100.0,
                    "median_change_pct": -10.0,
                    "ci95_change_pct": [-12.0, -8.0],
                    "improved": True,
                    "significant": True,
                }
            },
        }
        return run

    def test_a_single_comparison_needs_no_variant_names(self) -> None:
        self.assertEqual(record._headline(_run_document(), self._arguments()), -30.0)

    def test_a_named_pair_selects_that_pair(self) -> None:
        headline = record._headline(
            self._sweep_run(),
            self._arguments(control_variant="control", candidate_variant="four_threads"),
        )
        self.assertEqual(headline, -10.0)

    def test_a_sweep_without_variant_names_is_refused(self) -> None:
        with self.assertRaises(ValueError) as caught:
            record._headline(self._sweep_run(), self._arguments())
        self.assertIn("several comparisons", str(caught.exception))

    def test_half_a_pair_is_refused(self) -> None:
        with self.assertRaises(ValueError) as caught:
            record._headline(_run_document(), self._arguments(candidate_variant="candidate"))
        self.assertIn("together", str(caught.exception))

    def test_a_pair_the_run_does_not_hold_is_refused(self) -> None:
        with self.assertRaises(ValueError) as caught:
            record._headline(
                _run_document(),
                self._arguments(control_variant="control", candidate_variant="absent"),
            )
        self.assertIn("absent_vs_control", str(caught.exception))

    def test_an_existing_job_without_comparisons_is_non_comparative(self) -> None:
        run = _run_document()
        run["statistics"]["cold-scan-index"]["comparisons"] = {}

        self.assertIsNone(record._headline(run, self._arguments()))

    def test_a_missing_primary_job_is_refused(self) -> None:
        with self.assertRaises(ValueError) as caught:
            record._headline(_run_document(), self._arguments(primary_job="warm-revalidate"))
        self.assertIn("warm-revalidate", str(caught.exception))

    def test_a_missing_primary_metric_is_refused(self) -> None:
        with self.assertRaises(ValueError) as caught:
            record._headline(_run_document(), self._arguments(primary_metric="cpu_ns"))
        self.assertIn("cpu_ns", str(caught.exception))


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

    def test_iso_dates_are_quoted(self) -> None:
        # The contract declares `date: str`. Written plainly, YAML resolves the scalar
        # to a date object, so the artifact fails its own schema on the way back in --
        # which every artifact recorded before this quoting did.
        self.assertEqual(record._scalar("2026-08-13"), '"2026-08-13"')

    def test_a_rendered_artifact_carries_its_date_as_a_string(self) -> None:
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
        rendered = record._render(payload, "# body\n")

        self.assertIn(f'  date: "{payload["date"]}"', rendered)


class RenderRoundTripTests(unittest.TestCase):
    """The artifact must validate against its own compiled contract."""

    def setUp(self) -> None:
        try:
            self.validator = _validator()
        except Exception:  # noqa: BLE001 - the harness reports its own reason
            self.skipTest("softschema is not available; run through `make perf-test`")
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
            [*self.validator, "validate", str(destination)],
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

    def test_a_punctuated_reason_does_not_get_a_second_period(self) -> None:
        experiment = self._experiment()
        experiment["verdict"] = dict(experiment["verdict"], reason="already punctuated.")

        text = summary.render([experiment])

        self.assertIn("**Accepted:** already punctuated.", text)
        self.assertNotIn("already punctuated..", text)

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

    def test_reproduction_conditions_stay_with_the_cumulative_comparison(self) -> None:
        cumulative = self._experiment(title="Cumulative effect of accepted changes")
        later = self._experiment(title="Later experiment on another tree")
        later["subject"] = dict(
            later["subject"],
            tree_label="other-tree",
            tree_entries=999,
        )

        text = summary.render([cumulative, later])
        conditions = text.split("## Reproducing the cumulative comparison", 1)[1].split(
            "## Every experiment", 1
        )[0]

        self.assertIn("Label `fixture`, 100 entries", conditions)
        self.assertNotIn("other-tree", conditions)
        self.assertNotIn("999 entries", conditions)


class IdentifierCollisionTests(unittest.TestCase):
    """The failure mode that let two campaigns claim one experiment id.

    Each side's artifacts validated individually, so nothing caught it until the
    branches met and one campaign's rows silently stood in for the other's.
    """

    def _experiment(self, identifier: str, title: str, hypotheses):
        payload = experiment_model.from_run(
            _run_document(),
            experiment_id=identifier,
            title=title,
            hypotheses=list(hypotheses),
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
        payload["_path"] = f"docs/project/experiments/{identifier}-{title}.md"
        return payload

    def test_two_artifacts_claiming_one_id_fail_the_build(self) -> None:
        experiments = [
            self._experiment("exp-056", "adaptive diagnostics", ["H86-observability"]),
            self._experiment("exp-056", "extension memo", ["H89"]),
        ]
        with self.assertRaises(summary.SummaryError) as raised:
            summary.check_identifiers(experiments)
        message = str(raised.exception)
        self.assertIn("exp-056", message)
        self.assertIn("claimed by 2 artifacts", message)

    def test_one_hypothesis_across_many_experiments_is_not_a_collision(self) -> None:
        # The record's normal shape: H31 spans twelve experiments under twelve titles,
        # because cumulative and validation runs re-test the same claim. A rule keyed on
        # differing titles would reject the entire committed ledger.
        experiments = [
            self._experiment("exp-015", "post-BFS worker depth", ["H31"]),
            self._experiment("exp-023", "cumulative through adaptive scanning", ["H31"]),
            self._experiment("exp-032", "cumulative through bounded parallel", ["H31"]),
        ]
        self.assertEqual(summary.check_identifiers(experiments), [])

    def test_one_number_under_two_spellings_warns(self) -> None:
        experiments = [
            self._experiment("exp-059", "fixed worker knee", ["H87-fixed-worker-knee"]),
            self._experiment("exp-063", "share the index with the writer", ["H87"]),
        ]
        warnings = summary.check_identifiers(experiments)
        self.assertEqual(len(warnings), 1)
        self.assertIn("H87", warnings[0])
        self.assertIn("2 spellings", warnings[0])


class AbsoluteTimingTests(unittest.TestCase):
    """Absolute walls are recorded per artifact; the ledger has to show them."""

    def _experiment(self, identifier: str, *, decision: str = "accepted", **subject):
        payload = experiment_model.from_run(
            _run_document(),
            experiment_id=identifier,
            title=f"Experiment {identifier}",
            hypotheses=["H1"],
            control="before",
            candidate="after",
            complexity={"lines_changed": 10, "new_dependencies": [], "notes": ""},
            verdict={
                "decision": decision,
                "primary_job": "cold-scan-index",
                "primary_metric": "wall_ns",
                "change_pct": -30.0,
                "reason": "faster",
            },
        )
        payload["_path"] = f"docs/project/experiments/{identifier}-test.md"
        payload["subject"].update(subject)
        return payload

    def test_absolute_milliseconds_reach_the_report(self) -> None:
        text = summary.render([self._experiment("exp-042")])
        appendix = text.split("## Absolute timings", 1)[1]
        entry = _run_document()["statistics"]["cold-scan-index"]["variants"]
        control_ms = entry["control"]["metrics"]["wall_ns"]["median"] / 1e6
        self.assertIn(f"{control_ms:,.1f}", appendix)

    def test_subjects_are_grouped_so_absolutes_are_never_compared_across_trees(self) -> None:
        text = summary.render(
            [
                self._experiment("exp-042", tree_label="alpha", tree_entries=100),
                self._experiment("exp-043", tree_label="beta", tree_entries=999),
            ]
        )
        appendix = text.split("## Absolute timings", 1)[1]
        self.assertIn("alpha (100 entries)", appendix)
        self.assertIn("beta (999 entries)", appendix)

    def test_a_baseline_shows_one_value_because_it_measures_a_state(self) -> None:
        text = summary.render([self._experiment("exp-042", decision="baseline")])
        row = [
            line
            for line in text.split("## Absolute timings", 1)[1].splitlines()
            if line.startswith("| 042 ")
        ]
        self.assertEqual(len(row), 1)
        # Candidate and change columns are empty: there was no comparison to report.
        self.assertEqual(row[0].count("| — |"), 1)
        self.assertIn("— | —", row[0])


if __name__ == "__main__":
    unittest.main()
