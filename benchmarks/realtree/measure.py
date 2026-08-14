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
class Sample:
    variant: str
    job: str
    ordinal: int
    warmup: bool
    valid: bool
    reasons: List[str] = field(default_factory=list)
    metrics: Dict[str, Optional[int]] = field(default_factory=dict)
    probe: Dict[str, Any] = field(default_factory=dict)

    def as_json(self) -> Dict[str, Any]:
        return {
            "job": self.job,
            "metrics": self.metrics,
            "ordinal": self.ordinal,
            "probe": self.probe,
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
    ),
    "content-binary-gate": Job(
        id="content-binary-gate",
        argv=("{binary}", "content-binary-gate", "--root", "{root}"),
        start_state="cold",
        description="Exercise early binary admission on a binary-heavy immutable tree.",
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
    ),
    "content-disabled": Job(
        id="content-disabled",
        argv=("{binary}", "content-disabled", "--root", "{root}"),
        start_state="cold",
        description="Call the disabled content boundary after metadata setup.",
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
    ),
    "text-prose": Job(
        id="text-prose",
        argv=("{binary}", "text-prose", "--root", "{root}"),
        start_state="cold",
        description=(
            "Analyze plain text with raw and normalized words, paragraphs, and page "
            "sufficient statistics after metadata setup."
        ),
    ),
    "cold-scan-index": Job(
        id="cold-scan-index",
        argv=("{binary}", "scan-index", "--root", "{root}"),
        start_state="cold",
        description="Full walk with metadata into a complete index. No snapshot.",
    ),
    "cold-scan-producer": Job(
        id="cold-scan-producer",
        argv=("{binary}", "scan-producer", "--root", "{root}"),
        start_state="cold",
        description=(
            "Walk and metadata only, no index build. Isolates the syscall layer. "
            "Wall time includes an untimed exact validation scan; read component_ns."
        ),
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
    timeout_seconds: float = DEFAULT_TIMEOUT_SECONDS,
) -> Dict[str, Any]:
    """Measure every ``variant`` against every ``job``, interleaved, and return a run."""
    if not variants:
        raise MeasureError("at least one variant is required")
    if not jobs:
        raise MeasureError("at least one job is required")
    if trials < 1:
        raise MeasureError("trials must be at least 1")

    scratch.mkdir(parents=True, exist_ok=True)
    started = time.time()

    before = reference_tree.fingerprint(root, label=label)
    drift = (
        reference_tree.compare(before, baseline_fingerprint)
        if baseline_fingerprint is not None
        else []
    )

    snapshots = _prepare_snapshots(root, variants, jobs, scratch, timeout_seconds)

    samples: List[Sample] = []
    schedule = _interleave(variants, jobs, trials, warmups)
    total = len(schedule)
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
            timeout_seconds=timeout_seconds,
        )
        samples.append(sample)
        _progress(position, total, sample)

    after = reference_tree.fingerprint(root, label=label)
    mutation = reference_tree.compare(after, before)

    document = {
        "schema": RUN_SCHEMA,
        "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(started)),
        "duration_seconds": round(time.time() - started, 3),
        "note": note,
        "host": host_facts(),
        "conditions": {
            "os_cache": "purged-per-trial" if purge else "warm-steady",
            "trials": trials,
            "warmups": warmups,
            "interleaved": True,
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
                "start_state": job.start_state,
            }
            for job in jobs
        },
        "samples": [sample.as_json() for sample in samples],
    }
    document["statistics"] = summarize(document)
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


def _measure_once(
    *,
    variant: Variant,
    job: Job,
    root: Path,
    snapshot: Optional[Path],
    ordinal: int,
    warmup: bool,
    fingerprint_document: Dict[str, Any],
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
    result = _spawn(argv, timeout_seconds=timeout_seconds)

    reasons: List[str] = []
    if result["timed_out"]:
        reasons.append("command timed out")
    if result["exit_code"] not in job.allowed_exit_codes:
        reasons.append(f"command exited with {result['exit_code']}")

    probe: Dict[str, Any] = {}
    component_ns: Optional[int] = None
    if variant.kind == "fdu-probe":
        probe, probe_reasons = _read_probe_output(
            result["stdout"], allow_incomplete=job.allow_incomplete
        )
        reasons.extend(probe_reasons)
        component_ns = probe.get("component_ns")
        if job.verify_oracle and probe:
            disagreement = reference_tree.probe_agrees(
                fingerprint_document, probe.get("summary")
            )
            if disagreement is not None:
                reasons.append(disagreement)

    metrics = dict(result["resources"])
    metrics["wall_ns"] = result["wall_ns"]
    metrics["component_ns"] = component_ns
    user = metrics.get("user_cpu_ns")
    system = metrics.get("system_cpu_ns")
    if user is not None and system is not None:
        metrics["cpu_ns"] = user + system
        metrics["blocked_ns"] = max(0, result["wall_ns"] - user - system)
    else:
        metrics["cpu_ns"] = None
        metrics["blocked_ns"] = None

    return Sample(
        variant=variant.name,
        job=job.id,
        ordinal=ordinal,
        warmup=warmup,
        valid=not reasons,
        reasons=reasons,
        metrics=metrics,
        probe={
            key: value
            for key, value in probe.items()
            if key in {"component_ns", "attribution", "mode", "source", "summary"}
        },
    )


def _read_probe_output(stdout: bytes, *, allow_incomplete: bool = False):
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
    return document, reasons


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


def _spawn(argv: Sequence[str], *, timeout_seconds: float) -> Dict[str, Any]:
    """Run one command and collect wall time plus that child's own rusage.

    Output goes to temporary files rather than pipes. Pipes would need a draining
    thread to avoid deadlocking on a chatty reference tool, and that thread's
    scheduling would land inside the measured window. Files cost one write.

    ``os.wait4`` gives the resource usage of exactly this child, which is what a
    per-trial number has to mean. That is why the child is not reaped by
    ``subprocess``.
    """
    import tempfile

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
                env=dict(BASE_ENVIRONMENT),
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
        f"[{position:4d}/{total}] {sample.job:<22} {sample.variant:<14} "
        f"{tag} {shown}{flag}",
        file=sys.stderr,
        flush=True,
    )


# --------------------------------------------------------------------------------
# Statistics
# --------------------------------------------------------------------------------


def summarize(document: Mapping[str, Any]) -> Dict[str, Any]:
    """Per-job, per-variant distributions plus paired comparisons against the first."""
    samples = [
        Sample(**{**sample, "reasons": list(sample["reasons"])})
        for sample in document["samples"]
    ]
    variants = list(document["variants"])
    jobs = list(document["jobs"])
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
                    samples, job=job, control=control, candidate=variant
                )
            # Successive pairs, for stacked-change runs: variant i measured against
            # variant i-1 isolates the i-th change, while everything still shares one
            # interleaved schedule. Without this, a stack of three small changes could
            # only be judged in aggregate or in three runs on a drifting machine.
            for previous, current in zip(variants[1:], variants[2:]):
                comparisons[f"{current}_vs_{previous}"] = paired_comparison(
                    samples, job=job, control=previous, candidate=current
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
    samples: Sequence[Sample], *, job: str, control: str, candidate: str
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
        result["metrics"][metric] = {
            "pairs": len(pairs),
            "median_delta": statistics.median(deltas),
            "median_change_pct": round(median_ratio * 100, 3)
            if median_ratio is not None
            else None,
            "ci95_change_pct": [round(low * 100, 3), round(high * 100, 3)]
            if low is not None
            else None,
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
            "ci_excludes_zero": low is not None
            and high is not None
            and (high < 0 or low > 0),
            "direction": _direction(low, high),
            "significant": low is not None and high is not None and high < 0,
        }
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
    return [
        (left[ordinal], right[ordinal])
        for ordinal in sorted(set(left) & set(right))
        if left[ordinal]
    ]


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
