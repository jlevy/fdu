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
import tempfile
import unittest
from pathlib import Path

from benchmarks.realtree import compat, evidence, ledger, measure, profile, scale, tree


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
        self.assertEqual(document["max_depth"], 3)

    def test_fingerprint_discloses_no_path_or_name(self) -> None:
        document = tree.fingerprint(self.root, label="fixture")
        serialized = json.dumps(document)
        for secret in ("alpha", "beta", "gamma", "nested", "deeper", str(self.root)):
            self.assertNotIn(
                secret, serialized, f"fingerprint leaked {secret!r} from the tree"
            )

    def test_root_id_identifies_without_disclosing(self) -> None:
        document = tree.fingerprint(self.root, label="fixture")
        self.assertEqual(len(document["root_id"]), 64)
        self.assertEqual(document["root_id"], tree.root_id(self.root))
        self.assertNotEqual(document["root_id"], tree.root_id(self.scratch))

    def test_identical_observations_compare_equal(self) -> None:
        first = tree.fingerprint(self.root, label="fixture")
        second = tree.fingerprint(self.root, label="fixture")
        self.assertEqual(tree.compare(second, first), [])

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
            " ".join(tree.compare(tree.fingerprint(other, label="x"), tree.fingerprint(self.root, label="x"))),
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
            "engine_digest": document["engine_digest"],
            "rollup_digest": document["rollup_digest"],
        }
        self.assertIsNone(tree.probe_agrees(document, agreeing, mode="scan-index"))

        for field in (
            "entries",
            "files",
            "apparent_bytes",
            "engine_digest",
            "rollup_digest",
        ):
            wrong = dict(agreeing)
            wrong[field] = (
                "wrong" if field in {"engine_digest", "rollup_digest"} else 0
            )
            self.assertIsNotNone(
                tree.probe_agrees(document, wrong, mode="scan-index"),
                f"a probe reporting the wrong {field} was accepted",
            )

        producer = dict(agreeing)
        producer.pop("rollup_digest")
        self.assertIsNone(
            tree.probe_agrees(document, producer, mode="scan-producer")
        )
        self.assertIsNotNone(
            tree.probe_agrees(document, producer, mode="scan-index")
        )

    def test_missing_summary_is_a_disagreement(self) -> None:
        document = tree.fingerprint(self.root, label="fixture")
        self.assertIsNotNone(tree.probe_agrees(document, None))


class StatisticsTests(unittest.TestCase):
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
        self.assertEqual(entry["direction"], "improvement")
        self.assertTrue(entry["ci_excludes_zero"])
        self.assertTrue(entry["significant_improvement"])
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
        entry = comparison["metrics"]["wall_ns"]
        self.assertEqual(entry["direction"], "regression")
        self.assertTrue(entry["ci_excludes_zero"])
        self.assertFalse(entry["significant_improvement"])
        self.assertTrue(entry["significant"])
        decision = ledger.verdict(comparison)
        self.assertFalse(decision["accepted"])
        self.assertIn("statistically significant regression", decision["reason"])

    def test_parallel_process_cpu_does_not_invent_blocked_time(self) -> None:
        resources = {"user_cpu_ns": 80, "system_cpu_ns": 40}
        parallel = measure._process_metrics(
            resources,
            wall_ns=100,
            process_cpu_can_exceed_wall=True,
        )
        serial = measure._process_metrics(
            resources,
            wall_ns=200,
            process_cpu_can_exceed_wall=False,
        )
        self.assertEqual(parallel["cpu_ns"], 120)
        self.assertIsNone(parallel["blocked_ns"])
        self.assertEqual(serial["blocked_ns"], 80)

    def test_warmups_and_invalid_samples_never_reach_the_comparison(self) -> None:
        samples = self._samples("job", "control", [1000] * 12)
        samples += self._samples("job", "candidate", [700] * 12)
        # A warmup that would look like a huge win, and an invalid one likewise.
        samples += self._samples("job", "candidate", [1], warmup=True)
        rogue = measure.Sample(
            variant="candidate", job="job", ordinal=99, warmup=False, valid=False,
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
        return [
            measure.Variant(name=f"v{index}", path=Path(__file__))
            for index in range(count)
        ]

    def test_every_variant_runs_at_every_ordinal(self) -> None:
        variants = self._variants(3)
        jobs = [measure.PROBE_JOBS["cold-scan-index"]]
        schedule = measure._interleave(variants, jobs, trials=5, warmups=2)
        self.assertEqual(len(schedule), 3 * 7)
        for ordinal in range(-2, 5):
            at_ordinal = {
                variant.name for variant, _job, position, _warmup in schedule
                if position == ordinal
            }
            self.assertEqual(at_ordinal, {"v0", "v1", "v2"})

    def test_variant_order_alternates_so_neither_is_always_first(self) -> None:
        variants = self._variants(2)
        jobs = [measure.PROBE_JOBS["cold-scan-index"]]
        schedule = measure._interleave(variants, jobs, trials=4, warmups=0)
        first_at = [
            [variant.name for variant, _job, position, _warmup in schedule if position == ordinal][0]
            for ordinal in range(4)
        ]
        self.assertEqual(len(set(first_at)), 2, "one variant always ran first")

    def test_warmups_are_flagged(self) -> None:
        schedule = measure._interleave(
            self._variants(1), [measure.PROBE_JOBS["cold-scan-index"]], trials=3, warmups=2
        )
        self.assertEqual(sum(1 for *_rest, warmup in schedule if warmup), 2)

    def test_exact_schedule_has_a_stable_content_digest(self) -> None:
        variants = self._variants(2)
        jobs = [measure.PROBE_JOBS["cold-scan-index"]]
        schedule = measure._interleave(variants, jobs, trials=4, warmups=1)
        self.assertEqual(
            measure._schedule_sha256(schedule), measure._schedule_sha256(schedule)
        )
        self.assertNotEqual(
            measure._schedule_sha256(schedule),
            measure._schedule_sha256(list(reversed(schedule))),
        )


class EvidenceArchiveTests(unittest.TestCase):
    def setUp(self) -> None:
        self.scratch = Path(tempfile.mkdtemp(prefix="fdu-evidence-test-"))
        self.addCleanup(shutil.rmtree, self.scratch, ignore_errors=True)

    def test_archive_redacts_operator_tree_identity_but_preserves_samples(self) -> None:
        document = {
            "schema": measure.RUN_SCHEMA,
            "tree": {
                "label": "private-checkout",
                "root_id": "a" * 64,
                "engine_digest": "b" * 64,
            },
            "samples": [{"ordinal": 3, "metrics": {"wall_ns": 42}}],
        }
        destination = self.scratch / "run.json"
        archived, digest = evidence.archive_run(
            document,
            destination=destination,
            tree_label="reference-tree-60k",
        )
        self.assertEqual(archived["tree"]["label"], "reference-tree-60k")
        self.assertNotEqual(archived["tree"]["root_id"], document["tree"]["root_id"])
        self.assertEqual(archived["samples"], document["samples"])
        self.assertEqual(len(digest), 64)

    def test_archive_rejects_an_absolute_path_anywhere(self) -> None:
        document = {
            "tree": {"engine_digest": "b" * 64},
            "note": "/private/reference-tree",
        }
        with self.assertRaises(evidence.EvidenceError):
            evidence.archive_run(
                document,
                destination=self.scratch / "run.json",
                tree_label="reference-tree",
            )

    def test_archive_rejects_an_embedded_absolute_path(self) -> None:
        document = {
            "tree": {"engine_digest": "b" * 64},
            "note": "probe failed at /private/reference-tree/file.txt",
        }
        with self.assertRaises(evidence.EvidenceError):
            evidence.archive_run(
                document,
                destination=self.scratch / "run.json",
                tree_label="reference-tree",
            )

    def test_archive_never_overwrites_different_evidence(self) -> None:
        destination = self.scratch / "run.json"
        first = {"tree": {"engine_digest": "a" * 64}, "samples": []}
        second = {"tree": {"engine_digest": "b" * 64}, "samples": []}
        evidence.archive_run(first, destination=destination, tree_label="reference-tree")
        with self.assertRaisesRegex(evidence.EvidenceError, "immutable evidence"):
            evidence.archive_run(
                second, destination=destination, tree_label="reference-tree"
            )


class BinaryProvenanceTests(unittest.TestCase):
    def test_claim_manifest_is_normalized_into_binary_identity(self) -> None:
        provenance = {
            "schema": measure.BINARY_PROVENANCE_SCHEMA,
            "engine_revision": "a" * 40,
            "harness_revision": "b" * 40,
            "harness_sha256": "c" * 64,
            "target": "aarch64-apple-darwin",
            "build_profile": "release",
            "features": ["watch", "watch"],
            "build_command": "cargo build --release --example perf_probe",
        }
        normalized = measure._validated_provenance(provenance)
        self.assertEqual(normalized["features"], ["watch"])
        self.assertEqual(normalized["engine_revision"], "a" * 40)

    def test_incomplete_claim_manifest_is_rejected(self) -> None:
        with self.assertRaises(measure.MeasureError):
            measure._validated_provenance(
                {"schema": measure.BINARY_PROVENANCE_SCHEMA}
            )

    def test_embedded_absolute_build_path_is_rejected(self) -> None:
        provenance = {
            "schema": measure.BINARY_PROVENANCE_SCHEMA,
            "engine_revision": "a" * 40,
            "harness_revision": "b" * 40,
            "harness_sha256": "c" * 64,
            "target": "aarch64-apple-darwin",
            "build_profile": "release",
            "features": [],
            "build_command": "cargo build --manifest-path /private/repo/Cargo.toml",
        }
        with self.assertRaisesRegex(measure.MeasureError, "path-redacted"):
            measure._validated_provenance(provenance)

    def test_variant_binary_is_frozen_across_a_run(self) -> None:
        scratch = Path(tempfile.mkdtemp(prefix="fdu-variant-freeze-test-"))
        self.addCleanup(shutil.rmtree, scratch, ignore_errors=True)
        binary = scratch / "probe"
        binary.write_bytes(b"before")
        variant = measure.Variant(name="candidate", path=binary)
        frozen = measure._freeze_variant_identities([variant])
        binary.write_bytes(b"after")
        with self.assertRaisesRegex(measure.MeasureError, "changed"):
            measure._assert_variants_unchanged([variant], frozen)

    def test_variant_name_must_be_path_safe_and_unique(self) -> None:
        variant = measure.Variant(name="../candidate", path=Path(__file__))
        with self.assertRaisesRegex(measure.MeasureError, "path-safe"):
            measure._freeze_variant_identities([variant])


class SnapshotScaleTests(unittest.TestCase):
    def test_oracle_covers_the_full_rollup_digest(self) -> None:
        manifest = {
            "counts": {"total": 3},
            "sizes": {"entry_apparent_bytes": 7},
            "oracle": {"engine_digest": "a" * 64, "rollup_digest": "b" * 64},
        }
        probe = {
            "mode": "snapshot-load",
            "source": "snapshot",
            "summary": {
                "entries": 3,
                "index_len": 3,
                "apparent_bytes": 7,
                "engine_digest": "a" * 64,
                "rollup_digest": "c" * 64,
            },
        }
        reasons = scale._oracle_reasons(manifest, probe, mode="snapshot-load")
        self.assertEqual(len(reasons), 1)
        self.assertIn("rollup_digest", reasons[0])


class CompatibilityProbeTests(unittest.TestCase):
    def test_generator_removes_only_the_threads_parser_arm(self) -> None:
        scratch = Path(tempfile.mkdtemp(prefix="fdu-compat-probe-test-"))
        self.addCleanup(shutil.rmtree, scratch, ignore_errors=True)
        source = scratch / "source.rs"
        destination = scratch / "generated.rs"
        source.write_text(
            "before\n"
            "                Some(\"--threads\") => {\n"
            "                    scan.threads = Some(next_usize(&mut arguments, \"--threads\")?);\n"
            "                }\n"
            "after\n",
            encoding="utf-8",
        )
        compat.write_pr2_base_probe(source, destination)
        self.assertEqual(destination.read_text(encoding="utf-8"), "before\nafter\n")

    def test_generator_fails_when_the_expected_source_drifted(self) -> None:
        scratch = Path(tempfile.mkdtemp(prefix="fdu-compat-probe-test-"))
        self.addCleanup(shutil.rmtree, scratch, ignore_errors=True)
        source = scratch / "source.rs"
        source.write_text("different\n", encoding="utf-8")
        with self.assertRaises(compat.CompatibilityError):
            compat.write_pr2_base_probe(source, scratch / "generated.rs")


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
