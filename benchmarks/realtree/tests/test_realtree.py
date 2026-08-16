"""Tests for the real-tree performance harness.

The harness makes claims about a codebase's speed, so the harness itself has to be
worth believing. These tests cover the three things that would make a result a lie:
leaking a path from a private tree, failing to notice that the tree changed, and
calling a difference significant when it is noise.
"""

from __future__ import annotations

import json
import os
import shutil
import statistics
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

from benchmarks import corpus as corpus_tools
from benchmarks.realtree import __main__ as realtree_cli
from benchmarks.realtree import ledger, measure, profile, tree


def _write(path: Path, contents: bytes = b"x") -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(contents)


class ReferenceTreeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.scratch = Path(tempfile.mkdtemp(prefix="fdu-realtree-test-"))
        self.addCleanup(shutil.rmtree, self.scratch, ignore_errors=True)
        self.root = self.scratch / "tree"
        _write(self.root / "alpha.txt", b"hello")
        _write(self.root / "nested" / "beta.rs", b"fn main() {}")
        _write(self.root / "nested" / "deeper" / "gamma.json", b"{}")

    def test_fingerprint_counts_every_entry_once(self) -> None:
        document = tree.fingerprint(self.root, label="fixture")
        # 3 files + 2 directories + the root itself.
        self.assertEqual(document["counts"]["total"], 6)
        self.assertEqual(document["counts"]["files"], 3)
        self.assertEqual(document["counts"]["directories"], 3)
        self.assertEqual(document["sizes"]["apparent_bytes"], 5 + 12 + 2)
        self.assertEqual(
            document["newest_file_mtime_ns"],
            max(path.stat().st_mtime_ns for path in self.root.rglob("*") if path.is_file()),
        )
        self.assertEqual(document["max_depth"], 3)

    def test_allocated_bytes_use_apparent_size_when_blocks_are_unavailable(self) -> None:
        metadata = SimpleNamespace(st_size=7, st_blocks=2)
        with mock.patch("benchmarks.realtree.tree.os.name", "posix"):
            self.assertEqual(tree._allocated_bytes(metadata), 1024)
        with mock.patch("benchmarks.realtree.tree.os.name", "nt"):
            self.assertEqual(tree._allocated_bytes(metadata), 7)

    def test_fingerprint_discloses_no_path_or_name(self) -> None:
        document = tree.fingerprint(self.root, label="fixture")
        serialized = json.dumps(document)
        for secret in ("alpha", "beta", "gamma", "nested", "deeper", str(self.root)):
            self.assertNotIn(secret, serialized, f"fingerprint leaked {secret!r} from the tree")

    def test_fingerprint_quantifies_hardlinks_without_disclosing_names(self) -> None:
        linked = self.root / "nested" / "linked-copy.txt"
        os.link(self.root / "alpha.txt", linked)
        linked_allocated = int(linked.stat().st_blocks) * 512

        document = tree.fingerprint(self.root, label="fixture")

        self.assertEqual(
            document["hardlinks"],
            {
                "groups": 1,
                "linked_file_entries": 2,
                "duplicate_file_entries": 1,
                "duplicate_apparent_bytes": 5,
                "duplicate_allocated_bytes": linked_allocated,
            },
        )
        serialized = json.dumps(document)
        self.assertNotIn("linked-copy", serialized)
        self.assertNotIn("alpha", serialized)

    def test_root_id_identifies_without_disclosing(self) -> None:
        document = tree.fingerprint(self.root, label="fixture")
        self.assertEqual(len(document["root_id"]), 64)
        self.assertEqual(document["root_id"], tree.root_id(self.root))
        self.assertNotEqual(document["root_id"], tree.root_id(self.scratch))

    def test_identical_observations_compare_equal(self) -> None:
        first = tree.fingerprint(self.root, label="fixture")
        second = tree.fingerprint(self.root, label="fixture")
        self.assertEqual(tree.compare(second, first), [])

    def test_fingerprint_schema_drift_is_not_comparable(self) -> None:
        current = tree.fingerprint(self.root, label="fixture")
        baseline = dict(current)
        baseline["schema"] = "fdu-reference-tree-v1"

        self.assertIn("schema differs", " ".join(tree.compare(current, baseline)))

    def test_content_change_is_detected(self) -> None:
        before = tree.fingerprint(self.root, label="fixture")
        _write(self.root / "alpha.txt", b"hello there")
        after = tree.fingerprint(self.root, label="fixture")
        reasons = tree.compare(after, before)
        self.assertTrue(reasons)
        self.assertIn("contents changed", reasons[0])

    def test_added_entry_is_detected(self) -> None:
        before = tree.fingerprint(self.root, label="fixture")
        _write(self.root / "nested" / "new.txt", b"new")
        after = tree.fingerprint(self.root, label="fixture")
        self.assertTrue(tree.compare(after, before))

    def test_metadata_only_change_is_detected(self) -> None:
        # Size and name are identical; only mtime moved. A fingerprint that missed
        # this would let a rewritten file pass as an unchanged tree.
        before = tree.fingerprint(self.root, label="fixture")
        target = self.root / "alpha.txt"
        os.utime(target, ns=(1_000_000_000, 1_000_000_000))
        after = tree.fingerprint(self.root, label="fixture")
        self.assertTrue(tree.compare(after, before))

    def test_a_different_root_is_never_comparable(self) -> None:
        other = self.scratch / "other"
        shutil.copytree(self.root, other)
        self.assertIn(
            "root differs from the baseline root",
            " ".join(
                tree.compare(
                    tree.fingerprint(other, label="x"), tree.fingerprint(self.root, label="x")
                )
            ),
        )

    def test_probe_summary_must_match_the_oracle(self) -> None:
        document = tree.fingerprint(self.root, label="fixture")
        agreeing = {
            "dirs": document["counts"]["directories"],
            "entries": document["counts"]["total"],
            "files": document["counts"]["files"],
            "other": document["counts"]["other"],
            "symlinks": document["counts"]["symlinks"],
            "apparent_bytes": document["sizes"]["apparent_bytes"],
            "allocated_bytes": document["sizes"]["allocated_bytes"],
            "newest_file_mtime_ns": document["newest_file_mtime_ns"],
            "engine_digest": document["engine_digest"],
        }
        self.assertIsNone(tree.probe_agrees(document, agreeing))

        for field in ("entries", "files", "apparent_bytes", "engine_digest"):
            wrong = dict(agreeing)
            wrong[field] = "wrong" if field == "engine_digest" else 0
            self.assertIsNotNone(
                tree.probe_agrees(document, wrong),
                f"a probe reporting the wrong {field} was accepted",
            )

    def test_missing_summary_is_a_disagreement(self) -> None:
        document = tree.fingerprint(self.root, label="fixture")
        self.assertIsNotNone(tree.probe_agrees(document, None))

    def test_harness_state_must_live_outside_the_measured_tree(self) -> None:
        with self.assertRaisesRegex(SystemExit, "outside the measured --root"):
            realtree_cli._require_external(
                self.root, self.root / "results", description="result directory"
            )

        realtree_cli._require_external(
            self.root, self.scratch / "results", description="result directory"
        )

    def test_generated_corpus_binding_is_verified_and_path_free(self) -> None:
        run_root = corpus_tools.reserve_run_directory(self.scratch / "generated")
        corpus_tools.create_corpus(run_root, "phased-fast-slow", target_entries=40)
        manifest_path = run_root / corpus_tools.MANIFEST_NAME

        bound = realtree_cli._verified_corpus_manifest(
            run_root / corpus_tools.CORPUS_NAME, manifest_path
        )

        self.assertEqual(bound["recipe_id"], "phased-fast-slow")
        serialized = json.dumps(bound)
        self.assertNotIn(str(run_root), serialized)
        self.assertNotIn("records", bound)
        with self.assertRaisesRegex(SystemExit, "does not describe"):
            realtree_cli._verified_corpus_manifest(self.root, manifest_path)


class StatisticsTests(unittest.TestCase):
    @staticmethod
    def _diagnostic_probe(outcome: str = "fixed") -> bytes:
        windows = []
        if outcome != "fixed":
            windows = [
                {
                    "active_workers": 6,
                    "decision": {
                        "held": "hold",
                        "scaled_up": "scale_up",
                        "undecided": "undecided",
                    }.get(outcome, "hold"),
                    "end_entry_ordinal": 16_384,
                    "handoff_backlog": 1,
                    "in_flight_directories": 4,
                    "observed_chunks": 1,
                    "observed_entries": 16_384,
                    "observed_work_ns": 16_384_000,
                    "ready_directories": 8,
                    "requested_workers": 16 if outcome == "scaled_up" else None,
                    "sequence": 0,
                    "start_entry_ordinal": 0,
                    "work_ns_per_entry": 1_000,
                    "work_ns_per_entry_unavailable_reason": None,
                }
            ]
        document = {
            "component_ns": 1,
            "schema": "fdu-perf-probe-v1",
            "summary": {"complete": True, "dirs_read": 4, "errors": 0},
            "scan_diagnostics": {
                "schema": "fdu-scan-diagnostics-v1",
                "backend": {
                    "macos_bulk_attempts": 4,
                    "macos_bulk_successes": 3,
                    "macos_bulk_fallbacks": 1,
                    "portable_attempts": 1,
                    "portable_directory_reads": 1,
                    "unavailable_reason": None,
                },
                "worker_policy": {
                    "available_parallelism": 16,
                    "calibration_chunks": 0 if outcome == "fixed" else 1,
                    "calibration_entries": 0 if outcome == "fixed" else 16_384,
                    "calibration_window_entries": None if outcome == "fixed" else 16_384,
                    "calibration_work_ns": 0 if outcome == "fixed" else 16_384_000,
                    "controller": "shipped_one_shot",
                    "events_truncated": False,
                    "handoff_backlog_at_finish": 0,
                    "handoff_backlog_high_water": 2,
                    "in_flight_directories_at_finish": 0,
                    "initial_workers": 6,
                    "maximum_workers": 16,
                    "outcome": outcome,
                    "peak_active_workers": 16 if outcome == "scaled_up" else 6,
                    "ready_directories_at_finish": 0,
                    "slow_threshold_ns_per_entry": None if outcome == "fixed" else 20_000,
                    "windows": windows,
                    "worker_expansions": 1 if outcome == "scaled_up" else 0,
                    "workers_spawned": 16 if outcome == "scaled_up" else 6,
                },
            },
        }
        return json.dumps(document).encode()

    def test_claim_grade_scan_diagnostics_are_retained_and_validated(self) -> None:
        document, reasons = measure._read_probe_output(
            self._diagnostic_probe(),
            require_scan_diagnostics=True,
            require_macos_backend=True,
        )

        self.assertEqual(reasons, [])
        self.assertEqual(document["scan_diagnostics"]["schema"], "fdu-scan-diagnostics-v1")

    def test_claim_grade_scan_rejects_truncated_or_unobservable_policy(self) -> None:
        encoded = self._diagnostic_probe("undecided")
        document = json.loads(encoded)
        document["scan_diagnostics"]["worker_policy"]["events_truncated"] = True

        _, reasons = measure._read_probe_output(
            json.dumps(document).encode(), require_scan_diagnostics=True
        )

        joined = "\n".join(reasons)
        self.assertIn("trace is truncated", joined)
        self.assertIn("policy was not observable", joined)

    def test_claim_grade_macos_scan_rejects_null_backend_counts(self) -> None:
        document = json.loads(self._diagnostic_probe())
        backend = document["scan_diagnostics"]["backend"]
        backend["macos_bulk_attempts"] = None
        backend["macos_bulk_successes"] = None
        backend["macos_bulk_fallbacks"] = None
        backend["unavailable_reason"] = "not macOS"

        _, reasons = measure._read_probe_output(
            json.dumps(document).encode(),
            require_scan_diagnostics=True,
            require_macos_backend=True,
        )

        self.assertIn("required macOS bulk/fallback counts are unavailable", reasons)

    def test_claim_grade_scan_cross_checks_trace_and_backend_aggregates(self) -> None:
        document = json.loads(self._diagnostic_probe("held"))
        policy = document["scan_diagnostics"]["worker_policy"]
        policy["calibration_entries"] -= 1
        document["summary"]["dirs_read"] += 1

        _, reasons = measure._read_probe_output(
            json.dumps(document).encode(),
            require_scan_diagnostics=True,
            require_macos_backend=True,
        )

        self.assertIn("calibration entry aggregate disagrees with policy windows", reasons)
        self.assertIn("backend counts disagree with reported directories read", reasons)

    def test_generated_phase_claim_is_validated_from_the_trace(self) -> None:
        document = json.loads(self._diagnostic_probe("held"))
        policy = document["scan_diagnostics"]["worker_policy"]
        policy["windows"].append(
            {
                "active_workers": 6,
                "decision": "observe_slow",
                "end_entry_ordinal": 32_768,
                "handoff_backlog": 1,
                "in_flight_directories": 4,
                "observed_chunks": 1,
                "observed_entries": 16_384,
                "observed_work_ns": 655_360_000,
                "ready_directories": 8,
                "requested_workers": None,
                "sequence": 1,
                "start_entry_ordinal": 16_384,
                "work_ns_per_entry": 40_000,
                "work_ns_per_entry_unavailable_reason": None,
            }
        )

        _, reasons = measure._read_probe_output(
            json.dumps(document).encode(),
            require_scan_diagnostics=True,
            require_macos_backend=True,
            expected_phase_profile="fast-prefix-slow-suffix",
        )
        self.assertEqual(reasons, [])

        _, reversed_reasons = measure._read_probe_output(
            json.dumps(document).encode(),
            require_scan_diagnostics=True,
            require_macos_backend=True,
            expected_phase_profile="slow-prefix-fast-suffix",
        )
        self.assertIn(
            "generated-corpus phase hypothesis was not observed",
            "\n".join(reversed_reasons),
        )

    def test_alternating_phase_claim_requires_every_transition(self) -> None:
        complete = {
            "windows": [
                {"decision": "hold"},
                {"decision": "observe_slow"},
                {"decision": "observe_fast"},
                {"decision": "observe_slow"},
            ]
        }
        incomplete = {
            "windows": [
                {"decision": "hold"},
                {"decision": "observe_slow"},
                {"decision": "observe_fast"},
            ]
        }
        self.assertEqual(measure._validate_phase_history(complete, "alternating-regions"), [])
        self.assertTrue(measure._validate_phase_history(incomplete, "alternating-regions"))

    def test_document_jobs_are_catalogued_with_their_cache_seed(self) -> None:
        text = measure.PROBE_JOBS["text-prose"]
        markdown = measure.PROBE_JOBS["markdown-prose"]
        cached = measure.PROBE_JOBS["document-cache-hit"]

        self.assertEqual(text.argv[1], "text-prose")
        self.assertEqual(markdown.argv[1], "markdown-prose")
        self.assertTrue(cached.needs_snapshot)
        self.assertEqual(cached.snapshot_preparation_mode, "document-seed")

    def test_a_parallel_job_reports_no_blocked_time(self) -> None:
        # The failure this guards: getrusage sums CPU across threads, so a parallel walk
        # that keeps four cores busy for 1s reports 4s of CPU against 1s of wall. The
        # old clamp turned that into "blocked_ns: 0" and the ledger printed it under
        # "blocked (I/O+sched)" as though the walk had never waited on the filesystem.
        metrics = {"user_cpu_ns": 3_000_000_000, "system_cpu_ns": 1_000_000_000}
        measure.add_cpu_metrics(metrics, wall_ns=1_000_000_000, parallel_cpu=True)

        self.assertEqual(metrics["cpu_ns"], 4_000_000_000)
        self.assertIsNone(metrics["blocked_ns"])

    def test_a_serial_job_still_reports_blocked_time(self) -> None:
        metrics = {"user_cpu_ns": 200_000_000, "system_cpu_ns": 100_000_000}
        measure.add_cpu_metrics(metrics, wall_ns=1_000_000_000, parallel_cpu=False)

        self.assertEqual(metrics["cpu_ns"], 300_000_000)
        self.assertEqual(metrics["blocked_ns"], 700_000_000)

    def test_cpu_above_wall_withdraws_blocked_time_even_when_undeclared(self) -> None:
        # The safety net for a probe mode that becomes parallel without the catalogue
        # being updated: the sample itself proves more than one thread ran.
        metrics = {"user_cpu_ns": 900_000_000, "system_cpu_ns": 400_000_000}
        measure.add_cpu_metrics(metrics, wall_ns=1_000_000_000, parallel_cpu=False)

        self.assertIsNone(metrics["blocked_ns"])

    def test_every_job_that_walks_or_analyses_declares_parallel_cpu(self) -> None:
        # These are the jobs whose measured process runs the scan worker pool or the
        # content analyzer pool. Left undeclared, each would publish a zero.
        for job_id in (
            "cold-scan-index",
            "cold-scan-producer",
            "warm-revalidate",
            "cold-snapshot-save",
            "content-basic",
            "code-sloc",
            "markdown-prose",
            "text-prose",
        ):
            with self.subTest(job=job_id):
                self.assertTrue(measure.PROBE_JOBS[job_id].parallel_cpu)

    @staticmethod
    def _samples(job: str, variant: str, values, warmup: bool = False):
        return [
            measure.Sample(
                variant=variant,
                job=job,
                ordinal=ordinal,
                warmup=warmup,
                valid=True,
                metrics={"wall_ns": value},
            )
            for ordinal, value in enumerate(values)
        ]

    @staticmethod
    def _qualification_samples(
        job: str,
        variant: str,
        wall_values,
        *,
        rss_multiplier: float = 1.0,
        policy_outcome: str = "fixed",
        policy_decisions=(),
    ):
        samples = []
        for ordinal, wall in enumerate(wall_values):
            metrics = {
                "wall_ns": wall,
                "component_ns": wall,
                "cpu_ns": 1_000,
                "user_cpu_ns": 500,
                "system_cpu_ns": 500,
                "peak_rss_bytes": round(1_000 * rss_multiplier),
                "major_faults": 0,
                "minor_faults": 100,
                "voluntary_context_switches": 100,
                "involuntary_context_switches": 100,
            }
            probe = {}
            if job == "adaptive-scan-index":
                probe = {
                    "scan_diagnostics": {
                        "worker_policy": {
                            "outcome": policy_outcome,
                            "windows": [{"decision": decision} for decision in policy_decisions],
                        }
                    }
                }
            samples.append(
                measure.Sample(
                    variant=variant,
                    job=job,
                    ordinal=ordinal,
                    warmup=False,
                    valid=True,
                    metrics=metrics,
                    probe=probe,
                )
            )
        return samples

    def test_noninferiority_contract_distinguishes_all_four_outcomes(self) -> None:
        self.assertEqual(
            measure._noninferiority_classification(-5.0, [-6.0, -4.0]),
            "superior",
        )
        self.assertEqual(
            measure._noninferiority_classification(1.0, [-1.0, 2.0]),
            "noninferior",
        )
        self.assertEqual(
            measure._noninferiority_classification(5.0, [4.0, 6.0]),
            "inferior",
        )
        self.assertEqual(
            measure._noninferiority_classification(3.0, [2.0, 4.0]),
            "inconclusive",
        )

    def test_resource_regression_rejects_a_faster_candidate(self) -> None:
        samples = self._qualification_samples("job", "control", [1_000] * 12)
        samples += self._qualification_samples("job", "candidate", [900] * 12, rss_multiplier=1.10)

        comparison = measure.paired_comparison(
            samples,
            job="job",
            control="control",
            candidate="candidate",
            campaign_stage="held-out",
            required_pairs=12,
        )

        self.assertEqual(comparison["metrics"]["wall_ns"]["noninferiority"], "superior")
        self.assertEqual(comparison["qualification"]["classification"], "inferior")
        self.assertFalse(comparison["qualification"]["confirmable"])
        self.assertIn("peak_rss_bytes exceeds", "\n".join(comparison["qualification"]["reasons"]))

    def test_missing_resource_field_makes_qualification_inconclusive(self) -> None:
        samples = self._samples("job", "control", [1_000] * 12)
        samples += self._samples("job", "candidate", [950] * 12)

        comparison = measure.paired_comparison(
            samples, job="job", control="control", candidate="candidate"
        )

        self.assertEqual(comparison["qualification"]["classification"], "inconclusive")
        self.assertIn("missing", "\n".join(comparison["qualification"]["reasons"]))

    def test_mixed_order_sensitivity_makes_adaptive_qualification_inconclusive(self) -> None:
        control = self._qualification_samples(
            "adaptive-scan-index",
            "control",
            [1_000] * 12,
            policy_outcome="held",
            policy_decisions=("hold", "observe_fast"),
        )
        # One identical-tree trial records the completion-order false negative.
        control[5].probe["scan_diagnostics"]["worker_policy"]["windows"].append(
            {"decision": "observe_slow"}
        )
        candidate = self._qualification_samples(
            "adaptive-scan-index",
            "candidate",
            [950] * 12,
            policy_outcome="fixed",
        )

        comparison = measure.paired_comparison(
            control + candidate,
            job="adaptive-scan-index",
            control="control",
            candidate="candidate",
            campaign_stage="held-out",
        )

        self.assertFalse(comparison["policy_stability"]["stable"])
        control_history = comparison["policy_stability"]["variants"]["control"]
        self.assertEqual(control_history["harmful_histories"], 0)
        self.assertEqual(control_history["order_sensitive_histories"], 1)
        self.assertEqual(comparison["qualification"]["classification"], "inconclusive")
        self.assertIn(
            "adaptive policy history is missing, structurally harmful, or unstable",
            "\n".join(comparison["qualification"]["reasons"]),
        )

    def test_reproducible_late_slow_history_is_sensitive_not_intrinsically_harmful(self) -> None:
        control = self._qualification_samples(
            "adaptive-scan-index",
            "control",
            [1_000] * 12,
            policy_outcome="held",
            policy_decisions=("hold", "observe_slow"),
        )
        candidate = self._qualification_samples(
            "adaptive-scan-index", "candidate", [990] * 12, policy_outcome="fixed"
        )

        comparison = measure.paired_comparison(
            control + candidate,
            job="adaptive-scan-index",
            control="control",
            candidate="candidate",
            campaign_stage="held-out",
            required_pairs=12,
        )

        history = comparison["policy_stability"]["variants"]["control"]
        self.assertTrue(comparison["policy_stability"]["stable"])
        self.assertEqual(history["harmful_histories"], 0)
        self.assertEqual(history["order_sensitive_histories"], 12)
        self.assertTrue(comparison["qualification"]["confirmable"])

    def test_held_out_qualification_requires_every_preregistered_pair_and_trace(self) -> None:
        control = self._qualification_samples("adaptive-scan-index", "control", [1_000] * 12)
        candidate = self._qualification_samples("adaptive-scan-index", "candidate", [950] * 12)
        candidate[-1].valid = False
        candidate[-1].reasons.append("fixture invalidation")

        comparison = measure.paired_comparison(
            control + candidate,
            job="adaptive-scan-index",
            control="control",
            candidate="candidate",
            campaign_stage="held-out",
            required_pairs=12,
        )

        self.assertFalse(comparison["policy_stability"]["stable"])
        self.assertEqual(
            comparison["policy_stability"]["variants"]["candidate"]["missing_histories"],
            1,
        )
        self.assertFalse(comparison["qualification"]["confirmable"])
        self.assertIn(
            "every fixed-N pair",
            "\n".join(comparison["qualification"]["reasons"]),
        )

    def test_complete_stable_held_out_evidence_is_confirmable(self) -> None:
        samples = self._qualification_samples("adaptive-scan-index", "control", [1_000] * 12)
        samples += self._qualification_samples("adaptive-scan-index", "candidate", [990] * 12)

        comparison = measure.paired_comparison(
            samples,
            job="adaptive-scan-index",
            control="control",
            candidate="candidate",
            campaign_stage="held-out",
            required_pairs=12,
        )

        self.assertTrue(comparison["policy_stability"]["stable"])
        self.assertEqual(comparison["qualification"]["classification"], "noninferior")
        self.assertTrue(comparison["qualification"]["confirmable"])

    def test_distribution_reports_median_and_spread(self) -> None:
        summary = measure.distribution([10, 20, 30, 40, 100])
        self.assertEqual(summary["count"], 5)
        self.assertEqual(summary["median"], 30)
        self.assertEqual(summary["min"], 10)
        self.assertEqual(summary["max"], 100)
        # MAD ignores the outlier that inflates stdev.
        self.assertLess(summary["mad"], summary["stdev"])

    def test_distribution_of_nothing_is_none(self) -> None:
        self.assertIsNone(measure.distribution([]))

    def test_a_real_improvement_is_significant(self) -> None:
        samples = self._samples("job", "control", [1000] * 12) + self._samples(
            "job", "candidate", [700] * 12
        )
        comparison = measure.paired_comparison(
            samples, job="job", control="control", candidate="candidate"
        )
        entry = comparison["metrics"]["wall_ns"]
        self.assertAlmostEqual(entry["median_change_pct"], -30.0, places=3)
        self.assertTrue(entry["significant"])
        self.assertTrue(ledger.verdict(comparison)["accepted"])

    def test_pure_noise_is_not_significant(self) -> None:
        control = [1000, 1100, 900, 1050, 950, 1000, 1080, 920, 1010, 990, 1040, 960]
        candidate = [1010, 1090, 910, 1040, 960, 990, 1070, 930, 1000, 1000, 1030, 970]
        samples = self._samples("job", "control", control) + self._samples(
            "job", "candidate", candidate
        )
        comparison = measure.paired_comparison(
            samples, job="job", control="control", candidate="candidate"
        )
        decision = ledger.verdict(comparison)
        self.assertFalse(decision["accepted"])

    def test_a_small_but_certain_win_is_still_rejected(self) -> None:
        # 1% with no variance at all: real, reproducible, and not worth complexity.
        samples = self._samples("job", "control", [1000] * 12) + self._samples(
            "job", "candidate", [990] * 12
        )
        comparison = measure.paired_comparison(
            samples, job="job", control="control", candidate="candidate"
        )
        decision = ledger.verdict(comparison)
        self.assertFalse(decision["accepted"])
        self.assertIn("under the", decision["reason"])

    def test_a_regression_is_never_accepted(self) -> None:
        samples = self._samples("job", "control", [1000] * 12) + self._samples(
            "job", "candidate", [1400] * 12
        )
        comparison = measure.paired_comparison(
            samples, job="job", control="control", candidate="candidate"
        )
        self.assertFalse(ledger.verdict(comparison)["accepted"])

    def test_warmups_and_invalid_samples_never_reach_the_comparison(self) -> None:
        samples = self._samples("job", "control", [1000] * 12)
        samples += self._samples("job", "candidate", [700] * 12)
        # A warmup that would look like a huge win, and an invalid one likewise.
        samples += self._samples("job", "candidate", [1], warmup=True)
        rogue = measure.Sample(
            variant="candidate",
            job="job",
            ordinal=99,
            warmup=False,
            valid=False,
            metrics={"wall_ns": 1},
        )
        samples.append(rogue)
        comparison = measure.paired_comparison(
            samples, job="job", control="control", candidate="candidate"
        )
        self.assertEqual(comparison["metrics"]["wall_ns"]["pairs"], 12)

    def test_too_few_pairs_yields_no_verdict(self) -> None:
        samples = self._samples("job", "control", [1000, 900]) + self._samples(
            "job", "candidate", [500, 450]
        )
        comparison = measure.paired_comparison(
            samples, job="job", control="control", candidate="candidate"
        )
        self.assertIsNone(comparison["metrics"]["wall_ns"])
        self.assertFalse(ledger.verdict(comparison)["accepted"])

    def test_the_bootstrap_is_deterministic(self) -> None:
        values = [-0.10, -0.12, -0.05, -0.20, -0.08, -0.11, -0.09, -0.15]
        first = measure._bootstrap_median_interval(values)
        second = measure._bootstrap_median_interval(values)
        self.assertEqual(first, second)
        self.assertLess(first[0], statistics.median(values) + 1e-9)


class HostRegimeTests(unittest.TestCase):
    def test_spawn_applies_only_explicit_environment_overrides(self) -> None:
        result = measure._spawn(
            [
                sys.executable,
                "-c",
                "import os; print(os.environ.get('FDU_FIXTURE', 'missing'))",
            ],
            timeout_seconds=5,
            environment_overrides={"FDU_FIXTURE": "present"},
        )

        self.assertEqual(result["exit_code"], 0)
        self.assertEqual(result["stdout"], b"present\n")

    def test_held_out_measurement_rejects_an_uncontrolled_host(self) -> None:
        with self.assertRaisesRegex(measure.MeasureError, "held-out evidence"):
            measure.run(
                root=Path("/does/not-matter"),
                label="fixture",
                variants=[measure.Variant("control", Path("/bin/true"))],
                jobs=[measure.PROBE_JOBS["cold-scan-index"]],
                trials=1,
                warmups=0,
                scratch=Path("/does/not-matter-either"),
                campaign_stage="held-out",
                host_regime="uncontrolled",
            )

    def test_quiet_host_rejects_pressure_at_either_sample_boundary(self) -> None:
        regime = measure.HostRegime(name="quiet", initial={})

        reasons = measure._host_pressure_reasons(
            regime,
            {
                "system": "Darwin",
                "cpu_busy_pct": measure.QUIET_MAX_CPU_BUSY_PCT + 0.01,
                "power_source": "AC",
                "thermal_pressure": "normal",
            },
            {
                "system": "Darwin",
                "cpu_busy_pct": 0.01,
                "power_source": "AC",
                "thermal_pressure": "normal",
            },
        )

        self.assertEqual(len(reasons), 1)
        self.assertIn("before the sample", reasons[0])

    def test_controlled_host_rejects_load_beyond_the_predeclared_envelope(self) -> None:
        regime = measure.HostRegime(name="controlled-interactive", initial={}, load_workers=2)
        within = {
            "controlled_load_alive": True,
            "cpu_busy_pct": 40.0,
            "logical_cpu_count": 10,
            "power_source": "AC",
            "system": "Darwin",
            "thermal_pressure": "normal",
        }
        excess = {**within, "cpu_busy_pct": 46.0}

        self.assertEqual(measure._host_pressure_reasons(regime, within, within), [])
        reasons = measure._host_pressure_reasons(regime, within, excess)
        self.assertEqual(len(reasons), 1)
        self.assertIn("synthetic-load envelope", reasons[0])

    def test_controlled_load_lifecycle_is_observable(self) -> None:
        quiet = {
            "controlled_load_alive": None,
            "cpu_busy_pct": 0.0,
            "load_1m": 0.0,
            "load_1m_per_cpu": 0.0,
            "load_5m": 0.0,
            "load_15m": 0.0,
            "system": "Darwin",
            "unavailable_reason": None,
        }
        with mock.patch("benchmarks.realtree.measure._host_pressure_snapshot", return_value=quiet):
            with measure._host_regime("controlled-interactive", 1) as regime:
                process = regime.process
                self.assertEqual(regime.load_workers, 1)
                self.assertTrue(regime.load_alive())

        self.assertIsNotNone(process)
        self.assertIsNotNone(process.poll())

    def test_darwin_cpu_pressure_uses_a_short_counter_delta(self) -> None:
        with (
            mock.patch.object(
                measure,
                "_darwin_cpu_ticks",
                side_effect=[((100, 50, 850, 0), None), ((125, 75, 900, 0), None)],
            ),
            mock.patch.object(measure.time, "sleep") as sleep,
        ):
            busy, reason = measure._darwin_cpu_busy_pct()

        self.assertEqual(busy, 50.0)
        self.assertIsNone(reason)
        sleep.assert_called_once_with(measure.HOST_CPU_BOUNDARY_INTERVAL_SECONDS)


class ProfileParsingTests(unittest.TestCase):
    SAMPLE = """
Call graph:
    100 start  (in dyld) + 1  [0x1]
      100 main  (in probe) + 2  [0x2]
        70 walk  (in probe) + 3  [0x3]
        + 50 fstatat  (in libsystem) + 4  [0x4]
        + 10 malloc  (in libsystem) + 5  [0x5]
        30 build_index  (in probe) + 6  [0x6]
"""

    def test_self_time_excludes_children(self) -> None:
        parsed = profile.parse(self.SAMPLE)
        by_symbol = {entry["symbol"]: entry["samples"] for entry in parsed["self_time"]}
        # walk had 70 inclusive with 60 in children, so 10 is its own.
        self.assertEqual(by_symbol["walk"], 10)
        self.assertEqual(by_symbol["fstatat"], 50)
        self.assertEqual(by_symbol["build_index"], 30)
        # start and main are pure pass-through frames and hold no self time.
        self.assertNotIn("start", by_symbol)
        self.assertEqual(parsed["total_samples"], 100)

    def test_percentages_sum_to_one_hundred(self) -> None:
        parsed = profile.parse(self.SAMPLE)
        self.assertAlmostEqual(
            sum(entry["percent"] for entry in parsed["self_time"]), 100.0, places=1
        )

    def test_counter_report_is_structured_without_raw_text(self) -> None:
        parsed = profile.parse_counter_report(
            "[filesystem operations]\n"
            "  directory opens             17\n\n"
            "[adaptive scan policy]\n"
            "  reserve expansions           1\n"
        )

        self.assertEqual(parsed["groups"]["filesystem operations"]["directory opens"], 17)
        self.assertEqual(parsed["groups"]["adaptive scan policy"]["reserve expansions"], 1)
        self.assertIsNone(parsed["unavailable_reason"])

    def test_profile_command_redacts_subject_and_snapshot_paths(self) -> None:
        shown = profile._redacted_command(
            [
                "/tmp/bin/perf_probe",
                "scan-index",
                "--root",
                "/private/subject",
                "--snapshot",
                "/private/snapshot.fdu",
                "--repeat",
                "4",
            ],
            Path("/tmp/bin/perf_probe"),
        )

        self.assertEqual(
            shown,
            [
                "{binary}",
                "scan-index",
                "--root",
                "{root}",
                "--snapshot",
                "{snapshot}",
                "--repeat",
                "4",
            ],
        )

    def test_layers_partition_the_samples(self) -> None:
        parsed = profile.parse(self.SAMPLE)
        self.assertEqual(
            sum(layer["samples"] for layer in parsed["by_layer"]),
            parsed["total_samples"],
        )

    def test_empty_input_does_not_divide_by_zero(self) -> None:
        parsed = profile.parse("no frames here")
        self.assertEqual(parsed["self_time"], [])
        self.assertEqual(parsed["total_samples"], 1)


class ScheduleTests(unittest.TestCase):
    def _variants(self, count: int):
        return [measure.Variant(name=f"v{index}", path=Path(__file__)) for index in range(count)]

    def test_every_variant_runs_at_every_ordinal(self) -> None:
        variants = self._variants(3)
        jobs = [measure.PROBE_JOBS["cold-scan-index"]]
        schedule = measure._interleave(variants, jobs, trials=5, warmups=2)
        self.assertEqual(len(schedule), 3 * 7)
        for ordinal in range(-2, 5):
            at_ordinal = {
                variant.name for variant, _job, position, _warmup in schedule if position == ordinal
            }
            self.assertEqual(at_ordinal, {"v0", "v1", "v2"})

    def test_variant_order_alternates_so_neither_is_always_first(self) -> None:
        variants = self._variants(2)
        jobs = [measure.PROBE_JOBS["cold-scan-index"]]
        schedule = measure._interleave(variants, jobs, trials=4, warmups=0)
        first_at = [
            [variant.name for variant, _job, position, _warmup in schedule if position == ordinal][
                0
            ]
            for ordinal in range(4)
        ]
        self.assertEqual(len(set(first_at)), 2, "one variant always ran first")

    def test_warmups_are_flagged(self) -> None:
        schedule = measure._interleave(
            self._variants(1), [measure.PROBE_JOBS["cold-scan-index"]], trials=3, warmups=2
        )
        self.assertEqual(sum(1 for *_rest, warmup in schedule if warmup), 2)


class ArgumentExpansionTests(unittest.TestCase):
    def test_placeholders_expand_and_extra_flags_append(self) -> None:
        expanded = measure._expand(
            ("{binary}", "scan-index", "--root", "{root}"),
            binary=Path("/bin/probe"),
            root=Path("/tmp/tree"),
            snapshot=None,
            extra=["--threads", "4"],
        )
        self.assertEqual(
            expanded, ["/bin/probe", "scan-index", "--root", "/tmp/tree", "--threads", "4"]
        )

    def test_a_missing_snapshot_is_an_error_not_an_empty_string(self) -> None:
        with self.assertRaises(measure.MeasureError):
            measure._expand(
                ("{binary}", "--snapshot", "{snapshot}"),
                binary=Path("/bin/probe"),
                root=Path("/tmp/tree"),
                snapshot=None,
            )

    def test_an_unknown_placeholder_is_rejected(self) -> None:
        with self.assertRaises(measure.MeasureError):
            measure._expand(
                ("{binary}", "{nonsense}"),
                binary=Path("/bin/probe"),
                root=Path("/tmp/tree"),
                snapshot=None,
            )


if __name__ == "__main__":
    unittest.main()
