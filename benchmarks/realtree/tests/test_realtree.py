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
from types import SimpleNamespace
from unittest import mock

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
            self.assertNotIn(
                secret, serialized, f"fingerprint leaked {secret!r} from the tree"
            )

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


class StatisticsTests(unittest.TestCase):
    def test_document_jobs_are_catalogued_with_their_cache_seed(self) -> None:
        text = measure.PROBE_JOBS["text-prose"]
        markdown = measure.PROBE_JOBS["markdown-prose"]
        cached = measure.PROBE_JOBS["document-cache-hit"]

        self.assertEqual(text.argv[1], "text-prose")
        self.assertEqual(markdown.argv[1], "markdown-prose")
        self.assertTrue(cached.needs_snapshot)
        self.assertEqual(cached.snapshot_preparation_mode, "document-seed")

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
