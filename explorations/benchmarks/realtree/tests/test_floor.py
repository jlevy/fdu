"""The floor scoreboard: what it reconciles, what it refuses, and what it flags.

Every case here is a way the scoreboard could print a plausible number that means
something other than what its column heading says.
"""

from __future__ import annotations

import contextlib
import io
import json
import re
import unittest
from pathlib import Path
from unittest import mock

from benchmarks.realtree import floor

BINARIES = {
    "parfloor": Path("/b/parfloor"),
    "arena_spike": Path("/b/arena_spike"),
    "probe": Path("/b/perf_probe"),
}


class ReadsInstrumentOutput(unittest.TestCase):
    def test_parfloor_tallies_and_timer(self):
        line = json.dumps({
            "variant": "stat", "threads": 4, "dirs": 7842, "files": 68134,
            "other": 8559, "bytes": 3573889208, "allocated": 3751350272,
            "wall_ns": 39_260_000,
        })
        parsed = floor.INSTRUMENTS["parfloor-stat"].read(line)
        self.assertEqual(parsed["elapsed_ns"], 39_260_000)
        self.assertEqual(parsed["tallies"]["dirs"], 7842)
        self.assertEqual(parsed["tallies"]["apparent_bytes"], 3573889208)

    def test_arena_spike_milliseconds_become_nanoseconds(self):
        line = json.dumps({
            "files": 68134, "dirs": 7842, "bytes": 3573889208,
            "allocated": 3751350272, "wall_ms": 153.75,
        })
        parsed = floor.INSTRUMENTS["arena-spike"].read(line)
        self.assertEqual(parsed["elapsed_ns"], 153_750_000)

    def test_probe_timer_is_the_component_not_the_spawn(self):
        """The probe's own timer, not its wall: process startup is harness cost."""
        line = json.dumps({
            "component_ns": 62_210_000, "mode": "summary",
            "summary": {"dirs": 7842, "files": 68134,
                        "apparent_bytes": 3573889208, "allocated_bytes": 3751350272},
        })
        parsed = floor.INSTRUMENTS["aggregate"].read(line)
        self.assertEqual(parsed["elapsed_ns"], 62_210_000)


class ReconcilesDefinitionalDifferences(unittest.TestCase):
    """An index retains the root as an entry; a tallying walk does not count it.

    The offset is applied before the oracle compares, so the comparison stays exact
    rather than being given slack that would hide a real disagreement.
    """

    def test_index_tier_drops_the_root_directory(self):
        line = json.dumps({
            "component_ns": 110_600_000,
            "summary": {"dirs": 7843, "files": 68134,
                        "apparent_bytes": 3573889208, "allocated_bytes": 3751350272},
        })
        parsed = floor.INSTRUMENTS["index"].read(line)
        self.assertEqual(parsed["tallies"]["dirs"], 7842)

    def test_the_reconciliation_says_why(self):
        self.assertIn("root", floor.INSTRUMENTS["index"].tally_notes)


class EveryInstrumentRunsTheSamePool(unittest.TestCase):
    """A floor is a lower bound for a parallel walker only at the pool size it runs.

    Review FLOOR-1: the probe tiers were given no `--threads`, so fdu ran its automatic
    pool while the floor ran one worker per CPU, and every ratio was biased wherever
    the two differ.
    """

    def test_every_instrument_is_handed_the_worker_count(self):
        for instrument in floor.INSTRUMENTS.values():
            with self.subTest(instrument=instrument.id):
                argv = instrument.command(binaries=BINARIES, root=Path("/r"), workers=7)
                self.assertIn("7", argv)

    def test_the_probe_tiers_pin_fdus_pool_rather_than_its_automatic_policy(self):
        for name in ("aggregate", "index"):
            with self.subTest(instrument=name):
                argv = floor.INSTRUMENTS[name].command(binaries=BINARIES, root=Path("/r"), workers=7)
                self.assertEqual(argv[argv.index("--threads") + 1], "7")

    def test_the_default_counts_the_cpus_this_process_may_run_on(self):
        with mock.patch.object(floor.os, "process_cpu_count", create=True, return_value=3), \
                mock.patch.object(floor.os, "cpu_count", return_value=16):
            self.assertEqual(floor.default_workers(), 3)

    def test_without_process_cpu_count_the_affinity_mask_decides(self):
        """Python before 3.13, and the case the review names: a container whose
        affinity mask is narrower than the host's CPU count."""
        with mock.patch.object(floor.os, "process_cpu_count", None, create=True), \
                mock.patch.object(floor.os, "sched_getaffinity", create=True, return_value={0, 1}), \
                mock.patch.object(floor.os, "cpu_count", return_value=16):
            self.assertEqual(floor.default_workers(), 2)

    def test_the_default_never_exceeds_what_fdu_will_actually_run(self):
        with mock.patch.object(floor.os, "process_cpu_count", create=True, return_value=96):
            self.assertEqual(floor.default_workers(), floor.MAX_WORKERS)

    def test_a_pool_fdu_would_silently_clamp_is_refused(self):
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr), \
                mock.patch.object(floor, "run", side_effect=AssertionError("must refuse first")):
            status = floor.main(["--subject", "t=/", "--workers", str(floor.MAX_WORKERS + 1)])
        self.assertEqual(status, 2)
        self.assertIn(str(floor.MAX_WORKERS), stderr.getvalue())

    def test_the_cap_is_fdus_own_clamp(self):
        """Read from the engine, so the two cannot drift apart unnoticed."""
        scan = (floor.PROJECT_ROOT / "crates" / "fdu-core" / "src" / "scan.rs").read_text()
        match = re.search(r"const MAX_SCAN_THREADS: usize = (\d+);", scan)
        self.assertIsNotNone(match)
        self.assertEqual(int(match.group(1)), floor.MAX_WORKERS)

    def test_the_table_names_the_regime_it_measured(self):
        document = {
            "host": {"system": "Linux", "machine": "x86_64", "logical_cpu_count": 8},
            "workers": 4, "recorded_at": "t", "commit": "c", "host_regime": "quiet",
            "trials": 30, "warmups": 3, "subjects": [],
        }
        self.assertIn("fixed pool of 4 workers", floor.render(document))


class RefusesRatherThanSubstituting(unittest.TestCase):
    def test_non_linux_names_the_decision_instead_of_falling_back(self):
        with mock.patch("platform.system", return_value="Darwin"):
            with self.assertRaises(floor.FloorError) as raised:
                floor.require_linux()
        message = str(raised.exception)
        self.assertIn("getattrlistbulk", message)
        self.assertIn("fdu-33ri", message)

    def test_a_busy_host_is_refused_under_the_quiet_regime(self):
        with self.assertRaises(floor.FloorError):
            floor._require_quiet({"load_1m_per_cpu": 0.9})

    def test_a_quiet_host_passes(self):
        floor._require_quiet({"load_1m_per_cpu": 0.04})


class FlagsMoreThanOnePopulation(unittest.TestCase):
    """A median describes one hump. `arena_spike` on a shared container has two."""

    def _summary(self, samples):
        trials = [
            floor.Trial(instrument="arena-spike", ordinal=i, warmup=False,
                        elapsed_ns=value, spawn_wall_ns=value + 2_000_000,
                        max_rss_bytes=15 << 20, tallies={})
            for i, value in enumerate(samples)
        ]
        return floor._summarize(trials, floor.INSTRUMENTS["arena-spike"])

    def test_bimodal_samples_are_flagged(self):
        # The two modes actually measured on a four-core container: ~63 ms and ~150 ms.
        summary = self._summary([63e6, 64e6, 150e6, 152e6, 63e6, 151e6])
        self.assertTrue(summary["multimodal_suspect"])
        self.assertGreaterEqual(summary["spread"], floor.SPREAD_SUSPECT)

    def test_a_tight_unimodal_instrument_is_not_flagged(self):
        summary = self._summary([38e6, 39e6, 40e6, 39e6, 41e6])
        self.assertFalse(summary["multimodal_suspect"])

    def test_p95_over_median_can_look_calm_while_spread_does_not(self):
        """Both humps are individually narrow, so the tail ratio reassures wrongly."""
        summary = self._summary([150e6, 151e6, 152e6, 63e6, 64e6, 65e6])
        self.assertLess(summary["elapsed_ns"]["p95_over_median"], 1.5)
        self.assertTrue(summary["multimodal_suspect"])


class ScoresAgainstTheFloor(unittest.TestCase):
    def _subject(self, medians):
        return {
            "label": "usr-tree", "entries": 75_976,
            "instruments": {
                name: {
                    "role": floor.INSTRUMENTS[name].role,
                    "description": "", "samples": 30, "spread": 1.2,
                    "multimodal_suspect": False,
                    "elapsed_ns": {"median": value, "min": value, "max": value,
                                   "p95": value, "p95_over_median": 1.0},
                    "spawn_wall_ns": {"median": value}, "harness_overhead_ns": 0,
                    "max_rss_bytes": None,
                }
                for name, value in medians.items()
            },
        }

    def test_ratios_divide_by_the_floor_instrument(self):
        scored = floor.score(self._subject({
            "parfloor-stat": 39_260_000, "aggregate": 62_210_000, "index": 110_600_000,
        }))
        rows = {row["instrument"]: row for row in scored["rows"]}
        self.assertEqual(rows["parfloor-stat"]["x_floor"], 1.0)
        self.assertAlmostEqual(rows["aggregate"]["x_floor"], 1.585, places=2)
        self.assertAlmostEqual(rows["index"]["x_floor"], 2.817, places=2)

    def test_thresholds_decide_only_for_tiers(self):
        scored = floor.score(self._subject({
            "parfloor-stat": 39_260_000, "aggregate": 62_210_000, "index": 110_600_000,
        }))
        rows = {row["instrument"]: row for row in scored["rows"]}
        # 1.58x against a 1.25x threshold, and 2.82x against 1.40x: neither is closed.
        self.assertFalse(rows["aggregate"]["meets_threshold"])
        self.assertFalse(rows["index"]["meets_threshold"])
        # The floor is not a contestant and has no threshold to meet.
        self.assertIsNone(rows["parfloor-stat"]["meets_threshold"])

    def test_a_tier_inside_its_threshold_is_closed(self):
        scored = floor.score(self._subject({
            "parfloor-stat": 40_000_000, "aggregate": 46_000_000,
        }))
        rows = {row["instrument"]: row for row in scored["rows"]}
        self.assertTrue(rows["aggregate"]["meets_threshold"])

    def test_no_floor_measurement_is_refused(self):
        with self.assertRaises(floor.FloorError):
            floor.score(self._subject({"aggregate": 62_210_000}))


if __name__ == "__main__":
    unittest.main()
