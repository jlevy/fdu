#!/usr/bin/env python3
"""Compare fdu's totals on real trees with other disk-usage tools, and explain each gap.

Manual release QA, not part of `make check`: real trees change while they are measured,
and peer tools are not pinned. See tests/qa/cli-installed-e2e.qa.md, the peer-agreement
phase, for how to read the table.

    python3 scripts/qa_peer_agreement.py ~/Library ~/.rustup /Applications

Every tool runs once per root, one at a time, and fdu runs again last, so the spread of
fdu's two readings bounds how much the tree changed during the comparison.

The reference is GNU du with `--count-links`: like fdu, it counts a file once for each
path that reaches it, so on APFS (where directories and symbolic links occupy no blocks)
its allocated total must equal fdu's to within that drift. Plain GNU du counts a
hard-linked file once, and the difference between the two readings is exactly what every
tool that deduplicates hard links should come in under fdu by. The script also measures
the tree's symbolic links, whose target text du and diskus count as apparent size and fdu
does not, so on a quiet tree every verdict is exact rather than within a tolerance.

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
from dataclasses import asdict, dataclass, field
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
    note: str = ""


def run(command: list[str], timeout: int) -> tuple[str, float]:
    started = time.monotonic()
    result = subprocess.run(command, capture_output=True, text=True, timeout=timeout, check=False)
    return result.stdout, time.monotonic() - started


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
    out, seconds = run(command, timeout)
    report = json.loads(out)["reports"]
    tree = (report[0] if isinstance(report, list) else report)["tree"]
    readings = []
    for metric, key in (("allocated", "allocated"), ("apparent", "bytes")):
        children = {child["name"]: child[key] for child in tree["children"]}
        readings.append(
            Reading(label, metric, PER_PATH, tree[key], seconds, " ".join(command), children)
        )
    return readings


def gnu_reading(du: str, root: Path, timeout: int, metric: str, links: bool) -> Reading:
    size = ["-b"] if metric == "apparent" else ["-B1"]
    command = [du, *size, "--max-depth=1", *(["-l"] if links else []), str(root)]
    out, seconds = run(command, timeout)
    children = {}
    total = None
    for line in out.splitlines():
        value, _, path = line.partition("\t")
        if Path(path) == root:
            total = int(value)
        else:
            children[Path(path).name] = int(value)
    label = "GNU du -l" if links else "GNU du"
    return Reading(
        label, metric, PER_PATH if links else PER_INODE, total, seconds, " ".join(command), children
    )


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
    out, seconds = run(command, timeout)
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
    return Reading("dust", metric, counting, total, seconds, " ".join(command), children)


def simple_reading(
    tool: str,
    command: list[str],
    metric: str,
    counting: str,
    pattern: str,
    timeout: int,
    scale: int = 1,
) -> Reading:
    out, seconds = run(command, timeout)
    value = first_group(re.search(pattern, out, re.M))
    total = value * scale if value is not None else None
    return Reading(tool, metric, counting, total, seconds, " ".join(command))


def peers(root: Path, timeout: int, du: str | None) -> list[Reading]:
    readings: list[Reading] = []
    if du:
        for metric in ("allocated", "apparent"):
            readings.append(gnu_reading(du, root, timeout, metric, links=True))
            readings.append(gnu_reading(du, root, timeout, metric, links=False))
    if shutil.which("dust"):
        readings += [dust_reading(root, timeout, m) for m in ("allocated", "apparent")]
    if shutil.which("pdu"):
        for metric, quantity in (("allocated", "block-size"), ("apparent", "apparent-size")):
            readings.append(
                simple_reading(
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
            readings.append(
                simple_reading(
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
            readings.append(
                simple_reading(
                    "diskus",
                    ["diskus", *flags, str(root)],
                    metric,
                    PER_INODE,
                    r"\((\d[\d,]*) bytes\)|^(\d+)$",
                    timeout,
                )
            )
    readings.append(
        simple_reading(
            "BSD du" if sys.platform == "darwin" else "du",
            ["/usr/bin/du", "-sk", str(root)],
            "allocated",
            PER_INODE,
            r"^(\d+)",
            timeout,
            scale=1024,
        )
    )
    return readings


def first_group(match: re.Match[str] | None) -> int | None:
    if match is None:
        return None
    value = next(group for group in match.groups() if group)
    return int(value.replace(",", ""))


def symlinks(root: Path) -> dict[str, int]:
    """The apparent and allocated bytes of the symbolic links under `root`."""
    apparent = allocated = count = 0
    stack = [root]
    while stack:
        directory = stack.pop()
        try:
            entries = list(os.scandir(directory))
        except OSError:
            continue
        for entry in entries:
            try:
                if entry.is_symlink():
                    info = entry.stat(follow_symlinks=False)
                    count += 1
                    apparent += info.st_size
                    allocated += getattr(info, "st_blocks", 0) * 512
                elif entry.is_dir(follow_symlinks=False):
                    stack.append(Path(entry.path))
            except OSError:
                continue
    return {"count": count, "apparent": apparent, "allocated": allocated}


def human(n: float) -> str:
    sign = "-" if n < 0 else ""
    n = abs(n)
    for unit in ("B", "KiB", "MiB", "GiB", "TiB", "PiB"):
        if n < 1024 or unit == "PiB":
            return f"{sign}{n:.0f} {unit}" if unit == "B" else f"{sign}{n:.1f} {unit}"
        n /= 1024
    return f"{sign}{n}"


def measure(root: Path, fdu: str, timeout: int) -> dict[str, object]:
    du = gnu_du()
    first = fdu_reading(fdu, root, timeout, "fdu")
    others = peers(root, timeout, du)
    last = fdu_reading(fdu, root, timeout, "fdu (again)")
    return {
        "root": str(root),
        "symlinks": symlinks(root),
        "readings": [asdict(r) for r in [*first, *others, *last]],
    }


# Tools that count a symbolic link's target text as apparent size, as du does.
COUNTS_SYMLINKS = {"GNU du -l", "GNU du", "diskus", "BSD du"}


def judge(record: dict[str, object]) -> str:
    readings = [Reading(**r) for r in record["readings"]]  # type: ignore[arg-type]
    links_seen: dict[str, int] = record["symlinks"]  # type: ignore[assignment]
    first = {r.metric: r for r in readings if r.tool == "fdu"}
    last = {r.metric: r for r in readings if r.tool == "fdu (again)"}
    drift = {m: abs((last[m].total or 0) - (first[m].total or 0)) for m in first}
    dedup: dict[str, int | None] = {}
    for metric in ("allocated", "apparent"):
        per_path = next(
            (r.total for r in readings if r.tool == "GNU du -l" and r.metric == metric), None
        )
        per_inode = next(
            (r.total for r in readings if r.tool == "GNU du" and r.metric == metric), None
        )
        dedup[metric] = (
            per_path - per_inode if per_path is not None and per_inode is not None else None
        )
    root = record["root"]
    lines = [
        f"### `{root}`",
        "",
        f"fdu: allocated {human(first['allocated'].total or 0)}, apparent "
        f"{human(first['apparent'].total or 0)}. Drift between fdu's two runs: "
        f"{human(drift['allocated'])} allocated, {human(drift['apparent'])} apparent. "
        f"Hard-link duplication: {human(dedup['allocated'] or 0)} allocated. "
        f"Symbolic links: {links_seen['count']:,}, {human(links_seen['apparent'])} "
        f"apparent, {human(links_seen['allocated'])} allocated.",
        "",
        "| Tool | Metric | Counts | Total | Δ vs fdu | Expected Δ | Verdict | Time |",
        "| --- | --- | --- | ---: | ---: | ---: | --- | ---: |",
    ]
    for reading in readings:
        base = first[reading.metric].total or 0
        if reading.total is None:
            lines.append(
                f"| {reading.tool} | {reading.metric} | {reading.counting} | — | "
                f"— | — | no reading | {reading.seconds:.1f} s |"
            )
            continue
        delta = reading.total - base
        expected = 0
        causes = []
        if PER_INODE in reading.counting and dedup[reading.metric]:
            expected -= dedup[reading.metric] or 0
            causes.append("hard links")
        if reading.tool in COUNTS_SYMLINKS and links_seen[reading.metric]:
            expected += links_seen[reading.metric]
            causes.append("symbolic links")
        # A quiet tree must agree exactly; a live one within what fdu itself saw move,
        # plus 0.1% for changes that came and went between fdu's two readings.
        live = drift[reading.metric] > 0
        tolerance = drift[reading.metric] + (base // 1000 if live else 0)
        within = "exactly" if not live else "within drift"
        if DIRS in reading.counting:
            verdict = "adds directory sizes" if delta >= expected - tolerance else "UNEXPLAINED"
        elif abs(delta - expected) <= tolerance:
            verdict = f"agrees {within}" + (f" after {' and '.join(causes)}" if causes else "")
        else:
            verdict = "UNEXPLAINED"
        lines.append(
            f"| {reading.tool} | {reading.metric} | {reading.counting} | "
            f"{human(reading.total)} | {human(delta)} | {human(expected)} | "
            f"{verdict} | {reading.seconds:.1f} s |"
        )

    # Top-level directories, against the per-path reference, so a gap is placed.
    reference = next(
        (r for r in readings if r.tool == "GNU du -l" and r.metric == "allocated"), None
    )
    moved = []
    if reference is not None:
        for name, size in sorted(first["allocated"].children.items()):
            theirs = reference.children.get(name)
            if theirs is None:
                continue
            gap = size - theirs
            if abs(gap) > max(size // 1000, 1024 * 1024):
                moved.append(f"| `{name}` | {human(size)} | {human(theirs)} | {human(gap)} |")
    lines += [
        "",
        "Top-level directories whose allocated size differs from GNU du -l by more "
        f"than 0.1% and 1 MiB: {len(moved)}.",
    ]
    if moved:
        lines += ["", "| Directory | fdu | GNU du -l | Δ |", "| --- | ---: | ---: | ---: |", *moved]
    return "\n".join(lines)


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
