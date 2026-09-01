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

import hashlib
import json
import os
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
    require_diagnostics: bool = False,
    counters_enabled: bool = True,
    oracle_enabled: bool = True,
) -> Dict[str, Any]:
    """Sample ``argv`` and return a ranked self-time attribution.

    ``repeat`` makes the process live long enough to be sampled properly. It is the
    probe's ``--repeat``, so the work is genuinely re-executed rather than the
    process being padded with sleep. Counters and the semantic oracle default to on for
    backward compatibility. Disable both for the cleanest engine attribution; the
    returned artifact labels that unverified mode, and timing runs reject it.
    """
    if sys.platform != "darwin":
        raise ProfileError(
            "sampling is implemented for macOS /usr/bin/sample; on Linux use "
            "`perf record -g` against the same profiling build"
        )
    sampler = shutil.which("sample") or "/usr/bin/sample"
    if not Path(sampler).exists():
        raise ProfileError("/usr/bin/sample is not available")
    resolved_binary = binary.resolve(strict=True)
    if not argv or Path(argv[0]).resolve(strict=False) != resolved_binary:
        raise ProfileError("profile argv does not execute the declared binary")
    if seconds < 1 or repeat < 1:
        raise ProfileError("profile seconds and repeat must be positive")

    command = _profile_command(argv, repeat=repeat, oracle_enabled=oracle_enabled)
    with tempfile.TemporaryDirectory(prefix="fdu-profile-") as scratch:
        report = Path(scratch) / "sample.txt"
        stdout_path = Path(scratch) / "stdout"
        stderr_path = Path(scratch) / "stderr"
        environment = _profile_environment(counters_enabled=counters_enabled)
        with stdout_path.open("xb") as stdout_file, stderr_path.open("xb") as stderr_file:
            process = subprocess.Popen(
                command,
                stdin=subprocess.DEVNULL,
                stdout=stdout_file,
                stderr=stderr_file,
                env=environment,
            )
            # Give the process a moment to get past startup and into steady state, so the
            # profile describes the walk rather than dyld.
            time.sleep(0.3)
            if process.poll() is not None:
                raise ProfileError("probe exited before it could be sampled")
            sampled = subprocess.run(
                [sampler, str(process.pid), str(seconds), "-mayDie", "-f", str(report)],
                capture_output=True,
                env={**environment, "PATH": os.environ.get("PATH", "")},
            )
            try:
                exit_code = process.wait(timeout=max(60, seconds * 4))
            except subprocess.TimeoutExpired as error:
                process.kill()
                process.wait(timeout=10)
                raise ProfileError("profiled probe did not finish after sampling") from error
        if sampled.returncode != 0 or not report.is_file():
            raise ProfileError(
                "sample failed: " + sampled.stderr.decode("utf-8", errors="replace")[:400]
            )
        text = report.read_text(encoding="utf-8", errors="replace")
        stdout = stdout_path.read_bytes()
        stderr = stderr_path.read_bytes()

    if exit_code != 0:
        raise ProfileError(f"profiled probe exited with {exit_code}")

    frames = parse(text)
    probe = _probe_evidence(stdout)
    if require_diagnostics:
        from benchmarks.realtree import measure

        summary = probe.get("summary")
        reasons = measure._validate_scan_diagnostics(
            probe.get("scan_diagnostics"),
            require_macos_backend=True,
            expected_dirs_read=summary.get("dirs_read") if isinstance(summary, dict) else None,
        )
        if reasons:
            raise ProfileError("profiled probe diagnostics are invalid: " + "; ".join(reasons))
    return {
        "schema": PROFILE_SCHEMA,
        "label": label,
        "binary": str(resolved_binary.name),
        "binary_sha256": _sha256_file(resolved_binary),
        "binary_size_bytes": resolved_binary.stat().st_size,
        "command": _redacted_command(command, binary),
        "counters_enabled": counters_enabled,
        "counters": parse_counter_report(stderr.decode("utf-8", errors="replace")),
        "counter_report_bytes": len(stderr),
        "counter_report_sha256": hashlib.sha256(stderr).hexdigest(),
        "seconds": seconds,
        "repeat": repeat,
        "oracle_enabled": oracle_enabled,
        "probe": probe,
        "total_samples": frames["total_samples"],
        "self_time": frames["self_time"],
        "by_layer": frames["by_layer"],
        "threads": frames["threads"],
    }


def _profile_command(
    argv: Sequence[str], *, repeat: int, oracle_enabled: bool
) -> List[str]:
    """Build the repeated probe command while labelling attribution-only runs."""
    command = list(argv)
    if oracle_enabled and "--no-oracle" in command:
        raise ProfileError(
            "profile argv disables the oracle; select `--oracle disabled` so the "
            "attribution artifact records that choice"
        )
    if not oracle_enabled and "--no-oracle" not in command:
        command.append("--no-oracle")
    command.extend(("--repeat", str(repeat)))
    return command


def _profile_environment(*, counters_enabled: bool) -> Dict[str, str]:
    """Return a complete, stable environment for one sampling run."""
    return {
        "FDU_COUNTERS": "1" if counters_enabled else "0",
        "LANG": "C",
        "LC_ALL": "C",
        "TZ": "UTC",
    }


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _redacted_command(command: Sequence[str], binary: Path) -> List[str]:
    """Retain operational flags while removing every filesystem path."""
    result: List[str] = []
    previous = ""
    resolved_binary = binary.resolve(strict=False)
    for index, argument in enumerate(command):
        if index == 0 or Path(argument).resolve(strict=False) == resolved_binary:
            shown = "{binary}"
        elif previous == "--root":
            shown = "{root}"
        elif previous == "--snapshot":
            shown = "{snapshot}"
        elif Path(argument).is_absolute():
            shown = "{absolute-path}"
        else:
            shown = argument
        result.append(shown)
        previous = argument
    return result


def _probe_evidence(stdout: bytes) -> Dict[str, Any]:
    """Retain only path-free probe fields needed to interpret a stack sample."""
    text = stdout.decode("utf-8", errors="replace").strip()
    if not text:
        return {"unavailable_reason": "profiled process emitted no probe output"}
    try:
        document = json.loads(text.splitlines()[-1])
    except json.JSONDecodeError as error:
        return {"unavailable_reason": f"profiled probe output was not JSON: {error}"}
    if not isinstance(document, dict):
        return {"unavailable_reason": "profiled probe output was not an object"}
    return {
        key: document.get(key)
        for key in (
            "attribution",
            "component_ns",
            "mode",
            "oracle_enabled",
            "scan_diagnostics",
            "schema",
            "source",
            "summary",
        )
    }


def parse_counter_report(text: str) -> Dict[str, Any]:
    """Parse the stable grouped integer rows emitted by ``FDU_COUNTERS``."""
    groups: Dict[str, Dict[str, int]] = {}
    current: Optional[Dict[str, int]] = None
    for raw in text.splitlines():
        line = raw.strip()
        if line.startswith("[") and line.endswith("]"):
            current = groups.setdefault(line[1:-1], {})
            continue
        if current is None or not line:
            continue
        label, separator, raw_value = line.rpartition("  ")
        if separator and raw_value.strip().isdigit():
            current[label.strip()] = int(raw_value.strip())
    return {
        "groups": groups,
        "schema": "fdu-counter-report-v1",
        "unavailable_reason": None if groups else "counter report contained no grouped rows",
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
