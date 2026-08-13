"""Attribute time to functions, using a sampling profiler and a symbol-bearing build.

Profiling and timing answer different questions and must not be mixed. Timing asks
"is the candidate faster?", and its answer has to come from an unmodified release
build running the work exactly once, because that is what a user gets. Profiling asks
"where does the time go?", and its answer needs symbols and thousands of stacks,
which means a debug-info build repeating the work in one warm process.

So this module deliberately produces no timings. It produces a ranked attribution,
and the loop uses it to choose what to try next — never to decide whether a change
worked.

macOS is the supported host today: ``/usr/bin/sample`` ships with the OS and needs no
entitlement to attach to a process the operator owns, which ``dtrace`` does under
System Integrity Protection. On Linux the same role is played by ``perf record``.
"""

from __future__ import annotations

import re
import shutil
import subprocess
import sys
import tempfile
import time
from collections import Counter
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence

PROFILE_SCHEMA = "fdu-realtree-profile-v1"

#: How long to sample. The probe is asked to repeat its work enough to outlive this.
DEFAULT_SAMPLE_SECONDS = 8

#: ``sample`` reports one line per frame, indented, of the form
#: ``    1234 symbol  (in binary) + 56  [0x...]``.
_FRAME = re.compile(
    r"^(?P<indent>[\s+!:|]*)(?P<count>\d+)\s+(?P<symbol>.*?)\s+\(in (?P<image>[^)]*)\)"
)


class ProfileError(RuntimeError):
    """A profile could not be collected."""


def capture(
    *,
    binary: Path,
    argv: Sequence[str],
    seconds: int = DEFAULT_SAMPLE_SECONDS,
    repeat: int = 20,
    label: str = "",
) -> Dict[str, Any]:
    """Sample ``argv`` and return a ranked self-time attribution.

    ``repeat`` makes the process live long enough to be sampled properly. It is the
    probe's ``--repeat``, so the work is genuinely re-executed rather than the
    process being padded with sleep.
    """
    if sys.platform != "darwin":
        raise ProfileError(
            "sampling is implemented for macOS /usr/bin/sample; on Linux use "
            "`perf record -g` against the same profiling build"
        )
    sampler = shutil.which("sample") or "/usr/bin/sample"
    if not Path(sampler).exists():
        raise ProfileError("/usr/bin/sample is not available")

    command = list(argv) + ["--repeat", str(repeat)]
    with tempfile.TemporaryDirectory(prefix="fdu-profile-") as scratch:
        report = Path(scratch) / "sample.txt"
        process = subprocess.Popen(
            command,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
        )
        # Give the process a moment to get past startup and into steady state, so the
        # profile describes the walk rather than dyld.
        time.sleep(0.3)
        if process.poll() is not None:
            stderr = (process.stderr.read() if process.stderr else b"").decode(
                "utf-8", errors="replace"
            )
            raise ProfileError(
                f"probe exited before it could be sampled: {stderr[:400]}"
            )
        sampled = subprocess.run(
            [sampler, str(process.pid), str(seconds), "-mayDie", "-f", str(report)],
            capture_output=True,
        )
        process.wait(timeout=max(60, seconds * 4))
        if sampled.returncode != 0 or not report.is_file():
            raise ProfileError(
                "sample failed: "
                + sampled.stderr.decode("utf-8", errors="replace")[:400]
            )
        text = report.read_text(encoding="utf-8", errors="replace")

    frames = parse(text)
    return {
        "schema": PROFILE_SCHEMA,
        "label": label,
        "binary": str(binary.name),
        "command": command,
        "seconds": seconds,
        "repeat": repeat,
        "total_samples": frames["total_samples"],
        "self_time": frames["self_time"],
        "by_layer": frames["by_layer"],
        "threads": frames["threads"],
    }


def parse(text: str) -> Dict[str, Any]:
    """Turn a ``sample`` call graph into self-time per symbol.

    ``sample`` prints an inclusive tree: each line's count includes its children. Self
    time is therefore a line's count minus the sum of its direct children's counts,
    which the indentation tells us. Reporting inclusive counts instead would put
    ``main`` at 100% and teach nobody anything.
    """
    lines: List[tuple] = []
    for raw in text.splitlines():
        match = _FRAME.match(raw)
        if match is None:
            continue
        depth = len(match.group("indent"))
        lines.append(
            (depth, int(match.group("count")), match.group("symbol").strip(), match.group("image"))
        )

    # One pass with an explicit stack: every line contributes its count to its
    # parent's child total, and what is left over after the whole tree is read is
    # that frame's self time.
    child_totals = [0] * len(lines)
    stack: List[int] = []
    for position, (depth, count, _symbol, _image) in enumerate(lines):
        while stack and lines[stack[-1]][0] >= depth:
            stack.pop()
        if stack:
            child_totals[stack[-1]] += count
        stack.append(position)

    self_time: Counter = Counter()
    images: Counter = Counter()
    for position, (_depth, count, symbol, image) in enumerate(lines):
        own = max(0, count - child_totals[position])
        if own:
            self_time[_clean(symbol)] += own
            images[image] += own

    grand_total = sum(self_time.values()) or 1
    ranked = [
        {
            "symbol": symbol,
            "samples": count,
            "percent": round(100.0 * count / grand_total, 2),
        }
        for symbol, count in self_time.most_common(40)
    ]
    return {
        "total_samples": grand_total,
        "self_time": ranked,
        "by_layer": _layers(self_time, grand_total),
        "threads": images.most_common(10),
    }


def _clean(symbol: str) -> str:
    """Strip the generic noise that makes two identical frames look different."""
    symbol = symbol.split(" + ")[0].strip()
    symbol = re.sub(r"::h[0-9a-f]{16}$", "", symbol)
    return symbol


#: Grouping symbols into layers is what makes a profile actionable. A list of forty
#: mangled Rust symbols does not tell you whether to attack syscalls or allocation;
#: "58% kernel, 21% allocator" does.
#: Order matters: the first pattern that matches wins. Rust symbols arrive in v0
#: mangled form (``_RNv...3fdu5index...``) as often as demangled, so every layer
#: matches both spellings.
#:
#: ``probe/oracle`` is listed first on purpose. The probe hashes the whole index to
#: prove it saw the same tree the oracle saw, and under ``--repeat`` it does that on
#: every iteration. That work is real, but it is the harness proving itself, not fdu
#: doing its job, and it sits outside the component timer. Naming it keeps it from
#: being silently averaged into the engine's cost.
_LAYERS = (
    ("probe/oracle", (r"Sha256", r"sha2", r"summarize_index", r"10perf_probe")),
    (
        "kernel/syscall",
        (r"^__", r"syscall", r"mach_", r"kevent", r"getdirentries", r"fstatat", r"^_platform_"),
    ),
    ("allocator", (r"malloc", r"free", r"realloc", r"nanov2", r"szone", r"tiny_", r"xzm")),
    ("fdu::scan", (r"fdu::scan", r"3fdu4scan", r"scan_into_index", r"metadata_for_fingerprint")),
    (
        "fdu::index",
        (
            r"fdu::index",
            r"3fdu5index",
            r"Index<",
            r"apply_baseline",
            r"apply_validated",
            r"merge_upward",
            r"unmerge_upward",
            r"normalize",
            r"rollup",
            r"RollUp",
        ),
    ),
    ("fdu::snapshot", (r"fdu::snapshot", r"3fdu8snapshot")),
    (
        "fdu::content",
        (r"fdu::content", r"3fdu7content", r"BasicAccumulator", r"analyze_candidate"),
    ),
    ("std::fs", (r"std::fs", r"std::sys", r"ReadDir", r"DirEntry", r"2fs", r"3sys")),
    ("collections", (r"BTreeMap", r"HashMap", r"btree", r"hashbrown", r"7btree", r"4hash")),
    ("path", (r"PathBuf", r"OsString", r"std::path", r"components", r"4path", r"Components")),
)


def _layers(self_time: Counter, grand_total: int) -> List[Dict[str, Any]]:
    assigned: Counter = Counter()
    for symbol, count in self_time.items():
        for name, patterns in _LAYERS:
            if any(re.search(pattern, symbol) for pattern in patterns):
                assigned[name] += count
                break
        else:
            assigned["other"] += count
    return [
        {
            "layer": name,
            "samples": count,
            "percent": round(100.0 * count / grand_total, 2),
        }
        for name, count in assigned.most_common()
    ]
