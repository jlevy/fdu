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
which, and `--self-test` proves every model exactly on a tree built with each case.

GNU du with `--count-links` is the reference: it counts as fdu does apart from links'
and directories' own sizes, so on a quiet tree its allocated total equals fdu's to the
byte on APFS. It is required.

A real tree may change while it is measured, so fdu runs just before each tool and once
at the end, and each tool is judged against the two fdu readings around it. On a quiet
tree, one whose every fdu reading was the same, agreement must be exact. On a live one, a
tool may fall within its two fdu readings, or outside them by no more than the largest
change fdu saw between consecutive readings; and a tool that reports giving up on
directories after interrupted reads may fall short by no more than what those
directories hold, measured afterwards. Anything else is UNEXPLAINED, and the script exits
non-zero.

The JSON written by `--json` holds absolute paths; keep it out of the repository.
Stdlib only, like scripts/run_installed_cli_qa.py, so it runs against a PATH binary.
"""

from __future__ import annotations

import argparse
import json
import os
import re
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


# Verified by `--self-test` on APFS, where directories and symbolic links occupy no
# blocks, so their allocated terms are zero there and only the apparent terms are tested.
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
    interrupted: list[str] = field(default_factory=list)
    skipped: int = 0  # bytes in the interrupted directories, measured afterwards
    exit: int | None = None


def run(command: list[str], timeout: int) -> tuple[str, str, int, float]:
    started = time.monotonic()
    result = subprocess.run(command, capture_output=True, text=True, timeout=timeout, check=False)
    return result.stdout, result.stderr, result.returncode, time.monotonic() - started


ANSI = re.compile(r"\x1b\[[0-9;]*m")
QUOTED = re.compile(r"['\"]([^'\"]+)['\"]")


def tool_errors(stdout: str, stderr: str) -> tuple[dict[str, int], list[str]]:
    """The errors a peer tool reported, by kind, and the directories it gave up on after
    an interrupted read."""
    counts: Counter[str] = Counter()
    interrupted: list[str] = []
    for line in stderr.splitlines():
        lowered = line.lower()
        if lowered.strip().startswith("did not have permissions for directories:"):
            # dust --print-errors lists them on one line.
            counts["denied"] += line.split(": ", 1)[1].count(", ") + 1
            continue
        path = QUOTED.search(line)
        if "interrupted system call" in lowered or "(os error 4)" in lowered:
            counts["interrupted"] += 1
            if path:
                interrupted.append(path[1])
        elif "not permitted" in lowered or "permission denied" in lowered:
            counts["denied"] += 1
        elif "could not read" in lowered or "cannot" in lowered or "error" in lowered:
            counts["unreadable, reason not given"] += 1
    io_errors = re.search(r"<(\d+) IO Errors?>", ANSI.sub("", stdout))
    if io_errors:  # dua reports only a count, on stdout
        counts["unreadable, reason not given"] += int(io_errors[1])
    return dict(counts), interrupted


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
    command = [
        fdu,
        "--cache",
        "off",
        "--progress",
        "never",
        "--view",
        "tree",
        "--depth",
        "1",
        "--limit",
        "all",
        "--format",
        "json",
        str(root),
    ]
    out, _, code, seconds = run(command, timeout)
    document = json.loads(out)
    report = document["reports"]
    tree = (report[0] if isinstance(report, list) else report)["tree"]
    errors: Counter[str] = Counter()
    interrupted = []
    status = document.get("status", {})
    for error in status.get("errors", []):
        if error.get("os_error") == 4:
            errors["interrupted"] += 1
            interrupted.append(str(root / error.get("path", "")))
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
            interrupted,
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
    errors, interrupted = tool_errors(out, err)
    label = "GNU du -l" if links else "GNU du"
    return Reading(label, metric, total, seconds, " ".join(command), children, errors, interrupted)


def peer_reading(
    tool: str, metric: str, command: list[str], pattern: str, timeout: int, scale: int = 1
) -> Reading:
    out, err, _, seconds = run(command, timeout)
    value = largest(out, pattern)
    errors, interrupted = tool_errors(out, err)
    total = value * scale if value is not None else None
    return Reading(tool, metric, total, seconds, " ".join(command), {}, errors, interrupted)


def peers(root: Path, timeout: int, du: str) -> list[Callable[[], Reading]]:
    """One deferred run per tool and metric, so fdu can be run between them."""
    runs: list[Callable[[], Reading]] = []
    for metric in METRICS:
        for links in (True, False):
            runs.append(partial(gnu_reading, du, root, timeout, metric, links))
    path = str(root)
    if shutil.which("dust"):
        for metric, flags in (("allocated", []), ("apparent", ["-s"])):
            command = ["dust", "-d", "1", "-o", "b", "-c", "-b", "--print-errors", *flags, path]
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


def tree_facts(root: Path) -> dict[str, dict[str, int]]:
    """What the tools count differently from fdu, measured in one walk of `root`."""
    links: Counter[str] = Counter()
    dirs: Counter[str] = Counter()
    unlisted: Counter[str] = Counter()
    shared: dict[tuple[int, int], list[int]] = {}
    unstatted = 0
    root_info = root.lstat()
    dirs.update(count=1, apparent=root_info.st_size, allocated=root_info.st_blocks * 512)
    stack = [(str(root), True)]
    while stack:
        directory, at_root = stack.pop()
        entries, failure = listing(directory)
        if entries is None:
            unlisted[failure or "other"] += 1
            continue
        for entry in entries:
            try:
                info = entry.stat(follow_symlinks=False)
            except OSError:
                unstatted += 1
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
        "unstatted": {"entries": unstatted},
    }


def measure(root: Path, fdu: str, timeout: int, du: str) -> dict[str, object]:
    readings = fdu_reading(fdu, root, timeout)
    for run_tool in peers(root, timeout, du):
        readings.append(run_tool())
        readings += fdu_reading(fdu, root, timeout)
    # What each tool gave up on after interrupted reads, measured now so a shortfall can
    # be checked against it.
    sizes: dict[tuple[str, str], int] = {}
    for reading in readings:
        if reading.tool == "fdu":
            continue
        for path in reading.interrupted:
            if (path, reading.metric) not in sizes:
                for measured in fdu_reading(fdu, Path(path), timeout):
                    sizes[(path, measured.metric)] = measured.total or 0
        reading.skipped = sum(sizes.get((p, reading.metric), 0) for p in reading.interrupted)
    return {
        "root": str(root),
        "facts": tree_facts(root),
        "readings": [asdict(r) for r in readings],
    }


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


def judge(record: dict[str, object]) -> tuple[str, int]:
    """The record as a table, and how many readings failed."""
    readings = [Reading(**r) for r in record["readings"]]  # type: ignore[arg-type]
    facts: dict[str, dict[str, int]] = record["facts"]  # type: ignore[assignment]
    fdu_at = [(i, r) for i, r in enumerate(readings) if r.tool == "fdu"]
    series = {m: [r.total or 0 for _, r in fdu_at if r.metric == m] for m in METRICS}
    quiet = {m: len(set(v)) == 1 for m, v in series.items()}
    churn = {m: max((abs(b - a) for a, b in pairwise(v)), default=0) for m, v in series.items()}

    def around(position: int, metric: str) -> tuple[int, int]:
        before = next(r for i, r in reversed(fdu_at) if i < position and r.metric == metric)
        after = next(r for i, r in fdu_at if i > position and r.metric == metric)
        return before.total or 0, after.total or 0

    first = fdu_at[0][1]
    moved = {m: max(v) - min(v) for m, v in series.items()}
    lines = [
        f"### `{record['root']}`",
        "",
        f"fdu read {human(series['allocated'][0])} allocated and "
        f"{human(series['apparent'][0])} apparent, exiting {first.exit}; across its "
        f"{len(series['allocated'])} readings, one before each tool and one at the end, the "
        f"tree moved {human(moved['allocated'])} allocated and {human(moved['apparent'])} "
        f"apparent, at most {human(churn['allocated'])} and {human(churn['apparent'])} "
        f"between consecutive readings. fdu errors: {describe(first.errors)}.",
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
    failures = 0
    for position, reading in enumerate(readings):
        if reading.tool == "fdu":
            continue
        model = MODELS[(reading.tool, reading.metric)]
        parts = expected_delta(model, reading.metric, facts)
        expected = sum(amount for _, amount in parts)
        made_of = ", ".join(f"{name} {signed(amount)}" for name, amount in parts) or "—"
        before, after = around(position, reading.metric)
        low, high = sorted((before, after))
        verdict = verdict_for(
            reading, low + expected, high + expected, quiet[reading.metric], churn[reading.metric]
        )
        failures += verdict in ("UNEXPLAINED", "no reading")
        total = human(reading.total) if reading.total is not None else "—"
        delta = signed(reading.total - before) if reading.total is not None else "—"
        lines.append(
            f"| {reading.tool} | {reading.metric} | {total} | {delta} | {signed(expected)} | "
            f"{made_of} | {verdict} | {describe(reading.errors)} | {reading.seconds:.1f} s |"
        )
    gave_up = [r for r in readings if r.tool != "fdu" and r.interrupted]
    if gave_up:
        lines += ["", "Directories a tool gave up on after an interrupted read:", ""]
        for reading in gave_up:
            names = ", ".join(f"`{relative(p, record['root'])}`" for p in reading.interrupted)
            lines.append(f"- {reading.tool} ({reading.metric}): {names}")
    failures += top_level(readings, quiet["allocated"], churn["allocated"], lines)
    return "\n".join(lines), failures


def verdict_for(reading: Reading, low: int, high: int, quiet: bool, churn: int) -> str:
    total = reading.total
    if total is None:
        return "no reading"
    if quiet:
        return "agrees exactly" if total == low else "UNEXPLAINED"
    if low <= total <= high:
        return "agrees within the tree's movement"
    if low - churn <= total <= high + churn:
        return f"agrees within observed churn ({human(churn)})"
    if reading.interrupted and low - reading.skipped - churn <= total < low:
        return (
            f"short by what it gave up on: {len(reading.interrupted):,} interrupted "
            f"directories holding {human(reading.skipped)}"
        )
    return "UNEXPLAINED"


def top_level(readings: list[Reading], quiet: bool, churn: int, lines: list[str]) -> int:
    """Top-level directories against the reference and the fdu reading just before it."""
    position, reference = next(
        (i, r) for i, r in enumerate(readings) if r.tool == "GNU du -l" and r.metric == "allocated"
    )
    ours = next(
        r for r in reversed(readings[:position]) if r.tool == "fdu" and r.metric == "allocated"
    )
    limit = 0 if quiet else churn
    rows = []
    for name in sorted(set(ours.children) | set(reference.children)):
        mine, theirs = ours.children.get(name), reference.children.get(name)
        if mine is None or theirs is None or abs(mine - theirs) > limit:
            rows.append(
                f"| `{name}` | {human(mine) if mine is not None else 'absent'} | "
                f"{human(theirs) if theirs is not None else 'absent'} | "
                f"{signed(mine - theirs) if mine is not None and theirs is not None else '—'} |"
            )
    bound = "exactly" if quiet else f"within {human(limit)}"
    lines += [
        "",
        f"Top-level directories whose allocated size does not agree with GNU du -l {bound}: "
        f"{len(rows)} of {len(ours.children):,}.",
    ]
    if rows:
        lines += ["", "| Directory | fdu | GNU du -l | Δ |", "| --- | ---: | ---: | ---: |", *rows]
    return len(rows) if quiet else 0


def relative(path: str, root: object) -> str:
    try:
        return str(Path(path).relative_to(str(root)))
    except ValueError:
        return path


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
        with open(probe / "sparse", "wb") as sparse:
            sparse.truncate(1 << 30)
        (probe / "name with spaces").mkdir()
        (probe / "name with spaces" / "small").write_bytes(b"small")
        (probe / "locked").mkdir()
        (probe / "locked" / "hidden").write_bytes(b"hidden")
        (probe / "locked").chmod(0)
        text, failures = judge(measure(probe, fdu, timeout, du))
        print(text + "\n")
        exact = text.count("| agrees exactly |")
        print(f"Self-test: {exact} readings agree exactly; {failures} failed.")
        return 1 if failures else 0
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
        help="prove the counting models on a tree built with each case",
    )
    parser.add_argument("--scratch", type=Path, help="where --self-test builds its tree")
    args = parser.parse_args()

    if args.rejudge:
        failures = 0
        for record in json.loads(args.rejudge.read_text(encoding="utf-8")):
            text, failed = judge(record)
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
        text, failed = judge(record)
        failures += failed
        print(text + "\n", flush=True)
    print(f"{failures} readings failed." if failures else "Every reading is explained.")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
