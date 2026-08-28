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
  retained paths, no per-entry allocation, no delta contract. Every tier is read against
  it at the same thread count, because a one-thread floor is not a lower bound for a
  parallel walker.
- **`arena_spike`** is the measured ceiling for the representation change: it retains an
  index-shaped result and is what H86 (`fdu-xde5`) is trying to reach. It is a reference
  row, never a denominator.
- **fdu's own tiers**, through `perf_probe`, which is the thing being scored.

`peerwalk` is deliberately absent. It takes third-party dependencies the shipped crate
does not have, and its README says it is never built by `make`; the ecosystem anchor is a
question for the floor *report*, not for a scoreboard that has to run unattended.

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
"""

from __future__ import annotations

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

PROJECT_ROOT = Path(__file__).resolve().parents[3]
SPIKES = PROJECT_ROOT / "explorations" / "benchmarks" / "spikes"

#: The tallies every instrument emits, and the only ones all three agree on.
ORACLE_KEYS = ("dirs", "files", "apparent_bytes", "allocated_bytes")

DEFAULT_TRIALS = 8
DEFAULT_WARMUPS = 2
DEFAULT_TIMEOUT_SECONDS = 900.0

#: Refuse to start above this per-core load. The scoreboard is a ratio of two absolute
#: numbers measured minutes apart, so it is *more* exposed to host drift than a paired
#: comparison, not less: interleaving cancels drift between arms, but nothing cancels a
#: host that was quiet for the denominator and busy for the numerator.
QUIET_LOAD_PER_CPU = 0.25

#: max/min past which a median is summarizing more than one population. See `_summarize`.
SPREAD_SUSPECT = 2.0


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
        argv=("{probe}", "summary", "--root", "{root}"),
        tally_map={"dirs": "dirs", "files": "files", "apparent_bytes": "apparent_bytes",
                   "allocated_bytes": "allocated_bytes"},
        elapsed_key="component_ns",
        payload_key="summary",
    ),
    "index": Instrument(
        id="index",
        role="tier",
        description="fdu index tier: full walk with metadata into a complete index.",
        argv=("{probe}", "scan-index", "--root", "{root}"),
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

#: The order the scoreboard runs and prints them in: denominator first, so a reader of
#: the raw sample list sees the number everything else is divided by before the ratios.
DEFAULT_INSTRUMENTS = ("parfloor-stat", "parfloor-enum", "arena-spike", "aggregate", "index")


# --------------------------------------------------------------------------------------
# Building the instruments
# --------------------------------------------------------------------------------------


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


def build_instruments(destination: Path, *, cargo: str = "cargo") -> Dict[str, Path]:
    """Compile the spikes and the probe, and say which compiler produced each.

    The spikes are single files outside the workspace on purpose -- they take no
    dependencies and are not shipped -- so they are built here rather than by cargo.
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

    # The probe is built exactly as `make perf-probe-release` builds it, so a scoreboard
    # and a verdict run are scoring the same binary shape.
    _run_build(
        [cargo, "build", "--locked", "--release", "-p", "fdu-core",
         "--example", "perf_probe", "--no-default-features"],
        what="perf_probe",
        cwd=PROJECT_ROOT,
    )
    probe = PROJECT_ROOT / "target" / "release" / "examples" / "perf_probe"
    if not probe.is_file():
        raise FloorError(f"perf_probe did not appear at {probe}")
    binaries["probe"] = probe
    return binaries


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
    timeout_seconds: float = DEFAULT_TIMEOUT_SECONDS,
) -> Trial:
    argv = instrument.command(binaries=binaries, root=root, workers=workers)
    outcome = _spawn(argv, timeout_seconds=timeout_seconds)
    parsed = instrument.read(outcome["stdout"])
    return Trial(
        instrument=instrument.id,
        ordinal=ordinal,
        warmup=warmup,
        elapsed_ns=parsed["elapsed_ns"],
        spawn_wall_ns=outcome["spawn_wall_ns"],
        max_rss_bytes=outcome["max_rss_bytes"],
        tallies=parsed["tallies"],
    )


# --------------------------------------------------------------------------------------
# One subject
# --------------------------------------------------------------------------------------


def _host_pressure() -> Dict[str, Any]:
    cpu_count = os.cpu_count() or 1
    try:
        load_1m, load_5m, _ = os.getloadavg()
    except (AttributeError, OSError):
        return {"logical_cpu_count": cpu_count, "load_1m": None, "load_1m_per_cpu": None}
    return {
        "logical_cpu_count": cpu_count,
        "load_1m": round(load_1m, 3),
        "load_5m": round(load_5m, 3),
        "load_1m_per_cpu": round(load_1m / cpu_count, 4),
    }


def _require_quiet(pressure: Mapping[str, Any]) -> None:
    per_cpu = pressure.get("load_1m_per_cpu")
    if per_cpu is not None and per_cpu > QUIET_LOAD_PER_CPU:
        raise FloorError(
            f"host is {per_cpu:.0%} busy per core, over the {QUIET_LOAD_PER_CPU:.0%} bar. "
            "The scoreboard divides two absolute numbers measured minutes apart, so host "
            "drift does not cancel the way it does in a paired run. Wait, or pass "
            "--host-regime uncontrolled to record a screening-grade table."
        )


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
) -> Dict[str, Any]:
    """Run every instrument over one subject, interleaved, and enforce the oracle.

    Interleaving is not a nicety here. Run sequentially, the first instrument pays the
    page-cache miss for the whole tree and every later one reads a warm cache: measured
    that way on a 76k-entry subject, the *ceiling* came out 1.6x faster than the floor it
    is supposed to sit above. Round-robin plus discarded warmups is what makes the ratio
    a property of the programs rather than of their order.
    """
    pressure_before = _host_pressure()
    if quiet:
        _require_quiet(pressure_before)

    trials_by_instrument: Dict[str, List[Trial]] = {item.id: [] for item in instruments}
    oracle: Optional[Dict[str, int]] = None
    oracle_source: Optional[str] = None
    disagreements: List[str] = []

    for ordinal in range(-warmups, trials):
        warmup = ordinal < 0
        for instrument in instruments:
            trial = run_cell(
                instrument,
                binaries=binaries,
                root=root,
                workers=workers,
                ordinal=ordinal,
                warmup=warmup,
            )
            if not warmup:
                trials_by_instrument[instrument.id].append(trial)

            # Every trial faces the oracle, not just the first: a subject that changes
            # underneath the run is the failure mode this catches, and it can start at
            # any point. `parfloor enum` contributes only `dirs` -- it makes no metadata
            # call, so it has no byte counts to agree about.
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
                        + ", ".join(f"{k} {a} vs {b}" for k, (a, b) in sorted(differing.items()))
                    )

    pressure_after = _host_pressure()
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
        "host_pressure_before": pressure_before,
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
    # four-core container has two -- a ~63 ms mode and a ~150 ms mode, selected by how
    # much memory the preceding process churned -- and its median is then whichever mode
    # the run happened to land in more often, reported with a reassuringly tight
    # p95/median because both humps are individually narrow. `spread` is max/min, and it
    # is the cheap tell: a unimodal instrument on a quiet host sits near 1.2, and
    # anything past `SPREAD_SUSPECT` means the median is summarizing two populations and
    # should not be quoted as one number.
    spread = round(elapsed[-1] / elapsed[0], 2) if elapsed and elapsed[0] else None
    return {
        "role": instrument.role,
        "description": instrument.description,
        "samples": len(elapsed),
        "spread": spread,
        "multimodal_suspect": (spread is not None and spread >= SPREAD_SUSPECT),
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
            "multimodal_suspect": result["multimodal_suspect"],
            "max_rss_bytes": result["max_rss_bytes"],
            "threshold": threshold,
            # A tier is closed when it reaches its threshold. Only tiers have one; the
            # floor, the ceiling and the enum reference are context, not contestants.
            "meets_threshold": (ratio <= threshold) if threshold else None,
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
                 f"{document['host']['logical_cpu_count']} logical CPUs, "
                 f"{document['workers']} workers.")
    lines.append(f"Recorded {document['recorded_at']} from commit {document['commit']}.")
    lines.append("")
    lines.append(f"Regime: **{document['host_regime']}**. "
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
            spread = (f"{row['spread']:.2f}⚠" if row["multimodal_suspect"]
                      else f"{row['spread']:.2f}" if row["spread"] else "—")
            lines.append(
                f"| `{row['instrument']}` | {row['role']} | {row['median_ms']:.2f} ms | "
                f"**{row['x_floor']:.2f}**{mark} | {row['ns_per_entry']:.0f} | "
                f"{spread} | {row['p95_over_median'] or '—'} | {rss} |"
            )
        lines.append("")
        if any(row["multimodal_suspect"] for row in scored["rows"]):
            lines.append(
                "⚠ max/min at or past "
                f"{SPREAD_SUSPECT:.0f}×: the median summarizes more than one "
                "population and is not a single number about this instrument. "
                "Read it as a range, or re-run where the modes can be separated."
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
    build_dir: Path,
    host_regime: str,
    instruments: Sequence[str] = DEFAULT_INSTRUMENTS,
) -> Dict[str, Any]:
    require_linux()
    binaries = build_instruments(build_dir)
    selected = [INSTRUMENTS[name] for name in instruments]
    measured = []
    for label, root in subjects:
        subject = measure_subject(
            root=root, label=label, binaries=binaries, instruments=selected,
            workers=workers, trials=trials, warmups=warmups,
            quiet=(host_regime == "quiet"),
        )
        subject["scored"] = score(subject)
        measured.append(subject)
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
        "host_regime": host_regime,
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
    parser.add_argument("--workers", type=int, default=os.cpu_count() or 4)
    parser.add_argument("--trials", type=int, default=DEFAULT_TRIALS)
    parser.add_argument("--warmups", type=int, default=DEFAULT_WARMUPS)
    parser.add_argument("--build-dir", type=Path,
                        default=Path("/tmp/fdu-floor/bin"))
    parser.add_argument("--host-regime", choices=("quiet", "uncontrolled"), default="quiet")
    parser.add_argument("--output", type=Path, help="write the scoreboard JSON here")
    parser.add_argument("--markdown", type=Path, help="write the rendered table here")
    arguments = parser.parse_args(list(argv))

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
            subjects=subjects, workers=arguments.workers, trials=arguments.trials,
            warmups=arguments.warmups, build_dir=arguments.build_dir,
            host_regime=arguments.host_regime,
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
