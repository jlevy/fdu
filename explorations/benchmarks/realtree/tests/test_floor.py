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


def probe_line(dirs, files, *, apparent=1000, allocated=2000, component_ns=5_000_000):
    return json.dumps({"component_ns": component_ns, "mode": "x", "summary": {
        "dirs": dirs, "files": files, "apparent_bytes": apparent, "allocated_bytes": allocated,
    }})


def parfloor_line(dirs, files, *, other=3, apparent=1000, allocated=2000, wall_ns=1_000_000,
                  variant="stat"):
    return json.dumps({"variant": variant, "threads": 4, "dirs": dirs, "files": files,
                       "other": other, "bytes": apparent, "allocated": allocated,
                       "wall_ns": wall_ns})


def arena_line(dirs, files, *, apparent=1000, allocated=2000, wall_ms=2.5):
    return json.dumps({"files": files, "dirs": dirs, "bytes": apparent,
                       "allocated": allocated, "wall_ms": wall_ms})


#: One tree as each instrument reports it: 10 directories under the root, 50 files and 3
#: symlinks. `parfloor enum` makes no metadata call, so it counts directories only, and
#: the index retains the root as an entry of its own (11 - 1 == 10).
CONSISTENT = {
    "parfloor-stat": parfloor_line(10, 50),
    "parfloor-enum": parfloor_line(10, 0, other=0, variant="enum"),
    "arena-spike": arena_line(10, 50),
    "aggregate": probe_line(10, 50),
    "index": probe_line(11, 50),
}


def instrument_key(argv):
    name = Path(argv[0]).name
    if name == "parfloor":
        return f"parfloor-{argv[1]}"
    if name == "arena_spike":
        return "arena-spike"
    return {"summary": "aggregate", "scan-index": "index"}[argv[1]]


def pressure(load_per_cpu):
    """A Linux host-pressure snapshot in the shape `measure` records."""
    return {"system": "Linux", "logical_cpu_count": 4, "load_1m": None,
            "load_1m_per_cpu": load_per_cpu, "cpu_busy_pct": None,
            "power_source": None, "thermal_pressure": None, "controlled_load_alive": None}


QUIET = pressure(0.01)
BUSY = pressure(0.9)


def pressure_sequence(*snapshots):
    """Serve `snapshots` in order, then repeat the last one forever."""
    remaining = list(snapshots)

    def next_snapshot(_regime):
        return dict(remaining.pop(0) if len(remaining) > 1 else remaining[0])

    return next_snapshot


def run_subject(outputs, *, instruments=floor.DEFAULT_INSTRUMENTS, trials=2, warmups=1,
                snapshots=(QUIET,), **overrides):
    """Run the real `measure_subject` with only process spawning and pressure replaced.

    Adapted from the proof tests published with the PR #49 review. Host pressure is
    replaced because sampling it for real sleeps a second per snapshot on macOS.
    """
    order = []

    def fake_spawn(argv, *, timeout_seconds):
        key = instrument_key(argv)
        order.append(key)
        return {"stdout": outputs[key] + "\n", "spawn_wall_ns": 7_000_000,
                "max_rss_bytes": 1 << 20}

    arguments = dict(
        root=Path("/r"), label="t", binaries=BINARIES,
        instruments=[floor.INSTRUMENTS[name] for name in instruments],
        workers=4, trials=trials, warmups=warmups, quiet=False,
    )
    arguments.update(overrides)
    with mock.patch.object(floor, "_spawn", fake_spawn), \
            mock.patch("benchmarks.realtree.measure._host_pressure_snapshot",
                       side_effect=pressure_sequence(*snapshots)):
        subject = floor.measure_subject(**arguments)
    return subject, order


def run_document(*, outputs=CONSISTENT, snapshots=(QUIET,), host_regime="uncontrolled",
                 trials=2, warmups=1, subjects=(("t", Path("/r")),)):
    """Run the real `run` end to end with nothing built, spawned, or sampled for real."""

    def fake_spawn(argv, *, timeout_seconds):
        return {"stdout": outputs[instrument_key(argv)] + "\n", "spawn_wall_ns": 7_000_000,
                "max_rss_bytes": 1 << 20}

    with mock.patch.object(floor, "require_linux"), \
            mock.patch.object(floor, "build_instruments", return_value=dict(BINARIES)), \
            mock.patch.object(floor, "_spawn", fake_spawn), \
            mock.patch.object(floor.time, "sleep"), \
            mock.patch("benchmarks.realtree.measure._host_pressure_snapshot",
                       side_effect=pressure_sequence(*snapshots)):
        return floor.run(subjects=list(subjects), workers=4, trials=trials, warmups=warmups,
                         build_dir=Path("/b"), host_regime=host_regime)


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


class EveryInstrumentMeetsEveryPredecessor(unittest.TestCase):
    """What the previous process left behind is a predecessor effect, and the order
    decides whose effect each instrument inherits.

    Review FLOOR-2: every round ran the same order, so each instrument followed the same
    predecessor in every round and that effect was baked into its median. A rotation by
    the round's ordinal does not fix it: a cyclic shift keeps adjacent pairs adjacent.
    """

    @staticmethod
    def _adjacency(rounds, names):
        sequence = [name for order in rounds for name in order]
        counts = {(before, after): 0 for before in names for after in names if before != after}
        repeats = 0
        for before, after in zip(sequence, sequence[1:]):
            if before == after:
                repeats += 1
            else:
                counts[(before, after)] += 1
        return counts, repeats

    def test_every_round_runs_every_instrument_once(self):
        for count in range(1, 6):
            names = [f"i{k}" for k in range(count)]
            rounds = floor.schedule(names, trials=7, warmups=2)
            self.assertEqual([ordinal for ordinal, _ in rounds], list(range(-2, 7)))
            for _, order in rounds:
                self.assertEqual(sorted(order), names)

    def test_every_instrument_follows_every_other_equally_often(self):
        for count in range(2, 6):
            names = [f"i{k}" for k in range(count)]
            for trials in (1, 4, 11, 30):
                with self.subTest(instruments=count, trials=trials):
                    rounds = [order for _, order in floor.schedule(names, trials=trials, warmups=3)]
                    counts, repeats = self._adjacency(rounds, names)
                    self.assertEqual(repeats, 0, "an instrument followed itself")
                    self.assertLessEqual(max(counts.values()) - min(counts.values()), 1, counts)

    def test_no_instrument_keeps_a_single_predecessor(self):
        _, order = run_subject(CONSISTENT, trials=7, warmups=1)
        for name in floor.DEFAULT_INSTRUMENTS:
            with self.subTest(instrument=name):
                predecessors = {before for before, after in zip(order, order[1:]) if after == name}
                self.assertEqual(len(predecessors), len(floor.DEFAULT_INSTRUMENTS) - 1)

    def test_the_run_follows_the_schedule_it_records(self):
        subject, order = run_subject(CONSISTENT, trials=3, warmups=1)
        recorded = subject["schedule"]
        self.assertEqual(recorded["scheme"], floor.SCHEDULE_SCHEME)
        self.assertEqual(len(recorded["rounds"]), 4)
        self.assertEqual([name for names in recorded["rounds"] for name in names], order)


class RefusesRatherThanSubstituting(unittest.TestCase):
    def test_non_linux_names_the_decision_instead_of_falling_back(self):
        with mock.patch("platform.system", return_value="Darwin"):
            with self.assertRaises(floor.FloorError) as raised:
                floor.require_linux()
        message = str(raised.exception)
        self.assertIn("getattrlistbulk", message)
        self.assertIn("fdu-33ri", message)


class HoldsEveryTrialToTheQuietBar(unittest.TestCase):
    """`quiet` is the loop's contract -- the bar holds before and after every sample --
    checked by `measure`'s own gate rather than a copy of it.

    Review FLOOR-3: the bar was checked once per subject, before any trial, and a load
    average that could not be read passed silently. Review FLOOR-7: that one check came
    straight after the harness's own release build, so a first run refused on an idle host.
    """

    def setUp(self):
        sleep = mock.patch.object(floor.time, "sleep")
        self.sleep = sleep.start()
        self.addCleanup(sleep.stop)

    def test_a_quiet_host_runs_every_trial_valid(self):
        subject, _ = run_subject(CONSISTENT, quiet=True)
        self.assertEqual(subject["host_regime"], "quiet")
        self.assertEqual(subject["invalid_trials"], 0)
        self.sleep.assert_not_called()

    def test_quiet_is_refused_when_load_cannot_be_read(self):
        with self.assertRaises(floor.FloorError) as raised:
            run_subject(CONSISTENT, quiet=True, snapshots=(pressure(None),))
        self.assertIn("unavailable", str(raised.exception))
        self.sleep.assert_not_called()  # waiting cannot make it readable

    def test_a_trial_that_breaches_the_bar_is_invalid(self):
        # Quiet at entry and around the whole first round, busy from then on.
        entry = [QUIET] * 2
        first_round = [QUIET] * (2 * len(floor.DEFAULT_INSTRUMENTS))
        subject, _ = run_subject(CONSISTENT, quiet=True, warmups=1, trials=2,
                                 snapshots=(*entry, *first_round, BUSY))
        # The warmup round stayed quiet; both measured rounds breached.
        self.assertEqual(subject["invalid_trials"], 2 * len(floor.DEFAULT_INSTRUMENTS))
        self.assertTrue(any("load/core exceeded" in reason for reason in subject["invalid_reasons"]))

    def test_an_uncontrolled_run_records_pressure_without_judging_it(self):
        subject, _ = run_subject(CONSISTENT, quiet=False, snapshots=(BUSY,))
        self.assertEqual(subject["host_regime"], "uncontrolled")
        self.assertEqual(subject["invalid_trials"], 0)

    def test_a_host_still_settling_after_a_build_is_given_time(self):
        subject, _ = run_subject(CONSISTENT, quiet=True, snapshots=(BUSY, BUSY, QUIET))
        self.assertEqual(subject["invalid_trials"], 0)
        self.assertEqual(self.sleep.call_count, 2)
        self.sleep.assert_called_with(floor.QUIET_POLL_SECONDS)

    def test_a_host_that_never_settles_is_refused_after_the_stated_bound(self):
        clock = iter(range(0, 10_000, 60))
        with mock.patch.object(floor.time, "monotonic", side_effect=lambda: next(clock)):
            with self.assertRaises(floor.FloorError) as raised:
                run_subject(CONSISTENT, quiet=True, snapshots=(BUSY,), quiet_wait_seconds=180)
        message = str(raised.exception)
        self.assertIn("not quiet enough", message)
        self.assertIn("180", message)

    def test_one_breaching_trial_downgrades_the_whole_scoreboard(self):
        """A table cannot say quiet when one of its samples was not."""
        # Settle check and regime entry are quiet; the first measured trial is not.
        document = run_document(snapshots=(QUIET, QUIET, BUSY, QUIET), host_regime="quiet",
                                warmups=0)
        self.assertEqual(document["host_regime"], "uncontrolled")
        self.assertEqual(document["host_regime_requested"], "quiet")
        rendered = floor.render(document)
        self.assertIn("Regime: **uncontrolled**", rendered)
        self.assertIn("quiet was requested", rendered)

    def test_a_scoreboard_whose_every_trial_held_the_bar_stays_quiet(self):
        document = run_document(snapshots=(QUIET,), host_regime="quiet")
        self.assertEqual(document["host_regime"], "quiet")
        self.assertNotIn("was requested", floor.render(document))


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
