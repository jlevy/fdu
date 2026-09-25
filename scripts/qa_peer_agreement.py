#!/usr/bin/env python3
"""Compare fdu's totals on real trees with other disk-usage tools, and explain each gap.

Manual release QA, not part of `make check`: real trees change while they are measured,
and peer tools are not pinned. See tests/qa/cli-installed-e2e.qa.md, the peer-agreement
phase, for how to read the output.

    python3 scripts/qa_peer_agreement.py --self-test
    python3 scripts/qa_peer_agreement.py . ~/.rustup /Applications ~/Library --json out.json

Each tool counts some things differently from fdu, and each difference is computed from
the tree itself rather than allowed for with a tolerance: fdu counts a hard-linked file
once per path and only regular files, while other tools count a hard-linked file once, a
symbolic link's own size, or each directory's own size. `MODELS` records which tool does
which, and `--self-test` checks every model exactly on a tree built with each case.

GNU du with `--count-links` is the reference: it counts as fdu does apart from links'
and directories' own sizes, so on a quiet tree its allocated total equals fdu's to the
byte on APFS. It is required.

A real tree may change while it is measured, so fdu runs just before each tool and once
at the end, and each tool is judged against the two fdu readings around it. On a quiet
tree, one whose every fdu reading was the same, agreement must be exact; a single fdu
reading that disagrees with all the others fails on its own row rather than loosening
the judgement of every tool. On a live tree, a tool may fall within its two fdu
readings, or outside them by no more than fdu moved during or next to them.

A tool may also fall short by what it skipped: every directory a tool reports it could
not read, and that this script's own walk could list, is measured afterwards with fdu,
and the tool may be short by exactly that. Anything else is UNEXPLAINED, and the script
exits non-zero.

The JSON written by `--json` holds absolute paths, and directory names can identify
people; keep the JSON out of the repository, and review any table before publishing it.
Stdlib only, like scripts/run_installed_cli_qa.py, so it runs against a PATH binary.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import shutil
import stat
import subprocess
import sys
import tempfile
import time
from collections import Counter
from collections.abc import Callable
from dataclasses import asdict, dataclass, field
from functools import partial
from itertools import pairwise
from pathlib import Path

METRICS = ("allocated", "apparent")


@dataclass(frozen=True)
class Model:
    """How a tool counts, relative to fdu, which counts regular files once per path."""

    per_path: bool  # a hard-linked file once per path, as fdu does; else once per inode
    # Also counts each symbolic link's own size: "none", "all", or "below_root", which
    # leaves out links directly inside the root (dua takes the root's entries as inputs).
    symlinks: str
    dirs: str  # also counts each directory's own size: "none", "all", or "all_but_root"


# Checked by `--self-test` on APFS. There directories and symbolic links occupy no
# blocks, so the allocated rows cannot tell whether a tool counts them; only the
# apparent rows test those terms.
MODELS: dict[tuple[str, str], Model] = {
    ("GNU du -l", "allocated"): Model(per_path=True, symlinks="all", dirs="all"),
    ("GNU du -l", "apparent"): Model(per_path=True, symlinks="all", dirs="none"),
    ("GNU du", "allocated"): Model(per_path=False, symlinks="all", dirs="all"),
    ("GNU du", "apparent"): Model(per_path=False, symlinks="all", dirs="none"),
    ("dust", "allocated"): Model(per_path=False, symlinks="all", dirs="all"),
    ("dust", "apparent"): Model(per_path=True, symlinks="all", dirs="all"),
    ("pdu", "allocated"): Model(per_path=True, symlinks="all", dirs="all"),
    ("pdu", "apparent"): Model(per_path=True, symlinks="all", dirs="all"),
    ("dua", "allocated"): Model(per_path=False, symlinks="below_root", dirs="all_but_root"),
    ("dua", "apparent"): Model(per_path=False, symlinks="below_root", dirs="all_but_root"),
    ("diskus", "allocated"): Model(per_path=False, symlinks="all", dirs="all"),
    ("diskus", "apparent"): Model(per_path=False, symlinks="all", dirs="none"),
    ("BSD du", "allocated"): Model(per_path=False, symlinks="all", dirs="all"),
}


@dataclass
class Reading:
    tool: str
    metric: str
    total: int | None
    seconds: float
    command: str
    children: dict[str, int] = field(default_factory=dict)
    errors: dict[str, int] = field(default_factory=dict)
    failed: list[str] = field(default_factory=list)  # paths it reported it could not read
    gave_up: list[str] = field(default_factory=list)  # of those, what the walk could list
    skipped: int = 0  # what the tool would have counted there, measured afterwards
    unmeasured: bool = False  # a directory it gave up on could not be measured
    exit: int | None = None


def run(command: list[str], timeout: int) -> tuple[str, str, int, float]:
    started = time.monotonic()
    result = subprocess.run(command, capture_output=True, text=True, timeout=timeout, check=False)
    return result.stdout, result.stderr, result.returncode, time.monotonic() - started


ANSI = re.compile(r"\x1b\[[0-9;]*m")
# Each tool's failure message, anchored, with the path in its own quoting.
GNU_FAILURE = re.compile(r"^\S*du: cannot (?:read directory|access) (.+): ([^:]+)$")
BSD_FAILURE = re.compile(r"^du: (.+): ([^:]+)$")
PDU_FAILURE = re.compile(r'\[error\] \w+ (".*"): (.*)$')
DISKUS_FAILURE = re.compile(r"could not read contents of directory '(.*)'$")


def reason(text: str) -> str:
    lowered = text.lower()
    if "interrupted" in lowered or "os error 4)" in lowered:
        return "interrupted"
    if "not permitted" in lowered or "permission denied" in lowered:
        return "denied"
    return "unreadable, reason not given"


def tool_errors(tool: str, stdout: str, stderr: str) -> tuple[dict[str, int], list[str]]:
    """The errors a peer tool reported, by kind, and the paths it said it could not read."""
    counts: Counter[str] = Counter()
    paths: list[str] = []
    for line in stderr.splitlines():
        if tool.startswith("GNU du") and (match := GNU_FAILURE.match(line)):
            try:
                paths.append(shlex.split(match[1])[0])
            except ValueError:
                paths.append(match[1])
            counts[reason(match[2])] += 1
        elif tool == "BSD du" and (match := BSD_FAILURE.match(line)):
            paths.append(match[1])
            counts[reason(match[2])] += 1
        elif tool == "pdu" and (match := PDU_FAILURE.search(line)):
            try:
                paths.append(json.loads(match[1]))
            except json.JSONDecodeError:
                paths.append(match[1].strip('"'))
            counts[reason(match[2])] += 1
        elif tool == "diskus" and (match := DISKUS_FAILURE.search(line)):
            paths.append(match[1])
            counts[reason("")] += 1
        elif tool == "dust" and "did not have permissions for directories:" in line.lower():
            counts["denied"] += line.split(": ", 1)[1].count(", ") + 1
        elif line.strip():
            counts[reason(line)] += 1
    io_errors = re.findall(r"<(\d+) IO Errors?>", ANSI.sub("", stdout))
    if io_errors:  # dua reports only a count, on stdout, with its total the largest
        counts["unreadable, reason not given"] += max(int(n) for n in io_errors)
    return dict(counts), paths


def gnu_du() -> str | None:
    """GNU du, which macOS installs from coreutils as `gdu`."""
    for name in ("gdu", "du"):
        path = shutil.which(name)
        if path is None:
            continue
        version = subprocess.run([path, "--version"], capture_output=True, text=True)
        if "GNU coreutils" in version.stdout:
            return path
    return None


def fdu_reading(fdu: str, root: Path, timeout: int) -> list[Reading]:
    """fdu's allocated and apparent readings of `root`; totals are None if it failed."""
    command = [
        *(fdu, "--cache", "off", "--progress", "never", "--view", "tree"),
        *("--depth", "1", "--limit", "all", "--format", "json", str(root)),
    ]
    out, _, code, seconds = run(command, timeout)
    try:
        document = json.loads(out)
    except json.JSONDecodeError:
        return [Reading("fdu", m, None, seconds, " ".join(command), exit=code) for m in METRICS]
    report = document["reports"]
    tree = (report[0] if isinstance(report, list) else report)["tree"]
    errors: Counter[str] = Counter()
    failed = []
    status = document.get("status", {})
    for error in status.get("errors", []):
        failed.append(str(root / error.get("path", "")))
        if error.get("os_error") == 4:
            errors["interrupted"] += 1
        elif error.get("kind") == "permission":
            errors["denied"] += 1
        else:
            errors["other"] += 1
    if status.get("errors_omitted"):
        # fdu details its first errors and counts the rest.
        errors["more, not detailed"] = status["errors_omitted"]
    return [
        Reading(
            "fdu",
            metric,
            tree[key],
            seconds,
            " ".join(command),
            {child["name"]: child[key] for child in tree["children"]},
            dict(errors),
            failed,
            exit=code,
        )
        for metric, key in (("allocated", "allocated"), ("apparent", "bytes"))
    ]


def largest(out: str, pattern: str) -> int | None:
    """The largest number the pattern captures: a root's total is never below a child's,
    whatever order or truncation a tool prints them in."""
    values = [
        int(next(group for group in m.groups() if group))
        for m in re.finditer(pattern, ANSI.sub("", out), re.M)
    ]
    return max(values) if values else None


def gnu_reading(du: str, root: Path, timeout: int, metric: str, links: bool) -> Reading:
    size = ["-b"] if metric == "apparent" else ["-B1"]
    command = [du, *size, "--max-depth=1", *(["-l"] if links else []), str(root)]
    out, err, _, seconds = run(command, timeout)
    children = {}
    total = None
    for line in out.splitlines():
        value, _, path = line.partition("\t")
        if Path(path) == root:
            total = int(value)
        elif path:
            children[Path(path).name] = int(value)
    label = "GNU du -l" if links else "GNU du"
    errors, failed = tool_errors(label, out, err)
    return Reading(label, metric, total, seconds, " ".join(command), children, errors, failed)


def peer_reading(
    tool: str, metric: str, command: list[str], pattern: str, timeout: int, scale: int = 1
) -> Reading:
    out, err, _, seconds = run(command, timeout)
    value = largest(out, pattern)
    errors, failed = tool_errors(tool, out, err)
    total = value * scale if value is not None else None
    return Reading(tool, metric, total, seconds, " ".join(command), {}, errors, failed)


def peers(root: Path, timeout: int, du: str) -> list[Callable[[], Reading]]:
    """One deferred run per tool and metric, so fdu can be run between them."""
    runs: list[Callable[[], Reading]] = []
    for metric in METRICS:
        for links in (True, False):
            runs.append(partial(gnu_reading, du, root, timeout, metric, links))
    path = str(root)
    if shutil.which("dust"):
        for metric, flags in (("allocated", []), ("apparent", ["-s"])):
            command = [
                "dust",
                "-d",
                "1",
                "-o",
                "b",
                "-c",
                "-b",
                "-P",
                "--print-errors",
                *flags,
                path,
            ]
            runs.append(partial(peer_reading, "dust", metric, command, r"^\s*(\d+)B\s", timeout))
    if shutil.which("pdu"):
        for metric, quantity in (("allocated", "block-size"), ("apparent", "apparent-size")):
            command = ["pdu", "-b", "plain", "--max-depth", "1", "-q", quantity, path]
            runs.append(partial(peer_reading, "pdu", metric, command, r"^\s*(\d+)\s", timeout))
    if shutil.which("dua"):
        for metric, flags in (("allocated", []), ("apparent", ["-A"])):
            command = ["dua", "-f", "bytes", *flags, "aggregate", path]
            runs.append(partial(peer_reading, "dua", metric, command, r"^\s*(\d+) b\b", timeout))
    if shutil.which("diskus"):
        for metric, flags in (("allocated", []), ("apparent", ["-b"])):
            command = ["diskus", "-v", *flags, path]
            pattern = r"^\s*(\d+)\s*$|\((\d+) bytes\)"
            runs.append(partial(peer_reading, "diskus", metric, command, pattern, timeout))
    if sys.platform == "darwin":
        command = ["/usr/bin/du", "-sk", path]
        pattern = r"^(\d+)\s"
        runs.append(
            partial(peer_reading, "BSD du", "allocated", command, pattern, timeout, scale=1024)
        )
    return runs


def listing(directory: str) -> tuple[list[os.DirEntry[str]] | None, str | None]:
    """A directory's entries, retrying an interrupted read, or why it could not be read."""
    for _ in range(5):
        try:
            with os.scandir(directory) as entries:
                return list(entries), None
        except InterruptedError:
            continue
        except PermissionError:
            return None, "denied"
        except OSError:
            return None, "other"
    return None, "interrupted"


def tree_facts(root: Path) -> dict[str, object]:
    """What the tools count differently from fdu, measured in one walk of `root`, and the
    directories no tool can list."""
    links: Counter[str] = Counter()
    dirs: Counter[str] = Counter()
    unlisted: Counter[str] = Counter()
    unlisted_paths: list[str] = []
    unstatted_paths: list[str] = []
    shared: dict[tuple[int, int], list[int]] = {}
    root_info = root.lstat()
    dirs.update(count=1, apparent=root_info.st_size, allocated=root_info.st_blocks * 512)
    stack = [(str(root), True)]
    while stack:
        directory, at_root = stack.pop()
        entries, failure = listing(directory)
        if entries is None:
            unlisted[failure or "other"] += 1
            unlisted_paths.append(os.path.normpath(directory))
            continue
        for entry in entries:
            try:
                info = entry.stat(follow_symlinks=False)
            except OSError:
                unstatted_paths.append(os.path.normpath(entry.path))
                continue
            if stat.S_ISLNK(info.st_mode):
                links.update(count=1, apparent=info.st_size, allocated=info.st_blocks * 512)
                if at_root:
                    links.update(root_apparent=info.st_size, root_allocated=info.st_blocks * 512)
            elif stat.S_ISDIR(info.st_mode):
                dirs.update(count=1, apparent=info.st_size, allocated=info.st_blocks * 512)
                stack.append((entry.path, False))
            elif stat.S_ISREG(info.st_mode) and info.st_nlink > 1:
                size = [info.st_size, info.st_blocks * 512, 0]
                shared.setdefault((info.st_dev, info.st_ino), size)[2] += 1
    duplicated: Counter[str] = Counter()
    for apparent, allocated, paths in shared.values():
        if paths > 1:
            duplicated.update(
                inodes=1, apparent=apparent * (paths - 1), allocated=allocated * (paths - 1)
            )
    return {
        "hard_links": dict(duplicated),
        "symlinks": dict(links),
        "dirs": dict(dirs)
        | {"root_apparent": root_info.st_size, "root_allocated": root_info.st_blocks * 512},
        "unlisted": dict(unlisted),
        "unlisted_paths": unlisted_paths,
        "unstatted": {"entries": len(unstatted_paths)},
        "unstatted_paths": unstatted_paths,
    }


def measure(root: Path, fdu: str, timeout: int, du: str) -> dict[str, object]:
    readings = fdu_reading(fdu, root, timeout)
    for run_tool in peers(root, timeout, du):
        readings.append(run_tool())
        readings += fdu_reading(fdu, root, timeout)
    facts = tree_facts(root)
    # A path a tool reported it could not read, but that the walk could read, is one it
    # gave up on: measure what the tool would have counted there, so a shortfall can be
    # checked against it. A path the walk could not read either is one no tool can.
    unlisted = set(facts["unlisted_paths"]) | set(facts["unstatted_paths"])  # type: ignore[arg-type]
    cache: dict[str, tuple[dict[str, int], dict[str, object]] | None] = {}
    for reading in readings:
        if reading.tool == "fdu":
            continue
        reading.gave_up = sorted(
            {os.path.normpath(p) for p in reading.failed} - unlisted - {os.path.normpath(root)}
        )
        model = MODELS[(reading.tool, reading.metric)]
        for path in reading.gave_up:
            if path not in cache:
                cache[path] = subtree(fdu, Path(path), timeout)
            measured = cache[path]
            if measured is None:
                reading.unmeasured = True
                continue
            totals, sub_facts = measured
            extras = expected_delta(model, reading.metric, sub_facts)  # type: ignore[arg-type]
            reading.skipped += totals[reading.metric] + sum(amount for _, amount in extras)
        if os.path.normpath(root) in {os.path.normpath(p) for p in reading.failed}:
            reading.unmeasured = True
    # What every folder some tool gave up on holds: the most a tool that skipped folders
    # without naming them can have lost, if it failed where the others did.
    pool = {
        metric: sum(measured[0][metric] for measured in cache.values() if measured)
        for metric in METRICS
    }
    return {
        "root": str(root),
        "facts": facts,
        "gave_up_pool": pool,
        "readings": [asdict(r) for r in readings],
    }


def subtree(fdu: str, path: Path, timeout: int) -> tuple[dict[str, int], dict[str, object]] | None:
    """fdu's totals and the walk's facts for one directory, or None if it cannot be
    measured: gone, not a directory, or unreadable now."""
    try:
        if not path.is_dir():
            return None
        readings = fdu_reading(fdu, path, timeout)
        if any(r.total is None for r in readings):
            return None
        return {r.metric: r.total or 0 for r in readings}, tree_facts(path)
    except OSError:
        return None


def expected_delta(
    model: Model, metric: str, facts: dict[str, dict[str, int]]
) -> list[tuple[str, int]]:
    """The parts of a tool's expected difference from fdu, each measured from the tree."""
    parts = []
    if not model.per_path and facts["hard_links"].get(metric):
        parts.append(("hard links once", -facts["hard_links"][metric]))
    links = facts["symlinks"].get(metric, 0)
    if model.symlinks == "below_root":
        links -= facts["symlinks"].get(f"root_{metric}", 0)
    if model.symlinks != "none" and links:
        parts.append(("symbolic links", links))
    directories = facts["dirs"].get(metric, 0)
    if model.dirs == "all_but_root":
        directories -= facts["dirs"][f"root_{metric}"]
    if model.dirs != "none" and directories:
        parts.append(("directories", directories))
    return parts


def saved(fields: dict[str, object]) -> Reading:
    """A reading from saved JSON, including one saved before `failed` replaced `interrupted`."""
    known = {k: v for k, v in fields.items() if k in Reading.__dataclass_fields__}
    known.setdefault("failed", fields.get("interrupted", []))
    return Reading(**known)  # type: ignore[arg-type]


def fdu_series(readings: list[Reading], metric: str) -> tuple[list[int], int | None]:
    """fdu's readings of `metric` in order, with one outlier replaced, and its position.

    When every reading but one agrees, the odd one is fdu disagreeing with itself, not the
    tree moving: it fails on its own row, and the tools are judged against the others."""
    values = [r.total or 0 for r in readings if r.tool == "fdu" and r.metric == metric]
    common, count = Counter(values).most_common(1)[0]
    if len(values) >= 3 and count == len(values) - 1:
        odd = next(i for i, v in enumerate(values) if v != common)
        return [common] * len(values), odd
    return values, None


def judge(record: dict[str, object], full_paths: bool = False) -> tuple[str, int]:
    """The record as a table, and how many readings failed."""
    readings = [saved(r) for r in record["readings"]]  # type: ignore[union-attr]
    facts: dict[str, dict[str, int]] = record["facts"]  # type: ignore[assignment]
    fdu_readings = [r for r in readings if r.tool == "fdu"]
    failures = 0
    notes: list[str] = []
    series: dict[str, list[int]] = {}
    for metric in METRICS:
        series[metric], odd = fdu_series(readings, metric)
        if odd is not None:
            failures += 1
            value = [r for r in fdu_readings if r.metric == metric][odd].total or 0
            notes.append(
                f"- UNEXPLAINED: fdu's {metric} reading {odd + 1} of {len(series[metric])} was "
                f"{human(value)}, and every other reading was {human(series[metric][0])}."
            )
    if any(r.total is None for r in fdu_readings):
        failures += 1
        notes.append("- UNEXPLAINED: an fdu reading failed.")
    error_counts = {sum(r.errors.values()) for r in fdu_readings}
    if len(error_counts) > 1:
        failures += 1
        notes.append(f"- UNEXPLAINED: fdu's error count varied: {sorted(error_counts)}.")
    quiet = {m: len(set(v)) == 1 for m, v in series.items()}
    steps = {m: [abs(b - a) for a, b in pairwise(v)] for m, v in series.items()}
    first = fdu_readings[0]
    moved = {m: max(v) - min(v) for m, v in series.items()}
    lines = [
        f"### `{record['root']}`",
        "",
        f"fdu read {human(series['allocated'][0])} allocated and "
        f"{human(series['apparent'][0])} apparent, exiting {first.exit}; across its "
        f"{len(series['allocated'])} readings, one before each tool and one at the end, the "
        f"tree moved {human(moved['allocated'])} allocated and {human(moved['apparent'])} "
        f"apparent. fdu errors: {describe(first.errors)}.",
        "",
        f"Measured from the tree: {facts['hard_links'].get('inodes', 0):,} hard-linked "
        f"inodes, which per-path counting adds {human(facts['hard_links'].get('allocated', 0))}"
        f" allocated to; {facts['symlinks'].get('count', 0):,} symbolic links, "
        f"{human(facts['symlinks'].get('apparent', 0))} apparent; "
        f"{facts['dirs'].get('count', 0):,} directories, "
        f"{human(facts['dirs'].get('apparent', 0))} apparent; directories that could not be "
        f"listed: {describe(facts['unlisted'])}; entries that could not be stat'd: "
        f"{facts['unstatted']['entries']:,}.",
        "",
        "| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |",
        "| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |",
    ]
    tool_index = -1
    for reading in readings:
        if reading.tool == "fdu":
            continue
        tool_index += 1
        model = MODELS[(reading.tool, reading.metric)]
        parts = expected_delta(model, reading.metric, facts)
        expected = sum(amount for _, amount in parts)
        made_of = ", ".join(f"{name} {signed(amount)}" for name, amount in parts) or "—"
        values, own = series[reading.metric], steps[reading.metric]
        before, after = values[tool_index], values[tool_index + 1]
        # The most fdu moved during this tool's bracket or the ones either side of it.
        churn = max(
            (own[j] for j in (tool_index - 1, tool_index, tool_index + 1) if 0 <= j < len(own)),
            default=0,
        )
        low, high = sorted((before + expected, after + expected))
        # A tool that reports more failures than there are unreadable paths, without
        # naming them all, skipped folders it did not say.
        unreadable = set(facts.get("unlisted_paths", [])) | set(facts.get("unstatted_paths", []))
        named = {os.path.normpath(p) for p in reading.failed}
        unnamed = sum(reading.errors.values()) - len(named) - len(unreadable - named)
        pool = record.get("gave_up_pool", {}).get(reading.metric, 0)  # type: ignore[union-attr]
        verdict = verdict_for(
            reading, low, high, 0 if quiet[reading.metric] else churn, max(unnamed, 0), pool
        )
        failures += verdict in ("UNEXPLAINED", "no reading")
        total = human(reading.total) if reading.total is not None else "—"
        delta = signed(reading.total - before) if reading.total is not None else "—"
        lines.append(
            f"| {reading.tool} | {reading.metric} | {total} | {delta} | {signed(expected)} | "
            f"{made_of} | {verdict} | {describe(reading.errors)} | {reading.seconds:.1f} s |"
        )
    if notes:
        lines += ["", *notes]
    gave_up = [r for r in readings if r.gave_up]
    if gave_up:
        lines += [
            "",
            "Directories a tool reported it could not read and the walk could list:",
            "",
        ]
        for reading in gave_up:
            lines.append(
                f"- {reading.tool} ({reading.metric}), holding {human(reading.skipped)}"
                f"{', not all measurable' if reading.unmeasured else ''}: "
                f"{places(reading.gave_up, str(record['root']), full_paths)}"
            )
    failures += top_level(readings, series["allocated"], quiet["allocated"], steps, lines)
    return "\n".join(lines), failures


def verdict_for(
    reading: Reading, low: int, high: int, slack: int, unnamed: int = 0, pool: int = 0
) -> str:
    total = reading.total
    if total is None:
        return "no reading"
    within = "exactly" if slack == 0 and low == high else "within the tree's movement"
    if low <= total <= high:
        return f"agrees {within}"
    gone = 0 if reading.unmeasured else reading.skipped
    short = f"short by what it skipped: {len(reading.gave_up):,} directories holding {human(gone)}"
    if gone and low - gone <= total <= high - gone:
        return short
    if slack and low - slack <= total <= high + slack:
        return f"agrees within fdu's nearby movement ({human(slack)})"
    if gone and slack and low - gone - slack <= total <= high - gone + slack:
        return f"{short}, within fdu's nearby movement ({human(slack)})"
    if unnamed and pool and low - gone - pool - slack <= total < low:
        return (
            f"short after skipping {unnamed:,} folders it did not name, by no more than the "
            f"{human(pool)} the other tools gave up on"
        )
    return "UNEXPLAINED"


def top_level(
    readings: list[Reading],
    allocated: list[int],
    quiet: bool,
    steps: dict[str, list[int]],
    lines: list[str],
) -> int:
    """Top-level directories against the reference, between the fdu readings around it."""
    fdu_before: Reading | None = None
    tool_index = -1
    for index, reading in enumerate(readings):
        if reading.tool == "fdu":
            if reading.metric == "allocated":
                fdu_before = reading
            continue
        tool_index += 1
        if reading.tool == "GNU du -l" and reading.metric == "allocated":
            reference = reading
            fdu_after = next(
                r for r in readings[index + 1 :] if r.tool == "fdu" and r.metric == "allocated"
            )
            break
    else:
        return 0
    assert fdu_before is not None
    own = steps["allocated"]
    churn = (
        0
        if quiet
        else max(
            (own[j] for j in (tool_index - 1, tool_index, tool_index + 1) if 0 <= j < len(own)),
            default=0,
        )
    )
    rows = []
    names = set(fdu_before.children) | set(fdu_after.children) | set(reference.children)
    for name in sorted(names):
        before, after = fdu_before.children.get(name), fdu_after.children.get(name)
        theirs = reference.children.get(name)
        if before is None or after is None or theirs is None:
            rows.append(f"| `{name}` | {size(before)} to {size(after)} | {size(theirs)} | — |")
            continue
        low, high = sorted((before, after))
        if not low - churn <= theirs <= high + churn:
            rows.append(f"| `{name}` | {size(before)} to {size(after)} | {size(theirs)} | "
                        f"{signed(theirs - before)} |")  # fmt: skip
    bound = "exactly" if quiet else f"within fdu's readings around it, ± {human(churn)}"
    lines += [
        "",
        f"Top-level directories whose allocated size does not agree with GNU du -l {bound}: "
        f"{len(rows)} of {len(names):,}.",
    ]
    if rows:
        lines += ["", "| Directory | fdu | GNU du -l | Δ |", "| --- | ---: | ---: | ---: |", *rows]
    return len(rows)


def places(paths: list[str], root: str, full: bool) -> str:
    """Where a tool gave up, as the first two path components under the root unless asked
    for more: directory names can identify people."""
    relative = []
    for path in paths:
        try:
            relative.append(str(Path(path).relative_to(root)))
        except ValueError:
            relative.append(path)
    if full:
        return ", ".join(f"`{p}`" for p in relative)
    grouped = Counter("/".join(Path(p).parts[:2]) for p in relative)
    return ", ".join(f"`{place}` ({n})" for place, n in sorted(grouped.items()))


def size(n: int | None) -> str:
    return "absent" if n is None else human(n)


def describe(errors: dict[str, int]) -> str:
    return ", ".join(f"{n:,} {kind}" for kind, n in errors.items()) or "none"


def human(n: float) -> str:
    sign = "-" if n < 0 else ""
    n = abs(n)
    for unit in ("B", "KiB", "MiB", "GiB", "TiB", "PiB"):
        if n < 1024 or unit == "PiB":
            return f"{sign}{n:.0f} {unit}" if unit == "B" else f"{sign}{n:.1f} {unit}"
        n /= 1024
    return f"{sign}{n}"


def signed(n: int) -> str:
    return "0 B" if n == 0 else ("+" if n > 0 else "") + human(n)


def self_test(fdu: str, du: str, timeout: int, scratch: Path | None) -> int:
    """Build a tree with each case the models name, and require exact agreement."""
    base = Path(tempfile.mkdtemp(prefix="fdu-peer-agreement-", dir=scratch))
    probe = base / "probe root"
    try:
        (probe / "a").mkdir(parents=True)
        (probe / "b").mkdir()
        (probe / "a" / "file").write_bytes(os.urandom(10_000))
        os.link(probe / "a" / "file", probe / "a" / "same inode")
        os.link(probe / "a" / "file", probe / "b" / "same inode across directories")
        (probe / "link").symlink_to("a/file")
        (probe / "directory link").symlink_to("a")
        (probe / "long link").symlink_to("x" * 1000)
        (probe / "b" / "inner link").symlink_to("y" * 37)
        with open(probe / "sparse", "wb") as sparse:
            sparse.truncate(1 << 30)
        (probe / "name with spaces").mkdir()
        (probe / "name with spaces" / "small").write_bytes(b"small")
        (probe / "locked").mkdir()
        (probe / "locked" / "hidden").write_bytes(b"hidden")
        (probe / "locked").chmod(0)
        record = measure(probe, fdu, timeout, du)
        text, failures = judge(record)
        print(text + "\n")
        present = {(r["tool"], r["metric"]) for r in record["readings"]}  # type: ignore[index, union-attr]
        missing = sorted(set(MODELS) - present)
        exact = text.count("| agrees exactly |")
        print(f"Self-test: {exact} readings agree exactly; {failures} failed.")
        if missing:
            print("Not installed, so not checked: " + ", ".join(f"{t} {m}" for t, m in missing))
        return 1 if failures or missing else 0
    finally:
        (probe / "locked").chmod(0o755)
        shutil.rmtree(base)


def main() -> int:
    parser = argparse.ArgumentParser(description=(__doc__ or "").split("\n\n")[0])
    parser.add_argument("roots", nargs="*", type=Path)
    parser.add_argument("--fdu", default=os.environ.get("FDU", "fdu"))
    parser.add_argument("--timeout", type=int, default=1800, help="seconds per command")
    parser.add_argument("--json", type=Path, help="also write every reading here")
    parser.add_argument("--rejudge", type=Path, help="judge readings saved by --json again")
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="check the counting models on a tree built with each case",
    )
    parser.add_argument("--scratch", type=Path, help="where --self-test builds its tree")
    parser.add_argument(
        "--full-paths",
        action="store_true",
        help="name every directory a tool gave up on, not just where (names can identify people)",
    )
    args = parser.parse_args()

    if args.rejudge:
        failures = 0
        for record in json.loads(args.rejudge.read_text(encoding="utf-8")):
            text, failed = judge(record, args.full_paths)
            failures += failed
            print(text + "\n")
        return 1 if failures else 0
    du = gnu_du()
    if du is None:
        print("GNU du is required as the reference (`brew install coreutils`).", file=sys.stderr)
        return 2
    version = subprocess.run([args.fdu, "--version"], capture_output=True, text=True).stdout
    print(f"## Peer agreement ({version.strip()}, {time.strftime('%Y-%m-%d')})\n", flush=True)
    if args.self_test:
        return self_test(args.fdu, du, args.timeout, args.scratch)
    failures = 0
    data = []
    for root in args.roots:
        record = measure(root.expanduser().resolve(), args.fdu, args.timeout, du)
        data.append(record)
        if args.json:
            args.json.write_text(json.dumps(data, indent=1), encoding="utf-8")
        text, failed = judge(record, args.full_paths)
        failures += failed
        print(text + "\n", flush=True)
    print(f"{failures} readings failed." if failures else "Every reading is explained.")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
