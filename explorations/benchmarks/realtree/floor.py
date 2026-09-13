"""The tier-by-subject floor scoreboard: what the machine costs, and where fdu sits.

Campaign 2 orders work by each tier's measured distance to the parallel syscall floor
(`docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md`), which makes
that distance the scoreboard. Deriving it used to be a by-hand session with the spikes,
recorded once in `docs/project/reports/report-2026-08-23-metadata-walk-floor.md`; this
module is that session automated, so the shared-cost re-screen and the termination
criteria are checkable rather than asserted.

What it is not: a verdict harness. `measure.py` decides whether a *change* is kept, under
the accept rule's paired 3% gate. This decides where a *tier* stands against the machine,
which is a ratio of two absolute numbers rather than a paired difference. The two answer
different questions and neither substitutes for the other.

## Why this is Linux-only, and why it refuses rather than falls back

`parfloor.c` -- the denominator every x-floor threshold in campaign 2 is defined against
-- is Linux-only: it issues `SYS_getdents64` and `statx` directly, and neither has a
Darwin equivalent. A macOS scoreboard therefore needs either a `getattrlistbulk` port of
the floor (`fdu-9hdc`) or a different floor set with the regime difference recorded.

That is a decision for the plan, not for a harness quietly substituting a different
denominator and continuing to print a column headed "xfloor". So on a non-Linux host this
module exits with that sentence rather than measuring something else. The bead recording
the choice is `fdu-33ri`; the blocked half is tracked as `fdu-9hdc`.

## The instruments, and why these three

- **`parfloor stat`** is the denominator: N threads over a shared directory queue, raw
  `getdents64` plus one `statx` per entry into four integer accumulators. No index, no
  retained paths, no per-entry allocation, no delta contract.
- **`arena_spike`** is the measured ceiling for the representation change: it retains an
  index-shaped result and is what H86 (`fdu-xde5`) is trying to reach. It is a reference
  row, never a denominator.
- **fdu's own tiers**, through `perf_probe`, which is the thing being scored.

`peerwalk` is deliberately absent. It takes third-party dependencies the shipped crate
does not have, and its README says it is never built by `make`; the ecosystem anchor is a
question for the floor *report*, not for a scoreboard that has to run unattended.

## The pool: a fixed N for every instrument

A one-thread floor is not a lower bound for a parallel walker, and a floor at N threads
is a lower bound only for a walker that also runs N. So every instrument runs a fixed
pool of the same N workers: the floor and the ceiling take N as an argument, and fdu's
tiers take `--threads N`.

That makes the scoreboard a reading of fdu *at a fixed pool of N*, not of its shipped
automatic pool, which starts at a capped share of the cores and grows when calibration
finds a slow path. The two readings coincide only where the automatic pool would pick N
and never grow; anywhere else, leaving fdu's pool to its policy would divide one pool
size by another.

N defaults to the CPUs this process may run on -- its affinity mask, not the host's CPU
count, which a pinned container does not shrink -- and may not exceed `MAX_WORKERS`,
fdu's own clamp, above which fdu would quietly run fewer workers than the floor.

## What is timed

Each instrument reports its own internal elapsed time, and that -- not the spawn wall --
is the primary metric. Process startup, argument parsing and JSON rendering are harness
cost, and on a small subject they are a large fraction of a spawn: the loop guide's
warning that a probe's own oracle digest has measured 31.9% of a profile is the same
mistake one level up. Spawn wall is recorded beside it so the gap stays visible.

## The oracle

Every instrument emits the same tallies over the same tree, so any two that disagree mean
one of them is broken rather than fast. The four all three emit -- directories, files,
apparent bytes, allocated bytes -- are enforced on every trial, not just the first: a
subject that changes underneath the run invalidates it, and a live working directory is a
subject this loop is expected to be handed.

`parfloor` counts symlinks and other non-regular entries in its own `other` bucket, which
fdu excludes from `files`/`dirs` entirely; that difference is structural and reconciled
here rather than treated as drift.

## The host regime

The scoreboard divides two absolute numbers measured minutes apart, so it is *more*
exposed to host drift than a paired comparison, not less: interleaving spreads drift
across instruments, but nothing cancels a host that was quiet for the denominator and
busy for the numerator.

So `quiet` here is the loop's contract rather than a local variant of it. `measure`'s own
gate -- on Linux, a one-minute load average of at most 0.25 per core -- must hold before
and after every trial, and a load average that cannot be read refuses the regime rather
than passing it. A trial that breaches the gate is invalid, and one invalid measured trial
anywhere downgrades the whole scoreboard to `uncontrolled`: a table cannot say quiet when
one of its samples was not. It is still written, as the screening-grade table
`--host-regime uncontrolled` would have recorded.

Before each subject's first trial the harness waits, within a stated bound
(`--quiet-wait`), for a settling host to meet the gate, because `make perf-floor` builds
the probe immediately beforehand and a load average remembers a build for minutes. The
wait decides when measurement starts, never what it accepts. A load average also
remembers this harness's own instruments, which run N workers back to back, so a long run
on few cores can cross the gate on its own load and be downgraded; `measure` shares that
property on Linux.
"""

from __future__ import annotations

import contextlib
import functools
import itertools
import json
import os
import platform
import shutil
import statistics
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence

from benchmarks.realtree import measure

PROJECT_ROOT = Path(__file__).resolve().parents[3]
SPIKES = PROJECT_ROOT / "explorations" / "benchmarks" / "spikes"

#: The tallies every instrument emits, and the only ones all three agree on.
ORACLE_KEYS = ("dirs", "files", "apparent_bytes", "allocated_bytes")

DEFAULT_TRIALS = 8
DEFAULT_WARMUPS = 2
DEFAULT_TIMEOUT_SECONDS = 900.0

#: How long `--host-regime quiet` waits for a settling host before it refuses.
#:
#: A load average is a one-minute exponential average, so after load stops it stays over
#: a 0.25-per-core bar for ln(load per core / 0.25) minutes: 83 s after a build that held
#: every core busy, 125 s after one that ran twice oversubscribed. `make perf-floor` builds
#: the probe immediately before this starts. Three minutes covers a build of up to five
#: runnable threads per core; `--quiet-wait` lifts it.
QUIET_WAIT_SECONDS = 180.0

#: How often that wait re-reads the load average, which the kernel updates every 5 s.
QUIET_POLL_SECONDS = 5.0

#: max/min at or past which the samples span more than a median can stand for. See
#: `_summarize`.
SPREAD_SUSPECT = 2.0

#: The largest pool every instrument runs exactly. fdu clamps `--threads` to
#: `MAX_SCAN_THREADS` (`crates/fdu-core/src/scan.rs`) without saying so, parfloor clamps
#: at 64, and arena_spike does not clamp -- so fdu's is the clamp that binds, and above
#: it the floor would run more workers than the tiers divided by it.
MAX_WORKERS = 32


class FloorError(RuntimeError):
    """The scoreboard cannot be produced, with the reason a reader needs."""


@dataclass
class Instrument:
    """One measured program: how to run it, and how to read its answer."""

    id: str
    role: str  # "floor", "ceiling", or "tier"
    description: str
    argv: Sequence[str]
    #: Maps this instrument's own JSON keys onto ORACLE_KEYS.
    tally_map: Mapping[str, str]
    #: Constant reconciliations applied before the oracle compares, with the reason in
    #: `tally_notes`. These are definitional differences between instruments measuring
    #: the same tree, not slack in the oracle: the comparison is exact after them.
    tally_offsets: Mapping[str, int] = field(default_factory=dict)
    tally_notes: str = ""
    #: Key holding the instrument's internal elapsed time, and its scale to nanoseconds.
    elapsed_key: str = "wall_ns"
    elapsed_scale: float = 1.0
    #: `perf_probe` nests its answer under "summary".
    payload_key: Optional[str] = None

    def command(self, *, binaries: Mapping[str, Path], root: Path, workers: int) -> List[str]:
        return [
            part.format(
                parfloor=binaries["parfloor"],
                arena_spike=binaries["arena_spike"],
                probe=binaries["probe"],
                root=root,
                workers=workers,
            )
            for part in self.argv
        ]

    def read(self, stdout: str) -> Dict[str, Any]:
        """Pull the tallies and the internal timer out of one run's JSON line."""
        document = json.loads(stdout.strip().splitlines()[-1])
        elapsed_source = document
        payload = document[self.payload_key] if self.payload_key else document
        tallies = {target: payload[source] for source, target in self.tally_map.items()}
        for key, offset in self.tally_offsets.items():
            if key in tallies:
                tallies[key] += offset
        elapsed = elapsed_source.get(self.elapsed_key)
        if elapsed is None:
            elapsed = payload.get(self.elapsed_key)
        if elapsed is None:
            raise FloorError(f"{self.id}: no {self.elapsed_key} in its output")
        return {"tallies": tallies, "elapsed_ns": int(float(elapsed) * self.elapsed_scale)}


#: `parfloor enum` is included because the gap between it and `stat` is the price of the
#: metadata call -- 91% of this workload's kernel cost, and the whole reason a search
#: tool's numbers are not comparable with a disk-usage tool's.
INSTRUMENTS: Dict[str, Instrument] = {
    "parfloor-stat": Instrument(
        id="parfloor-stat",
        role="floor",
        description="Raw getdents64 + statx per entry into four accumulators. The floor.",
        argv=("{parfloor}", "stat", "{root}", "{workers}"),
        tally_map={"dirs": "dirs", "files": "files", "bytes": "apparent_bytes",
                   "allocated": "allocated_bytes"},
    ),
    "parfloor-enum": Instrument(
        id="parfloor-enum",
        role="reference",
        description="The same walk with the metadata call removed: a search tool's floor.",
        argv=("{parfloor}", "enum", "{root}", "{workers}"),
        tally_map={"dirs": "dirs"},
    ),
    "arena-spike": Instrument(
        id="arena-spike",
        role="ceiling",
        description="An index-shaped result in arena records: the H86 ceiling.",
        argv=("{arena_spike}", "{root}", "{workers}"),
        tally_map={"dirs": "dirs", "files": "files", "bytes": "apparent_bytes",
                   "allocated": "allocated_bytes"},
        elapsed_key="wall_ms",
        elapsed_scale=1e6,
    ),
    "aggregate": Instrument(
        id="aggregate",
        role="tier",
        description="fdu aggregate tier: five exact tallies, no retained index.",
        argv=("{probe}", "summary", "--root", "{root}", "--threads", "{workers}"),
        tally_map={"dirs": "dirs", "files": "files", "apparent_bytes": "apparent_bytes",
                   "allocated_bytes": "allocated_bytes"},
        elapsed_key="component_ns",
        payload_key="summary",
    ),
    "index": Instrument(
        id="index",
        role="tier",
        description="fdu index tier: full walk with metadata into a complete index.",
        argv=("{probe}", "scan-index", "--root", "{root}", "--threads", "{workers}"),
        tally_map={"dirs": "dirs", "files": "files", "apparent_bytes": "apparent_bytes",
                   "allocated_bytes": "allocated_bytes"},
        tally_offsets={"dirs": -1},
        tally_notes=(
            "An index retains the root directory as an entry of its own -- the tree root "
            "node, with an identity and a roll-up -- while a tallying walk counts only what "
            "it enumerates inside the root. Measured on two subjects the difference is "
            "exactly one directory, and index_len confirms the decomposition: on /usr, "
            "84,536 = 7,843 dirs + 68,134 files + 8,559 symlinks + the root."
        ),
        elapsed_key="component_ns",
        payload_key="summary",
    ),
}

#: The instruments a scoreboard runs, denominator first. This is the order of the first
#: round only; later rounds follow `schedule`, and the table sorts rows by ratio.
DEFAULT_INSTRUMENTS = ("parfloor-stat", "parfloor-enum", "arena-spike", "aggregate", "index")

#: Names the ordering `schedule` produces, recorded with every subject.
SCHEDULE_SCHEME = "carryover-balanced-v1"


# --------------------------------------------------------------------------------------
# Building the instruments
# --------------------------------------------------------------------------------------


def default_workers() -> int:
    """The CPUs this process may run on, capped at the pool every instrument runs exactly.

    The affinity mask rather than `os.cpu_count()`, which counts the host's CPUs even for
    a process pinned to fewer. `os.process_cpu_count()` reads that mask on Python 3.13
    and later; `sched_getaffinity` reads it before that, where the platform has one.
    """
    process_cpu_count = getattr(os, "process_cpu_count", None)
    count = process_cpu_count() if process_cpu_count is not None else None
    if count is None:
        affinity = getattr(os, "sched_getaffinity", None)
        if affinity is not None:
            count = len(affinity(0))
    if count is None:
        count = os.cpu_count()
    return max(1, min(count or 1, MAX_WORKERS))


def require_linux() -> None:
    """Refuse on a host whose floor this denominator does not describe.

    See the module docstring: substituting a different floor and still printing a column
    headed "xfloor" would make the scoreboard say something it has not measured.
    """
    if platform.system() != "Linux":
        raise FloorError(
            f"the floor scoreboard is Linux-only ({platform.system()} here): parfloor.c "
            "issues SYS_getdents64 and statx directly, and neither has a Darwin "
            "equivalent. A macOS scoreboard needs a getattrlistbulk floor (fdu-9hdc) or "
            "a different floor set with the regime difference recorded -- a decision for "
            "the campaign plan, not a fallback this harness may pick. See fdu-33ri."
        )


def build_instruments(destination: Path) -> Dict[str, Path]:
    """Compile the two spikes into `destination`.

    The spikes are single files outside the workspace on purpose -- they take no
    dependencies and are not shipped -- so they are built here rather than by cargo.
    The probe is not: it is whatever `make perf-probe-release` built, handed to `run` by
    path, so a scoreboard and a verdict run score one binary. A second copy of that cargo
    line here would build a different probe the moment the make target changes, into the
    same output path.
    """
    destination.mkdir(parents=True, exist_ok=True)
    binaries: Dict[str, Path] = {}

    for tool, missing in (("gcc", "gcc"), ("rustc", "rustc")):
        if shutil.which(tool) is None:
            raise FloorError(f"{missing} is required to build the floor instruments")

    parfloor = destination / "parfloor"
    _run_build(
        ["gcc", "-O2", "-pthread", "-o", str(parfloor), str(SPIKES / "parfloor.c")],
        what="parfloor",
    )
    binaries["parfloor"] = parfloor

    arena = destination / "arena_spike"
    _run_build(
        ["rustc", "-O", "-o", str(arena), str(SPIKES / "arena_spike.rs")],
        what="arena_spike",
    )
    binaries["arena_spike"] = arena
    return binaries


def default_build_dir() -> Path:
    """Where the spikes are built unless `--build-dir` says otherwise: under cargo's own
    target directory.

    Asked of cargo rather than assumed to be `target/`, because `CARGO_TARGET_DIR` and
    `build.target-dir` move it. And not a fixed path in `/tmp`, where anyone on the host
    could create the directory first and replace a binary between its build and its run.
    """
    try:
        completed = subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--no-deps"],
            cwd=str(PROJECT_ROOT), capture_output=True, text=True,
        )
    except OSError as error:
        raise FloorError(f"cannot ask cargo for its target directory: {error}") from error
    target = None
    if completed.returncode == 0:
        try:
            document = json.loads(completed.stdout)
        except ValueError:
            document = None
        if isinstance(document, dict):
            target = document.get("target_directory")
    if not isinstance(target, str) or not target:
        raise FloorError(
            "cargo metadata did not name a target directory, so there is no safe default "
            f"for the spike build; pass --build-dir. {completed.stderr.strip()[-500:]}"
        )
    return Path(target) / "fdu-floor"


def _run_build(argv: Sequence[str], *, what: str, cwd: Optional[Path] = None) -> None:
    outcome = subprocess.run(
        list(argv), cwd=str(cwd) if cwd else None, capture_output=True, text=True
    )
    if outcome.returncode != 0:
        raise FloorError(f"building {what} failed:\n{outcome.stderr.strip()[-2000:]}")


# --------------------------------------------------------------------------------------
# Running one cell
# --------------------------------------------------------------------------------------


@dataclass
class Trial:
    instrument: str
    ordinal: int
    warmup: bool
    elapsed_ns: int
    spawn_wall_ns: int
    max_rss_bytes: Optional[int]
    tallies: Dict[str, int]
    #: Whether the host regime's gate held before and after this trial, and why not.
    valid: bool = True
    reasons: List[str] = field(default_factory=list)


def _spawn(argv: Sequence[str], *, timeout_seconds: float) -> Dict[str, Any]:
    """Run one instrument and collect its stdout, its wall time and its own rusage.

    `os.wait4` gives the resource usage of exactly this child, which is what a per-trial
    peak-RSS number has to mean. `RUSAGE_CHILDREN` cannot: it is a high-water mark across
    every child this process has ever reaped, so after the first large instrument it stops
    moving and every later one reads back that first peak. Peak RSS is a pre-registered
    target of the structural experiment, so it has to be the child's own.

    Output goes to temporary files rather than pipes, for the reason `measure._spawn`
    gives: a pipe would need a draining thread, and that thread's scheduling would land
    inside the measured window.
    """
    import tempfile
    import threading

    with tempfile.TemporaryDirectory(prefix="fdu-floor-") as scratch:
        out_path = Path(scratch) / "stdout"
        err_path = Path(scratch) / "stderr"
        with out_path.open("xb") as out, err_path.open("xb") as err:
            start = time.perf_counter_ns()
            process = subprocess.Popen(
                list(argv), stdin=subprocess.DEVNULL, stdout=out, stderr=err,
            )
            # Block in a thread rather than polling `WNOHANG`, as `measure._spawn` does.
            # A poll loop wakes the parent a thousand times a second, which on a
            # four-core host is measurable pressure on a four-worker child; it was not
            # the cause of the bimodality found while writing this module (that survived
            # the change and is a property of `arena_spike`, recorded in `_summarize`),
            # but a harness should not be adding load it then measures.
            state = _WaitState()
            waiter = threading.Thread(target=_wait_for_child, args=(process.pid, state))
            waiter.start()
            waiter.join(timeout_seconds)
            elapsed = time.perf_counter_ns() - start
            if waiter.is_alive():
                process.kill()
                waiter.join(10.0)
                raise FloorError(f"{Path(argv[0]).name} exceeded {timeout_seconds}s")
        # `wait4` already reaped it; keep Popen from trying to do so again at exit.
        process.returncode = state.exit_code if state.exit_code is not None else -1
        stdout = out_path.read_text()
        stderr_text = err_path.read_text()

    if state.error:
        raise FloorError(f"{Path(argv[0]).name}: {state.error}")
    if state.exit_code != 0:
        raise FloorError(f"{Path(argv[0]).name} exited {state.exit_code}: {stderr_text[-500:]}")
    return {
        "stdout": stdout,
        "spawn_wall_ns": elapsed,
        # Linux reports ru_maxrss in kibibytes.
        "max_rss_bytes": int(state.usage.ru_maxrss) * 1024 if state.usage else None,
    }


class _WaitState:
    """Where the waiting thread leaves what it reaped."""

    def __init__(self) -> None:
        self.exit_code: Optional[int] = None
        self.usage: Any = None
        self.error: Optional[str] = None


def _wait_for_child(pid: int, state: "_WaitState") -> None:
    try:
        _waited, status, usage = os.wait4(pid, 0)
        state.exit_code = os.waitstatus_to_exitcode(status)
        state.usage = usage
    except (ChildProcessError, OSError, ValueError) as error:
        state.error = f"cannot wait for child process: {error}"


def run_cell(
    instrument: Instrument,
    *,
    binaries: Mapping[str, Path],
    root: Path,
    workers: int,
    ordinal: int,
    warmup: bool,
    regime: measure.HostRegime,
    timeout_seconds: float = DEFAULT_TIMEOUT_SECONDS,
) -> Trial:
    argv = instrument.command(binaries=binaries, root=root, workers=workers)
    pressure_before = measure._host_pressure_snapshot(regime)
    outcome = _spawn(argv, timeout_seconds=timeout_seconds)
    pressure_after = measure._host_pressure_snapshot(regime)
    parsed = instrument.read(outcome["stdout"])
    reasons = measure._host_pressure_reasons(regime, pressure_before, pressure_after)
    return Trial(
        instrument=instrument.id,
        ordinal=ordinal,
        warmup=warmup,
        elapsed_ns=parsed["elapsed_ns"],
        spawn_wall_ns=outcome["spawn_wall_ns"],
        max_rss_bytes=outcome["max_rss_bytes"],
        tallies=parsed["tallies"],
        valid=not reasons,
        reasons=reasons,
    )


# --------------------------------------------------------------------------------------
# One subject
# --------------------------------------------------------------------------------------


def _await_quiet(wait_seconds: float) -> bool:
    """Give a host that is only settling up to `wait_seconds` to meet the quiet gate.

    Returns whether it waited at all. It judges nothing: the regime entry that follows is
    `measure`'s own check and refuses a host that did not settle, and every trial is
    still held to the gate. A load average that cannot be read is not waited for, since
    time will not make it readable.
    """
    regime = measure.HostRegime(name="quiet", initial={})
    started = time.monotonic()
    waited = False
    while True:
        snapshot = measure._host_pressure_snapshot(regime)
        if snapshot.get("load_1m_per_cpu") is None:
            return waited
        if not measure._host_pressure_reasons(regime, snapshot, snapshot):
            return waited
        if time.monotonic() - started >= wait_seconds:
            return waited
        time.sleep(QUIET_POLL_SECONDS)
        waited = True


def _enter_regime(
    stack: contextlib.ExitStack, name: str, wait_seconds: float
) -> measure.HostRegime:
    """Enter `measure`'s host regime, after a settling host has had its bounded wait."""
    waited = _await_quiet(wait_seconds) if name == "quiet" else False
    try:
        return stack.enter_context(measure._host_regime(name, 0))
    except measure.MeasureError as error:
        after = f", after waiting up to {wait_seconds:.0f} s for it to settle" if waited else ""
        raise FloorError(
            f"{error}{after}. The scoreboard divides two absolute numbers measured minutes "
            "apart, so host drift does not cancel the way it does in a paired run. Wait, "
            "or pass --host-regime uncontrolled to record a screening-grade table."
        ) from error


def schedule(instruments: Sequence[Any], *, trials: int, warmups: int) -> List[tuple]:
    """The order of every round, warmups first, as `(ordinal, instruments)` pairs.

    Every instrument runs once per round, and across rounds every instrument follows
    every other equally often -- round boundaries included, since the first process of a
    round runs straight after the last one of the round before -- and never follows
    itself. Over any stretch of rounds, how often one instrument follows another differs
    by at most one between pairs.

    That is the property a fixed order lacks: it gives each instrument the same
    predecessor in every round, and whatever that predecessor leaves behind -- memory
    churned, page cache displaced -- lands on the same instrument's median every time. A
    rotation by the round's ordinal does not supply it either. A cyclic shift keeps
    adjacent instruments adjacent, so each would still follow one predecessor in all but
    one round of every n.
    """
    block = _balanced_block(len(instruments))
    return [
        (ordinal, [instruments[index] for index in block[(ordinal + warmups) % len(block)]])
        for ordinal in range(-warmups, trials)
    ]


@functools.lru_cache(maxsize=None)
def _balanced_block(count: int) -> tuple:
    """`count - 1` orderings that, repeated end to end, balance every predecessor.

    Concatenated and read cyclically, the block has every ordered pair of distinct
    instruments adjacent exactly once, and no instrument adjacent to itself: `count - 1`
    rounds of `count - 1` pairs each, plus the `count - 1` boundaries between rounds,
    is exactly the `count * (count - 1)` ordered pairs there are. The first ordering is
    the declared one.

    Found by a depth-first search rather than a formula, because the closed forms cover
    only some counts (stepping by k modulo a prime count, for one) and a scoreboard can
    be handed any subset of the catalogue. The search is exhaustive and returns at once
    at these sizes -- well under a second through eight instruments.
    """
    if count < 2:
        return (tuple(range(count)),)
    declared = tuple(range(count))
    orderings = list(itertools.permutations(range(count)))
    used = set(zip(declared, declared[1:]))
    block = [declared]

    def extend() -> bool:
        if len(block) == count - 1:
            closing = (block[-1][-1], block[0][0])
            return closing[0] != closing[1] and closing not in used
        for ordering in orderings:
            pairs = [(block[-1][-1], ordering[0]), *zip(ordering, ordering[1:])]
            if any(before == after or (before, after) in used for before, after in pairs):
                continue
            used.update(pairs)
            block.append(ordering)
            if extend():
                return True
            block.pop()
            used.difference_update(pairs)
        return False

    if not extend():
        raise FloorError(f"no predecessor-balanced order exists for {count} instruments")
    return tuple(block)


def measure_subject(
    *,
    root: Path,
    label: str,
    binaries: Mapping[str, Path],
    instruments: Sequence[Instrument],
    workers: int,
    trials: int,
    warmups: int,
    quiet: bool,
    quiet_wait_seconds: float = QUIET_WAIT_SECONDS,
) -> Dict[str, Any]:
    """Run every instrument over one subject, interleaved, and enforce the oracle.

    Interleaving is not a nicety here. Run sequentially, the first instrument pays the
    page-cache miss for the whole tree and every later one reads a warm cache: measured
    that way on a 76k-entry subject, the *ceiling* came out 1.6x faster than the floor it
    is supposed to sit above. Rounds plus discarded warmups are what make the ratio a
    property of the programs rather than of their order -- and the rounds follow
    `schedule`, so no instrument inherits one predecessor's leftovers every time.
    """
    regime_name = "quiet" if quiet else "uncontrolled"
    trials_by_instrument: Dict[str, List[Trial]] = {item.id: [] for item in instruments}
    oracle: Optional[Dict[str, int]] = None
    oracle_source: Optional[str] = None
    disagreements: List[str] = []
    invalid_trials = 0
    invalid_reasons: List[str] = []

    rounds = schedule(instruments, trials=trials, warmups=warmups)
    with contextlib.ExitStack() as stack:
        regime = _enter_regime(stack, regime_name, quiet_wait_seconds)
        for ordinal, order in rounds:
            warmup = ordinal < 0
            for instrument in order:
                trial = run_cell(
                    instrument,
                    binaries=binaries,
                    root=root,
                    workers=workers,
                    ordinal=ordinal,
                    warmup=warmup,
                    regime=regime,
                )
                if not warmup:
                    trials_by_instrument[instrument.id].append(trial)
                    if not trial.valid:
                        invalid_trials += 1
                        invalid_reasons.extend(
                            reason for reason in trial.reasons if reason not in invalid_reasons
                        )

                # Every trial faces the oracle, not just the first: a subject that changes
                # underneath the run is the failure mode this catches, and it can start at
                # any point. `parfloor enum` contributes only `dirs` -- it makes no
                # metadata call, so it has no byte counts to agree about.
                comparable = {k: v for k, v in trial.tallies.items() if k in ORACLE_KEYS}
                if oracle is None:
                    oracle, oracle_source = dict(comparable), instrument.id
                else:
                    shared = set(comparable) & set(oracle)
                    differing = {k: (oracle[k], comparable[k]) for k in shared
                                 if oracle[k] != comparable[k]}
                    if differing:
                        disagreements.append(
                            f"{instrument.id} trial {ordinal} disagrees with {oracle_source}: "
                            + ", ".join(
                                f"{k} {a} vs {b}" for k, (a, b) in sorted(differing.items())
                            )
                        )
        pressure_after = measure._host_pressure_snapshot(regime)

    results = {
        instrument.id: _summarize(trials_by_instrument[instrument.id], instrument)
        for instrument in instruments
    }
    return {
        "label": label,
        "entries": (oracle or {}).get("dirs", 0) + (oracle or {}).get("files", 0),
        "oracle": oracle,
        "oracle_source": oracle_source,
        "oracle_disagreements": disagreements,
        "workers": workers,
        "trials": trials,
        "warmups": warmups,
        # Every round's order, warmups first, so a reader can see which instrument ran
        # after which rather than trusting that the balance held.
        "schedule": {
            "scheme": SCHEDULE_SCHEME,
            "rounds": [[instrument.id for instrument in order] for _, order in rounds],
        },
        "host_regime": regime_name,
        # Measured trials whose regime gate did not hold, and the distinct reasons why.
        "invalid_trials": invalid_trials,
        "invalid_reasons": invalid_reasons,
        "host_pressure_before": regime.initial,
        "host_pressure_after": pressure_after,
        "instruments": results,
    }


def _summarize(trials: Sequence[Trial], instrument: Instrument) -> Dict[str, Any]:
    elapsed = sorted(trial.elapsed_ns for trial in trials)
    spawn = sorted(trial.spawn_wall_ns for trial in trials)
    rss = [t.max_rss_bytes for t in trials if t.max_rss_bytes is not None]
    median = statistics.median(elapsed) if elapsed else None
    p95 = elapsed[min(len(elapsed) - 1, int(round(0.95 * (len(elapsed) - 1))))] if elapsed else None
    # A median describes a distribution with one hump. `arena_spike` on a shared
    # four-core container had two -- a ~63 ms mode and a ~150 ms mode -- and its median
    # was then whichever mode the run happened to land in more often, reported with a
    # reassuringly tight p95/median because both humps are individually narrow. `spread`
    # is max/min, and it is the cheap tell that a median may not stand for its samples: a
    # unimodal instrument on a quiet host sits near 1.2, and at or past `SPREAD_SUSPECT`
    # the samples span more than one number can summarize.
    #
    # It says that much and no more. It cannot tell a second mode from one wild sample --
    # a single outlier among thirty trips it, and two modes closer than 2x do not -- so
    # the flag is named for what it measures. It compares the ratio itself, not the
    # two-decimal figure the table prints, which would let 1.995 trip a 2.0 bar.
    ratio = elapsed[-1] / elapsed[0] if elapsed and elapsed[0] else None
    return {
        "role": instrument.role,
        "description": instrument.description,
        "samples": len(elapsed),
        "spread": round(ratio, 2) if ratio is not None else None,
        "spread_suspect": ratio is not None and ratio >= SPREAD_SUSPECT,
        "elapsed_ns": {
            "median": median,
            "min": elapsed[0] if elapsed else None,
            "max": elapsed[-1] if elapsed else None,
            "p95": p95,
            # The right tail as a multiple of the middle -- the thing a user waits
            # through, and a pre-registered target of the structural experiment.
            "p95_over_median": round(p95 / median, 3) if median else None,
        },
        # Spawn wall minus the instrument's own timer is harness cost: process startup,
        # argument parsing, JSON rendering. Recorded so a reader can see how much of a
        # spawn-timed number would not have been engine work at all.
        "spawn_wall_ns": {"median": statistics.median(spawn) if spawn else None},
        "harness_overhead_ns": (
            round(statistics.median(spawn) - median) if spawn and median else None
        ),
        "max_rss_bytes": max(rss) if rss else None,
    }


# --------------------------------------------------------------------------------------
# The scoreboard
# --------------------------------------------------------------------------------------

#: Campaign 2's termination thresholds, in x-floor. A tier at or under its threshold on
#: the nominated real subjects is closed for that regime. Kept here beside the arithmetic
#: that produces the ratio so the plan and the scoreboard cannot drift apart silently.
THRESHOLDS = {"aggregate": 1.25, "index": 1.40}

FLOOR_INSTRUMENT = "parfloor-stat"


def score(subject: Mapping[str, Any]) -> Dict[str, Any]:
    """Divide every instrument's median by the floor's, on one subject."""
    instruments = subject["instruments"]
    floor = instruments.get(FLOOR_INSTRUMENT, {}).get("elapsed_ns", {}).get("median")
    if not floor:
        raise FloorError(f"{subject['label']}: no floor measurement to divide by")
    entries = subject["entries"] or 1
    rows = []
    for name, result in instruments.items():
        median = result["elapsed_ns"]["median"]
        if median is None:
            continue
        threshold = THRESHOLDS.get(name)
        ratio = median / floor
        rows.append({
            "instrument": name,
            "role": result["role"],
            "median_ms": round(median / 1e6, 2),
            "x_floor": round(ratio, 3),
            "ns_per_entry": round(median / entries, 1),
            "p95_over_median": result["elapsed_ns"]["p95_over_median"],
            "spread": result["spread"],
            "spread_suspect": result["spread_suspect"],
            "max_rss_bytes": result["max_rss_bytes"],
            "threshold": threshold,
            # A tier is closed when it reaches its threshold. Only tiers have one; the
            # floor, the ceiling and the enum reference are context, not contestants.
            # A tier whose samples spread past SPREAD_SUSPECT is left undecided: its
            # median is whichever mode or outlier it landed near, and that must not
            # close a tier -- or keep one open -- by the luck of a run.
            "meets_threshold": (
                None if not threshold or result["spread_suspect"] else ratio <= threshold
            ),
        })
    rows.sort(key=lambda row: row["x_floor"])
    return {"label": subject["label"], "entries": subject["entries"],
            "floor_ns": floor, "rows": rows}


def render(document: Mapping[str, Any]) -> str:
    """The x-floor table, in the shape the floor report established."""
    lines: List[str] = []
    lines.append("# The floor scoreboard")
    lines.append("")
    lines.append(f"Host: {document['host']['system']} {document['host']['machine']}, "
                 f"{document['host']['logical_cpu_count']} logical CPUs. "
                 f"Every instrument ran a fixed pool of {document['workers']} workers.")
    lines.append(f"Recorded {document['recorded_at']} from commit {document['commit']}.")
    lines.append("")
    regime = f"Regime: **{document['host_regime']}**"
    requested = document.get("host_regime_requested", document["host_regime"])
    if requested != document["host_regime"]:
        reasons = []
        for subject in document["subjects"]:
            reasons.extend(r for r in subject.get("invalid_reasons", []) if r not in reasons)
        breached = document.get("invalid_trials", 0)
        regime += (f" ({requested} was requested, and {breached} measured "
                   f"{'trial' if breached == 1 else 'trials'} breached it: "
                   f"{'; '.join(reasons)}; this table is screening-grade)")
    lines.append(f"{regime}. "
                 f"{document['trials']} trials, {document['warmups']} warmups, interleaved.")
    lines.append("")

    for subject in document["subjects"]:
        scored = subject["scored"]
        lines.append(f"## {scored['label']} — {scored['entries']:,} entries")
        lines.append("")
        if subject["oracle_disagreements"]:
            lines.append("> **The oracle disagreed. These numbers do not compare.**")
            for reason in subject["oracle_disagreements"][:5]:
                lines.append(f"> - {reason}")
            lines.append("")
        lines.append(
            "| Instrument | Role | Median | ×floor | ns/entry | spread | p95/median | Peak RSS |"
        )
        lines.append("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |")
        for row in scored["rows"]:
            rss = (f"{row['max_rss_bytes'] / 1048576:.0f} MiB"
                   if row["max_rss_bytes"] else "—")
            mark = ""
            if row["meets_threshold"] is True:
                mark = f" ✓≤{row['threshold']}"
            elif row["meets_threshold"] is False:
                mark = f" ✗>{row['threshold']}"
            elif row["threshold"]:
                mark = f" ?≤{row['threshold']}"
            spread = (f"{row['spread']:.2f}⚠" if row["spread_suspect"]
                      else f"{row['spread']:.2f}" if row["spread"] else "—")
            lines.append(
                f"| `{row['instrument']}` | {row['role']} | {row['median_ms']:.2f} ms | "
                f"**{row['x_floor']:.2f}**{mark} | {row['ns_per_entry']:.0f} | "
                f"{spread} | {row['p95_over_median'] or '—'} | {rss} |"
            )
        lines.append("")
        if any(row["spread_suspect"] for row in scored["rows"]):
            lines.append(
                f"⚠ max/min at or past {SPREAD_SUSPECT:.0f}×: the samples span more than "
                "the median can stand for -- a second mode or an outlier; the spread "
                "cannot say which. Read that row as a range, and a tier marked ? as "
                "neither closed nor open, or re-run where the cause can be separated."
            )
            lines.append("")
    return "\n".join(lines) + "\n"


def _commit() -> str:
    try:
        return subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"], cwd=str(PROJECT_ROOT),
            capture_output=True, text=True, check=True,
        ).stdout.strip()
    except (subprocess.CalledProcessError, OSError):
        return "unknown"


def run(
    *,
    subjects: Sequence[tuple],
    workers: int,
    trials: int,
    warmups: int,
    build_dir: Optional[Path],
    probe: Path,
    host_regime: str,
    instruments: Sequence[str] = DEFAULT_INSTRUMENTS,
    quiet_wait_seconds: float = QUIET_WAIT_SECONDS,
) -> Dict[str, Any]:
    require_linux()
    if not probe.is_file() or not os.access(probe, os.X_OK):
        raise FloorError(
            f"no executable probe at {probe}. The scoreboard scores the probe "
            "`make perf-probe-release` builds, so that a scoreboard and a verdict run "
            "score one binary; `make perf-floor` builds it first."
        )
    binaries = build_instruments(build_dir if build_dir is not None else default_build_dir())
    binaries["probe"] = probe
    selected = [INSTRUMENTS[name] for name in instruments]
    measured = []
    for label, root in subjects:
        subject = measure_subject(
            root=root, label=label, binaries=binaries, instruments=selected,
            workers=workers, trials=trials, warmups=warmups,
            quiet=(host_regime == "quiet"), quiet_wait_seconds=quiet_wait_seconds,
        )
        subject["scored"] = score(subject)
        measured.append(subject)
    breached = sum(subject["invalid_trials"] for subject in measured)
    return {
        "schema": "fdu-floor-scoreboard-v1",
        "recorded_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "commit": _commit(),
        "host": {
            "system": platform.system(),
            "machine": platform.machine(),
            "logical_cpu_count": os.cpu_count(),
            "kernel": platform.release(),
        },
        # The regime the table can claim rather than the one requested: one measured trial
        # that breached the gate makes the whole table screening-grade.
        "host_regime": "uncontrolled" if breached else host_regime,
        "host_regime_requested": host_regime,
        "invalid_trials": breached,
        "workers": workers,
        "trials": trials,
        "warmups": warmups,
        "subjects": measured,
    }


def main(argv: Sequence[str]) -> int:
    import argparse

    parser = argparse.ArgumentParser(
        prog="benchmarks.realtree.floor", description=__doc__.splitlines()[0]
    )
    parser.add_argument("--subject", action="append", default=[], metavar="LABEL=PATH",
                        help="a tree to score; repeat. Deciding subjects are dense and >=50k entries.")
    parser.add_argument(
        "--workers", type=int, default=None,
        help=("the fixed pool every instrument runs; default: the CPUs this process may "
              f"run on, at most {MAX_WORKERS}"),
    )
    parser.add_argument("--trials", type=int, default=DEFAULT_TRIALS)
    parser.add_argument("--warmups", type=int, default=DEFAULT_WARMUPS)
    parser.add_argument(
        "--probe", type=Path, required=True,
        help="the perf_probe to score: the one `make perf-probe-release` built",
    )
    parser.add_argument(
        "--build-dir", type=Path, default=None,
        help="where to build the spikes; default: fdu-floor under cargo's target directory",
    )
    parser.add_argument("--host-regime", choices=("quiet", "uncontrolled"), default="quiet")
    parser.add_argument(
        "--quiet-wait", type=float, default=QUIET_WAIT_SECONDS, metavar="SECONDS",
        help=("under --host-regime quiet, how long to wait for a settling host before "
              "refusing (default: %(default)s)"),
    )
    parser.add_argument("--output", type=Path, help="write the scoreboard JSON here")
    parser.add_argument("--markdown", type=Path, help="write the rendered table here")
    arguments = parser.parse_args(list(argv))

    workers = arguments.workers if arguments.workers is not None else default_workers()
    if not 1 <= workers <= MAX_WORKERS:
        print(
            f"--workers must be between 1 and {MAX_WORKERS}: fdu clamps its pool at "
            f"{MAX_WORKERS} without saying so, so a larger pool would leave the floor "
            "running more workers than the tiers divided by it",
            file=sys.stderr,
        )
        return 2

    if not arguments.subject:
        print("at least one --subject LABEL=PATH is required", file=sys.stderr)
        return 2

    subjects = []
    for specification in arguments.subject:
        label, separator, path = specification.partition("=")
        if not separator:
            print(f"subject {specification!r} must be LABEL=PATH", file=sys.stderr)
            return 2
        resolved = Path(path).expanduser().resolve()
        if not resolved.is_dir():
            print(f"subject {label!r} is not a directory: {resolved}", file=sys.stderr)
            return 2
        subjects.append((label, resolved))

    try:
        document = run(
            subjects=subjects, workers=workers, trials=arguments.trials,
            warmups=arguments.warmups, build_dir=arguments.build_dir,
            probe=arguments.probe.expanduser().resolve(),
            host_regime=arguments.host_regime, quiet_wait_seconds=arguments.quiet_wait,
        )
    except FloorError as error:
        # A refused scoreboard is the harness doing its job. It should read as a verdict
        # about the run, not as a traceback.
        print(f"floor scoreboard refused: {error}", file=sys.stderr)
        return 1

    text = render(document)
    print(text)
    if arguments.output:
        arguments.output.parent.mkdir(parents=True, exist_ok=True)
        arguments.output.write_text(json.dumps(document, indent=2, sort_keys=True), encoding="utf-8")
        print(f"wrote {arguments.output}", file=sys.stderr)
    if arguments.markdown:
        arguments.markdown.parent.mkdir(parents=True, exist_ok=True)
        arguments.markdown.write_text(text, encoding="utf-8")
        print(f"wrote {arguments.markdown}", file=sys.stderr)

    if any(subject["oracle_disagreements"] for subject in document["subjects"]):
        return 3
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
