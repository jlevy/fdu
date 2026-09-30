"""The record has to agree with itself, and the gate has to have checked something.

Every case here is one the drift gates cannot catch. They prove the generated views
match the records; regenerate and they match a wrong record just as happily. So the
checks below compare a record with its own measurements, and the corpus gate refuses to
report success over nothing.
"""

from __future__ import annotations

import contextlib
import gzip
import io
import json
import re
import shutil
import tempfile
import unittest
from pathlib import Path
from typing import Any

from benchmarks.realtree import experiment as experiment_model
from benchmarks.realtree import ledger, record, summary, validate
from benchmarks.realtree.experiment import VERDICT_TOLERANCE_PCT, kept_arm
from pydantic import ValidationError
from test_experiment import _run_document

#: The figure exp-116 shipped with: a cross-job ratio (opened second report against a
#: one-shot walk, 1630x apart) written where the paired default-tree change belonged.
HISTORICAL_CROSS_JOB_FIGURE = -99.919

SCHEMA = Path("docs/project/experiments/experiment.schema.yaml")
EXPERIMENTS = Path("docs/project/experiments")


def payload(**verdict: Any) -> dict[str, Any]:
    """An artifact whose one measured job, cold-scan-index, moved -30.0% on wall."""
    stated = {
        "decision": "accepted",
        "primary_job": "cold-scan-index",
        "primary_metric": "wall_ns",
        "change_pct": -30.0,
        "reason": "faster",
    }
    stated.update(verdict)
    return experiment_model.from_run(
        _run_document(),
        experiment_id="exp-042",
        title="Test experiment",
        hypotheses=["H1"],
        control="before",
        candidate="after",
        complexity={"lines_changed": 10},
        verdict=stated,
    )


def validate_payload(document: dict[str, Any]) -> experiment_model.Experiment:
    return experiment_model.Experiment.model_validate(document)


class VerdictConsistencyTests(unittest.TestCase):
    """`verdict.change_pct` is the paired figure for the job and metric it names."""

    def test_the_recorded_figure_validates(self) -> None:
        experiment = validate_payload(payload())
        self.assertEqual(experiment.verdict.change_pct, -30.0)

    def test_the_historical_cross_job_figure_is_refused(self) -> None:
        with self.assertRaises(ValidationError) as raised:
            validate_payload(payload(change_pct=HISTORICAL_CROSS_JOB_FIGURE))
        message = str(raised.exception)
        self.assertIn("-99.919", message)
        self.assertIn("-30.0", message)
        self.assertIn("cold-scan-index", message)
        self.assertIn("wall_ns", message)

    def test_rounding_to_what_the_ledger_prints_is_not_a_disagreement(self) -> None:
        # The ledger prints two decimals. A figure copied at that precision agrees.
        validate_payload(payload(change_pct=-30.004))
        validate_payload(payload(change_pct=-29.996))

    def test_a_figure_that_would_print_differently_is_refused(self) -> None:
        for figure in (-30.02, -29.98, -30.5, -31.0):
            with self.subTest(figure=figure), self.assertRaises(ValidationError):
                validate_payload(payload(change_pct=figure))

    def test_the_tolerance_is_one_unit_in_the_ledgers_last_digit(self) -> None:
        self.assertEqual(VERDICT_TOLERANCE_PCT, 0.01)

    def test_a_verdict_may_state_no_headline(self) -> None:
        # exp-000 is a baseline with nothing to compare; None is never a wrong number.
        validate_payload(payload(decision="baseline", change_pct=None))
        validate_payload(payload(change_pct=None))

    def test_a_job_the_record_did_not_measure_is_refused(self) -> None:
        with self.assertRaises(ValidationError) as raised:
            validate_payload(payload(primary_job="warm-revalidate"))
        self.assertIn("warm-revalidate", str(raised.exception))
        self.assertIn("cold-scan-index", str(raised.exception))

    def test_a_metric_the_job_did_not_measure_is_refused(self) -> None:
        with self.assertRaises(ValidationError) as raised:
            validate_payload(payload(primary_metric="cpu_ns"))
        self.assertIn("cpu_ns", str(raised.exception))

    def test_a_figure_with_no_results_behind_it_is_refused(self) -> None:
        document = payload()
        document["results"] = []
        with self.assertRaises(ValidationError) as raised:
            validate_payload(document)
        self.assertIn("results is empty", str(raised.exception))

    def test_no_results_and_no_figure_is_an_honest_record(self) -> None:
        # A blocked hypothesis may have nothing measured and nothing claimed.
        document = payload(decision="blocked", change_pct=None)
        document["results"] = []
        validate_payload(document)


class KeptArmTests(unittest.TestCase):
    """The arm on the product's axis is a property of the record, not of a list."""

    def test_omitted_kept_reads_as_the_decision_implies(self) -> None:
        self.assertEqual(kept_arm({"decision": "accepted"}), "candidate")
        for decision in ("rejected", "superseded", "blocked", "in-progress", "baseline"):
            self.assertEqual(kept_arm({"decision": decision}), "control", decision)

    def test_a_stated_kept_overrides_the_decision(self) -> None:
        self.assertEqual(kept_arm({"decision": "accepted", "kept": "control"}), "control")
        self.assertEqual(kept_arm({"decision": "rejected", "kept": "candidate"}), "candidate")
        self.assertIsNone(kept_arm({"decision": "rejected", "kept": "neither"}))
        self.assertIsNone(kept_arm({"decision": "accepted", "kept": "neither"}))

    def test_the_model_accepts_each_stated_arm(self) -> None:
        for decision, kept in (
            ("accepted", "control"),
            ("accepted", "neither"),
            ("rejected", "candidate"),
            ("rejected", "neither"),
            ("baseline", "control"),
        ):
            with self.subTest(decision=decision, kept=kept):
                experiment = validate_payload(payload(decision=decision, kept=kept))
                self.assertEqual(experiment.verdict.kept, kept)

    def test_a_baseline_cannot_keep_a_candidate(self) -> None:
        with self.assertRaises(ValidationError) as raised:
            validate_payload(payload(decision="baseline", change_pct=None, kept="candidate"))
        self.assertIn("baseline", str(raised.exception))

    def test_an_unknown_arm_is_refused(self) -> None:
        with self.assertRaises(ValidationError):
            validate_payload(payload(kept="both"))

    def test_the_recorder_states_the_kept_arm_on_every_artifact(self) -> None:
        # Written even when it is what the decision implies, so the claim is in the diff.
        for decision, kept in (("accepted", "candidate"), ("rejected", "control")):
            written = kept_arm({"decision": decision})
            self.assertEqual(written, kept, decision)
        rendered = record._render(payload(kept="candidate"), "# body\n")
        self.assertIn("    kept: candidate\n", rendered)


class CorpusGateTests(unittest.TestCase):
    """The gate reads through the ledger's loader, and cannot pass by checking nothing.

    These go through the softschema CLI exactly as `make perf-ledger` does, because the
    laundering path is `make perf-ledger && make perf-report`: the defect is only fixed
    if the loader those commands share is the thing that refuses.
    """

    def setUp(self) -> None:
        try:
            summary._validator()
        except Exception:
            self.skipTest("softschema is not available; run through `make perf-test`")
        if not SCHEMA.is_file():
            self.skipTest("compiled schema is not present; run from the repository root")
        self.scratch = Path(tempfile.mkdtemp(prefix="fdu-evidence-test-"))
        self.addCleanup(shutil.rmtree, self.scratch, ignore_errors=True)
        shutil.copy(SCHEMA, self.scratch / SCHEMA.name)

    def _write(self, name: str, document: dict[str, Any]) -> Path:
        destination = self.scratch / f"{name}.md"
        destination.write_text(record._render(document, "# body\n"), encoding="utf-8")
        return destination

    def test_a_consistent_corpus_reports_what_it_checked(self) -> None:
        self._write("exp-042-a", payload())
        self._write("exp-043-b", payload(change_pct=None))
        document = payload(decision="rejected", kept="neither")
        document["id"] = "exp-044"
        self._write("exp-044-c", document)
        for index, name in enumerate(("exp-042-a", "exp-043-b")):
            text = (self.scratch / f"{name}.md").read_text(encoding="utf-8")
            (self.scratch / f"{name}.md").write_text(
                text.replace("id: exp-042", f"id: exp-{42 + index:03d}"), encoding="utf-8"
            )

        report = validate.validate_corpus(self.scratch)

        self.assertEqual(report.records, 3)
        self.assertEqual(report.compared, 2)
        self.assertEqual(report.stated_kept, 1)
        self.assertIn("3 records", report.summary())

    def test_the_loader_the_ledger_uses_refuses_a_laundered_headline(self) -> None:
        # This is the regeneration path. `summary.load_experiments` is what
        # `make perf-ledger` and `make perf-report` both read through, so a record that
        # passes here would be regenerated into every published view.
        self._write("exp-042-bad", payload(change_pct=HISTORICAL_CROSS_JOB_FIGURE))

        with self.assertRaises(summary.SummaryError) as raised:
            summary.load_experiments(self.scratch)
        self.assertIn("-99.919", str(raised.exception))
        self.assertIn("exp-042-bad.md", str(raised.exception))

    def test_the_gate_refuses_a_laundered_headline(self) -> None:
        self._write("exp-042-bad", payload(change_pct=HISTORICAL_CROSS_JOB_FIGURE))

        with self.assertRaises(validate.CorpusError) as raised:
            validate.validate_corpus(self.scratch)
        self.assertIn("-99.919", str(raised.exception))

    def test_zero_records_is_a_failure_not_a_pass(self) -> None:
        with self.assertRaises(validate.CorpusError) as raised:
            validate.validate_corpus(self.scratch)
        self.assertIn("no records", str(raised.exception))

    def test_records_that_compare_nothing_are_a_failure_not_a_pass(self) -> None:
        self._write("exp-042-silent", payload(change_pct=None))

        with self.assertRaises(validate.CorpusError) as raised:
            validate.validate_corpus(self.scratch)
        self.assertIn("nothing was checked", str(raised.exception))

    def test_the_committed_record_that_shipped_the_figure_is_now_refused_with_it(self) -> None:
        # exp-116 is the committed record that carried -99.9 for a paired -2.158. Put the
        # historical figure back into a scratch copy and the gate names it.
        matches = sorted(EXPERIMENTS.glob("exp-116-*.md"))
        if len(matches) != 1:
            self.skipTest("exp-116 is not in the committed corpus")
        text = matches[0].read_text(encoding="utf-8")
        # Anchored at the line start: the same figure sits deeper-indented in the
        # results entry, and rewriting both would leave the record consistent.
        verdict_line = "\n    change_pct: -2.158\n"
        self.assertEqual(text.count(verdict_line), 1)
        (self.scratch / matches[0].name).write_text(
            text.replace(verdict_line, f"\n    change_pct: {HISTORICAL_CROSS_JOB_FIGURE}\n"),
            encoding="utf-8",
        )

        with self.assertRaises(validate.CorpusError) as raised:
            validate.validate_corpus(self.scratch)
        message = str(raised.exception)
        self.assertIn("exp-116", message)
        self.assertIn("-99.919", message)
        self.assertIn("-2.158", message)
        self.assertIn("default-tree", message)


def gzip_deterministically(data: bytes) -> bytes:
    """Compress the way the performance loop stores a run: no file name, an mtime of 0."""
    buffer = io.BytesIO()
    with gzip.GzipFile(filename="", mode="wb", fileobj=buffer, compresslevel=9, mtime=0) as out:
        out.write(data)
    return buffer.getvalue()


class CompressedRunArtifactTests(unittest.TestCase):
    """A run stored as ``.json.gz`` reads, records, and validates as its plain form does.

    Committed run artifacts are gzipped, because hundreds of thousands of lines of raw
    JSON made up most of one pull request's diff; older artifacts stay plain. Both forms
    have to give the same answer, or compressing a run would change the evidence.
    """

    def setUp(self) -> None:
        self.scratch = Path(tempfile.mkdtemp(prefix="fdu-run-artifact-test-"))
        self.addCleanup(shutil.rmtree, self.scratch, ignore_errors=True)
        self.document = _run_document()
        encoded = json.dumps(self.document, indent=2, sort_keys=True).encode("utf-8")
        self.plain = self.scratch / "plain" / "run.json"
        self.compressed = self.scratch / "compressed" / "run.json.gz"
        self.plain.parent.mkdir()
        self.compressed.parent.mkdir()
        self.plain.write_bytes(encoded)
        self.compressed.write_bytes(gzip_deterministically(encoded))

    def _record(self, run: Path) -> Path:
        output = run.parent / "records"
        argv = [
            "--run",
            str(run),
            "--id",
            "exp-042",
            "--title",
            "Test experiment",
            "--hypothesis",
            "H1",
            "--control",
            "before",
            "--candidate",
            "after",
            "--decision",
            "accepted",
            "--primary-job",
            "cold-scan-index",
            "--reason",
            "faster",
            "--tree-provenance",
            "fixture",
            "--output-dir",
            str(output),
            "--no-validate",
        ]
        with contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(record.main(argv), 0)
        (written,) = sorted(output.glob("exp-*.md"))
        return written

    def test_the_loader_reads_both_forms_to_the_same_document(self) -> None:
        self.assertEqual(ledger.load(self.plain), self.document)
        self.assertEqual(ledger.load(self.compressed), self.document)

    def test_storing_a_run_compresses_it_deterministically_and_whole(self) -> None:
        stored = self.scratch / "stored" / "run.json.gz"
        ledger.store(self.plain, stored)
        self.assertEqual(stored.read_bytes(), self.compressed.read_bytes())
        self.assertEqual(ledger.load(stored), self.document)
        ledger.store(self.plain, stored)
        self.assertEqual(stored.read_bytes(), self.compressed.read_bytes())
        self.assertEqual([entry.name for entry in stored.parent.iterdir()], ["run.json.gz"])

        with self.assertRaisesRegex(ValueError, r"must end in \.gz"):
            ledger.store(self.plain, self.scratch / "stored" / "run.json")

    def test_a_gz_suffix_on_plain_json_fails_rather_than_being_guessed(self) -> None:
        mislabelled = self.scratch / "mislabelled.json.gz"
        mislabelled.write_bytes(self.plain.read_bytes())
        with self.assertRaises(gzip.BadGzipFile):
            ledger.load(mislabelled)

    def test_recording_from_a_gzipped_run_writes_the_same_record(self) -> None:
        from_plain = self._record(self.plain).read_text(encoding="utf-8")
        from_compressed = self._record(self.compressed).read_text(encoding="utf-8")

        self.assertIn(f"    run_artifact: {self.compressed}\n", from_compressed)
        self.assertIn(f"    run_artifact: {self.plain}\n", from_plain)
        self.assertEqual(from_compressed.replace(str(self.compressed), str(self.plain)), from_plain)

    def test_a_record_naming_a_gzipped_artifact_validates_with_the_same_figures(self) -> None:
        try:
            summary._validator()
        except Exception:
            self.skipTest("softschema is not available; run through `make perf-test`")
        if not SCHEMA.is_file():
            self.skipTest("compiled schema is not present; run from the repository root")

        loaded: dict[str, dict[str, Any]] = {}
        for form, run in (("plain", self.plain), ("compressed", self.compressed)):
            written = self._record(run)
            shutil.copy(SCHEMA, written.parent / SCHEMA.name)
            report = validate.validate_corpus(written.parent)
            self.assertEqual((report.records, report.compared), (1, 1), form)
            (experiment,) = summary.load_experiments(written.parent)
            self.assertEqual(experiment["method"]["run_artifact"], str(run), form)
            loaded[form] = experiment

        plain, compressed = loaded["plain"], loaded["compressed"]
        self.assertEqual(compressed["verdict"]["change_pct"], -30.0)
        self.assertEqual(compressed["results"], plain["results"])
        for experiment in (plain, compressed):
            del experiment["_path"]
            del experiment["method"]["run_artifact"]
        self.assertEqual(compressed, plain)

    def test_every_committed_run_artifact_loads_and_is_stored_deterministically(self) -> None:
        # A record's `run_artifact` is only a string to the contract, so nothing else
        # notices a record left pointing at a run that was compressed and removed.
        if not EXPERIMENTS.is_dir():
            self.skipTest("the committed corpus is not present; run from the repository root")
        committed = re.compile(r"^    run_artifact: (docs/project/experiments/evidence/\S+)$", re.M)
        artifacts = sorted(
            {
                Path(match)
                for path in EXPERIMENTS.glob("exp-*.md")
                for match in committed.findall(path.read_text(encoding="utf-8"))
            }
        )
        compressed = [path for path in artifacts if path.suffix == ".gz"]
        self.assertTrue(artifacts, "no record names a committed run artifact")
        self.assertTrue(compressed, "no record names a gzipped run artifact")

        for path in artifacts:
            with self.subTest(artifact=str(path)):
                self.assertTrue(path.is_file(), f"{path} is named by a record but missing")
                run = ledger.load(path)
                self.assertIn("statistics", run)
                self.assertIn("jobs", run)
        for path in compressed:
            with self.subTest(artifact=str(path)):
                header = path.read_bytes()[:8]
                # No flags (so no stored file name) and a zero mtime: the same run always
                # compresses to the same header, whoever stores it and whenever.
                self.assertEqual(header[3], 0, f"{path} stores a file name or other field")
                self.assertEqual(header[4:8], b"\0\0\0\0", f"{path} stores a timestamp")


if __name__ == "__main__":
    unittest.main()
