"""Time fdu milestone builds against each other on one performance-index component.

    python -m benchmarks.realtree.history --component code \\
        --root TREE --label linux-v6.12-macos \\
        --build prework=path/to/fdu-prework ... --build v0.3.0=path/to/fdu-v0.3.0 \\
        --probe v0.3.0=path/to/perf_probe-v0.3.0 \\
        --trials 12 --output-dir RESULTS --name macos-code-internal

One history cell times every build in one interleaved session, anchored on the
manifest's reference build, so every ratio in it is within-session. The suite manifest
(``explorations/benchmarks/index-suite.json``) names the component's command and cache
state; this module decides how each build spells that command, checks every build's
answer before timing, and hands the timing to ``benchmarks.realtree.compare_tools``,
whose schedule, warm-ups, fingerprints, host-pressure records and statistics are used
unchanged. The cell it writes is the history-cell summary the evidence page reads.

**Command shapes per era.** A build is asked what it can do (``--help``, ``--version``)
and never forced:

- the cache-off switch is ``--cache off``, or ``--no-cache`` before 0.1.0;
- ``--cache on`` (persist the snapshot) is spelled ``--cache auto`` where the build's
  policies predate the cost model (auto, refresh, read-only, only, off: auto persisted
  the snapshot after a complete scan), and is the bare command before 0.1.0, whose
  default read and wrote it. What each build then does with the snapshot is its own: a
  one-shot metadata report from 0.1.0 on does not load it, and the performance line the
  answer check records says so;
- a content view is requested with its analyzer named (``--analyze code --view code``,
  ``--analyze words --view documents``), because no build so far implies the analysis
  from the view;
- a build without a view, an analyzer, a view list, a cache policy, or the probe mode a
  component needs is recorded as unsupported for that component, with the reason.

**Probe jobs.** A ``perf_probe JOB...`` command (or ``perf_probe A; perf_probe B``)
names jobs of the real-tree harness (``measure.PROBE_JOBS``), by job id or by the probe
mode the job runs, and runs each exactly as the harness does, oracle included, so every
sample's answer is checked against the fingerprint. Every process gets a fresh cache
directory; a job that reads a snapshot (``warm-revalidate``) gets one written by the same
build's preparation mode into that directory just before, untimed, and a job that writes
one (``cold-open-save``) starts with none. One cell times one job (``--job``); a
component of several jobs is one cell per job. The timed metric is the whole process's
wall time, which includes discovery, snapshot load and save, and the oracle;
``--timing component`` times the probe's own ``component_ns`` instead. Both are recorded
either way.

**Cache states** are the manifest's, implemented exactly: caches empty for every sample
is a fresh empty cache directory per process (with the cache switched off, as first runs
are measured, whether or not the manifest spells ``--cache off``); the steady state after
the warm-ups is one cache directory per build for the whole cell under the default
policy; a cache filled by an untimed run just before is, for every sample, an untimed run
of the same build into a fresh directory immediately before the timed run that reuses
it. A probe job's start state is the job's own, as above.

A component's command, cache state, and timing can be overridden for one cell
(``--command``, ``--cache-state``, ``--timing``, with ``--override-note`` saying why);
the cell records the manifest's definition and the override side by side.

See ``docs/project/specs/active/plan-2026-10-05-fdu-performance-index.md``.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import plistlib
import re
import statistics
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, Dict, FrozenSet, List, Mapping, Optional, Sequence, Tuple, Union

from benchmarks.atomic_write import write_text_atomic
from benchmarks.realtree import compare_tools, ledger, measure, tree

MANIFEST = Path(__file__).resolve().parents[1] / "index-suite.json"
SUMMARY_SCHEMA = "fdu-history-summary-v1"
MANIFEST_SCHEMA = "fdu-performance-index-suite/1"
DEFAULT_RESULTS = Path(tempfile.gettempdir()) / "fdu-history" / "results"

#: The manifest's cache states, by the text it records them with. A new state in the
#: manifest is a new index version and needs code here: an unknown one fails closed.
CACHE_STATES = {
    "fdu's caches empty for every sample": "empty",
    "default cache policy, steady state after the warm-ups": "steady",
    "content cache filled by an untimed run of the same build just before": "filled",
    "snapshot written by an untimed run of the same build just before": "filled",
    # Probe components: each job's start state is the job's own (see ``probe_job_argv``);
    # these texts describe the component and are recorded.
    "the library's open, then a second report": "job",
    "the library's open, then a second report and an applied change": "job",
    "the library's open, then a second report; then an applied change": "job",
    "a first run that scans and writes the snapshot; then a run that loads and "
    "revalidates a snapshot written by an untimed run of the same build just before": "job",
}

#: The analyzer each content view displays. Every build so far refuses the view without
#: it ("views never enable content analysis implicitly"), so the era command names it.
VIEW_ANALYZERS = {"code": "code", "documents": "words"}
#: The view a build shows for an analyzer when no view is named.
ANALYZER_VIEWS = {"code": "code", "words": "documents"}
ANALYZER_ORDER = ("lines", "code", "words")

#: The policy set fdu shipped before the cost model (0.2.0): `auto` persisted the
#: snapshot after a complete scan, which is what `--cache on` does from 0.2.0 on.
PRE_COST_MODEL_POLICIES = frozenset({"auto", "refresh", "read-only", "only", "off"})

#: Where a probe job that reads or writes a snapshot keeps it: inside the sample's own
#: cache directory, so it is created and discarded with that directory.
PROBE_SNAPSHOT = "{cache}/snapshot.fdu"

#: The metric a cell times, by ``--timing``.
TIMINGS = {"wall": "wall_ns", "component": "component_ns"}


class HistoryError(RuntimeError):
    """The requested cell cannot produce trustworthy evidence."""


# --------------------------------------------------------------------------------------
# The manifest


def load_manifest(path: Path = MANIFEST) -> Dict[str, Any]:
    document = json.loads(path.read_text(encoding="utf-8"))
    if document.get("schema") != MANIFEST_SCHEMA:
        raise HistoryError(f"{path.name} is not a {MANIFEST_SCHEMA} manifest")
    return document


def component(manifest: Mapping[str, Any], component_id: str) -> Dict[str, Any]:
    """The manifest's entry for one timed component."""
    for entry in manifest["components"]:
        if entry["id"] == component_id:
            if entry.get("from_components"):
                raise HistoryError(
                    f"component {component_id!r} is derived from "
                    f"{', '.join(entry['from_components'])} and is not timed on its own"
                )
            if entry.get("cache_state") not in CACHE_STATES:
                raise HistoryError(
                    f"component {component_id!r} has cache state "
                    f"{entry.get('cache_state')!r}, which this driver does not implement"
                )
            return dict(entry)
    known = ", ".join(entry["id"] for entry in manifest["components"])
    raise HistoryError(f"unknown component {component_id!r}; the manifest has {known}")


@dataclass(frozen=True)
class Request:
    """What a manifest command asks for, independent of how any build spells it."""

    program: str
    views: Tuple[str, ...] = ()
    analyzers: Tuple[str, ...] = ()
    cache_policy: Optional[str] = None
    probe_jobs: Tuple[str, ...] = ()


def resolve_probe_job(token: str) -> str:
    """The harness job a probe command names, by job id or by the probe mode it runs.

    ``warm-revalidate`` is a job and ``revalidate`` the mode it runs (``measure.py``);
    either names the job, as long as exactly one job runs that mode.
    """
    if token in measure.PROBE_JOBS:
        return token
    runs_mode = sorted(job_id for job_id, job in measure.PROBE_JOBS.items() if job.argv[1] == token)
    if len(runs_mode) == 1:
        return runs_mode[0]
    if runs_mode:
        raise HistoryError(f"probe mode {token!r} is run by jobs {runs_mode}; name one of them")
    raise HistoryError(f"{token!r} is neither a harness job nor a probe mode one runs")


def parse_command(command: str) -> Request:
    """Read a manifest command such as ``fdu --view code,documents PATH``.

    ``perf_probe JOB...`` (or ``perf_probe A; perf_probe B``) names harness jobs or the
    probe modes they run; ``fdu [--analyze LIST] [--view LIST] [--cache POLICY] PATH`` a
    command-line report.
    """
    words = command.split()
    if not words:
        raise HistoryError("empty manifest command")
    program, rest = words[0], words[1:]
    if program == "perf_probe":
        tokens = [
            token
            for part in command.split(";")
            for index, token in enumerate(part.replace(",", " ").split())
            if not (index == 0 and token == "perf_probe")
        ]
        if not tokens:
            raise HistoryError(f"probe command {command!r} names no job")
        return Request(program=program, probe_jobs=tuple(resolve_probe_job(t) for t in tokens))
    if program != "fdu" or not rest or rest[-1] != "PATH":
        raise HistoryError(f"manifest command {command!r} must be `fdu [FLAGS] PATH`")
    flags = rest[:-1]
    values: Dict[str, str] = {}
    index = 0
    while index < len(flags):
        flag = flags[index]
        if flag in ("--view", "--cache", "--analyze") and index + 1 < len(flags):
            values[flag] = flags[index + 1]
            index += 2
            continue
        raise HistoryError(f"manifest command {command!r} has a flag this driver cannot map")

    def listed(flag: str) -> Tuple[str, ...]:
        return tuple(item for item in values.get(flag, "").split(",") if item)

    return Request(
        program=program,
        views=listed("--view"),
        analyzers=listed("--analyze"),
        cache_policy=values.get("--cache"),
    )


# --------------------------------------------------------------------------------------
# What a build can do


@dataclass(frozen=True)
class Capabilities:
    """What one fdu build's help says it accepts."""

    version: str
    cache_policies: FrozenSet[str] = frozenset()
    no_cache_flag: bool = False
    views: FrozenSet[str] = frozenset()
    view_list: bool = False
    analyzers: FrozenSet[str] = frozenset()
    reads_gitignore: bool = False
    json_format: bool = False
    json_flag: bool = False

    def cache_off(self) -> Optional[Tuple[str, ...]]:
        """This build's spelling of "no persisted cache", or None."""
        if "off" in self.cache_policies:
            return ("--cache", "off")
        if self.no_cache_flag:
            return ("--no-cache",)
        return None

    def cache_on(self) -> Optional[Tuple[str, ...]]:
        """This build's spelling of "read and write the snapshot", or None."""
        if "on" in self.cache_policies:
            return ("--cache", "on")
        if self.cache_policies == PRE_COST_MODEL_POLICIES:
            return ("--cache", "auto")
        if not self.cache_policies and self.no_cache_flag:
            # Before 0.1.0 the default read and wrote the snapshot unless --no-cache.
            return ()
        return None


def _flat(text: str) -> str:
    return " ".join(text.split())


def _choices(flat: str, lead: str, stop: str) -> FrozenSet[str]:
    """The comma-and-or list of words after ``lead`` and before ``stop``."""
    start = flat.find(lead)
    if start < 0:
        return frozenset()
    rest = flat[start + len(lead) :]
    end = rest.find(stop)
    listed = rest if end < 0 else rest[:end]
    listed = re.sub(r"\([^)]*\)", " ", listed)
    words = re.split(r",|\bor\b", listed)
    return frozenset(word.strip() for word in words if re.fullmatch(r"[a-z][a-z-]*", word.strip()))


def capabilities(help_text: str, version: str) -> Capabilities:
    """Parse one build's ``--help`` into the capabilities the suite needs."""
    flat = _flat(help_text)
    return Capabilities(
        version=version,
        cache_policies=_choices(flat, "--cache <POLICY> Cache policy:", "[default"),
        no_cache_flag=re.search(r"(?:^|\s)--no-cache\s", flat) is not None,
        views=_choices(flat, "--view <LIST> Views:", "."),
        view_list="--view <LIST>" in flat,
        analyzers=_choices(flat, "--analyze <LIST> Analyzers to run:", "[default"),
        reads_gitignore=re.search(r"(?:^|\s)--no-gitignore\s", flat) is not None,
        json_format=re.search(r"--format <FORMAT>[^-]*\bjson\b", flat) is not None,
        json_flag=re.search(r"(?:^|\s)--json\s", flat) is not None,
    )


def _run_text(argv: Sequence[str], *, timeout: float = 30.0) -> Tuple[int, str]:
    completed = subprocess.run(
        list(argv),
        stdin=subprocess.DEVNULL,
        capture_output=True,
        timeout=timeout,
        check=False,
        env={**os.environ, **measure.BASE_ENVIRONMENT},
    )
    text = (completed.stdout + completed.stderr).decode("utf-8", errors="replace")
    return completed.returncode, text


def probe_build(binary: Path) -> Capabilities:
    """Ask a build what it accepts. Nothing is inferred from its name or position."""
    code, help_text = _run_text([str(binary), "--help"])
    if code != 0:
        raise HistoryError(f"{binary.name} --help exited with {code}")
    _code, version = _run_text([str(binary), "--version"])
    return capabilities(help_text, " ".join(version.split())[:300])


def probe_job_argv(job_id: str) -> Tuple[Tuple[str, ...], Tuple[str, ...]]:
    """The timed and setup command templates of one harness job.

    The timed command is the harness's own, oracle included. A job that reads a
    snapshot gets a setup that writes one with the same build, as the harness prepares
    it, into the sample's cache directory.
    """
    job = measure.PROBE_JOBS[job_id]
    argv = tuple(PROBE_SNAPSHOT if item == "{snapshot}" else item for item in job.argv)
    setup: Tuple[str, ...] = ()
    if job.needs_snapshot:
        setup = (
            "{binary}",
            job.snapshot_preparation_mode,
            "--root",
            "{root}",
            "--snapshot",
            PROBE_SNAPSHOT,
        )
    return argv, setup


def probe_support(probe: Path, job_id: str) -> Optional[str]:
    """Why a perf_probe build cannot run harness job ``job_id``, or None if it can.

    The probe is pointed at a root that does not exist, so nothing is measured: a known
    mode fails on the root, an unknown mode or flag fails first. Each mode is asked about
    alone, then with the job's flags, because early probes reject an unknown flag before
    they look at the mode.
    """
    argv, setup = probe_job_argv(job_id)
    with tempfile.TemporaryDirectory(prefix="fdu-history-probe-") as scratch:
        missing = str(Path(scratch) / "missing")
        for template in (argv, setup):
            if not template:
                continue
            mode = template[1]
            _code, text = _run_text([str(probe), mode, "--root", missing])
            if "unknown mode" in text:
                return f"this build's perf_probe has no {mode} mode"
            expanded = [
                str(probe)
                if item == "{binary}"
                else missing
                if item == "{root}"
                else item.replace("{cache}", missing)
                for item in template
            ]
            _code, text = _run_text(expanded)
            if "unknown argument" in text:
                return f"this build's perf_probe does not accept the {job_id} flags"
    return None


# --------------------------------------------------------------------------------------
# Command shapes


@dataclass(frozen=True)
class Shape:
    """How one build runs one component."""

    argv: Tuple[str, ...]
    setup_argv: Tuple[str, ...]
    cache_scope: str
    state: str
    variant: str
    uses_probe: bool = False
    #: The untimed command the answer check reads root totals from, or () for a probe.
    totals_argv: Tuple[str, ...] = ()
    #: The harness job a probe shape runs.
    job: Optional[str] = None


@dataclass(frozen=True)
class Unsupported:
    reason: str


def _totals_argv(caps: Capabilities, cache_off: Tuple[str, ...]) -> Tuple[str, ...]:
    """The era's JSON root-totals command, run untimed with the cache off."""
    if "summary" in caps.views and caps.json_format:
        middle: Tuple[str, ...] = ("--format", "json", "--view", "summary")
    elif caps.json_format:
        middle = ("--format", "json", "--depth", "0")
    elif caps.json_flag:
        return ("{binary}", *cache_off, "--json", "--depth", "0", "{root}")
    else:
        return ()
    return ("{binary}", *cache_off, *middle, "--color", "never", "{root}")


def select_job(request: Request, job: Optional[str]) -> str:
    """The one harness job a cell times, out of the component's."""
    if job is None:
        if len(request.probe_jobs) != 1:
            raise HistoryError(
                f"this component times {len(request.probe_jobs)} jobs "
                f"({', '.join(request.probe_jobs)}); choose one with --job"
            )
        return request.probe_jobs[0]
    if job not in request.probe_jobs:
        raise HistoryError(f"job {job!r} is not one of {', '.join(request.probe_jobs)}")
    return job


def shape_for(
    entry: Mapping[str, Any],
    caps: Capabilities,
    *,
    probe_check: Optional[Callable[[str], Optional[str]]] = None,
    job: Optional[str] = None,
) -> Union[Shape, Unsupported]:
    """Map one manifest component onto one build, or say why the build cannot run it.

    ``probe_check`` answers, for a harness job, why this build's perf_probe cannot run
    it, or None; it is absent when no probe was supplied for the build. ``job`` picks
    one job of a probe component that has several.
    """
    state = CACHE_STATES[entry["cache_state"]]
    request = parse_command(entry["command"])
    cache_off = caps.cache_off()
    if request.probe_jobs:
        job_id = select_job(request, job)
        # The job decides its start state: a fresh directory per process, with an
        # untimed snapshot written into it first when the job reads one.
        argv, setup = probe_job_argv(job_id)
        if probe_check is None:
            return Unsupported("no perf_probe build was supplied for this milestone")
        reason = probe_check(job_id)
        if reason is not None:
            return Unsupported(reason)
        return Shape(
            argv=argv,
            setup_argv=setup,
            cache_scope="sample",
            state="filled" if setup else "empty",
            variant=f"probe-{job_id}",
            uses_probe=True,
            job=job_id,
        )
    if state == "job":
        raise HistoryError(f"cache state {entry['cache_state']!r} needs a probe command")

    views = request.views or tuple(
        ANALYZER_VIEWS[name] for name in request.analyzers if name in ANALYZER_VIEWS
    )
    missing_views = [view for view in views if view not in caps.views]
    if missing_views:
        return Unsupported(f"no {', '.join(missing_views)} view")
    if len(request.views) > 1 and not caps.view_list:
        return Unsupported("--view takes no list")
    needed = {VIEW_ANALYZERS[view] for view in views if view in VIEW_ANALYZERS}
    if request.analyzers:
        # The manifest's analyzers, as written; they must cover what its views display.
        uncovered = needed - set(request.analyzers)
        if uncovered and "all" not in request.analyzers:
            raise HistoryError(
                f"manifest command {entry['command']!r} shows views that need "
                f"{', '.join(sorted(uncovered))}, which it does not analyze"
            )
        analyzers = list(request.analyzers)
    else:
        analyzers = sorted(needed, key=ANALYZER_ORDER.index)
    missing_analyzers = [name for name in analyzers if name not in caps.analyzers]
    if missing_analyzers:
        return Unsupported(f"no {', '.join(missing_analyzers)} analyzer")
    if cache_off is None:
        return Unsupported("no way to switch the persisted cache off for the answer check")

    if state == "empty":
        # A first run with empty caches runs with the cache off, whether or not the
        # manifest spells it; any other policy contradicts the state.
        if request.cache_policy not in (None, "off"):
            raise HistoryError(
                f"an empty-cache component cannot run --cache {request.cache_policy}"
            )
        cache: Tuple[str, ...] = cache_off
        variant = "no-cache" if cache_off == ("--no-cache",) else "cache-off"
        scope = "sample"
    elif state == "steady":
        if request.cache_policy is not None:
            raise HistoryError("a default-policy component cannot also name a cache policy")
        cache, variant, scope = (), "default", "tool"
    elif state == "filled":
        if request.cache_policy is None:
            cache, variant = (), "default"
        elif request.cache_policy == "on":
            spelled = caps.cache_on()
            if spelled is None:
                return Unsupported("no cache policy that reads and writes the snapshot")
            cache = spelled
            variant = "cache-" + spelled[1] if spelled else "default"
        else:
            raise HistoryError(f"cannot map --cache {request.cache_policy}")
        scope = "sample"
    else:
        raise HistoryError(f"cache state {entry['cache_state']!r} needs a probe command")

    flags: List[str] = list(cache)
    if analyzers:
        flags += ["--analyze", ",".join(analyzers)]
    if request.views:
        flags += ["--view", ",".join(request.views)]
    argv = ("{binary}", *flags, "--color", "never", "{root}")
    return Shape(
        argv=argv,
        setup_argv=argv if state == "filled" else (),
        cache_scope=scope,
        state=state,
        variant=variant,
        totals_argv=_totals_argv(caps, cache_off),
    )


# --------------------------------------------------------------------------------------
# Builds


@dataclass
class Build:
    label: str
    binary: Path
    probe: Optional[Path] = None
    caps: Optional[Capabilities] = None
    shape: Optional[Union[Shape, Unsupported]] = None
    #: The command-line binary's hash, taken when the build is asked what it accepts; a
    #: probe cell's timed binary is the probe, so the build itself is identified here.
    cli_sha256: Optional[str] = None

    @property
    def supported(self) -> bool:
        return isinstance(self.shape, Shape)

    @property
    def timed_binary(self) -> Path:
        assert isinstance(self.shape, Shape)
        if self.shape.uses_probe:
            assert self.probe is not None
            return self.probe
        return self.binary


def parse_builds(specifications: Sequence[str], probes: Sequence[str]) -> List[Build]:
    builds: List[Build] = []
    for specification in specifications:
        label, separator, path = specification.partition("=")
        if not separator or not label or not path:
            raise HistoryError(f"--build {specification!r} must be LABEL=PATH")
        binary = Path(path).resolve()
        if not binary.is_file():
            raise HistoryError(f"build {label} does not exist: {binary.name}")
        builds.append(Build(label=label, binary=binary))
    labels = [build.label for build in builds]
    if len(set(labels)) != len(labels):
        raise HistoryError("build labels must be unique")
    by_label = {build.label: build for build in builds}
    for specification in probes:
        label, separator, path = specification.partition("=")
        if not separator or label not in by_label:
            raise HistoryError(f"--probe {specification!r} must be LABEL=PATH for a --build")
        probe = Path(path).resolve()
        if not probe.is_file():
            raise HistoryError(f"probe for {label} does not exist: {probe.name}")
        by_label[label].probe = probe
    return builds


def map_builds(
    builds: Sequence[Build], entry: Mapping[str, Any], *, job: Optional[str] = None
) -> None:
    """Probe every build and decide its shape for this component."""
    for build in builds:
        build.caps = probe_build(build.binary)
        build.cli_sha256 = _sha256(build.binary)
        probe = build.probe
        check = (lambda job_id, probe=probe: probe_support(probe, job_id)) if probe else None
        build.shape = shape_for(entry, build.caps, probe_check=check, job=job)


def probe_output_reader(
    job_id: str,
) -> Callable[[bytes, Mapping[str, Any]], Tuple[Dict[str, Optional[int]], List[str]]]:
    """Read one probe sample as the harness does: its component timer and its oracle."""
    job = measure.PROBE_JOBS[job_id]

    def read(
        stdout: bytes, fingerprint: Mapping[str, Any]
    ) -> Tuple[Dict[str, Optional[int]], List[str]]:
        probe, problems = measure._read_probe_output(stdout, allow_incomplete=job.allow_incomplete)
        reasons = list(problems)
        component_ns = probe.get("component_ns") if probe else None
        if not isinstance(component_ns, int):
            component_ns = None
            reasons.append("probe reported no component_ns")
        if probe and job.verify_oracle:
            check = measure._ORACLES.get(job.oracle)
            if check is None:
                reasons.append(f"job declares unknown oracle {job.oracle!r}")
            else:
                summary = _legacy_summary(job.oracle, probe.get("summary"), fingerprint)
                disagreement = check(dict(fingerprint), summary)
                if disagreement is not None:
                    reasons.append(disagreement)
        return {"component_ns": component_ns}, reasons

    return read


#: Summary fields the probe gained after the first milestone builds. A probe that does
#: not report one at all is held to the rest of the index-digest oracle, whose engine
#: digest covers every entry's size, times, and identity and so implies the field; a
#: probe that reports a wrong value still fails.
LEGACY_ABSENT_FIELDS = ("newest_file_mtime_ns",)


def _legacy_summary(oracle: str, summary: Any, fingerprint: Mapping[str, Any]) -> Any:
    if oracle != "index-digest" or not isinstance(summary, dict):
        return summary
    if summary.get("engine_digest") is None:
        return summary
    absent = [field for field in LEGACY_ABSENT_FIELDS if field not in summary]
    return {**summary, **{field: fingerprint.get(field) for field in absent}}


def contract_for(
    component_id: str, entry: Mapping[str, Any], shape: Shape, *, timing: str = "wall"
) -> compare_tools.ToolContract:
    """The compare_tools contract one build's shape runs under."""
    if timing not in TIMINGS:
        raise HistoryError(f"unknown timing {timing!r}; choose from {', '.join(TIMINGS)}")
    if timing != "wall" and not shape.uses_probe:
        raise HistoryError("only a probe job reports a component timer")
    return compare_tools.ToolContract(
        name=f"fdu-index/{component_id}/{shape.variant}",
        work_class=f"index-{component_id}",
        description=(
            f"{entry['title']}: `{entry['command']}` as this build spells it, "
            f"{entry['cache_state']}"
        ),
        argv=shape.argv,
        version_argv=() if shape.uses_probe else ("{binary}", "--version"),
        writes_cache=True,
        measures=f"fdu-index:{component_id}",
        cache_scope=shape.cache_scope,
        setup_argv=shape.setup_argv,
        fdu_anchor=True,
        stdout_metrics=probe_output_reader(shape.job) if shape.job else None,
        primary_metric=TIMINGS[timing],
    )


# --------------------------------------------------------------------------------------
# Where the timed inputs live


def storage_location(path: Path) -> str:
    """``internal``, ``external``, or ``unknown`` for the volume holding ``path``."""
    if sys.platform != "darwin":
        return "unknown"
    try:
        completed = subprocess.run(
            ["df", "-P", str(path)], capture_output=True, text=True, timeout=10, check=True
        )
        device = completed.stdout.strip().splitlines()[-1].split()[0]
        info = subprocess.run(
            ["diskutil", "info", "-plist", device], capture_output=True, timeout=10, check=True
        )
        document = plistlib.loads(info.stdout)
    except (OSError, subprocess.SubprocessError, IndexError, plistlib.InvalidFileException):
        return "unknown"
    internal = document.get("Internal")
    if internal is True:
        return "internal"
    if internal is False:
        return "external"
    return "unknown"


def storage_check(
    root: Path, builds: Sequence[Build], locate: Callable[[Path], str] = storage_location
) -> Dict[str, str]:
    """Where every timed input lives, by role; no path is recorded."""
    roles: Dict[str, Path] = {
        "subject": root,
        "harness": Path(__file__).resolve().parent,
        "python": Path(sys.executable),
        "temporary": Path(tempfile.gettempdir()),
    }
    for build in builds:
        if build.supported:
            roles[f"binary:{build.label}"] = build.timed_binary
    return {role: locate(path) for role, path in roles.items()}


# --------------------------------------------------------------------------------------
# The answer check


_FOOTER = re.compile(r"^(?:perf:|Performance:).*$", re.MULTILINE)


def footer(text: str, root: Path) -> Optional[str]:
    """The run's performance line, with the root redacted, or None."""
    lines = _FOOTER.findall(text)
    if not lines:
        return None
    return lines[-1].replace(str(root), "ROOT")


def cache_evidence(line: Optional[str]) -> str:
    """What a performance line says the run started from: warm, cold, or unknown.

    Builds before 0.1.0 print no such line, so their state is unknown from output alone.
    """
    if line is None:
        return "unknown"
    if re.search(r"\bwarm\b", line):
        return "warm"
    if "cold scan" in line:
        return "cold"
    return "unknown"


_ANALYSIS = re.compile(
    r"analysis ([0-9][0-9,]*) fresh(?: at [^;]*?files/s)?, ([0-9][0-9,]*) cached"
)


def analysis_counts(line: Optional[str]) -> Optional[Dict[str, int]]:
    """Fresh and cached analysis counts from a performance line, or None."""
    matched = _ANALYSIS.search(line or "")
    if matched is None:
        return None
    fresh, cached = (int(value.replace(",", "")) for value in matched.groups())
    return {"fresh": fresh, "cached": cached}


def root_totals(document: Mapping[str, Any]) -> Dict[str, Any]:
    """The five root tallies and completeness from any era's JSON report."""
    totals = document.get("tree")
    complete = document.get("complete")
    if not isinstance(totals, Mapping):
        reports = document.get("reports") or []
        first = reports[0] if reports and isinstance(reports[0], Mapping) else {}
        totals = first.get("summary") or first.get("tree")
        status = document.get("status")
        if isinstance(status, Mapping):
            complete = status.get("complete")
    if not isinstance(totals, Mapping):
        raise HistoryError("the JSON report carries no root totals")
    return {
        "files": totals.get("files"),
        "dirs": totals.get("dirs"),
        "apparent_bytes": totals.get("bytes"),
        "allocated_bytes": totals.get("allocated"),
        "complete": complete,
        "ignored": totals.get("ignored"),
    }


def fingerprint_totals(fingerprint: Mapping[str, Any]) -> Dict[str, Any]:
    """The four tallies an fdu root report must agree with the independent walk on."""
    counts, sizes = fingerprint["counts"], fingerprint["sizes"]
    return {
        "files": counts["files"],
        # A root report counts descendants; the fingerprint counts the root too.
        "dirs": counts["directories"] - 1,
        "apparent_bytes": sizes["apparent_bytes"],
        "allocated_bytes": sizes["allocated_bytes"],
    }


Runner = Callable[[Sequence[str], Mapping[str, str]], Tuple[int, bytes, bytes]]


def _spawn_untimed(argv: Sequence[str], overrides: Mapping[str, str]) -> Tuple[int, bytes, bytes]:
    completed = subprocess.run(
        list(argv),
        stdin=subprocess.DEVNULL,
        capture_output=True,
        timeout=measure.DEFAULT_TIMEOUT_SECONDS,
        check=False,
        env={**measure.BASE_ENVIRONMENT, **overrides},
    )
    return completed.returncode, completed.stdout, completed.stderr


def _expand(
    template: Sequence[str], binary: Path, root: Path, cache: Optional[str] = None
) -> List[str]:
    return compare_tools._expand(template, binary, root, cache=Path(cache) if cache else None)


def check_build(
    build: Build, root: Path, fingerprint: Mapping[str, Any], run: Runner = _spawn_untimed
) -> Dict[str, Any]:
    """Run one build's component once, untimed, from its declared start state."""
    shape = build.shape
    assert isinstance(shape, Shape) and build.caps is not None
    binary = build.timed_binary
    record: Dict[str, Any] = {
        "label": build.label,
        "reads_gitignore": build.caps.reads_gitignore,
    }
    with tempfile.TemporaryDirectory(prefix="fdu-history-check-") as cache:
        overrides = {"XDG_CACHE_HOME": cache}
        exits: List[int] = []
        if shape.setup_argv:
            exits.append(run(_expand(shape.setup_argv, binary, root, cache), overrides)[0])
        if shape.state == "steady":
            # The steady state is what the command finds after earlier runs of itself.
            exits.append(run(_expand(shape.argv, binary, root, cache), overrides)[0])
        code, stdout, stderr = run(_expand(shape.argv, binary, root, cache), overrides)
        exits.append(code)
        cache_files = sum(len(files) for _base, _dirs, files in os.walk(cache))
    record["exit_codes"] = exits
    record["exit_ok"] = all(code == 0 for code in exits)
    record["cache_files_after"] = cache_files
    text = (stdout + stderr).decode("utf-8", errors="replace")
    line = footer(text, root)
    record["performance_line"] = line
    record["cache_evidence"] = cache_evidence(line)
    record["analysis"] = analysis_counts(line)
    totals, _error = compare_tools._code_table_totals("fdu", stdout, "")
    record["code_totals"] = totals

    if shape.uses_probe:
        # The timed probe command runs the job's oracle itself, as every sample will.
        assert shape.job is not None
        reported, problems = probe_output_reader(shape.job)(stdout, fingerprint)
        record["oracle_exit_code"] = code
        record["oracle_error"] = "; ".join(problems) or None
        record["component_ns"] = reported.get("component_ns")
        record["totals"] = None
        return record

    with tempfile.TemporaryDirectory(prefix="fdu-history-check-") as cache:
        code, stdout, _stderr = run(
            _expand(shape.totals_argv, build.binary, root), {"XDG_CACHE_HOME": cache}
        )
    record["totals_exit_code"] = code
    try:
        record["totals"] = root_totals(json.loads(stdout))
    except (json.JSONDecodeError, HistoryError) as error:
        record["totals"] = None
        record["totals_error"] = str(error)
    return record


def answer_groups(
    records: Sequence[Mapping[str, Any]], fingerprint: Mapping[str, Any]
) -> Dict[str, Any]:
    """Group the checked builds by whether they read .gitignore, as before."""
    expected = fingerprint_totals(fingerprint)
    groups: Dict[str, Dict[str, Any]] = {}
    for record in records:
        key = "with_gitignore" if record["reads_gitignore"] else "without_gitignore"
        totals = record.get("totals")
        group = groups.setdefault(
            key,
            {
                "labels": [],
                "totals": totals,
                "all_identical": True,
                "exit_codes_zero": True,
                "matches_fingerprint": True,
                "oracle_errors": [],
            },
        )
        group["labels"].append(record["label"])
        group["all_identical"] &= group["totals"] == totals
        is_probe = "oracle_exit_code" in record
        group["exit_codes_zero"] &= (
            bool(record["exit_ok"])
            and record.get("totals_exit_code", record.get("oracle_exit_code", 0)) == 0
            # A command-line build whose totals did not parse has no answer to compare.
            and (is_probe or totals is not None)
        )
        if totals is not None:
            group["matches_fingerprint"] &= all(totals.get(k) == v for k, v in expected.items())
        if record.get("oracle_error"):
            group["oracle_errors"].append(f"{record['label']}: {record['oracle_error']}")
    for group in groups.values():
        if group["totals"] is None:
            group["matches_fingerprint"] = None
            group.pop("totals")
    return groups


def answer_problems(groups: Mapping[str, Mapping[str, Any]]) -> List[str]:
    """Why the cell must not be timed: a failed run or two answers in one group."""
    problems: List[str] = []
    for key, group in groups.items():
        if not group["exit_codes_zero"]:
            problems.append(f"{key}: a build exited non-zero or its totals did not parse")
        if not group["all_identical"]:
            problems.append(f"{key}: builds disagree on the root totals")
        problems.extend(f"{key}: {error}" for error in group.get("oracle_errors") or [])
    return problems


# --------------------------------------------------------------------------------------
# The history cell


def _ms(value: Optional[float]) -> Optional[float]:
    return None if value is None else round(value / 1e6, 2)


def _quartiles(values: Sequence[float]) -> Optional[List[Optional[float]]]:
    if len(values) < 4:
        return None
    q = statistics.quantiles(values, n=4, method="inclusive")
    return [_ms(q[0]), _ms(q[2])]


def _boot_ms(values: Sequence[float]) -> Optional[List[Optional[float]]]:
    low, high = measure._bootstrap_median_interval(list(values))
    return None if low is None else [_ms(low), _ms(high)]


def _pct_interval(ratios: Sequence[float]) -> Optional[List[float]]:
    low, high = measure._bootstrap_median_interval(list(ratios))
    return None if low is None else [round(low * 100, 2), round(high * 100, 2)]


def reference_key(reference: str) -> str:
    """``vs_v0_3_0_paired_harness`` for reference build ``v0.3.0``."""
    return "vs_" + re.sub(r"[^A-Za-z0-9]", "_", reference) + "_paired_harness"


METADATA_FIELDS = (
    "role",
    "short",
    "includes",
    "after_experiment",
    "ref",
    "commit",
    "commit_choice",
    "author_date",
    "commit_date",
)
SUBJECT_FIELDS = (
    "title",
    "provenance",
    "record_shape",
    "shape_difference",
    "gitignore_files",
    "recipe",
    "generator_command",
    "generator_revision",
)


def milestone_metadata(source: Optional[Mapping[str, Any]]) -> Dict[str, Dict[str, Any]]:
    """Per-label descriptive fields from an earlier history cell, which the run cannot know."""
    result: Dict[str, Dict[str, Any]] = {}
    for item in (source or {}).get("milestones") or []:
        result[item["label"]] = {key: item[key] for key in METADATA_FIELDS if key in item}
    return result


def _platform(host: Mapping[str, Any]) -> str:
    system = host.get("system")
    return {"Darwin": "macOS", "Linux": "Linux"}.get(str(system), str(system))


def build_cell(
    document: Mapping[str, Any],
    *,
    builds: Sequence[Build],
    entry: Mapping[str, Any],
    manifest: Mapping[str, Any],
    reference: str,
    metadata_source: Optional[Mapping[str, Any]],
    answer_check: Mapping[str, Any],
    storage: Mapping[str, Any],
    harness_revision: Optional[str],
    run_artifact: str,
    manifest_entry: Optional[Mapping[str, Any]] = None,
    override_note: Optional[str] = None,
) -> Dict[str, Any]:
    """Summarize one compare_tools run as a history cell.

    Every comparison is on the cell's timed metric: the process's wall time, or the
    probe's ``component_ns`` for a component-timed probe job. ``wall_ms`` is always the
    process's wall time; a component-timed cell adds ``component_ms`` and keeps the wall
    comparison beside the timed one.
    """
    samples = [s for s in document["samples"] if not s["warmup"] and s["valid"]]
    supported = [build.label for build in builds if build.supported]
    first = supported[0]
    metadata = milestone_metadata(metadata_source)
    vs_reference = reference_key(reference)
    metric = document["tools"][reference].get("primary_metric", "wall_ns")
    timed_key = "component_ms" if metric == "component_ns" else "wall_ms"

    def own(name: str) -> Dict[Tuple[str, int], Mapping[str, Any]]:
        return {
            (s["pair"], s["ordinal"]): s
            for s in samples
            if s["tool"] == name and s["metrics"].get(metric) is not None
        }

    def by_round(name: str, partner: str) -> Dict[int, int]:
        """One timed value per round: a competitor's own sample, or for the anchor the
        sample adjacent to ``partner`` in that round."""
        pair = partner if name == reference else name
        return {o: s["metrics"][metric] for (p, o), s in own(name).items() if p == pair}

    def paired_versus_reference(label: str, measured: str) -> Optional[Dict[str, Any]]:
        statistics_entry = document["statistics"][label]
        paired = statistics_entry["competitor_vs_fdu"].get(measured)
        anchor_in_pair = statistics_entry["tools"][reference]["metrics"].get(measured)
        if paired is None:
            return None
        return {
            "meaning": f"(this - adjacent {reference}) / adjacent {reference} in {measured}, "
            "median of adjacent pairs; positive = this binary slower",
            "pairs": paired["pairs"],
            "median_change_pct": paired["median_change_pct"],
            "ci95_change_pct": paired["ci95_change_pct"],
            "direction": paired["direction"],
            "anchor_median_ms_in_these_pairs": _ms(anchor_in_pair["median"])
            if anchor_in_pair
            else None,
            "speed_ratio_this_over_reference": round(1 + paired["median_change_pct"] / 100, 4),
        }

    def anchor_normalized(name: str) -> Dict[int, float]:
        if name == reference:
            return {}
        mine = by_round(name, name)
        anchors = by_round(reference, name)
        return {o: mine[o] / anchors[o] for o in mine if o in anchors}

    def round_matched(name: str, other: str) -> Dict[str, Any]:
        """Same-round ratios name/other - 1; adjacent only when one is the anchor."""
        if name == reference:
            mine, theirs = by_round(reference, other), by_round(other, other)
        elif other == reference:
            mine, theirs = by_round(name, name), by_round(reference, name)
        else:
            mine, theirs = by_round(name, name), by_round(other, other)
        ordinals = sorted(set(mine) & set(theirs))
        ratios = [(mine[o] - theirs[o]) / theirs[o] for o in ordinals]
        adjacent = reference in (name, other)
        median = statistics.median(ratios) if ratios else None
        return {
            "method": "adjacent pairs (one side is the anchor)"
            if adjacent
            else "same schedule round, not adjacent",
            "pairs": len(ratios),
            "median_change_pct": round(median * 100, 2) if median is not None else None,
            "ci95_change_pct": _pct_interval(ratios),
            "median_ratio_x": round(1 + median, 4) if median is not None else None,
        }

    def versus_first(label: str) -> Optional[Dict[str, Any]]:
        if label == first:
            return None
        if label == reference:
            first_norm = anchor_normalized(first)
            norm_ratios = [1.0 / first_norm[o] - 1.0 for o in sorted(first_norm)]
        else:
            norm, first_norm = anchor_normalized(label), anchor_normalized(first)
            norm_ratios = [
                norm[o] / first_norm[o] - 1.0 for o in sorted(set(norm) & set(first_norm))
            ]
        matched = round_matched(label, first)
        matched["median_speedup_x"] = (
            round(1 / matched["median_ratio_x"], 3) if matched["median_ratio_x"] else None
        )
        return {
            "round_matched": matched,
            "anchor_normalized": {
                "method": "per round, (this / its adjacent anchor) / "
                "(first build / its adjacent anchor) - 1",
                "pairs": len(norm_ratios),
                "median_change_pct": round(statistics.median(norm_ratios) * 100, 2)
                if norm_ratios
                else None,
                "ci95_change_pct": _pct_interval(norm_ratios),
            },
        }

    checked = {record["label"]: record for record in answer_check["builds"]}
    milestones: List[Dict[str, Any]] = []
    previous: Optional[str] = None
    for build in builds:
        assert build.caps is not None
        item: Dict[str, Any] = {"label": build.label, **metadata.get(build.label, {})}
        item["version"] = build.caps.version
        item["reads_gitignore"] = build.caps.reads_gitignore
        if not isinstance(build.shape, Shape):
            assert isinstance(build.shape, Unsupported)
            item.update(
                {
                    "supported": False,
                    "unsupported_reason": build.shape.reason,
                    "contract": None,
                    "command": None,
                    "timed_samples": 0,
                    "wall_ms": None,
                    "peak_rss_mib": None,
                    "cpu_s_median": None,
                    vs_reference: None,
                    "vs_first_derived": None,
                    "vs_previous_milestone_derived": None,
                }
            )
            milestones.append(item)
            continue
        label = build.label
        tool = document["tools"][label]
        walls = [s["metrics"]["wall_ns"] for s in own(label).values()]
        rss = [
            s["metrics"]["peak_rss_bytes"]
            for s in own(label).values()
            if s["metrics"].get("peak_rss_bytes") is not None
        ]
        overall = document["overall"][label]["metrics"]
        wall = overall.get("wall_ns") or {}
        rss_overall = overall.get("peak_rss_bytes")
        item.update(
            {
                "supported": True,
                "binary_sha256": tool["binary_sha256"],
                "binary_size_bytes": tool["binary_size_bytes"],
                "contract": tool["contract"],
                "command": tool["command"],
                "setup_command": tool.get("setup_command"),
                "cache_scope": tool.get("cache_scope"),
                "cache_evidence": (checked.get(label) or {}).get("cache_evidence"),
                "timed_samples": len(walls),
                "wall_ms": {
                    "source": "harness median of this binary's valid timed samples"
                    + (
                        f"; the anchor's {len(walls)} samples beside every competitor"
                        if label == reference
                        else ""
                    ),
                    "median": _ms(wall.get("median")),
                    "min": _ms(wall.get("min")),
                    "max": _ms(wall.get("max")),
                    "iqr_derived": _quartiles(walls),
                    "median_bootstrap95_derived": _boot_ms(walls),
                },
                "peak_rss_mib": {
                    "source": "harness rusage max RSS of the child, median of timed samples",
                    "median": round(rss_overall["median"] / 2**20, 1) if rss_overall else None,
                    "max": round(max(rss) / 2**20, 1) if rss else None,
                },
                "cpu_s_median": {
                    "user": round(overall["user_cpu_ns"]["median"] / 1e9, 3)
                    if overall.get("user_cpu_ns")
                    else None,
                    "system": round(overall["system_cpu_ns"]["median"] / 1e9, 3)
                    if overall.get("system_cpu_ns")
                    else None,
                },
            }
        )
        if build.shape.uses_probe:
            item["cli_binary_sha256"] = build.cli_sha256
            item["job"] = build.shape.job
        if overall.get("component_ns"):
            component = overall["component_ns"]
            values = [
                s["metrics"]["component_ns"]
                for s in own(label).values()
                if s["metrics"].get("component_ns") is not None
            ]
            item["component_ms"] = {
                "source": "probe-reported component_ns, harness median of valid timed samples",
                "median": _ms(component.get("median")),
                "min": _ms(component.get("min")),
                "max": _ms(component.get("max")),
                "iqr_derived": _quartiles(values),
                "median_bootstrap95_derived": _boot_ms(values),
            }
        item[vs_reference] = None if label == reference else paired_versus_reference(label, metric)
        # The metric the cell does not time, compared the same way, where it was recorded.
        other = "wall_ns" if metric == "component_ns" else "component_ns"
        if overall.get(other):
            suffix = "_wall" if other == "wall_ns" else "_component"
            item[vs_reference + suffix] = (
                None if label == reference else paired_versus_reference(label, other)
            )
        derived = versus_first(label)
        if derived is not None:
            derived = {"first": first, **derived}
        item["vs_first_derived"] = derived
        if builds[0].label == first and builds[0].supported:
            # The committed cells' name for the same figure when the first build ran.
            item["vs_prework_derived"] = (
                None
                if derived is None
                else {key: value for key, value in derived.items() if key != "first"}
            )
        item["vs_previous_milestone_derived"] = (
            None if previous is None else {"previous": previous, **round_matched(label, previous)}
        )
        previous = label
        milestones.append(item)

    def boundary(key: str) -> List[Any]:
        return [
            s["host_pressure"][side][key]
            for s in document["samples"]
            for side in ("before", "after")
            if (s.get("host_pressure") or {}).get(side, {}).get(key) is not None
        ]

    pressure, loads = boundary("cpu_busy_pct"), boundary("load_1m")
    conditions = document["conditions"]
    quiet = conditions["host_acceptance"]["quiet_max_cpu_busy_pct"]
    by_label = {item["label"]: item for item in milestones}
    headline: Optional[Dict[str, Any]] = None
    if first != reference:
        first_item, anchor_item = by_label[first], by_label[reference]
        pair = first_item[vs_reference]
        fdu_vs = document["statistics"][first]["fdu_vs_competitor"].get(metric)
        if pair is not None and fdu_vs is not None:
            headline = {
                "first_label": first,
                "timed_metric": metric,
                "median_ratio_prework_over_v0_3_0": round(
                    first_item[timed_key]["median"] / anchor_item[timed_key]["median"], 3
                ),
                "median_ratio_using_anchor_samples_in_prework_pairs": round(
                    first_item[timed_key]["median"] / pair["anchor_median_ms_in_these_pairs"], 3
                ),
                "paired_harness_prework_vs_v0_3_0_change_pct": pair["median_change_pct"],
                "paired_harness_prework_vs_v0_3_0_ci95_pct": pair["ci95_change_pct"],
                "paired_speedup_x": round(1 + pair["median_change_pct"] / 100, 3),
                "paired_speedup_x_ci95": [
                    round(1 + value / 100, 3) for value in pair["ci95_change_pct"]
                ],
                "paired_harness_v0_3_0_vs_prework_change_pct": fdu_vs["median_change_pct"],
                "paired_harness_v0_3_0_vs_prework_ci95_pct": fdu_vs["ci95_change_pct"],
                "naming": "for schema compatibility the keys say prework and v0_3_0; they "
                "mean first_label and the reference build",
            }

    tree_document = document["tree"]
    source_subject = (metadata_source or {}).get("subject") or {}
    subject: Dict[str, Any] = {"label": tree_document.get("label")}
    if source_subject.get("label") == tree_document.get("label"):
        subject.update(
            {key: source_subject[key] for key in SUBJECT_FIELDS if key in source_subject}
        )
    locations = set(storage["locations"].values())
    subject.update(
        {
            "storage": "internal SSD"
            if locations == {"internal"}
            else "not established"
            if "unknown" in locations
            else "includes external storage",
            "storage_detail": conditions.get("storage") or "",
            "storage_check": dict(storage["locations"]),
            "fingerprint_schema": tree_document.get("schema"),
            "root_id": tree_document.get("root_id"),
            "engine_digest": tree_document.get("engine_digest"),
            "engine_digest_after": document["tree_after_digest"],
            "counts": tree_document["counts"],
            "sizes": tree_document["sizes"],
            "max_depth": tree_document.get("max_depth"),
            "hardlinks": tree_document.get("hardlinks"),
            "baseline_drift": document["baseline_drift"],
            "mutated_during_run": document["tree_mutated_during_run"],
            "answer_check_groups": answer_check["groups"],
        }
    )

    defined = manifest_entry or entry
    overridden = any(entry.get(key) != defined.get(key) for key in ("command", "cache_state")) or (
        override_note is not None
    )
    jobs = sorted(
        {build.shape.job for build in builds if isinstance(build.shape, Shape) and build.shape.job}
    )
    return {
        "schema": SUMMARY_SCHEMA,
        "component": entry["id"],
        "component_title": entry["title"],
        "manifest_version": manifest["version"],
        "manifest_command": defined["command"],
        "cache_state": defined["cache_state"],
        "override": {
            "note": override_note,
            "command": entry["command"],
            "cache_state": entry["cache_state"],
        }
        if overridden
        else None,
        "timed_metric": metric,
        "job": jobs[0] if jobs else None,
        "tree_key": entry["tree"],
        "platform": _platform(document["host"]),
        "reference_build": reference,
        "run_artifact": run_artifact,
        "harness": {
            "module": "benchmarks.realtree.history, timing through benchmarks.realtree."
            "compare_tools with per-build command contracts",
            "harness_revision": harness_revision,
            "artifact_schema": document["schema"],
            "schedule": conditions["schedule"],
            "stopping_rule": conditions["stopping_rule"],
            "confidence_interval": conditions["confidence_interval"],
        },
        "subject": subject,
        "host": document["host"],
        "regime": {
            "campaign_stage": conditions["campaign_stage"],
            "host_regime": conditions["host_regime"],
            "os_cache": conditions["os_cache"],
            "quiet_gate_cpu_busy_pct": quiet,
            "initial": conditions["host_acceptance"]["initial"],
            "final": conditions["host_acceptance"]["final"],
            "cpu_busy_pct_at_sample_boundaries": {
                "n": len(pressure),
                "min": min(pressure),
                "median": statistics.median(pressure),
                "max": max(pressure),
                "share_above_quiet_gate": round(
                    sum(p > quiet for p in pressure) / len(pressure), 3
                ),
            }
            if pressure
            else None,
            "load_1m_at_sample_boundaries": {
                "min": min(loads),
                "median": statistics.median(loads),
                "max": max(loads),
            }
            if loads
            else None,
            "thermal_pressure_seen": sorted(set(map(str, boundary("thermal_pressure")))),
            "power_source_seen": sorted(set(map(str, boundary("power_source")))),
        },
        "rounds": {
            "trials": conditions["trials"],
            "warmups": conditions["warmups"],
            "processes": len(document["samples"]),
            "setup_processes": sum(1 for s in document["samples"] if s.get("setup")),
            "started_utc": document["started_utc"],
            "duration_seconds": document["duration_seconds"],
        },
        "invalid_samples": document["invalid_samples"],
        "semantic_mismatches": len(document.get("semantic_mismatches", [])),
        "answer_check": {
            "builds": answer_check["builds"],
            "problems": answer_check["problems"],
        },
        "milestones": milestones,
        "headline": headline,
    }


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rebuild_cell(
    output_dir: Path,
    name: str,
    *,
    builds: Sequence[Build],
    entry: Mapping[str, Any],
    manifest: Mapping[str, Any],
    reference: str,
    metadata_source: Optional[Mapping[str, Any]],
    summary_revision: Optional[str],
    manifest_entry: Optional[Mapping[str, Any]] = None,
    override_note: Optional[str] = None,
) -> Path:
    """Summarize a stored run again, so a cell gains new fields without new timing.

    The run artifact, the answer check, and the earlier cell (for the storage check and
    the timing harness's revision) are read from ``output_dir``. Every build must map to
    exactly the command the run timed, or the summary would describe another cell.
    """
    document = json.loads((output_dir / f"run-{name}.json").read_text(encoding="utf-8"))
    answer_check = json.loads(
        (output_dir / f"answer-check-{name}.json").read_text(encoding="utf-8")
    )
    earlier = json.loads((output_dir / f"{name}.json").read_text(encoding="utf-8"))
    for build in builds:
        recorded = (document["tools"].get(build.label) or {}).get("command")
        mapped = list(build.shape.argv) if isinstance(build.shape, Shape) else None
        if recorded != mapped:
            raise HistoryError(
                f"build {build.label} maps to {mapped}, but the run timed {recorded}"
            )
    locations = (earlier.get("subject") or {}).get("storage_check") or {}
    cell = build_cell(
        document,
        builds=builds,
        entry=entry,
        manifest=manifest,
        reference=reference,
        metadata_source=metadata_source,
        answer_check=answer_check,
        storage={"locations": locations, "allowed_external": "external" in locations.values()},
        harness_revision=(earlier.get("harness") or {}).get("harness_revision"),
        run_artifact=f"{name}.run.json.gz",
        manifest_entry=manifest_entry,
        override_note=override_note,
    )
    cell["harness"]["summary_revision"] = summary_revision
    cell_path = output_dir / f"{name}.json"
    write_text_atomic(cell_path, json.dumps(cell, indent=2) + "\n")
    return cell_path


# --------------------------------------------------------------------------------------
# Command line


def _harness_revision() -> Optional[str]:
    try:
        completed = subprocess.run(
            ["git", "-C", str(Path(__file__).resolve().parent), "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            timeout=10,
            check=True,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    return completed.stdout.strip() or None


def plan_lines(builds: Sequence[Build], reference: str, trials: int, warmups: int) -> List[str]:
    lines = []
    for build in builds:
        assert build.caps is not None
        shape = build.shape
        if isinstance(shape, Shape):
            setup = " (+ untimed setup)" if shape.setup_argv else ""
            lines.append(
                f"{build.label:<10} {shape.variant:<10} {shape.cache_scope:<7} "
                f"{' '.join(shape.argv)}{setup}"
            )
        else:
            assert isinstance(shape, Unsupported)
            lines.append(f"{build.label:<10} unsupported: {shape.reason}")
    competitors = [b for b in builds if b.supported and b.label != reference]
    processes = len(competitors) * 2 * (trials + warmups)
    setups = sum(
        2 * (trials + warmups)
        for b in competitors
        if isinstance(b.shape, Shape) and b.shape.setup_argv
    )
    lines.append(f"timed processes: {processes}; untimed setup processes: {setups}")
    return lines


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(prog="benchmarks.realtree.history", description=__doc__)
    parser.add_argument("--component", required=True)
    parser.add_argument("--manifest", type=Path, default=MANIFEST)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--label", required=True)
    parser.add_argument("--build", action="append", required=True, metavar="LABEL=PATH")
    parser.add_argument("--probe", action="append", default=[], metavar="LABEL=PATH")
    parser.add_argument("--reference", help="anchor build label; defaults to the manifest's")
    parser.add_argument(
        "--milestones",
        type=Path,
        help="an earlier history cell whose per-build descriptions this one copies",
    )
    parser.add_argument("--trials", type=int, default=measure.DEFAULT_TRIALS)
    parser.add_argument("--warmups", type=int, default=3)
    parser.add_argument(
        "--stage", choices=("exploratory", "discovery", "held-out"), default="exploratory"
    )
    parser.add_argument(
        "--host-regime", choices=sorted(measure.HOST_REGIMES), default="uncontrolled"
    )
    parser.add_argument("--baseline-fingerprint", type=Path)
    parser.add_argument("--baseline-output", type=Path)
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_RESULTS)
    parser.add_argument("--name", required=True, help="cell name: the files it writes")
    parser.add_argument("--storage", default="", help="non-sensitive storage description")
    parser.add_argument("--harness-revision", help="commit of this harness, if not a checkout")
    parser.add_argument("--timeout", type=float, default=measure.DEFAULT_TIMEOUT_SECONDS)
    parser.add_argument(
        "--allow-external-storage",
        action="store_true",
        help="time even when a timed input is not on internal storage (recorded)",
    )
    parser.add_argument(
        "--plan", action="store_true", help="print each build's command shape and stop"
    )
    parser.add_argument("--check-only", action="store_true", help="run the answer check and stop")
    parser.add_argument("--job", help="the harness job to time, for a probe component of several")
    parser.add_argument(
        "--timing",
        choices=sorted(TIMINGS),
        help="the timed metric: process wall time, or a probe's component_ns; "
        "defaults to the manifest's measures",
    )
    parser.add_argument("--command", help="override the manifest command for this cell")
    parser.add_argument(
        "--cache-state", choices=sorted(CACHE_STATES), help="override the manifest cache state"
    )
    parser.add_argument("--override-note", help="why this cell departs from the manifest")
    parser.add_argument(
        "--from-run",
        action="store_true",
        help="re-summarize the cell from this name's stored run, answer check, and earlier "
        "cell in --output-dir, without timing anything",
    )
    arguments = parser.parse_args(list(argv))

    try:
        manifest = load_manifest(arguments.manifest)
        defined = component(manifest, arguments.component)
        entry = dict(defined)
        if arguments.command:
            entry["command"] = arguments.command
        if arguments.cache_state:
            entry["cache_state"] = arguments.cache_state
        if entry != defined and not arguments.override_note:
            raise HistoryError("a cell that overrides the manifest needs --override-note")
        timing = arguments.timing or (
            "component" if defined.get("measures") in ("component", "component_ns") else "wall"
        )
        reference = arguments.reference or manifest["reference_build"]
        builds = parse_builds(arguments.build, arguments.probe)
        map_builds(builds, entry, job=arguments.job)
        labels = [build.label for build in builds]
        if reference not in labels:
            raise HistoryError(f"the reference build {reference} is not among the builds")
        anchor_build = builds[labels.index(reference)]
        if not anchor_build.supported:
            raise HistoryError(f"the reference build {reference} cannot run {entry['id']}")
        for line in plan_lines(builds, reference, arguments.trials, arguments.warmups):
            print(line, file=sys.stderr)
        if arguments.plan:
            return 0
        if arguments.from_run:
            cell_path = rebuild_cell(
                arguments.output_dir,
                arguments.name,
                builds=builds,
                entry=entry,
                manifest=manifest,
                reference=reference,
                metadata_source=json.loads(arguments.milestones.read_text(encoding="utf-8"))
                if arguments.milestones
                else None,
                summary_revision=arguments.harness_revision or _harness_revision(),
                manifest_entry=defined,
                override_note=arguments.override_note,
            )
            print(f"wrote {cell_path}", file=sys.stderr)
            return 0
        if len([b for b in builds if b.supported]) < 2:
            raise HistoryError("a cell needs the reference and at least one other build")
        compare_tools._require_external_output(arguments.root, arguments.output_dir)

        locations = storage_check(arguments.root, builds)
        external = sorted(role for role, where in locations.items() if where == "external")
        if external and not arguments.allow_external_storage:
            raise HistoryError(
                "timed inputs must be on internal storage; external: " + ", ".join(external)
            )
        storage = {"locations": locations, "allowed_external": bool(external)}

        fingerprint = tree.fingerprint(arguments.root, label=arguments.label)
        if arguments.baseline_fingerprint is not None:
            baseline = json.loads(arguments.baseline_fingerprint.read_text(encoding="utf-8"))
            drift = tree.compare(fingerprint, baseline)
            if drift:
                raise HistoryError("the tree differs from its baseline: " + "; ".join(drift))
        records = [
            check_build(build, arguments.root, fingerprint) for build in builds if build.supported
        ]
        groups = answer_groups(records, fingerprint)
        problems = answer_problems(groups)
        answer_check = {"builds": records, "groups": groups, "problems": problems}
        arguments.output_dir.mkdir(parents=True, exist_ok=True)
        check_path = arguments.output_dir / f"answer-check-{arguments.name}.json"
        write_text_atomic(check_path, json.dumps(answer_check, indent=2, sort_keys=True))
        for record in records:
            print(
                f"check {record['label']:<10} exit={record['exit_codes']} "
                f"evidence={record['cache_evidence']} totals={record.get('totals')}",
                file=sys.stderr,
            )
        if problems:
            for problem in problems:
                print(f"answer check failed: {problem}", file=sys.stderr)
            return 4
        if arguments.check_only:
            return 0

        anchor = compare_tools.Tool(
            reference,
            contract_for(entry["id"], entry, anchor_build.shape, timing=timing),  # type: ignore[arg-type]
            anchor_build.timed_binary,
        )
        competitors = [
            compare_tools.Tool(
                build.label,
                contract_for(entry["id"], entry, build.shape, timing=timing),  # type: ignore[arg-type]
                build.timed_binary,
            )
            for build in builds
            if build.supported and build.label != reference
        ]
        document = compare_tools.run(
            root=arguments.root,
            label=arguments.label,
            anchor=anchor,
            competitors=competitors,
            trials=arguments.trials,
            warmups=arguments.warmups,
            baseline_fingerprint=fingerprint,
            baseline_output=arguments.baseline_output,
            storage=arguments.storage,
            campaign_stage=arguments.stage,
            host_regime=arguments.host_regime,
            timeout_seconds=arguments.timeout,
        )
    except (
        HistoryError,
        compare_tools.ComparisonError,
        measure.MeasureError,
        OSError,
        tree.ReferenceTreeError,
    ) as error:
        print(f"history cell refused: {error}", file=sys.stderr)
        return 1

    document = dict(document)
    document["index_component"] = {
        "component": entry["id"],
        "manifest_version": manifest["version"],
        "command": entry["command"],
        "cache_state": entry["cache_state"],
        "override_note": arguments.override_note,
        "timed_metric": TIMINGS[timing],
        "unsupported": {
            build.label: build.shape.reason
            for build in builds
            if isinstance(build.shape, Unsupported)
        },
    }
    run_path = arguments.output_dir / f"run-{arguments.name}.json"
    write_text_atomic(run_path, json.dumps(document, indent=2, sort_keys=True))
    write_text_atomic(
        arguments.output_dir / f"run-{arguments.name}.md", compare_tools.render(document)
    )
    stored = arguments.output_dir / f"{arguments.name}.run.json.gz"
    ledger.store(run_path, stored)
    metadata_source = (
        json.loads(arguments.milestones.read_text(encoding="utf-8"))
        if arguments.milestones
        else None
    )
    cell = build_cell(
        document,
        builds=builds,
        entry=entry,
        manifest=manifest,
        reference=reference,
        metadata_source=metadata_source,
        answer_check=answer_check,
        storage=storage,
        harness_revision=arguments.harness_revision or _harness_revision(),
        run_artifact=stored.name,
        manifest_entry=defined,
        override_note=arguments.override_note,
    )
    cell_path = arguments.output_dir / f"{arguments.name}.json"
    write_text_atomic(cell_path, json.dumps(cell, indent=2) + "\n")
    print(f"\nwrote {run_path}\nwrote {stored}\nwrote {cell_path}", file=sys.stderr)
    print(compare_tools.render(document))
    if document["tree_mutated_during_run"] or document["baseline_drift"]:
        return 2
    if document["invalid_samples"]:
        return 3
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
