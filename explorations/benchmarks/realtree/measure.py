"""Measure one or more binaries against an immutable reference tree.

The design constraint that shapes everything here: a comparison between two builds
is only trustworthy if the machine drifted the same way under both. Thermal state,
background load, and page-cache warmth all move on a timescale of seconds to
minutes, which is the same timescale as a benchmark run. Measuring all of A and then
all of B therefore measures the machine as much as the code.

So variants are interleaved trial by trial, and the accept/reject decision is made
on *paired* differences at equal ordinals rather than on the difference of two
independently computed medians.
"""

from __future__ import annotations

import ctypes
import json
import os
import platform
import shutil
import signal
import statistics
import subprocess
import sys
import threading
import time
from contextlib import contextmanager
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence

from benchmarks.realtree import tree as reference_tree

RUN_SCHEMA = "fdu-realtree-run-v1"

#: A measured command inherits nothing from the operator's shell. Locale and timezone
#: change how much work formatting does; an inherited PATH changes which binary runs.
BASE_ENVIRONMENT = {"LANG": "C", "LC_ALL": "C", "TZ": "UTC"}

#: Metrics that are lower-is-better. The report and the accept rule both need to know
#: which direction counts as an improvement.
LOWER_IS_BETTER = (
    "wall_ns",
    "component_ns",
    "cpu_ns",
    "user_cpu_ns",
    "system_cpu_ns",
    "blocked_ns",
    "peak_rss_bytes",
    "major_faults",
    "minor_faults",
    "involuntary_context_switches",
    "voluntary_context_switches",
)

DEFAULT_TRIALS = 12
DEFAULT_WARMUPS = 3
DEFAULT_TIMEOUT_SECONDS = 600.0

# Pre-registered adaptive-controller decision boundaries. Positive percent change is
# worse for every metric here. The runtime noninferiority margin comes from fdu-8slr;
# resource limits are intentionally separate so a faster controller cannot hide an
# unbounded CPU, memory, fault, or scheduling regression behind wall time.
SUPERIORITY_FLOOR_PCT = -3.0
NONINFERIORITY_MARGIN_PCT = 3.0
RESOURCE_REGRESSION_LIMITS_PCT = {
    "cpu_ns": 50.0,
    "system_cpu_ns": 75.0,
    "peak_rss_bytes": 5.0,
    "minor_faults": 10.0,
    "voluntary_context_switches": 50.0,
    "involuntary_context_switches": 50.0,
}
# Major faults are usually zero on a warm-steady run. Relative ratios are undefined at
# zero, so this gate is expressed as an absolute paired-median increase.
MAJOR_FAULT_DELTA_LIMIT = 0.0
QUIET_MAX_LOAD_PER_CPU = 0.25
CONTROLLED_MAX_EXCESS_LOAD_PER_CPU = 0.25
QUIET_MAX_CPU_BUSY_PCT = 25.0
CONTROLLED_MAX_EXCESS_CPU_BUSY_PCT = 25.0
HOST_CPU_BOUNDARY_INTERVAL_SECONDS = 1.0
HOST_REGIMES = {"controlled-interactive", "quiet", "uncontrolled"}


class MeasureError(RuntimeError):
    """A measurement could not be established or completed."""


@dataclass(frozen=True)
class Job:
    """One measurable unit of work, expressed as an argv template.

    ``{root}`` expands to the reference tree, ``{snapshot}`` to a scratch snapshot
    path, and ``{binary}`` to the variant under test. ``needs_snapshot`` marks jobs
    whose scratch snapshot must be built by an untimed preparation run first — that
    build cost belongs to the setup, not to the warm-start measurement.
    """

    id: str
    argv: Sequence[str]
    start_state: str
    description: str
    allowed_exit_codes: Sequence[int] = (0,)
    allow_incomplete: bool = False
    needs_snapshot: bool = False
    verify_oracle: bool = True
    #: The job writes the snapshot itself, so it needs a path but not a prepared
    #: file — and the path must be empty at the start of every trial, or the job
    #: would be measured overwriting rather than creating.
    writes_snapshot: bool = False
    #: Probe mode used by untimed snapshot preparation. Content-cache jobs need both
    #: the metadata snapshot and its independently versioned content sidecar.
    snapshot_preparation_mode: str = "snapshot-save"
    #: Whether the measured process may run more than one thread.
    #:
    #: ``getrusage`` sums CPU across threads, so for a job that walks with the worker
    #: pool or analyses with the content pool, ``wall - cpu`` is not off-CPU time — it
    #: is wall minus aggregate thread CPU, which goes negative as soon as the job keeps
    #: two cores busy. Clamping that to zero publishes a confident "never blocked" for
    #: exactly the jobs whose I/O wait the ledger is trying to see, so those jobs report
    #: no ``blocked_ns`` at all instead.
    parallel_cpu: bool = False
    #: Require the opt-in scan trace and reject incomplete or unobservable policy state.
    require_scan_diagnostics: bool = False
    #: Require macOS bulk/fallback counts rather than the portable null+reason form.
    require_macos_backend: bool = False


@dataclass
class Variant:
    """A binary under test, identified by content rather than by where it sits."""

    name: str
    path: Path
    kind: str = "fdu-probe"
    notes: str = ""
    #: Extra flags appended to every job's argv. This is what lets one binary be
    #: measured under several configurations in a single interleaved run — a thread
    #: count, a batch size — which is both fairer and far quicker than building a
    #: binary per setting.
    extra_args: List[str] = field(default_factory=list)

    def identity(self) -> Dict[str, Any]:
        import hashlib

        digest = hashlib.sha256(self.path.read_bytes()).hexdigest()
        return {
            "args": list(self.extra_args),
            "kind": self.kind,
            "name": self.name,
            "notes": self.notes,
            "sha256": digest,
            "size_bytes": self.path.stat().st_size,
        }


@dataclass
class HostRegime:
    """One predeclared host-pressure cell and its optional controlled load."""

    name: str
    initial: Dict[str, Any]
    load_workers: int = 0
    process: Optional[subprocess.Popen[str]] = None

    def load_alive(self) -> Optional[bool]:
        if self.process is None:
            return None
        return self.process.poll() is None


@dataclass
class Sample:
    variant: str
    job: str
    ordinal: int
    warmup: bool
    valid: bool
    reasons: List[str] = field(default_factory=list)
    metrics: Dict[str, Optional[int]] = field(default_factory=dict)
    probe: Dict[str, Any] = field(default_factory=dict)
    host_pressure: Dict[str, Any] = field(default_factory=dict)

    def as_json(self) -> Dict[str, Any]:
        return {
            "job": self.job,
            "metrics": self.metrics,
            "ordinal": self.ordinal,
            "probe": self.probe,
            "host_pressure": self.host_pressure,
            "reasons": self.reasons,
            "valid": self.valid,
            "variant": self.variant,
            "warmup": self.warmup,
        }


# --------------------------------------------------------------------------------
# Job catalogue
# --------------------------------------------------------------------------------

#: ``start_state`` is the cache condition the job measures, and it is the axis the
#: whole loop is organised around:
#:
#: ``cold``  — no fdu snapshot exists. Everything must come from the filesystem.
#: ``warm``  — a compatible snapshot exists and is revalidated against the tree.
#:
#: Note what this does *not* claim: the operating system page cache is warm in both
#: cases. Dropping it on macOS needs root, so a run that does not opt into
#: ``--purge`` records ``os_cache: "warm-steady"`` and means it.
PROBE_JOBS: Dict[str, Job] = {
    "code-sloc": Job(
        id="code-sloc",
        argv=("{binary}", "code-sloc", "--root", "{root}"),
        start_state="cold",
        description="Analyze common-language code with code-sloc-v1 after metadata setup.",
        allowed_exit_codes=(0, 2),
        allow_incomplete=True,
        parallel_cpu=True,
    ),
    "code-sloc-cache-hit": Job(
        id="code-sloc-cache-hit",
        argv=(
            "{binary}",
            "code-sloc-cache-hit",
            "--root",
            "{root}",
            "--snapshot",
            "{snapshot}",
        ),
        start_state="warm",
        description="Load code-sloc-v1 metrics entirely from compatible sidecars.",
        needs_snapshot=True,
        snapshot_preparation_mode="code-sloc-seed",
    ),
    "content-basic": Job(
        id="content-basic",
        argv=("{binary}", "content-basic", "--root", "{root}"),
        start_state="cold",
        description="Analyze every eligible file with content-basic-v1 after metadata setup.",
        parallel_cpu=True,
    ),
    "content-binary-gate": Job(
        id="content-binary-gate",
        argv=("{binary}", "content-binary-gate", "--root", "{root}"),
        start_state="cold",
        description="Exercise early binary admission on a binary-heavy immutable tree.",
        parallel_cpu=True,
    ),
    "content-cache-hit": Job(
        id="content-cache-hit",
        argv=(
            "{binary}",
            "content-cache-hit",
            "--root",
            "{root}",
            "--snapshot",
            "{snapshot}",
        ),
        start_state="warm",
        description="Load metadata and basic content entirely from compatible sidecars.",
        needs_snapshot=True,
        snapshot_preparation_mode="content-seed",
    ),
    "content-query": Job(
        id="content-query",
        argv=("{binary}", "content-query", "--root", "{root}", "--queries", "100"),
        start_state="warm",
        description="Build type, family, language, and document summaries 100 times.",
        parallel_cpu=True,
    ),
    "content-disabled": Job(
        id="content-disabled",
        argv=("{binary}", "content-disabled", "--root", "{root}"),
        start_state="cold",
        description="Call the disabled content boundary after metadata setup.",
        parallel_cpu=True,
    ),
    "document-cache-hit": Job(
        id="document-cache-hit",
        argv=(
            "{binary}",
            "document-cache-hit",
            "--root",
            "{root}",
            "--snapshot",
            "{snapshot}",
        ),
        start_state="warm",
        description="Load reader-visible document metrics from compatible sidecars.",
        needs_snapshot=True,
        snapshot_preparation_mode="document-seed",
    ),
    "markdown-prose": Job(
        id="markdown-prose",
        argv=("{binary}", "markdown-prose", "--root", "{root}"),
        start_state="cold",
        description=(
            "Analyze Markdown with raw, logical, reader-visible, paragraph, and page "
            "sufficient statistics after metadata setup."
        ),
        parallel_cpu=True,
    ),
    "text-prose": Job(
        id="text-prose",
        argv=("{binary}", "text-prose", "--root", "{root}"),
        start_state="cold",
        description=(
            "Analyze plain text with raw and normalized words, paragraphs, and page "
            "sufficient statistics after metadata setup."
        ),
        parallel_cpu=True,
    ),
    "cold-scan-index": Job(
        id="cold-scan-index",
        argv=("{binary}", "scan-index", "--root", "{root}"),
        start_state="cold",
        description="Full walk with metadata into a complete index. No snapshot.",
        parallel_cpu=True,
    ),
    "cold-scan-producer": Job(
        id="cold-scan-producer",
        argv=("{binary}", "scan-producer", "--root", "{root}"),
        start_state="cold",
        description=(
            "Walk and metadata only, no index build. Isolates the syscall layer. "
            "Wall time includes an untimed exact validation scan; read component_ns."
        ),
        parallel_cpu=True,
    ),
    "adaptive-scan-index": Job(
        id="adaptive-scan-index",
        argv=("{binary}", "scan-index", "--root", "{root}", "--diagnostics"),
        start_state="cold",
        description=(
            "Full Apple Silicon/APFS policy-qualification walk with a bounded, "
            "fail-closed worker and directory-backend trace."
        ),
        parallel_cpu=True,
        require_scan_diagnostics=True,
        require_macos_backend=True,
    ),
    "warm-revalidate": Job(
        id="warm-revalidate",
        argv=("{binary}", "revalidate", "--root", "{root}", "--snapshot", "{snapshot}"),
        start_state="warm",
        description=(
            "Load a compatible snapshot and reconcile it against the tree. "
            "component_ns is reconciliation; wall_ns is the whole warm start."
        ),
        needs_snapshot=True,
        parallel_cpu=True,
    ),
    "warm-snapshot-load": Job(
        id="warm-snapshot-load",
        argv=(
            "{binary}",
            "snapshot-load",
            "--root",
            "{root}",
            "--snapshot",
            "{snapshot}",
        ),
        start_state="warm",
        description="Deserialize a snapshot into a usable index. No filesystem walk.",
        needs_snapshot=True,
    ),
    "cold-open-save": Job(
        id="cold-open-save",
        argv=(
            "{binary}",
            "cold-open-save",
            "--root",
            "{root}",
            "--snapshot",
            "{snapshot}",
        ),
        start_state="cold",
        description=(
            "A cold scan that also writes its cache, through the real open path: "
            "index build, the writer hand-off, and the join a one-shot caller makes "
            "before exiting. The shape of `fdu --cache refresh`."
        ),
        writes_snapshot=True,
        parallel_cpu=True,
    ),
    "cold-snapshot-save": Job(
        id="cold-snapshot-save",
        argv=(
            "{binary}",
            "snapshot-save",
            "--root",
            "{root}",
            "--snapshot",
            "{snapshot}",
        ),
        start_state="cold",
        description="Serialize a populated index. Wall includes the untimed setup scan.",
        writes_snapshot=True,
        parallel_cpu=True,
    ),
}

#: Reference tools are context, not a verdict. They answer a different question with
#: different guarantees, so their numbers live in their own table and never enter the
#: accept/reject rule. ``dust`` and ``gdu`` compute a size roll-up like fdu's cold
#: scan; ``du`` is the platform baseline everyone already has.
REFERENCE_JOBS: Dict[str, Job] = {
    "reference-rollup": Job(
        id="reference-rollup",
        argv=("{binary}", "{root}"),
        start_state="cold",
        description="Third-party whole-tree size roll-up, for context only.",
        verify_oracle=False,
        parallel_cpu=True,
    ),
}

REFERENCE_ARGV: Dict[str, Sequence[str]] = {
    "dust": ("{binary}", "-d", "1", "--no-progress", "{root}"),
    "gdu": ("{binary}", "--non-interactive", "--show-apparent-size", "{root}"),
    "du": ("{binary}", "-s", "-k", "{root}"),
}


# --------------------------------------------------------------------------------
# Running
# --------------------------------------------------------------------------------


@contextmanager
def _host_regime(name: str, load_workers: int):
    initial = _host_pressure_snapshot(None)
    if name in {"quiet", "controlled-interactive"}:
        if initial.get("system") == "Darwin":
            cpu_busy = initial.get("cpu_busy_pct")
            if not isinstance(cpu_busy, (int, float)):
                raise MeasureError(
                    "instantaneous CPU pressure is unavailable for a controlled macOS regime"
                )
            if cpu_busy > QUIET_MAX_CPU_BUSY_PCT:
                raise MeasureError(
                    "host is not quiet enough to begin a controlled cell: "
                    f"CPU busy {cpu_busy:.1f}% > {QUIET_MAX_CPU_BUSY_PCT:.1f}%"
                )
        else:
            load_per_cpu = initial.get("load_1m_per_cpu")
            if not isinstance(load_per_cpu, (int, float)):
                raise MeasureError("host load is unavailable for a controlled evidence regime")
            if load_per_cpu > QUIET_MAX_LOAD_PER_CPU:
                raise MeasureError(
                    "host is not quiet enough to begin a controlled cell: "
                    f"load/core {load_per_cpu:.3f} > {QUIET_MAX_LOAD_PER_CPU:.3f}"
                )

    regime = HostRegime(name=name, initial=initial)
    if name != "controlled-interactive":
        yield regime
        return
    if load_workers < 1 or load_workers > 64:
        raise MeasureError("controlled background workers must be between 1 and 64")
    environment = dict(BASE_ENVIRONMENT)
    environment["PATH"] = os.environ.get("PATH", "")
    # The import root for the `benchmarks` package, which is the directory holding it
    # (`explorations/`) rather than the repository root. Those coincided before the
    # harness moved under `explorations/`; they do not now.
    environment["PYTHONPATH"] = str(Path(__file__).resolve().parents[2])
    process = subprocess.Popen(
        [
            sys.executable,
            "-m",
            "benchmarks.realtree.load",
            "--workers",
            str(load_workers),
        ],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        env=environment,
        start_new_session=os.name == "posix",
    )
    try:
        ready = process.stdout.readline().strip() if process.stdout is not None else ""
        if ready != "ready" or process.poll() is not None:
            raise MeasureError("controlled background load failed to become ready")
        regime.load_workers = load_workers
        regime.process = process
        yield regime
    finally:
        if process.poll() is None:
            if os.name == "posix":
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
            else:
                process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            if os.name == "posix":
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
            else:
                process.kill()
            process.wait(timeout=10)
        if process.stdout is not None:
            process.stdout.close()


def _host_pressure_snapshot(regime: Optional[HostRegime]) -> Dict[str, Any]:
    cpu_count = os.cpu_count() or 1
    try:
        load_1m, load_5m, load_15m = os.getloadavg()
        load_unavailable_reason = None
    except (AttributeError, OSError):
        load_1m = load_5m = load_15m = None
        load_unavailable_reason = "load average is unavailable on this platform"
    cpu_busy_pct, cpu_busy_unavailable_reason = _darwin_cpu_busy_pct()
    unavailable = [
        reason
        for reason in (load_unavailable_reason, cpu_busy_unavailable_reason)
        if reason is not None
    ]
    return {
        "controlled_load_alive": regime.load_alive() if regime is not None else None,
        "cpu_busy_pct": cpu_busy_pct,
        "cpu_busy_unavailable_reason": cpu_busy_unavailable_reason,
        "logical_cpu_count": cpu_count,
        "load_1m": round(load_1m, 3) if load_1m is not None else None,
        "load_1m_per_cpu": round(load_1m / cpu_count, 4) if load_1m is not None else None,
        "load_5m": round(load_5m, 3) if load_5m is not None else None,
        "load_15m": round(load_15m, 3) if load_15m is not None else None,
        "power_source": _darwin_power_source(),
        "system": platform.system(),
        "thermal_pressure": _darwin_thermal_pressure(),
        "unavailable_reason": "; ".join(unavailable) if unavailable else None,
    }


def _darwin_cpu_busy_pct() -> tuple[Optional[float], Optional[str]]:
    """Measure current CPU occupancy, avoiding load average's minute-long memory.

    A fixed-N benchmark creates runnable workers by design. Darwin's load average keeps
    counting that work after the child exits, so it cannot distinguish later background
    pressure from the benchmark's own previous samples. A one-second Mach host-counter
    delta observes a short boundary interval after the child is gone; the lagging load
    averages remain in the artifact as context but do not accept or reject macOS
    samples.
    """
    first, reason = _darwin_cpu_ticks()
    if first is None:
        return None, reason
    time.sleep(HOST_CPU_BOUNDARY_INTERVAL_SECONDS)
    second, reason = _darwin_cpu_ticks()
    if second is None:
        return None, reason
    deltas = [(after - before) & 0xFFFF_FFFF for before, after in zip(first, second, strict=True)]
    total = sum(deltas)
    if total == 0:
        return None, "Mach CPU counters did not advance during the boundary interval"
    user, system, idle, nice = deltas
    busy = (user + system + nice) / total * 100.0
    return round(busy, 2), None


def _darwin_cpu_ticks() -> tuple[Optional[tuple[int, int, int, int]], Optional[str]]:
    """Read Darwin's user/system/idle/nice host tick counters."""
    if sys.platform != "darwin":
        return None, "instantaneous CPU occupancy is currently collected only on macOS"

    class HostCpuLoadInfo(ctypes.Structure):
        _fields_ = [
            ("user", ctypes.c_uint),
            ("system", ctypes.c_uint),
            ("idle", ctypes.c_uint),
            ("nice", ctypes.c_uint),
        ]

    try:
        system_library = ctypes.CDLL("/usr/lib/libSystem.B.dylib")
        system_library.mach_host_self.restype = ctypes.c_uint
        system_library.host_statistics.argtypes = [
            ctypes.c_uint,
            ctypes.c_int,
            ctypes.POINTER(ctypes.c_int),
            ctypes.POINTER(ctypes.c_uint),
        ]
        system_library.host_statistics.restype = ctypes.c_int
        info = HostCpuLoadInfo()
        count = ctypes.c_uint(ctypes.sizeof(info) // ctypes.sizeof(ctypes.c_int))
        result = system_library.host_statistics(
            system_library.mach_host_self(),
            3,  # HOST_CPU_LOAD_INFO
            ctypes.cast(ctypes.byref(info), ctypes.POINTER(ctypes.c_int)),
            ctypes.byref(count),
        )
    except (AttributeError, OSError) as error:
        return None, f"Mach CPU counters are unavailable: {type(error).__name__}"
    if result != 0 or count.value < 4:
        return None, f"host_statistics failed with code {result}"
    return (info.user, info.system, info.idle, info.nice), None


def _host_pressure_reasons(
    regime: HostRegime, before: Mapping[str, Any], after: Mapping[str, Any]
) -> List[str]:
    if regime.name == "uncontrolled":
        return []
    environmental_reasons: List[str] = []
    for boundary, snapshot in (("before", before), ("after", after)):
        if snapshot.get("system") != "Darwin":
            continue
        if snapshot.get("power_source") != "AC":
            environmental_reasons.append(f"AC power was not established {boundary} the sample")
        if snapshot.get("thermal_pressure") != "normal":
            environmental_reasons.append(
                f"normal thermal pressure was not established {boundary} the sample"
            )
    if regime.name == "controlled-interactive":
        reasons = list(environmental_reasons)
        if (
            before.get("controlled_load_alive") is not True
            or after.get("controlled_load_alive") is not True
        ):
            reasons.append("controlled background load was not alive for the whole sample")
        for boundary, snapshot in (("before", before), ("after", after)):
            cpu_count = snapshot.get("logical_cpu_count")
            if snapshot.get("system") == "Darwin":
                value = snapshot.get("cpu_busy_pct")
                if not isinstance(value, (int, float)) or not isinstance(cpu_count, int):
                    reasons.append(
                        f"controlled-host CPU occupancy was unavailable {boundary} the sample"
                    )
                    continue
                maximum = regime.load_workers / cpu_count * 100 + CONTROLLED_MAX_EXCESS_CPU_BUSY_PCT
                if value > maximum:
                    reasons.append(
                        "controlled-host CPU busy exceeded the synthetic-load envelope "
                        f"{maximum:.1f}% {boundary} the sample"
                    )
            else:
                value = snapshot.get("load_1m_per_cpu")
                if not isinstance(value, (int, float)) or not isinstance(cpu_count, int):
                    reasons.append(f"controlled-host load was unavailable {boundary} the sample")
                    continue
                maximum = regime.load_workers / cpu_count + CONTROLLED_MAX_EXCESS_LOAD_PER_CPU
                if value > maximum:
                    reasons.append(
                        "controlled-host load/core exceeded the synthetic-load envelope "
                        f"{maximum:.3f} {boundary} the sample"
                    )
        return reasons
    reasons = list(environmental_reasons)
    for boundary, snapshot in (("before", before), ("after", after)):
        if snapshot.get("system") == "Darwin":
            value = snapshot.get("cpu_busy_pct")
            if not isinstance(value, (int, float)):
                reasons.append(f"quiet-host CPU occupancy was unavailable {boundary} the sample")
            elif value > QUIET_MAX_CPU_BUSY_PCT:
                reasons.append(
                    f"quiet-host CPU busy exceeded {QUIET_MAX_CPU_BUSY_PCT:.1f}% "
                    f"{boundary} the sample"
                )
        else:
            value = snapshot.get("load_1m_per_cpu")
            if not isinstance(value, (int, float)):
                reasons.append(f"quiet-host load was unavailable {boundary} the sample")
            elif value > QUIET_MAX_LOAD_PER_CPU:
                reasons.append(
                    f"quiet-host load/core exceeded {QUIET_MAX_LOAD_PER_CPU:.3f} "
                    f"{boundary} the sample"
                )
    return reasons


def run(
    *,
    root: Path,
    label: str,
    variants: Sequence[Variant],
    jobs: Sequence[Job],
    trials: int = DEFAULT_TRIALS,
    warmups: int = DEFAULT_WARMUPS,
    scratch: Path,
    baseline_fingerprint: Optional[Dict[str, Any]] = None,
    purge: bool = False,
    note: str = "",
    campaign_stage: str = "exploratory",
    corpus: Optional[Dict[str, Any]] = None,
    host_regime: str = "uncontrolled",
    background_load_workers: int = 2,
    timeout_seconds: float = DEFAULT_TIMEOUT_SECONDS,
    provenance_document: Optional[Mapping[str, Any]] = None,
) -> Dict[str, Any]:
    """Measure every ``variant`` against every ``job``, interleaved, and return a run."""
    if not variants:
        raise MeasureError("at least one variant is required")
    if not jobs:
        raise MeasureError("at least one job is required")
    if trials < 1:
        raise MeasureError("trials must be at least 1")
    if host_regime not in HOST_REGIMES:
        raise MeasureError(f"unknown host regime {host_regime!r}")
    if campaign_stage == "held-out" and host_regime == "uncontrolled":
        raise MeasureError(
            "held-out evidence requires a quiet or controlled-interactive host regime"
        )
    if campaign_stage == "held-out" and (
        provenance_document is None or provenance_document.get("claim_grade") is not True
    ):
        raise MeasureError("held-out evidence requires claim-grade build and host provenance")
    if campaign_stage == "held-out":
        # Imported lazily because provenance itself uses this module's portable host
        # collectors. At execution time both modules are fully initialized.
        from benchmarks.realtree import provenance as provenance_contract

        scope_reasons = provenance_contract.apple_silicon_apfs_scope_reasons(provenance_document)
        if scope_reasons:
            raise MeasureError(
                "held-out Apple Silicon/APFS scope is not established: " + "; ".join(scope_reasons)
            )

    scratch.mkdir(parents=True, exist_ok=True)
    started = time.time()

    before = reference_tree.fingerprint(root, label=label)
    if corpus is not None and (
        (corpus.get("oracle") or {}).get("engine_digest") != before["engine_digest"]
    ):
        raise MeasureError("generated-corpus oracle disagrees with the measured tree")
    phase_profile = ((corpus or {}).get("recipe") or {}).get("phase_profile")
    drift = (
        reference_tree.compare(before, baseline_fingerprint)
        if baseline_fingerprint is not None
        else []
    )

    snapshots = _prepare_snapshots(root, variants, jobs, scratch, timeout_seconds)

    samples: List[Sample] = []
    schedule = _interleave(variants, jobs, trials, warmups)
    total = len(schedule)
    with _host_regime(host_regime, background_load_workers) as regime:
        for position, (variant, job, ordinal, warmup) in enumerate(schedule, start=1):
            if purge:
                _purge_page_cache()
            sample = _measure_once(
                variant=variant,
                job=job,
                root=root,
                snapshot=snapshots.get((variant.name, job.id)),
                ordinal=ordinal,
                warmup=warmup,
                fingerprint_document=before,
                expected_phase_profile=phase_profile,
                host_regime=regime,
                timeout_seconds=timeout_seconds,
            )
            samples.append(sample)
            _progress(position, total, sample)
        final_host_pressure = _host_pressure_snapshot(regime)

    after = reference_tree.fingerprint(root, label=label)
    mutation = reference_tree.compare(after, before)
    global_reasons: List[str] = []
    if drift:
        global_reasons.append("the measured tree disagreed with its preregistered baseline")
    if mutation:
        global_reasons.append("the measured tree changed during the run")
    if global_reasons:
        for sample in samples:
            sample.valid = False
            for reason in global_reasons:
                if reason not in sample.reasons:
                    sample.reasons.append(reason)

    document = {
        "schema": RUN_SCHEMA,
        "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(started)),
        "duration_seconds": round(time.time() - started, 3),
        "note": note,
        "host": host_facts(),
        "conditions": {
            "campaign_stage": campaign_stage,
            "confidence_interval": "paired-bootstrap-median-95-v1",
            "os_cache": "purged-per-trial" if purge else "warm-steady",
            "stopping_rule": "fixed-N-no-optional-stopping-v1",
            "trials": trials,
            "warmups": warmups,
            "interleaved": True,
            "host_regime": host_regime,
            "host_acceptance": {
                "controlled_load_workers": regime.load_workers,
                "initial": regime.initial,
                "final": final_host_pressure,
                "quiet_max_load_per_cpu": QUIET_MAX_LOAD_PER_CPU,
                "controlled_max_excess_load_per_cpu": (CONTROLLED_MAX_EXCESS_LOAD_PER_CPU),
                "quiet_max_cpu_busy_pct": QUIET_MAX_CPU_BUSY_PCT,
                "controlled_max_excess_cpu_busy_pct": (CONTROLLED_MAX_EXCESS_CPU_BUSY_PCT),
            },
            "schedule": "round-robin-by-ordinal-v1",
        },
        "tree": before,
        "tree_after_digest": after["engine_digest"],
        "tree_mutated_during_run": mutation,
        "baseline_drift": drift,
        # The order variants were declared in, which is what makes the first one the
        # control. `variants` is a mapping, and the run document is written with sorted
        # keys so it diffs cleanly, so the declaration order has to be recorded
        # separately or a reader recovers it alphabetically and inverts the comparison.
        "variant_order": [variant.name for variant in variants],
        "variants": {variant.name: variant.identity() for variant in variants},
        "jobs": {
            job.id: {
                "argv": list(job.argv),
                "description": job.description,
                "require_macos_backend": job.require_macos_backend,
                "require_scan_diagnostics": job.require_scan_diagnostics,
                "start_state": job.start_state,
            }
            for job in jobs
        },
        "samples": [sample.as_json() for sample in samples],
    }
    if provenance_document is not None:
        document["provenance"] = dict(provenance_document)
    document["invalid_samples"] = sum(
        1 for sample in samples if not sample.warmup and not sample.valid
    )
    document["statistics"] = summarize(document)
    if corpus is not None:
        document["corpus"] = corpus
    return document


def _prepare_snapshots(
    root: Path,
    variants: Sequence[Variant],
    jobs: Sequence[Job],
    scratch: Path,
    timeout_seconds: float,
) -> Dict[tuple, Path]:
    """Build one scratch snapshot per (variant, job) that needs one.

    The snapshot is written with the same binary that will read it, because a
    snapshot is invalidated by an engine fingerprint and a candidate build may not
    accept a baseline build's file. This preparation is untimed and happens once, not
    per trial: a warm start measures reading a snapshot that already exists.
    """
    snapshots: Dict[tuple, Path] = {}
    for variant in variants:
        for job in jobs:
            if not (job.needs_snapshot or job.writes_snapshot):
                continue
            path = scratch / f"snapshot-{variant.name}-{job.id}.fdu"
            if job.writes_snapshot:
                # The job creates this itself; hand it a name and nothing else.
                path.unlink(missing_ok=True)
                snapshots[(variant.name, job.id)] = path
                continue
            if path.exists():
                path.unlink()
            argv = _expand(
                (
                    "{binary}",
                    job.snapshot_preparation_mode,
                    "--root",
                    "{root}",
                    "--snapshot",
                    "{snapshot}",
                ),
                binary=variant.path,
                root=root,
                snapshot=path,
                extra=variant.extra_args,
            )
            result = _spawn(argv, timeout_seconds=timeout_seconds)
            if result["exit_code"] != 0 or not path.is_file():
                raise MeasureError(
                    f"could not prepare a snapshot for {variant.name}/{job.id}: "
                    f"exit {result['exit_code']} {result['stderr'][:400]}"
                )
            snapshots[(variant.name, job.id)] = path
    return snapshots


def _interleave(
    variants: Sequence[Variant],
    jobs: Sequence[Job],
    trials: int,
    warmups: int,
) -> List[tuple]:
    """Round-robin variants inside each ordinal so drift lands on both equally.

    Jobs stay grouped because switching job also switches the page-cache working set,
    and paying that transition once per ordinal would add variance to every sample.
    Within an ordinal the variant order alternates, so neither variant is permanently
    the one that runs first.
    """
    schedule: List[tuple] = []
    for job in jobs:
        for ordinal in range(-warmups, trials):
            order = list(variants)
            if ordinal % 2 == 1:
                order.reverse()
            for variant in order:
                schedule.append((variant, job, ordinal, ordinal < 0))
    return schedule


def add_cpu_metrics(metrics: Dict[str, Optional[int]], *, wall_ns: int, parallel_cpu: bool) -> None:
    """Derive ``cpu_ns`` and, where it means anything, ``blocked_ns``.

    Off-CPU time is ``wall - cpu`` only for a process that ran one thread. ``getrusage``
    sums CPU across threads, so for a parallel job the difference is wall minus
    aggregate thread CPU: negative as soon as two cores stay busy, and clamping it to
    zero would publish "never blocked" for exactly the jobs whose I/O wait matters. Such
    jobs report no blocked time rather than a fabricated one.
    """
    user = metrics.get("user_cpu_ns")
    system = metrics.get("system_cpu_ns")
    if user is None or system is None:
        metrics["cpu_ns"] = None
        metrics["blocked_ns"] = None
        return

    cpu_ns = user + system
    metrics["cpu_ns"] = cpu_ns
    # CPU above wall proves a second thread whatever the job declared, so a probe mode
    # that quietly becomes parallel stops reporting blocked time instead of reporting
    # zero.
    metrics["blocked_ns"] = None if parallel_cpu or cpu_ns > wall_ns else wall_ns - cpu_ns


def _measure_once(
    *,
    variant: Variant,
    job: Job,
    root: Path,
    snapshot: Optional[Path],
    ordinal: int,
    warmup: bool,
    fingerprint_document: Dict[str, Any],
    expected_phase_profile: Optional[str],
    host_regime: HostRegime,
    timeout_seconds: float,
) -> Sample:
    if job.writes_snapshot and snapshot is not None:
        snapshot.unlink(missing_ok=True)
    argv = _expand(
        job.argv,
        binary=variant.path,
        root=root,
        snapshot=snapshot,
        extra=variant.extra_args,
    )
    pressure_before = _host_pressure_snapshot(host_regime)
    result = _spawn(argv, timeout_seconds=timeout_seconds)
    pressure_after = _host_pressure_snapshot(host_regime)

    reasons: List[str] = []
    if result["timed_out"]:
        reasons.append("command timed out")
    if result["exit_code"] not in job.allowed_exit_codes:
        reasons.append(f"command exited with {result['exit_code']}")
    reasons.extend(_host_pressure_reasons(host_regime, pressure_before, pressure_after))

    probe: Dict[str, Any] = {}
    component_ns: Optional[int] = None
    if variant.kind == "fdu-probe":
        probe, probe_reasons = _read_probe_output(
            result["stdout"],
            allow_incomplete=job.allow_incomplete,
            require_scan_diagnostics=job.require_scan_diagnostics,
            require_macos_backend=job.require_macos_backend,
            expected_phase_profile=expected_phase_profile,
        )
        reasons.extend(probe_reasons)
        component_ns = probe.get("component_ns")
        if job.verify_oracle and probe:
            disagreement = reference_tree.probe_agrees(fingerprint_document, probe.get("summary"))
            if disagreement is not None:
                reasons.append(disagreement)

    metrics = dict(result["resources"])
    metrics["wall_ns"] = result["wall_ns"]
    metrics["component_ns"] = component_ns
    add_cpu_metrics(metrics, wall_ns=result["wall_ns"], parallel_cpu=job.parallel_cpu)

    return Sample(
        variant=variant.name,
        job=job.id,
        ordinal=ordinal,
        warmup=warmup,
        valid=not reasons,
        reasons=reasons,
        metrics=metrics,
        host_pressure={"after": pressure_after, "before": pressure_before},
        probe={
            key: value
            for key, value in probe.items()
            if key
            in {
                "component_ns",
                "attribution",
                "mode",
                "scan_diagnostics",
                "source",
                "summary",
            }
        },
    )


def _read_probe_output(
    stdout: bytes,
    *,
    allow_incomplete: bool = False,
    require_scan_diagnostics: bool = False,
    require_macos_backend: bool = False,
    expected_phase_profile: Optional[str] = None,
):
    reasons: List[str] = []
    text = stdout.decode("utf-8", errors="replace").strip()
    if not text:
        return {}, ["probe produced no output"]
    try:
        document = json.loads(text.splitlines()[-1])
    except json.JSONDecodeError as error:
        return {}, [f"probe output was not JSON: {error}"]
    if not isinstance(document, dict):
        return {}, ["probe output was not a JSON object"]
    if document.get("schema") != "fdu-perf-probe-v1":
        reasons.append(f"unexpected probe schema {document.get('schema')!r}")
    summary = document.get("summary")
    if isinstance(summary, dict):
        if summary.get("complete") is not True and not allow_incomplete:
            reasons.append("probe reported an incomplete traversal")
        if summary.get("errors"):
            reasons.append(f"probe reported {summary['errors']} traversal errors")
    if require_scan_diagnostics:
        reasons.extend(
            _validate_scan_diagnostics(
                document.get("scan_diagnostics"),
                require_macos_backend=require_macos_backend,
                expected_dirs_read=summary.get("dirs_read") if isinstance(summary, dict) else None,
                expected_phase_profile=expected_phase_profile,
            )
        )
    return document, reasons


def _validate_scan_diagnostics(
    diagnostics: Any,
    *,
    require_macos_backend: bool,
    expected_dirs_read: Any = None,
    expected_phase_profile: Optional[str] = None,
) -> List[str]:
    """Reject traces that cannot support a policy claim.

    Timing without the policy path that produced it describes a mixture, not a
    controller. This validator is intentionally stricter than the probe parser: the
    probe may emit an honest null or undecided result, while a claim-grade adaptive
    job must treat that honesty as an inconclusive sample rather than filling gaps.
    """
    reasons: List[str] = []
    if not isinstance(diagnostics, dict):
        return ["required scan diagnostics are missing"]
    if diagnostics.get("schema") != "fdu-scan-diagnostics-v1":
        reasons.append(f"unexpected scan diagnostics schema {diagnostics.get('schema')!r}")
    policy = diagnostics.get("worker_policy")
    backend = diagnostics.get("backend")
    if not isinstance(policy, dict):
        reasons.append("scan diagnostics worker_policy is missing")
    else:
        controller = policy.get("controller")
        if controller not in {
            "shipped_one_shot",
            "repeated_windows",
            "staged_gated_windows",
        }:
            reasons.append(f"unknown adaptive worker controller {controller!r}")
        if policy.get("events_truncated") is not False:
            reasons.append("scan diagnostics policy trace is truncated or unspecified")
        for field in (
            "ready_directories_at_finish",
            "in_flight_directories_at_finish",
            "handoff_backlog_at_finish",
        ):
            if policy.get(field) != 0:
                reasons.append(f"scan diagnostics ended with nonzero {field}")
        outcome = policy.get("outcome")
        if outcome in {None, "not_run", "undecided"}:
            reasons.append(f"adaptive worker policy was not observable: {outcome!r}")
        elif outcome not in {
            "fixed",
            "held",
            "scaled_up",
            "held_no_useful_work",
        }:
            reasons.append(f"unknown adaptive worker outcome {outcome!r}")
        windows = policy.get("windows")
        available = policy.get("available_parallelism")
        initial = policy.get("initial_workers")
        maximum = policy.get("maximum_workers")
        if (
            not all(isinstance(value, int) and value > 0 for value in (available, initial, maximum))
            or initial > maximum
        ):
            reasons.append("scan diagnostics worker configuration is inconsistent")
        calibration_window = policy.get("calibration_window_entries")
        slow_threshold = policy.get("slow_threshold_ns_per_entry")
        if outcome == "fixed":
            if calibration_window is not None or slow_threshold is not None:
                reasons.append("fixed worker policy unexpectedly reported adaptive thresholds")
        elif not (
            isinstance(calibration_window, int)
            and calibration_window > 0
            and isinstance(slow_threshold, int)
            and slow_threshold > 0
        ):
            reasons.append("adaptive worker thresholds are missing")
        calibration_chunks = policy.get("calibration_chunks")
        calibration_entries = policy.get("calibration_entries")
        calibration_work_ns = policy.get("calibration_work_ns")
        worker_expansions = policy.get("worker_expansions")
        aggregates = (
            calibration_chunks,
            calibration_entries,
            calibration_work_ns,
            worker_expansions,
        )
        if not all(isinstance(value, int) and value >= 0 for value in aggregates):
            reasons.append("scan diagnostics calibration aggregates are missing")
        if not isinstance(windows, list):
            reasons.append("scan diagnostics policy windows are missing")
        else:
            previous_end = 0
            traced_entries = 0
            traced_chunks = 0
            traced_work_ns = 0
            scale_decisions = 0
            previous_target = initial if isinstance(initial, int) else 0
            for sequence, window in enumerate(windows):
                if not isinstance(window, dict):
                    reasons.append(f"scan diagnostics policy window {sequence} is malformed")
                    continue
                if window.get("sequence") != sequence:
                    reasons.append("scan diagnostics policy window sequence is not contiguous")
                start = window.get("start_entry_ordinal")
                end = window.get("end_entry_ordinal")
                if not isinstance(start, int) or not isinstance(end, int) or end < start:
                    reasons.append(
                        f"scan diagnostics policy window {sequence} has invalid ordinals"
                    )
                elif start < previous_end or end < previous_end:
                    reasons.append("scan diagnostics policy windows overlap or moved backward")
                else:
                    previous_end = end
                decision = window.get("decision")
                if decision not in {
                    "undecided",
                    "hold",
                    "scale_up",
                    "hold_no_useful_work",
                    "hold_insufficient_frontier",
                    "hold_handoff_backlog",
                    "observe_fast",
                    "observe_slow",
                    "observe_incomplete",
                    "incomplete",
                }:
                    reasons.append(
                        f"scan diagnostics policy window {sequence} has unknown decision"
                    )
                requested = window.get("requested_workers")
                if decision == "scale_up" and not isinstance(requested, int):
                    reasons.append("scan diagnostics scale decision has no worker target")
                elif decision == "scale_up" and isinstance(requested, int):
                    if not (
                        requested > previous_target
                        and isinstance(maximum, int)
                        and requested <= maximum
                    ):
                        reasons.append("scan diagnostics scale target is not strictly bounded")
                    previous_target = max(previous_target, requested)
                elif decision != "scale_up" and requested is not None:
                    reasons.append("non-scale policy decision requested workers")
                observed_entries = window.get("observed_entries")
                observed_chunks = window.get("observed_chunks")
                observed_work_ns = window.get("observed_work_ns")
                if not isinstance(observed_entries, int) or observed_entries < 0:
                    reasons.append(
                        f"scan diagnostics policy window {sequence} has invalid entry work"
                    )
                elif (
                    isinstance(start, int)
                    and isinstance(end, int)
                    and (end - start != observed_entries)
                ):
                    reasons.append(
                        f"scan diagnostics policy window {sequence} entry ordinals disagree"
                    )
                if not isinstance(observed_work_ns, int) or observed_work_ns < 0:
                    reasons.append(
                        f"scan diagnostics policy window {sequence} has invalid timed work"
                    )
                if not isinstance(observed_chunks, int) or observed_chunks < 0:
                    reasons.append(
                        f"scan diagnostics policy window {sequence} has invalid chunk count"
                    )
                derived_work = window.get("work_ns_per_entry")
                unavailable = window.get("work_ns_per_entry_unavailable_reason")
                if isinstance(observed_entries, int) and observed_entries > 0:
                    expected_work = (
                        observed_work_ns // observed_entries
                        if isinstance(observed_work_ns, int)
                        else None
                    )
                    if derived_work != expected_work or unavailable is not None:
                        reasons.append(
                            f"scan diagnostics policy window {sequence} has an inconsistent signal"
                        )
                    if isinstance(slow_threshold, int) and isinstance(derived_work, int):
                        slow_decisions = {
                            "scale_up",
                            "hold_no_useful_work",
                            "hold_insufficient_frontier",
                            "hold_handoff_backlog",
                            "observe_slow",
                        }
                        fast_decisions = {"hold", "observe_fast"}
                        if decision in slow_decisions and derived_work < slow_threshold:
                            reasons.append(
                                f"scan diagnostics policy window {sequence} slow decision disagrees with its signal"
                            )
                        if decision in fast_decisions and derived_work >= slow_threshold:
                            reasons.append(
                                f"scan diagnostics policy window {sequence} fast decision disagrees with its signal"
                            )
                elif derived_work is not None or not isinstance(unavailable, str):
                    reasons.append(
                        f"scan diagnostics policy window {sequence} lacks a null-signal reason"
                    )
                for field in (
                    "ready_directories",
                    "in_flight_directories",
                    "active_workers",
                    "handoff_backlog",
                ):
                    value = window.get(field)
                    if not isinstance(value, int) or value < 0:
                        reasons.append(
                            f"scan diagnostics policy window {sequence} has invalid {field}"
                        )
                active = window.get("active_workers")
                backlog = window.get("handoff_backlog")
                high_water = policy.get("handoff_backlog_high_water")
                if isinstance(active, int) and isinstance(maximum, int) and active > maximum:
                    reasons.append(
                        f"scan diagnostics policy window {sequence} exceeded the worker bound"
                    )
                if (
                    isinstance(backlog, int)
                    and isinstance(high_water, int)
                    and backlog > high_water
                ):
                    reasons.append(
                        f"scan diagnostics policy window {sequence} exceeded the handoff high-water"
                    )
                if decision not in {
                    "observe_fast",
                    "observe_slow",
                    "observe_incomplete",
                }:
                    if isinstance(observed_entries, int) and observed_entries >= 0:
                        traced_entries += observed_entries
                    if isinstance(observed_chunks, int) and observed_chunks >= 0:
                        traced_chunks += observed_chunks
                    if isinstance(observed_work_ns, int) and observed_work_ns >= 0:
                        traced_work_ns += observed_work_ns
                if decision == "scale_up":
                    scale_decisions += 1
            if outcome in {"held", "scaled_up", "held_no_useful_work"} and not windows:
                reasons.append("observable adaptive outcome has no policy window")
            if isinstance(calibration_entries, int) and calibration_entries != traced_entries:
                reasons.append("calibration entry aggregate disagrees with policy windows")
            if isinstance(calibration_chunks, int) and calibration_chunks != traced_chunks:
                reasons.append("calibration chunk aggregate disagrees with policy windows")
            if isinstance(calibration_work_ns, int) and calibration_work_ns != traced_work_ns:
                reasons.append("calibration work aggregate disagrees with policy windows")
            if isinstance(worker_expansions, int) and not (
                0 <= worker_expansions <= scale_decisions
            ):
                reasons.append("worker expansion aggregate disagrees with policy windows")
            if outcome == "scaled_up" and worker_expansions == 0:
                reasons.append("scaled policy outcome performed no worker expansion")
        spawned = policy.get("workers_spawned")
        peak = policy.get("peak_active_workers")
        maximum = policy.get("maximum_workers")
        if not all(isinstance(value, int) for value in (spawned, peak, maximum)):
            reasons.append("scan diagnostics worker bounds are missing")
        elif not (0 <= peak <= spawned <= maximum):
            reasons.append("scan diagnostics worker bounds are inconsistent")
        high_water = policy.get("handoff_backlog_high_water")
        if not isinstance(high_water, int) or high_water < 0:
            reasons.append("scan diagnostics handoff high-water is missing")
        if expected_phase_profile is not None and outcome != "fixed":
            reasons.extend(_validate_phase_history(policy, expected_phase_profile))
    if not isinstance(backend, dict):
        reasons.append("scan diagnostics backend is missing")
    else:
        portable_attempts = backend.get("portable_attempts")
        portable_reads = backend.get("portable_directory_reads")
        if not isinstance(portable_attempts, int) or not isinstance(portable_reads, int):
            reasons.append("portable backend counts are missing")
        elif not 0 <= portable_reads <= portable_attempts:
            reasons.append("portable backend counts are inconsistent")
        attempts = backend.get("macos_bulk_attempts")
        successes = backend.get("macos_bulk_successes")
        fallbacks = backend.get("macos_bulk_fallbacks")
        if require_macos_backend and not all(
            isinstance(value, int) for value in (attempts, successes, fallbacks)
        ):
            reasons.append("required macOS bulk/fallback counts are unavailable")
        elif all(isinstance(value, int) for value in (attempts, successes, fallbacks)):
            if attempts != successes + fallbacks:
                reasons.append("macOS bulk/fallback counts are inconsistent")
            if isinstance(expected_dirs_read, int) and expected_dirs_read != (
                successes + portable_reads
            ):
                reasons.append("backend counts disagree with reported directories read")
        elif backend.get("unavailable_reason") in (None, ""):
            reasons.append("unavailable macOS backend counts have no reason")
        if (
            not isinstance(attempts, int)
            and isinstance(expected_dirs_read, int)
            and isinstance(portable_reads, int)
            and expected_dirs_read != portable_reads
        ):
            reasons.append("portable backend count disagrees with reported directories read")
    return reasons


def _validate_phase_history(policy: Mapping[str, Any], expected_phase_profile: str) -> List[str]:
    """Require a generated phase hypothesis to appear in the measured trace.

    Corpus path prefixes describe deterministic topology, not enumeration or completion
    order. Only the service-time trace can establish that the hypothesized phases
    occurred in one run, so a missing or reversed signal invalidates that sample.
    """
    decision_signal = {
        "hold": "fast",
        "observe_fast": "fast",
        "scale_up": "slow",
        "hold_no_useful_work": "slow",
        "hold_insufficient_frontier": "slow",
        "hold_handoff_backlog": "slow",
        "observe_slow": "slow",
    }
    signals = [
        decision_signal.get(window.get("decision"))
        for window in policy.get("windows", [])
        if isinstance(window, Mapping)
    ]
    signals = [signal for signal in signals if signal is not None]
    compressed = [
        signal for index, signal in enumerate(signals) if index == 0 or signal != signals[index - 1]
    ]
    expected = {
        "fast-prefix-slow-suffix": ["fast", "slow"],
        "slow-prefix-fast-suffix": ["slow", "fast"],
        "alternating-regions": ["fast", "slow", "fast", "slow"],
    }.get(expected_phase_profile)
    if expected is None:
        return [f"unknown generated-corpus phase profile {expected_phase_profile!r}"]
    cursor = 0
    for signal in compressed:
        if cursor < len(expected) and signal == expected[cursor]:
            cursor += 1
    if cursor != len(expected):
        return [
            "generated-corpus phase hypothesis was not observed: "
            f"expected {'/'.join(expected)}, observed {'/'.join(compressed) or 'none'}"
        ]
    return []


def _expand(
    argv: Sequence[str],
    *,
    binary: Path,
    root: Path,
    snapshot: Optional[Path],
    extra: Sequence[str] = (),
) -> List[str]:
    values = {
        "binary": str(binary),
        "root": str(root),
        "snapshot": str(snapshot) if snapshot is not None else "",
    }
    expanded: List[str] = []
    for item in argv:
        if item.startswith("{") and item.endswith("}"):
            key = item[1:-1]
            if key not in values:
                raise MeasureError(f"unknown argv placeholder {item!r}")
            if not values[key]:
                raise MeasureError(f"argv placeholder {item!r} has no value")
            expanded.append(values[key])
        else:
            expanded.append(item)
    return expanded + list(extra)


def _spawn(
    argv: Sequence[str],
    *,
    timeout_seconds: float,
    environment_overrides: Optional[Mapping[str, str]] = None,
) -> Dict[str, Any]:
    """Run one command and collect wall time plus that child's own rusage.

    Output goes to temporary files rather than pipes. Pipes would need a draining
    thread to avoid deadlocking on a chatty reference tool, and that thread's
    scheduling would land inside the measured window. Files cost one write.

    ``os.wait4`` gives the resource usage of exactly this child, which is what a
    per-trial number has to mean. That is why the child is not reaped by
    ``subprocess``.
    """
    import tempfile

    environment = dict(BASE_ENVIRONMENT)
    if environment_overrides is not None:
        environment.update(environment_overrides)

    timed_out = False
    exit_code: Optional[int] = None
    usage = None

    with tempfile.TemporaryDirectory(prefix="fdu-realtree-") as scratch:
        out_path = Path(scratch) / "stdout"
        err_path = Path(scratch) / "stderr"
        with out_path.open("xb") as out, err_path.open("xb") as err:
            start = time.perf_counter_ns()
            process = subprocess.Popen(
                list(argv),
                stdin=subprocess.DEVNULL,
                stdout=out,
                stderr=err,
                env=environment,
                start_new_session=os.name == "posix",
            )
            if os.name == "posix" and hasattr(os, "wait4"):
                # A blocking wait in a thread, not a poll loop: polling would burn a
                # core in the parent for the whole measured window and contend with
                # the very thing being measured.
                state = _WaitState()
                waiter = threading.Thread(
                    target=_wait_for_child,
                    args=(process.pid, state),
                    daemon=True,
                    name="fdu-realtree-wait4",
                )
                waiter.start()
                waiter.join(timeout=timeout_seconds)
                if waiter.is_alive():
                    timed_out = True
                    _kill_group(process)
                    waiter.join(timeout=10)
                exit_code = state.exit_code
                usage = state.usage
                process.returncode = exit_code if exit_code is not None else -1
            else:
                try:
                    exit_code = process.wait(timeout=timeout_seconds)
                except subprocess.TimeoutExpired:
                    timed_out = True
                    _kill_group(process)
                    exit_code = process.wait()
            wall_ns = time.perf_counter_ns() - start
        stdout = out_path.read_bytes()
        stderr = err_path.read_bytes().decode("utf-8", errors="replace")

    return {
        "argv": list(argv),
        "exit_code": exit_code,
        "resources": _resources_from_wait4(usage),
        "stderr": stderr,
        "stdout": stdout,
        "timed_out": timed_out,
        "wall_ns": wall_ns,
    }


class _WaitState:
    """Where the waiting thread leaves what it reaped."""

    def __init__(self) -> None:
        self.exit_code: Optional[int] = None
        self.usage: Any = None
        self.error: Optional[str] = None


def _wait_for_child(pid: int, state: _WaitState) -> None:
    try:
        _waited, status, usage = os.wait4(pid, 0)
        state.exit_code = os.waitstatus_to_exitcode(status)
        state.usage = usage
    except (ChildProcessError, OSError, ValueError) as error:
        state.error = f"cannot wait for child process: {error}"


_RESOURCE_FIELDS = (
    "user_cpu_ns",
    "system_cpu_ns",
    "peak_rss_bytes",
    "major_faults",
    "minor_faults",
    "input_blocks",
    "output_blocks",
    "voluntary_context_switches",
    "involuntary_context_switches",
)


def _resources_from_wait4(usage: Any) -> Dict[str, Optional[int]]:
    if usage is None:
        return {field_name: None for field_name in _RESOURCE_FIELDS}
    peak = int(usage.ru_maxrss)
    if sys.platform != "darwin":
        # Linux reports ru_maxrss in kilobytes; Darwin already reports bytes.
        peak *= 1024
    return {
        "user_cpu_ns": round(float(usage.ru_utime) * 1e9),
        "system_cpu_ns": round(float(usage.ru_stime) * 1e9),
        "peak_rss_bytes": peak,
        "major_faults": int(usage.ru_majflt),
        "minor_faults": int(usage.ru_minflt),
        "input_blocks": int(usage.ru_inblock),
        "output_blocks": int(usage.ru_oublock),
        "voluntary_context_switches": int(usage.ru_nvcsw),
        "involuntary_context_switches": int(usage.ru_nivcsw),
    }


def _kill_group(process: "subprocess.Popen[bytes]") -> None:
    if os.name == "posix":
        try:
            os.killpg(process.pid, signal.SIGKILL)
            return
        except (ProcessLookupError, PermissionError):
            pass
    process.kill()


def _purge_page_cache() -> None:
    """Best-effort page-cache drop. Never silently pretends it worked."""
    if sys.platform == "darwin":
        binary = shutil.which("purge")
        if binary is None:
            raise MeasureError("--purge needs /usr/sbin/purge on macOS")
        completed = subprocess.run([binary], capture_output=True)
        if completed.returncode != 0:
            raise MeasureError(
                "purge failed; macOS needs root to drop the page cache: "
                + completed.stderr.decode("utf-8", errors="replace")[:200]
            )
        return
    drop = Path("/proc/sys/vm/drop_caches")
    if not drop.exists():
        raise MeasureError("--purge is not supported on this platform")
    try:
        drop.write_text("3\n", encoding="ascii")
    except OSError as error:
        raise MeasureError(f"--purge needs root: {error}") from error


def _progress(position: int, total: int, sample: Sample) -> None:
    wall = sample.metrics.get("wall_ns")
    shown = f"{wall / 1e6:8.1f} ms" if wall else "     n/a"
    flag = "" if sample.valid else f"  INVALID: {'; '.join(sample.reasons)[:90]}"
    tag = "warmup" if sample.warmup else f"#{sample.ordinal:02d}  "
    print(
        f"[{position:4d}/{total}] {sample.job:<22} {sample.variant:<14} {tag} {shown}{flag}",
        file=sys.stderr,
        flush=True,
    )


# --------------------------------------------------------------------------------
# Statistics
# --------------------------------------------------------------------------------


def summarize(document: Mapping[str, Any]) -> Dict[str, Any]:
    """Per-job, per-variant distributions plus paired comparisons against the first."""
    samples = [
        Sample(**{**sample, "reasons": list(sample["reasons"])}) for sample in document["samples"]
    ]
    variant_mapping = document["variants"]
    declared_order = document.get("variant_order")
    if declared_order is not None:
        if (
            not isinstance(declared_order, list)
            or len(declared_order) != len(variant_mapping)
            or set(declared_order) != set(variant_mapping)
        ):
            raise MeasureError(
                "variant_order does not identify every measured variant exactly once"
            )
        variants = list(declared_order)
    else:
        variants = list(variant_mapping)
    jobs = list(document["jobs"])
    campaign_stage = (document.get("conditions") or {}).get("campaign_stage", "exploratory")
    required_pairs = (document.get("conditions") or {}).get("trials")
    statistics_document: Dict[str, Any] = {}
    for job in jobs:
        per_variant: Dict[str, Any] = {}
        for variant in variants:
            selected = [
                sample
                for sample in samples
                if sample.job == job
                and sample.variant == variant
                and not sample.warmup
                and sample.valid
            ]
            per_variant[variant] = {
                "samples": len(selected),
                "invalid": sum(
                    1
                    for sample in samples
                    if sample.job == job
                    and sample.variant == variant
                    and not sample.warmup
                    and not sample.valid
                ),
                "metrics": {
                    metric: distribution(
                        [
                            sample.metrics[metric]
                            for sample in selected
                            if sample.metrics.get(metric) is not None
                        ]
                    )
                    for metric in LOWER_IS_BETTER
                },
            }
        comparisons = {}
        if len(variants) > 1:
            control = variants[0]
            for variant in variants[1:]:
                comparisons[f"{variant}_vs_{control}"] = paired_comparison(
                    samples,
                    job=job,
                    control=control,
                    candidate=variant,
                    campaign_stage=campaign_stage,
                    required_pairs=required_pairs,
                )
            # Successive pairs, for stacked-change runs: variant i measured against
            # variant i-1 isolates the i-th change, while everything still shares one
            # interleaved schedule. Without this, a stack of three small changes could
            # only be judged in aggregate or in three runs on a drifting machine.
            for previous, current in zip(variants[1:], variants[2:]):
                comparisons[f"{current}_vs_{previous}"] = paired_comparison(
                    samples,
                    job=job,
                    control=previous,
                    candidate=current,
                    campaign_stage=campaign_stage,
                    required_pairs=required_pairs,
                )
        statistics_document[job] = {
            "variants": per_variant,
            "comparisons": comparisons,
        }
    return statistics_document


def distribution(values: Sequence[int]) -> Optional[Dict[str, float]]:
    if not values:
        return None
    ordered = sorted(values)
    median = statistics.median(ordered)
    return {
        "count": len(ordered),
        "min": ordered[0],
        "max": ordered[-1],
        "median": median,
        "mean": round(statistics.fmean(ordered), 1),
        "p90": ordered[min(len(ordered) - 1, int(round(0.9 * (len(ordered) - 1))))],
        "stdev": round(statistics.stdev(ordered), 1) if len(ordered) > 1 else 0.0,
        # Median absolute deviation resists the single slow trial that a background
        # process causes, which stdev does not.
        "mad": statistics.median([abs(value - median) for value in ordered]),
    }


def paired_comparison(
    samples: Sequence[Sample],
    *,
    job: str,
    control: str,
    candidate: str,
    campaign_stage: str = "exploratory",
    required_pairs: Optional[int] = None,
) -> Dict[str, Any]:
    """Compare two variants using differences at equal ordinals.

    Pairing is the point. Two trials at the same ordinal ran within milliseconds of
    each other, so whatever the machine was doing, it was doing to both.
    """
    result: Dict[str, Any] = {"control": control, "candidate": candidate, "metrics": {}}
    for metric in LOWER_IS_BETTER:
        pairs = _pairs(samples, job, control, candidate, metric)
        if len(pairs) < 3:
            result["metrics"][metric] = None
            continue
        deltas = [candidate_value - control_value for control_value, candidate_value in pairs]
        ratios = [
            (candidate_value - control_value) / control_value
            for control_value, candidate_value in pairs
            if control_value
        ]
        median_ratio = statistics.median(ratios) if ratios else None
        low, high = _bootstrap_median_interval(ratios) if ratios else (None, None)
        delta_low, delta_high = _bootstrap_median_interval(deltas)
        median_change_pct = round(median_ratio * 100, 3) if median_ratio is not None else None
        interval_pct = [round(low * 100, 3), round(high * 100, 3)] if low is not None else None
        result["metrics"][metric] = {
            "pairs": len(pairs),
            "median_delta": statistics.median(deltas),
            "ci95_delta": [round(delta_low, 3), round(delta_high, 3)],
            "median_change_pct": median_change_pct,
            "ci95_change_pct": interval_pct,
            "improved": median_ratio is not None and median_ratio < 0,
            # Three separate questions that used to be collapsed into one flag.
            #
            # `passes_acceptance` is the project's one-sided accept rule: the whole
            # interval below zero. `ci_excludes_zero` asks only whether the evidence is
            # clear in *either* direction, and `direction` says which. Reporting only
            # the accept rule made a measured regression render as "not significant",
            # which is how exp-012 came to be described as free when its own RSS
            # intervals sat entirely above zero.
            "passes_acceptance": low is not None and high is not None and high < 0,
            "ci_excludes_zero": low is not None and high is not None and (high < 0 or low > 0),
            "direction": _direction(low, high),
            "significant": low is not None and high is not None and high < 0,
            "noninferiority": _noninferiority_classification(
                median_change_pct,
                interval_pct,
            ),
        }
    policy_stability = _adaptive_policy_stability(
        samples,
        job,
        control,
        candidate,
        required_histories=required_pairs,
    )
    result["policy_stability"] = policy_stability
    result["qualification"] = _qualification(
        result,
        policy_stability,
        campaign_stage=campaign_stage,
        required_pairs=required_pairs,
    )
    return result


def _pairs(
    samples: Sequence[Sample],
    job: str,
    control: str,
    candidate: str,
    metric: str,
) -> List[tuple]:
    def indexed(variant: str) -> Dict[int, int]:
        return {
            sample.ordinal: sample.metrics[metric]
            for sample in samples
            if sample.job == job
            and sample.variant == variant
            and not sample.warmup
            and sample.valid
            and sample.metrics.get(metric) is not None
        }

    left = indexed(control)
    right = indexed(candidate)
    return [(left[ordinal], right[ordinal]) for ordinal in sorted(set(left) & set(right))]


def _noninferiority_classification(
    median_change_pct: Optional[float], interval: Optional[Sequence[float]]
) -> str:
    """Classify superiority separately from the +3% noninferiority decision."""
    if median_change_pct is None or interval is None or len(interval) != 2:
        return "inconclusive"
    low, high = interval
    if median_change_pct <= SUPERIORITY_FLOOR_PCT and high < 0:
        return "superior"
    if high <= NONINFERIORITY_MARGIN_PCT:
        return "noninferior"
    if low > NONINFERIORITY_MARGIN_PCT:
        return "inferior"
    return "inconclusive"


def _adaptive_policy_stability(
    samples: Sequence[Sample],
    job: str,
    control: str,
    candidate: str,
    *,
    required_histories: Optional[int] = None,
) -> Optional[Dict[str, Any]]:
    if job != "adaptive-scan-index":
        return None
    variants: Dict[str, Any] = {}
    for variant in (control, candidate):
        histories = []
        missing = 0
        for sample in samples:
            if sample.job != job or sample.variant != variant or sample.warmup:
                continue
            if not sample.valid:
                missing += 1
                continue
            diagnostics = sample.probe.get("scan_diagnostics")
            policy = diagnostics.get("worker_policy") if isinstance(diagnostics, dict) else None
            if not isinstance(policy, dict):
                missing += 1
                continue
            outcome = policy.get("outcome")
            windows = policy.get("windows")
            if not isinstance(windows, list):
                missing += 1
                continue
            sensitivity = _order_sensitive_policy_history(outcome, windows)
            harmful = _harmful_policy_history(outcome, windows)
            histories.append((outcome, sensitivity, harmful))
        signatures = sorted({history for history in histories}, key=repr)
        harmful_count = sum(harmful is not None for _outcome, _sensitivity, harmful in histories)
        order_sensitive_count = sum(
            sensitivity is not None for _outcome, sensitivity, _harmful in histories
        )
        variants[variant] = {
            "expected_histories": required_histories,
            "histories": len(histories),
            "harmful_histories": harmful_count,
            "harmful_frequency": (round(harmful_count / len(histories), 6) if histories else None),
            "order_sensitive_histories": order_sensitive_count,
            "order_sensitivity_frequency": (
                round(order_sensitive_count / len(histories), 6) if histories else None
            ),
            "missing_histories": missing,
            "signatures": [
                {"outcome": outcome, "sensitivity": sensitivity, "harm": harm}
                for outcome, sensitivity, harm in signatures
            ],
            # A late signal is completion-order sensitivity, not harm by itself. Harm
            # requires either a structurally invalid policy history here or measured
            # wall/resource impact in the paired qualification below. Treating every
            # held-before-late-slow trace as harmful assumes the very benefit the
            # experiment is meant to test.
            "stable": bool(histories)
            and missing == 0
            and harmful_count == 0
            and len(signatures) == 1
            and (required_histories is None or len(histories) == required_histories),
        }
    return {
        "rule": "zero-structurally-harmful-and-one-outcome-sensitivity-signature-v2",
        "variants": variants,
        "stable": all(entry["stable"] for entry in variants.values()),
    }


def _harmful_policy_history(outcome: Any, windows: Sequence[Any]) -> Optional[str]:
    """Return only harm established by the trace itself.

    Slow work after a hold, or fast work after scale-up, is completion-order
    sensitivity. Neither observation establishes performance harm without a paired
    control, so those patterns are retained separately by
    :func:`_order_sensitive_policy_history`.
    """
    decisions = [window.get("decision") for window in windows if isinstance(window, Mapping)]
    if outcome == "scaled_up" and "scale_up" not in decisions:
        return "scaled-without-recorded-decision"
    return None


def _order_sensitive_policy_history(outcome: Any, windows: Sequence[Any]) -> Optional[str]:
    """Describe a completion-order pattern without presuming its performance impact."""
    decisions = [window.get("decision") for window in windows if isinstance(window, Mapping)]
    if outcome in {"held", "held_no_useful_work"} and "observe_slow" in decisions:
        return "held-before-later-slow-window"
    if outcome == "scaled_up" and "scale_up" in decisions:
        scale_index = decisions.index("scale_up")
        later_fast = sum(
            decision in {"hold", "observe_fast"} for decision in decisions[scale_index + 1 :]
        )
        if later_fast >= 2 and "observe_slow" not in decisions[scale_index + 1 :]:
            return "scaled-before-two-later-fast-windows"
    return None


def _qualification(
    comparison: Mapping[str, Any],
    policy_stability: Optional[Mapping[str, Any]],
    *,
    campaign_stage: str,
    required_pairs: Optional[int] = None,
) -> Dict[str, Any]:
    metrics = comparison.get("metrics") or {}
    primary = metrics.get("wall_ns")
    if not isinstance(primary, Mapping):
        return {
            "classification": "inconclusive",
            "campaign_stage": campaign_stage,
            "confirmable": False,
            "reasons": ["missing wall_ns evidence"],
        }
    classification = primary.get("noninferiority", "inconclusive")
    reasons: List[str] = []
    if campaign_stage == "held-out" and (
        not isinstance(required_pairs, int) or primary.get("pairs") != required_pairs
    ):
        reasons.append("held-out wall evidence does not contain every fixed-N pair")
    resource_results: Dict[str, Any] = {}
    for metric, limit in RESOURCE_REGRESSION_LIMITS_PCT.items():
        entry = metrics.get(metric)
        if campaign_stage == "held-out" and (
            not isinstance(required_pairs, int)
            or not isinstance(entry, Mapping)
            or entry.get("pairs") != required_pairs
        ):
            resource_results[metric] = "inconclusive"
            reasons.append(f"{metric} does not contain every fixed-N pair")
            continue
        interval = entry.get("ci95_change_pct") if isinstance(entry, Mapping) else None
        if not isinstance(interval, list) or len(interval) != 2:
            resource_results[metric] = "inconclusive"
            reasons.append(f"{metric} is missing a paired percent interval")
            continue
        low, high = interval
        if high <= limit:
            resource_results[metric] = "within-limit"
        elif low > limit:
            resource_results[metric] = "rejected"
            reasons.append(f"{metric} exceeds its +{limit:g}% regression limit")
        else:
            resource_results[metric] = "inconclusive"
            reasons.append(f"{metric} straddles its +{limit:g}% regression limit")
    major = metrics.get("major_faults")
    if campaign_stage == "held-out" and (
        not isinstance(required_pairs, int)
        or not isinstance(major, Mapping)
        or major.get("pairs") != required_pairs
    ):
        resource_results["major_faults"] = "inconclusive"
        reasons.append("major_faults does not contain every fixed-N pair")
        major_interval = None
    else:
        major_interval = major.get("ci95_delta") if isinstance(major, Mapping) else None
    if not isinstance(major_interval, list) or len(major_interval) != 2:
        resource_results["major_faults"] = "inconclusive"
        reasons.append("major_faults is missing a paired absolute interval")
    elif major_interval[1] <= MAJOR_FAULT_DELTA_LIMIT:
        resource_results["major_faults"] = "within-limit"
    elif major_interval[0] > MAJOR_FAULT_DELTA_LIMIT:
        resource_results["major_faults"] = "rejected"
        reasons.append("major_faults establishes a positive paired increase")
    else:
        resource_results["major_faults"] = "inconclusive"
        reasons.append("major_faults does not establish non-regression")

    if policy_stability is not None and not policy_stability.get("stable"):
        reasons.append(
            "trace-defined adaptive policy history is missing, structurally harmful, or unstable"
        )
    if any(value == "rejected" for value in resource_results.values()):
        classification = "inferior"
    elif reasons or classification == "inconclusive":
        classification = "inconclusive"
    return {
        "classification": classification,
        "campaign_stage": campaign_stage,
        "confirmable": campaign_stage == "held-out"
        and classification in {"superior", "noninferior"}
        and not reasons,
        "noninferiority_margin_pct": NONINFERIORITY_MARGIN_PCT,
        "resource_limits_pct": dict(RESOURCE_REGRESSION_LIMITS_PCT),
        "major_fault_delta_limit": MAJOR_FAULT_DELTA_LIMIT,
        "resources": resource_results,
        "reasons": reasons,
    }


def _direction(low: Optional[float], high: Optional[float]) -> str:
    """Which way the evidence points, independent of whether we would accept it.

    "unclear" is the honest answer for an interval straddling zero: it covers both a
    small win and a small loss, and calling that "unchanged" is the overclaim this
    field exists to prevent.
    """
    if low is None or high is None:
        return "unknown"
    if high < 0:
        return "improved"
    if low > 0:
        return "regressed"
    return "unclear"


def _bootstrap_median_interval(
    values: Sequence[float], *, resamples: int = 2000, seed: int = 0x5EED
):
    """A deterministic percentile bootstrap of the median.

    Deterministic because a benchmark result that changes when you re-render the
    report is not a result. The seed is fixed and the generator is local.
    """
    if not values:
        return None, None
    import random

    generator = random.Random(seed)
    count = len(values)
    medians = []
    for _ in range(resamples):
        resample = [values[generator.randrange(count)] for _ in range(count)]
        medians.append(statistics.median(resample))
    medians.sort()
    low = medians[int(0.025 * (resamples - 1))]
    high = medians[int(0.975 * (resamples - 1))]
    return low, high


# --------------------------------------------------------------------------------
# Environment
# --------------------------------------------------------------------------------


def host_facts() -> Dict[str, Any]:
    """Machine facts that change timings. No hostname, no user, no paths."""
    facts: Dict[str, Any] = {
        "arch": platform.machine(),
        "cpu_count": os.cpu_count(),
        "python": platform.python_version(),
        "system": platform.system(),
        "release": platform.release(),
        "toolchain": _toolchain(),
    }
    if sys.platform == "darwin":
        facts["cpu_model"] = _sysctl("machdep.cpu.brand_string")
        facts["memory_bytes"] = _sysctl_int("hw.memsize")
        facts["max_vnodes"] = _sysctl_int("kern.maxvnodes")
        facts["performance_cores"] = _sysctl_int("hw.perflevel0.logicalcpu")
        facts["efficiency_cores"] = _sysctl_int("hw.perflevel1.logicalcpu")
        facts["filesystem"] = _darwin_filesystem()
    return facts


def _toolchain() -> str:
    """The compiler that built the binaries, which changes the numbers as much as the code does."""
    binary = shutil.which("rustc")
    if binary is None:
        return ""
    completed = subprocess.run([binary, "--version"], capture_output=True)
    if completed.returncode != 0:
        return ""
    return completed.stdout.decode("utf-8", errors="replace").strip()


def _sysctl(name: str) -> Optional[str]:
    binary = shutil.which("sysctl")
    if binary is None:
        return None
    completed = subprocess.run([binary, "-n", name], capture_output=True)
    if completed.returncode != 0:
        return None
    return completed.stdout.decode("utf-8", errors="replace").strip() or None


def _sysctl_int(name: str) -> Optional[int]:
    value = _sysctl(name)
    try:
        return int(value) if value is not None else None
    except ValueError:
        return None


def _darwin_filesystem() -> Optional[str]:
    binary = shutil.which("mount")
    if binary is None:
        return None
    completed = subprocess.run([binary], capture_output=True)
    if completed.returncode != 0:
        return None
    for line in completed.stdout.decode("utf-8", errors="replace").splitlines():
        if " on / " in line:
            start = line.find("(")
            if start != -1:
                return line[start + 1 :].split(",")[0]
    return None


def _darwin_power_source() -> Optional[str]:
    if sys.platform != "darwin":
        return None
    binary = shutil.which("pmset")
    if binary is None:
        return None
    completed = subprocess.run([binary, "-g", "batt"], capture_output=True, check=False)
    if completed.returncode != 0:
        return None
    output = completed.stdout.decode("utf-8", errors="replace")
    if "AC Power" in output:
        return "AC"
    if "Battery Power" in output:
        return "battery"
    return None


def _darwin_thermal_pressure() -> Optional[str]:
    """Read the current Foundation thermal state without compiling a helper."""
    if sys.platform != "darwin":
        return None
    binary = Path("/usr/bin/osascript")
    if not binary.is_file():
        return None
    completed = subprocess.run(
        [
            str(binary),
            "-l",
            "JavaScript",
            "-e",
            'ObjC.import("Foundation"); $.NSProcessInfo.processInfo.thermalState',
        ],
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        return None
    states = {"0": "normal", "1": "fair", "2": "serious", "3": "critical"}
    return states.get(completed.stdout.decode("ascii", errors="ignore").strip())
