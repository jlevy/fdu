#!/usr/bin/env python3
"""Compare fdu's totals on real trees with other disk-usage tools, and explain each gap.

Manual release QA, not part of `make check`: real trees change while they are measured,
and peer tools are not pinned. See tests/qa/cli-installed-e2e.qa.md, the peer-agreement
phase, for how to read the table.

    python3 scripts/qa_peer_agreement.py ~/Library ~/.rustup /Applications

Every tool runs once per root, one at a time, with fdu run immediately before each and
once more at the end, so each tool is judged against the two fdu readings that bracket
it: a live tree such as ~/Library moves while it is measured, and a comparison with a
reading taken minutes earlier would measure the tree's movement rather than the tool.

The reference is GNU du with `--count-links`: like fdu, it counts a file once for each
path that reaches it, so on APFS (where directories and symbolic links occupy no blocks)
its allocated total must equal fdu's to within that drift. Plain GNU du counts a
hard-linked file once, and the difference between the two readings is exactly what every
tool that deduplicates hard links should come in under fdu by. The script measures that
duplication itself, in one walk that also totals the tree's symbolic links (whose target
text du and diskus count as apparent size and fdu does not), so on a quiet tree every
verdict is exact rather than within a tolerance. Each tool's errors are counted from its
output: GNU du on macOS gives up on a directory whose read is interrupted and leaves its
whole subtree out, and a reading that falls short for that reason says so.

Stdlib only, like scripts/run_installed_cli_qa.py, so it runs against a PATH binary.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time
from collections.abc import Callable
from dataclasses import asdict, dataclass, field
from functools import partial
from pathlib import Path

# How a tool counts, which decides what it should agree with.
PER_PATH = "per path"  # a hard-linked file once per path, as fdu does
PER_INODE = "per inode"  # a hard-linked file once
DIRS = "+ directory sizes"  # also adds each directory's own apparent size


@dataclass
class Reading:
    tool: str
    metric: str  # "allocated" or "apparent"
    counting: str
    total: int | None
    seconds: float
    command: str
    children: dict[str, int] = field(default_factory=dict)
    errors: dict[str, int] = field(default_factory=dict)


def run(command: list[str], timeout: int) -> tuple[str, dict[str, int], float]:
    """The command's stdout, the errors its stderr reports by kind, and its wall time."""
    started = time.monotonic()
    result = subprocess.run(command, capture_output=True, text=True, timeout=timeout, check=False)
    return result.stdout, classify(result.stderr), time.monotonic() - started


def classify(stderr: str) -> dict[str, int]:
    """Count the errors a tool printed: denied, interrupted, or other."""
    counts = {"denied": 0, "interrupted": 0, "other": 0}
    for line in stderr.splitlines():
        lowered = line.lower()
        if "interrupted system call" in lowered:
            counts["interrupted"] += 1
        elif "not permitted" in lowered or "permission denied" in lowered:
            counts["denied"] += 1
        elif "error" in lowered or "cannot" in lowered:
            counts["other"] += 1
    return {kind: n for kind, n in counts.items() if n}


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


def fdu_reading(fdu: str, root: Path, timeout: int, label: str) -> list[Reading]:
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
    out, _, seconds = run(command, timeout)
    document = json.loads(out)
    report = document["reports"]
    tree = (report[0] if isinstance(report, list) else report)["tree"]
    status = document.get("status", {})
    errors: dict[str, int] = {}
    for error in status.get("errors", []):
        kind = "denied" if error.get("kind") == "permission" else "other"
        errors[kind] = errors.get(kind, 0) + 1
    if status.get("errors_omitted"):
        # fdu details the first errors and counts the rest.
        errors["more, not detailed"] = status["errors_omitted"]
    readings = []
    for metric, key in (("allocated", "allocated"), ("apparent", "bytes")):
        children = {child["name"]: child[key] for child in tree["children"]}
        readings.append(
            Reading(
                label, metric, PER_PATH, tree[key], seconds, " ".join(command), children, errors
            )
        )
    return readings


def gnu_reading(du: str, root: Path, timeout: int, metric: str, links: bool) -> Reading:
    size = ["-b"] if metric == "apparent" else ["-B1"]
    command = [du, *size, "--max-depth=1", *(["-l"] if links else []), str(root)]
    out, errors, seconds = run(command, timeout)
    children = {}
    total = None
    for line in out.splitlines():
        value, _, path = line.partition("\t")
        if Path(path) == root:
            total = int(value)
        else:
            children[Path(path).name] = int(value)
    label = "GNU du -l" if links else "GNU du"
    counting = PER_PATH if links else PER_INODE
    return Reading(label, metric, counting, total, seconds, " ".join(command), children, errors)


def dust_reading(root: Path, timeout: int, metric: str) -> Reading:
    command = [
        "dust",
        "-d",
        "1",
        "-o",
        "b",
        "-c",
        "-b",
        "-n",
        "100000",
        *(["-s"] if metric == "apparent" else []),
        str(root),
    ]
    out, errors, seconds = run(command, timeout)
    children = {}
    total = None
    for line in out.splitlines():
        match = re.match(r"\s*(\d+)B\s+\S+\s+(.*)$", line)
        if not match:
            continue
        value, name = int(match[1]), match[2].strip()
        if name in (str(root), root.name, "."):
            total = value
        else:
            children[Path(name).name] = value
    counting = f"{PER_PATH} {DIRS}" if metric == "apparent" else PER_INODE
    return Reading("dust", metric, counting, total, seconds, " ".join(command), children, errors)


def simple_reading(
    tool: str,
    command: list[str],
    metric: str,
    counting: str,
    pattern: str,
    timeout: int,
    scale: int = 1,
) -> Reading:
    out, errors, seconds = run(command, timeout)
    value = first_group(re.search(pattern, out, re.M))
    total = value * scale if value is not None else None
    return Reading(tool, metric, counting, total, seconds, " ".join(command), {}, errors)


def peers(root: Path, timeout: int, du: str | None) -> list[Callable[[], Reading]]:
    """One deferred run per tool and metric, so fdu can be run between them."""
    runs: list[Callable[[], Reading]] = []
    if du:
        for metric in ("allocated", "apparent"):
            for links in (True, False):
                runs.append(partial(gnu_reading, du, root, timeout, metric, links))
    if shutil.which("dust"):
        runs += [partial(dust_reading, root, timeout, m) for m in ("allocated", "apparent")]
    if shutil.which("pdu"):
        for metric, quantity in (("allocated", "block-size"), ("apparent", "apparent-size")):
            runs.append(
                partial(
                    simple_reading,
                    "pdu",
                    ["pdu", "-b", "plain", "--max-depth", "1", "-q", quantity, str(root)],
                    metric,
                    PER_PATH if metric == "allocated" else f"{PER_PATH} {DIRS}",
                    r"^\s*(\d+)\s+\S+\s*" + re.escape(str(root)) + r"\b|^\s*(\d+)\s+┌──",
                    timeout,
                )
            )
    if shutil.which("dua"):
        for metric, flags in (("allocated", []), ("apparent", ["-A"])):
            runs.append(
                partial(
                    simple_reading,
                    "dua",
                    ["dua", "-f", "bytes", *flags, "aggregate", str(root)],
                    metric,
                    PER_INODE if metric == "allocated" else f"{PER_INODE} {DIRS}",
                    r"(\d+) b\S*\s+total|(\d+) b\S*\s+" + re.escape(str(root)),
                    timeout,
                )
            )
    if shutil.which("diskus"):
        for metric, flags in (("allocated", []), ("apparent", ["-b"])):
            runs.append(
                partial(
                    simple_reading,
                    "diskus",
                    ["diskus", *flags, str(root)],
                    metric,
                    PER_INODE,
                    r"\((\d[\d,]*) bytes\)|^(\d+)$",
                    timeout,
                )
            )
    runs.append(
        partial(
            simple_reading,
            "BSD du" if sys.platform == "darwin" else "du",
            ["/usr/bin/du", "-sk", str(root)],
            "allocated",
            PER_INODE,
            r"^(\d+)",
            timeout,
            scale=1024,
        )
    )
    return runs


def first_group(match: re.Match[str] | None) -> int | None:
    if match is None:
        return None
    value = next(group for group in match.groups() if group)
    return int(value.replace(",", ""))


def tree_facts(root: Path) -> dict[str, dict[str, int]]:
    """What the tools count differently, measured in one walk of `root`.

    Symbolic links: their count and bytes. Hard links: how much counting a hard-linked
    file once per path adds over counting it once, from the paths under `root` that share
    an inode. Directories the walk could not list: the same ones every tool must skip.
    """
    links = {"count": 0, "apparent": 0, "allocated": 0}
    shared: dict[tuple[int, int], list[int]] = {}
    unreadable = 0
    stack = [root]
    while stack:
        directory = stack.pop()
        try:
            entries = list(os.scandir(directory))
        except OSError:
            unreadable += 1
            continue
        for entry in entries:
            try:
                if entry.is_symlink():
                    info = entry.stat(follow_symlinks=False)
                    links["count"] += 1
                    links["apparent"] += info.st_size
                    links["allocated"] += getattr(info, "st_blocks", 0) * 512
                elif entry.is_dir(follow_symlinks=False):
                    stack.append(Path(entry.path))
                elif entry.is_file(follow_symlinks=False):
                    info = entry.stat(follow_symlinks=False)
                    if info.st_nlink > 1:
                        key = (info.st_dev, info.st_ino)
                        size = [info.st_size, getattr(info, "st_blocks", 0) * 512, 0]
                        shared.setdefault(key, size)[2] += 1
            except OSError:
                continue
    duplicated = {"inodes": 0, "apparent": 0, "allocated": 0}
    for apparent, allocated, paths in shared.values():
        if paths > 1:
            duplicated["inodes"] += 1
            duplicated["apparent"] += apparent * (paths - 1)
            duplicated["allocated"] += allocated * (paths - 1)
    return {"symlinks": links, "hard_links": duplicated, "unreadable": {"dirs": unreadable}}


def human(n: float) -> str:
    sign = "-" if n < 0 else ""
    n = abs(n)
    for unit in ("B", "KiB", "MiB", "GiB", "TiB", "PiB"):
        if n < 1024 or unit == "PiB":
            return f"{sign}{n:.0f} {unit}" if unit == "B" else f"{sign}{n:.1f} {unit}"
        n /= 1024
    return f"{sign}{n}"


def measure(root: Path, fdu: str, timeout: int) -> dict[str, object]:
    readings = fdu_reading(fdu, root, timeout, "fdu")
    for run_tool in peers(root, timeout, gnu_du()):
        readings.append(run_tool())
        readings += fdu_reading(fdu, root, timeout, "fdu")
    return {
        "root": str(root),
        "facts": tree_facts(root),
        "readings": [asdict(r) for r in readings],
    }


# Tools that count a symbolic link's target text as apparent size, as du does.
COUNTS_SYMLINKS = {"GNU du -l", "GNU du", "diskus", "BSD du"}


def judge(record: dict[str, object]) -> str:
    readings = [Reading(**r) for r in record["readings"]]  # type: ignore[arg-type]
    facts: dict[str, dict[str, int]] = record["facts"]  # type: ignore[assignment]
    links_seen, duplicated = facts["symlinks"], facts["hard_links"]
    fdu_at = [(i, r) for i, r in enumerate(readings) if r.tool == "fdu"]

    def bracket(position: int, metric: str) -> tuple[Reading, Reading]:
        """The fdu readings of `metric` taken just before and just after `position`."""
        before = next(r for i, r in reversed(fdu_at) if i < position and r.metric == metric)
        after = next(r for i, r in fdu_at if i > position and r.metric == metric)
        return before, after

    totals = {
        metric: [r.total or 0 for _, r in fdu_at if r.metric == metric]
        for metric in ("allocated", "apparent")
    }
    moved = {metric: max(v) - min(v) for metric, v in totals.items()}
    first = {metric: v[0] for metric, v in totals.items()}
    fdu_errors = next(r for _, r in fdu_at).errors
    lines = [
        f"### `{record['root']}`",
        "",
        f"fdu: allocated {human(first['allocated'])}, apparent {human(first['apparent'])}; "
        f"across its {len(totals['allocated'])} readings, one before each tool and one at "
        f"the end, the tree moved {human(moved['allocated'])} allocated and "
        f"{human(moved['apparent'])} apparent. "
        f"fdu errors: {describe(fdu_errors)}. "
        f"Hard links: {duplicated['inodes']:,} shared inodes, which counting per path adds "
        f"{human(duplicated['allocated'])} allocated to. "
        f"Symbolic links: {links_seen['count']:,}, {human(links_seen['apparent'])} apparent, "
        f"{human(links_seen['allocated'])} allocated. "
        f"Directories that could not be listed: {facts['unreadable']['dirs']:,}.",
        "",
        "| Tool | Metric | Counts | Total | Δ vs fdu | Expected Δ | fdu moved | Verdict "
        "| Errors | Time |",
        "| --- | --- | --- | ---: | ---: | ---: | ---: | --- | --- | ---: |",
    ]
    for position, reading in enumerate(readings):
        if reading.tool == "fdu":
            continue
        errors = describe(reading.errors)
        before, after = bracket(position, reading.metric)
        if reading.total is None:
            lines.append(
                f"| {reading.tool} | {reading.metric} | {reading.counting} | — | — | — | — "
                f"| no reading | {errors} | {reading.seconds:.1f} s |"
            )
            continue
        low, high = sorted((before.total or 0, after.total or 0))
        delta = reading.total - (before.total or 0)
        expected = 0
        causes = []
        if PER_INODE in reading.counting and duplicated[reading.metric]:
            expected -= duplicated[reading.metric]
            causes.append("hard links")
        if reading.tool in COUNTS_SYMLINKS and links_seen[reading.metric]:
            expected += links_seen[reading.metric]
            causes.append("symbolic links")
        # Exact on a quiet tree, one whose every fdu reading was the same. On a live one,
        # within the range the two fdu readings around the tool span, plus 0.05% for a
        # change that came and went while the tool ran: equal readings on either side of
        # a tool do not show that nothing changed during it.
        quiet = moved[reading.metric] == 0
        slack = 0 if quiet else low // 2000
        within = "exactly" if quiet else "within the tree's movement"
        interrupted = reading.errors.get("interrupted", 0)
        if DIRS in reading.counting:
            adds = reading.total >= low + expected - slack
            verdict = "adds directory sizes" if adds else "UNEXPLAINED"
        elif low + expected - slack <= reading.total <= high + expected + slack:
            verdict = f"agrees {within}" + (f" after {' and '.join(causes)}" if causes else "")
        else:
            verdict = "UNEXPLAINED"
        if verdict == "UNEXPLAINED" and interrupted and reading.total < low + expected:
            verdict = f"short: gave up on {interrupted:,} directories after interrupted reads"
        lines.append(
            f"| {reading.tool} | {reading.metric} | {reading.counting} | "
            f"{human(reading.total)} | {human(delta)} | {human(expected)} | "
            f"{human((after.total or 0) - (before.total or 0))} | {verdict} | {errors} "
            f"| {reading.seconds:.1f} s |"
        )

    # Top-level directories, against the per-path reference and the fdu reading taken
    # just before it, so a gap is placed.
    moved_dirs = []
    reference = next(
        (
            (i, r)
            for i, r in enumerate(readings)
            if r.tool == "GNU du -l" and r.metric == "allocated"
        ),
        None,
    )
    if reference is not None:
        position, theirs_reading = reference
        ours = bracket(position, "allocated")[0].children
        for name, size in sorted(ours.items()):
            theirs = theirs_reading.children.get(name)
            if theirs is None:
                continue
            gap = size - theirs
            if abs(gap) > max(size // 1000, 1024 * 1024):
                moved_dirs.append(f"| `{name}` | {human(size)} | {human(theirs)} | {human(gap)} |")
    lines += [
        "",
        "Top-level directories whose allocated size differs from GNU du -l by more "
        f"than 0.1% and 1 MiB: {len(moved_dirs)}.",
    ]
    if moved_dirs:
        lines += [
            "",
            "| Directory | fdu | GNU du -l | Δ |",
            "| --- | ---: | ---: | ---: |",
            *moved_dirs,
        ]
    return "\n".join(lines)


def describe(errors: dict[str, int]) -> str:
    return ", ".join(f"{n:,} {kind}" for kind, n in errors.items()) or "none"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("roots", nargs="*", type=Path)
    parser.add_argument("--fdu", default=os.environ.get("FDU", "fdu"))
    parser.add_argument("--timeout", type=int, default=1800, help="seconds per command")
    parser.add_argument("--json", type=Path, help="also write every reading here")
    parser.add_argument(
        "--rejudge",
        type=Path,
        help="judge readings saved by an earlier --json instead of measuring",
    )
    args = parser.parse_args()

    if args.rejudge:
        for record in json.loads(args.rejudge.read_text(encoding="utf-8")):
            print(judge(record) + "\n", flush=True)
        return 0
    version = subprocess.run([args.fdu, "--version"], capture_output=True, text=True).stdout
    print(f"## Peer agreement ({version.strip()}, {time.strftime('%Y-%m-%d')})\n", flush=True)
    data = []
    for root in args.roots:
        record = measure(root.expanduser().resolve(), args.fdu, args.timeout)
        data.append(record)
        if args.json:
            args.json.write_text(json.dumps(data, indent=1), encoding="utf-8")
        print(judge(record) + "\n", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
