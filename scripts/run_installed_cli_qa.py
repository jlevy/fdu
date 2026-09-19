#!/usr/bin/env python3
"""Sequential installed-CLI QA harness.

One fdu process at a time. Records wall time and peak RSS. Not part of `make check`.

    python3 scripts/run_installed_cli_qa.py --small PATH [--medium PATH] [--large PATH]

Override the binary with `--fdu` or `FDU`. Isolate cache arms with a fresh
`XDG_CACHE_HOME` per arm so a run does not read or write the user cache directory.

Stdlib only: the playbook must run against a PATH binary without the package venv,
so this file does not depend on Strif or prettyfmt.
"""

from __future__ import annotations

import argparse
import csv
import io
import json
import os
import re
import shlex
import signal
import subprocess
import sys
import tempfile
import time
from collections.abc import Sequence
from dataclasses import asdict, dataclass, field
from enum import StrEnum
from pathlib import Path
from shutil import which

FOOTER_RE = re.compile(r"analysis (?P<fresh>[0-9,]+) fresh.*?(?P<cached>[0-9,]+) cached")
SOURCE_RE = re.compile(r"(cold scan|warm revalidation|cache only)")

# /usr/bin/time is the portable wall-time + peak-RSS source; GNU time's `-f` and
# BSD time's `-l` are not interchangeable, so the wrapper is platform-specific.
TIME_BIN = Path("/usr/bin/time")
BYTES_PER_MIB = 1024 * 1024
KIB_PER_MIB = 1024

# GNU timeout's convention; reused when the harness SIGINTs a hung fdu and then kills it.
TIMEOUT_EXIT_CODE = 124
# 137 is 128+SIGKILL; -9 is the raw signal on some Pythons.
SIGKILL_EXIT_CODES = frozenset({137, -9})
# 130 is 128+SIGINT; -2 is SIGINT on some Pythons.
WATCH_OK_EXIT_CODES = frozenset({0, 130, -2})
# Exit 2 is a documented partial/TCC result.
LARGE_OK_EXIT_CODES = frozenset({0, 2})
# `--view=documents` without `--analyze` is a usage error, not a crash.
DOCUMENTS_WITHOUT_ANALYZE_EXIT = 2

# Defaults match the playbook's time-boxes: medium metadata may take minutes; Library
# steps must fail closed rather than hang.
DEFAULT_MEDIUM_TIMEOUT_S = 600.0
DEFAULT_LARGE_TIMEOUT_S = 180.0
DEFAULT_RSS_LIMIT_MIB = 2048.0
VIEWS_TIMEOUT_S = 180.0
CACHE_ANALYZE_TIMEOUT_S = 300.0
CACHE_STATUS_TIMEOUT_S = 30.0
ANALYZE_EXTRA_TIMEOUT_S = 300.0
# After SIGINT, wait this long before SIGKILL so a clean shutdown can finish.
SIGINT_GRACE_S = 8.0

# Second cache-on run must beat this fraction of the first run's wall time, or reuse
# is not visible in the timing (the cached count is checked separately).
CACHE_REUSE_MAX_FRACTION_OF_FIRST = 0.9

# The installed binary has no injectable clock. --watch is wall-clock, so the harness
# waits through one interval after create before SIGINT.
WATCH_SETTLE_BEFORE_CREATE_S = 1.0
WATCH_SETTLE_AFTER_CREATE_S = 2.0
WATCH_INTERVAL = "200ms"

SNIPPET_HEAD_LINES = 16
SNIPPET_TAIL_LINES = 8
STDERR_SNIPPET_HEAD_LINES = 8
STDERR_SNIPPET_TAIL_LINES = 4
STDOUT_ECHO_LINES = 20
STDERR_ECHO_LINES = 12

DEFAULT_COLOR = "never"
SKILL_COMMAND_NAME = "skill"


class Phase(StrEnum):
    """Harness stages. Values are the `--phases` tokens and the results-table labels."""

    sanity = "sanity"
    views = "views"
    cache_analyze = "cache-analyze"
    analyze_extra = "analyze-extra"
    watch = "watch"
    medium = "medium"
    large = "large"


DEFAULT_PHASES = ",".join(phase.value for phase in Phase)


@dataclass(frozen=True, slots=True)
class Args:
    fdu: str
    small: Path | None
    medium: Path | None
    large: Path | None
    medium_analyze: Path | None
    out_dir: Path | None
    phases: frozenset[Phase]
    medium_timeout: float
    large_timeout: float
    rss_limit_mib: float
    color: str


@dataclass
class Row:
    phase: str
    name: str
    argv: str
    exit: int
    real_s: float | None
    rss_mib: float | None
    out_bytes: int
    note: str
    footer: str = ""
    verdict: str = "ok"


@dataclass
class Suite:
    version: str
    fdu: str
    host: str
    started: str
    rows: list[Row] = field(default_factory=list)
    stopped_reason: str = ""


def env_path(name: str) -> Path | None:
    raw = os.environ.get(name)
    if not raw:
        return None
    return Path(raw)


def parse_phases(raw: str) -> frozenset[Phase]:
    names = [part.strip() for part in raw.split(",") if part.strip()]
    known = {phase.value: phase for phase in Phase}
    unknown = [name for name in names if name not in known]
    if unknown:
        sys.exit(f"unknown phases: {', '.join(unknown)}")
    return frozenset(known[name] for name in names)


def parse_args(argv: Sequence[str] | None = None) -> Args:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fdu", default=os.environ.get("FDU", "fdu"))
    parser.add_argument("--small", type=Path, default=env_path("FDU_QA_SMALL"))
    parser.add_argument("--medium", type=Path, default=env_path("FDU_QA_MEDIUM"))
    parser.add_argument("--large", type=Path, default=env_path("FDU_QA_LARGE"))
    parser.add_argument(
        "--medium-analyze",
        type=Path,
        default=env_path("FDU_QA_MEDIUM_ANALYZE"),
        help="Subtree for content analysis on the medium tree (default: $medium/docs)",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=env_path("FDU_QA_OUT"),
        help="Directory for per-command transcripts and the results table",
    )
    parser.add_argument(
        "--phases",
        default=DEFAULT_PHASES,
        help="Comma-separated subset of phases to run",
    )
    parser.add_argument("--medium-timeout", type=float, default=DEFAULT_MEDIUM_TIMEOUT_S)
    parser.add_argument("--large-timeout", type=float, default=DEFAULT_LARGE_TIMEOUT_S)
    parser.add_argument("--rss-limit-mib", type=float, default=DEFAULT_RSS_LIMIT_MIB)
    parser.add_argument("--color", default=DEFAULT_COLOR)
    raw = parser.parse_args(argv)
    return Args(
        fdu=raw.fdu,
        small=raw.small,
        medium=raw.medium,
        large=raw.large,
        medium_analyze=raw.medium_analyze,
        out_dir=raw.out_dir,
        phases=parse_phases(raw.phases),
        medium_timeout=raw.medium_timeout,
        large_timeout=raw.large_timeout,
        rss_limit_mib=raw.rss_limit_mib,
        color=raw.color,
    )


def resolve_fdu(name: str) -> Path:
    """Resolve a PATH name or an explicit path to the binary under test."""
    explicit = Path(name).expanduser()
    if (explicit.is_absolute() or len(explicit.parts) > 1) and explicit.exists():
        return explicit.resolve()
    found = which(name)
    if found is None:
        sys.exit(f"fdu binary not found: {name}")
    return Path(found)


def run_cmd(
    argv: Sequence[str],
    *,
    env: dict[str, str],
    out_path: Path,
    err_path: Path,
    time_path: Path,
    timeout: float | None,
) -> tuple[int, float | None, float | None]:
    """Run one argv under `/usr/bin/time`. Live streams stay incremental on purpose."""
    if not TIME_BIN.exists():
        sys.exit("/usr/bin/time is required")
    if sys.platform == "darwin":
        wrapper = [str(TIME_BIN), "-l", "-o", str(time_path), "--"]
    else:
        wrapper = [str(TIME_BIN), "-f", "real %e\nmaxrss %M", "-o", str(time_path), "--"]
    cmd = wrapper + list(argv)
    with out_path.open("wb") as out, err_path.open("wb") as err:
        proc = subprocess.Popen(cmd, stdout=out, stderr=err, env=env)
        try:
            proc.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            proc.send_signal(signal.SIGINT)
            try:
                proc.communicate(timeout=SIGINT_GRACE_S)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.communicate()
            return TIMEOUT_EXIT_CODE, None, None
    real_s, rss_mib = parse_time_file(time_path)
    return proc.returncode, real_s, rss_mib


def parse_time_file(path: Path) -> tuple[float | None, float | None]:
    """Read BSD `-l` or GNU `-f` output. The two formats are not a shared dialect."""
    text = path.read_text(encoding="utf-8", errors="replace") if path.exists() else ""
    real_s = None
    rss_mib = None
    darwin = re.search(r"([0-9.]+)\s+real", text)
    if darwin:
        real_s = float(darwin.group(1))
    gnu = re.search(r"^real ([0-9.]+)", text, re.M)
    if gnu and real_s is None:
        real_s = float(gnu.group(1))
    rss_bytes = re.search(r"([0-9]+)\s+maximum resident set size", text)
    if rss_bytes:
        rss_mib = int(rss_bytes.group(1)) / BYTES_PER_MIB
    rss_kb = re.search(r"^maxrss ([0-9]+)", text, re.M)
    if rss_kb and rss_mib is None:
        rss_mib = int(rss_kb.group(1)) / KIB_PER_MIB
    return real_s, rss_mib


def snippet(path: Path, *, head: int = SNIPPET_HEAD_LINES, tail: int = SNIPPET_TAIL_LINES) -> str:
    if not path.exists() or path.stat().st_size == 0:
        return ""
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    if len(lines) <= head + tail:
        return "\n".join(lines)
    return "\n".join([*lines[:head], f"... ({len(lines)} lines) ...", *lines[-tail:]])


def footer_line(path: Path) -> str:
    if not path.exists():
        return ""
    for line in reversed(path.read_text(encoding="utf-8", errors="replace").splitlines()):
        if "Performance:" in line:
            return line.strip()
    return ""


def count_token(footer: str, kind: str) -> int | None:
    match = FOOTER_RE.search(footer)
    if not match:
        return None
    raw = match.group(kind).replace(",", "")
    return int(raw)


def cache_env(base: dict[str, str], cache_home: Path) -> dict[str, str]:
    env = dict(base)
    env["XDG_CACHE_HOME"] = str(cache_home)
    return env


def atomic_write_text(path: Path, text: str) -> None:
    """Install `text` at `path` only after the write succeeds.

    Results files are the dated-report source. A truncate-in-place write can leave
    an empty table if the process dies mid-write.
    """
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp_name = tempfile.mkstemp(prefix=f".{path.name}.", suffix=".tmp", dir=path.parent)
    tmp_path = Path(tmp_name)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            handle.write(text)
        os.replace(tmp_path, path)
    except Exception:
        tmp_path.unlink(missing_ok=True)
        raise


class Runner:
    def __init__(self, args: Args, out_dir: Path, fdu: Path) -> None:
        self.args = args
        self.out_dir = out_dir
        self.fdu = fdu
        self.base_env = os.environ.copy()
        self.suite = Suite(
            version="",
            fdu=str(fdu),
            host=os.uname().sysname,
            started=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        )
        self.stopped = False

    def fdu_argv(self, *parts: str) -> list[str]:
        argv = [str(self.fdu), *parts]
        if "--color" not in parts and self.args.color:
            argv.append(f"--color={self.args.color}")
        return argv

    def run(
        self,
        phase: Phase,
        name: str,
        argv: Sequence[str],
        *,
        env: dict[str, str] | None = None,
        timeout: float | None = None,
        expect: set[int] | None = None,
    ) -> Row:
        if self.stopped:
            row = Row(phase.value, name, " ".join(argv), -1, None, None, 0, "skipped: prior stop")
            row.verdict = "skip"
            self.suite.rows.append(row)
            return row
        expect = expect or {0}
        dest = self.out_dir / name
        dest.mkdir(parents=True, exist_ok=True)
        out_path = dest / "stdout.txt"
        err_path = dest / "stderr.txt"
        time_path = dest / "time.txt"
        print(f"\n== {name} ==", flush=True)
        print(f"  {' '.join(shlex.quote(a) for a in argv)}", flush=True)
        exit_code, real_s, rss_mib = run_cmd(
            argv,
            env=env or self.base_env,
            out_path=out_path,
            err_path=err_path,
            time_path=time_path,
            timeout=timeout,
        )
        note_parts: list[str] = []
        footer = footer_line(out_path)
        if exit_code == TIMEOUT_EXIT_CODE:
            note_parts.append("timeout/interrupted")
        if exit_code in SIGKILL_EXIT_CODES:
            note_parts.append("SIGKILL")
            self.stopped = True
            self.suite.stopped_reason = f"{name}: SIGKILL"
        if rss_mib is not None and rss_mib >= self.args.rss_limit_mib and phase == Phase.large:
            note_parts.append(f"rss {rss_mib:.1f} MiB >= limit")
            self.stopped = True
            self.suite.stopped_reason = f"{name}: RSS limit"
        out_bytes = out_path.stat().st_size if out_path.exists() else 0
        # --skill is large; others should print something.
        if (
            exit_code in expect
            and out_bytes == 0
            and name not in {SKILL_COMMAND_NAME}
            and not name.endswith(SKILL_COMMAND_NAME)
        ):
            note_parts.append("empty stdout")
        if footer:
            source = SOURCE_RE.search(footer)
            note_parts.append(source.group(1) if source else "")
            fresh = count_token(footer, "fresh")
            cached = count_token(footer, "cached")
            if fresh is not None:
                note_parts.append(f"fresh={fresh}")
            if cached is not None:
                note_parts.append(f"cached={cached}")
        note = "; ".join(p for p in note_parts if p)
        row = Row(
            phase=phase.value,
            name=name,
            argv=" ".join(argv),
            exit=exit_code,
            real_s=real_s,
            rss_mib=rss_mib,
            out_bytes=out_bytes,
            note=note,
            footer=footer,
        )
        if exit_code in {*SIGKILL_EXIT_CODES, TIMEOUT_EXIT_CODE} or exit_code not in expect:
            row.verdict = "fail"
        elif "empty stdout" in note:
            row.verdict = "warn"
        else:
            row.verdict = "ok"
        self.suite.rows.append(row)
        real_txt = f"{real_s:.3f}s" if real_s is not None else "?"
        rss_txt = f"{rss_mib:.1f}MiB" if rss_mib is not None else "?"
        print(
            f"  exit={exit_code} {real_txt} rss={rss_txt} {row.verdict} {note}",
            flush=True,
        )
        head = snippet(out_path)
        if head:
            print("  --- stdout ---", flush=True)
            for line in head.splitlines()[:STDOUT_ECHO_LINES]:
                print(f"  {line}", flush=True)
        err_head = snippet(err_path, head=STDERR_SNIPPET_HEAD_LINES, tail=STDERR_SNIPPET_TAIL_LINES)
        if err_head:
            print("  --- stderr ---", flush=True)
            for line in err_head.splitlines()[:STDERR_ECHO_LINES]:
                print(f"  {line}", flush=True)
        return row

    def record_version(self) -> None:
        proc = subprocess.run(
            [str(self.fdu), "--version"],
            check=False,
            capture_output=True,
            text=True,
        )
        version = (proc.stdout or proc.stderr).strip()
        self.suite.version = version
        print(f"binary: {self.fdu}", flush=True)
        print(f"version: {version}", flush=True)


def write_outputs(out_dir: Path, suite: Suite) -> None:
    """Publish the results table atomically so a mid-write death cannot empty it."""
    tsv_buf = io.StringIO()
    writer = csv.writer(tsv_buf, delimiter="\t", lineterminator="\n")
    writer.writerow(
        ["phase", "name", "verdict", "exit", "real_s", "rss_mib", "out_bytes", "note", "argv"]
    )
    for row in suite.rows:
        writer.writerow(
            [
                row.phase,
                row.name,
                row.verdict,
                row.exit,
                "" if row.real_s is None else row.real_s,
                "" if row.rss_mib is None else row.rss_mib,
                row.out_bytes,
                row.note,
                row.argv,
            ]
        )
    atomic_write_text(out_dir / "results.tsv", tsv_buf.getvalue())

    lines = [
        "| Phase | Name | Verdict | Exit | Real s | RSS MiB | Note |",
        "| --- | --- | --- | ---: | ---: | ---: | --- |",
    ]
    for row in suite.rows:
        real = f"{row.real_s:.3f}" if row.real_s is not None else ""
        rss = f"{row.rss_mib:.1f}" if row.rss_mib is not None else ""
        note = row.note.replace("|", "/")
        lines.append(
            f"| {row.phase} | {row.name} | {row.verdict} | {row.exit} | {real} | {rss} | {note} |"
        )
    atomic_write_text(out_dir / "results.md", "\n".join(lines) + "\n")
    payload = {
        "version": suite.version,
        "fdu": suite.fdu,
        "host": suite.host,
        "started": suite.started,
        "stopped_reason": suite.stopped_reason,
        "rows": [asdict(row) for row in suite.rows],
    }
    atomic_write_text(out_dir / "results.json", json.dumps(payload, indent=2) + "\n")


def phase_sanity(runner: Runner) -> None:
    runner.run(Phase.sanity, "help", runner.fdu_argv("--help"))
    runner.run(Phase.sanity, "version", runner.fdu_argv("--version"))
    runner.run(Phase.sanity, "docs", runner.fdu_argv("--docs"))
    runner.run(Phase.sanity, "skill", runner.fdu_argv("--skill"))


def phase_views(runner: Runner, tree: Path, cache_home: Path) -> None:
    env = cache_env(runner.base_env, cache_home)
    root = str(tree)
    pairs: list[tuple[str, list[str], set[int] | None]] = [
        ("tree-cold", [root], None),
        ("tree-warm", [root], None),
        ("summary", [root, "--view=summary"], None),
        ("languages", [root, "--view=languages"], None),
        ("families", [root, "--view=families"], None),
        ("types", [root, "--view=types"], None),
        ("extensions", [root, "--view=extensions"], None),
        ("documents-no-analyze", [root, "--view=documents"], {DOCUMENTS_WITHOUT_ANALYZE_EXIT}),
        ("recent", [root, "--view=recent", "--limit=10"], None),
        ("largest", [root, "--view=largest"], None),
        ("files", [root, "--view=files", "--limit=10"], None),
        ("full", [root, "--view=full"], None),
        ("combo-kinds", [root, "--view=families,types,extensions"], None),
        ("exclude-ignored-summary", [root, "--exclude-ignored", "--view=summary"], None),
        ("depth-limit-tree", [root, "--depth=1", "--limit=5"], None),
        ("scan-depth-1-summary", [root, "--scan-depth=1", "--view=summary"], None),
        ("json-summary", [root, "--view=summary", "--format=json"], None),
        ("yaml-summary", [root, "--view=summary", "--format=yaml"], None),
    ]
    for name, parts, expect in pairs:
        runner.run(
            Phase.views,
            name,
            runner.fdu_argv(*parts),
            env=env,
            timeout=VIEWS_TIMEOUT_S,
            expect=expect,
        )


def phase_cache_analyze(runner: Runner, tree: Path) -> Path:
    """Return the cache-on home so later analyze extras can reuse the warm sidecar."""
    root = str(tree)
    off_home = Path(tempfile.mkdtemp(prefix="fdu-qa-cache-off-"))
    on_home = Path(tempfile.mkdtemp(prefix="fdu-qa-cache-on-"))
    off_env = cache_env(runner.base_env, off_home)
    on_env = cache_env(runner.base_env, on_home)

    for analyzer, label in (("code", "code"), ("lines", "lines")):
        runner.run(
            Phase.cache_analyze,
            f"off-{label}-1",
            runner.fdu_argv(root, f"--analyze={analyzer}", "--cache=off"),
            env=off_env,
            timeout=CACHE_ANALYZE_TIMEOUT_S,
        )
        runner.run(
            Phase.cache_analyze,
            f"off-{label}-2",
            runner.fdu_argv(root, f"--analyze={analyzer}", "--cache=off"),
            env=off_env,
            timeout=CACHE_ANALYZE_TIMEOUT_S,
        )
    runner.run(
        Phase.cache_analyze,
        "off-cache-status",
        runner.fdu_argv(root, "--cache-status"),
        env=off_env,
        timeout=CACHE_STATUS_TIMEOUT_S,
    )

    for analyzer, label in (("code", "code"), ("lines", "lines")):
        runner.run(
            Phase.cache_analyze,
            f"on-{label}-1",
            runner.fdu_argv(root, f"--analyze={analyzer}", "--cache=auto"),
            env=on_env,
            timeout=CACHE_ANALYZE_TIMEOUT_S,
        )
        runner.run(
            Phase.cache_analyze,
            f"on-{label}-2",
            runner.fdu_argv(root, f"--analyze={analyzer}", "--cache=auto"),
            env=on_env,
            timeout=CACHE_ANALYZE_TIMEOUT_S,
        )
    runner.run(
        Phase.cache_analyze,
        "on-cache-status",
        runner.fdu_argv(root, "--cache-status"),
        env=on_env,
        timeout=CACHE_STATUS_TIMEOUT_S,
    )
    judge_cache_reuse(runner)
    return on_home


def judge_cache_reuse(runner: Runner) -> None:
    """Flag a cache-off leak or a cache-on second run that did not get cheaper."""
    by_name = {row.name: row for row in runner.suite.rows}

    def cached_count(name: str) -> int | None:
        row = by_name.get(name)
        if row is None:
            return None
        return count_token(row.footer, "cached")

    for label in ("code", "lines"):
        off2 = cached_count(f"off-{label}-2")
        on2 = cached_count(f"on-{label}-2")
        off_row = by_name.get(f"off-{label}-2")
        on_row = by_name.get(f"on-{label}-2")
        on1 = by_name.get(f"on-{label}-1")
        if off_row and off2 not in {None, 0}:
            off_row.verdict = "warn"
            off_row.note = f"{off_row.note}; cache-off second run reported cached={off2}"
        if on_row and on1 and on_row.real_s and on1.real_s:
            if on2 == 0:
                on_row.verdict = "warn"
                on_row.note = f"{on_row.note}; cache-on second run reported 0 cached"
            elif on_row.real_s >= on1.real_s * CACHE_REUSE_MAX_FRACTION_OF_FIRST:
                on_row.verdict = "warn"
                on_row.note = (
                    f"{on_row.note}; second run not materially faster "
                    f"({on_row.real_s:.3f}s vs {on1.real_s:.3f}s)"
                )


def phase_analyze_extra(runner: Runner, tree: Path, cache_home: Path) -> None:
    env = cache_env(runner.base_env, cache_home)
    root = str(tree)
    extras = [
        ("analyze-words", [root, "--analyze=words"]),
        ("analyze-all", [root, "--analyze=all"]),
        ("json-analyze-code", [root, "--analyze=code", "--view=languages", "--format=json"]),
        ("yaml-analyze-lines", [root, "--analyze=lines", "--view=summary", "--format=yaml"]),
        ("text-analyze-code-summary", [root, "--analyze=code", "--view=summary"]),
    ]
    for name, parts in extras:
        row = runner.run(
            Phase.analyze_extra,
            name,
            runner.fdu_argv(*parts),
            env=env,
            timeout=ANALYZE_EXTRA_TIMEOUT_S,
        )
        if name.startswith("json-") and row.exit == 0:
            check_json_analysis(runner.out_dir / name / "stdout.txt", row)


def check_json_analysis(path: Path, row: Row) -> None:
    """JSON `--analyze` must carry a filled `physical_lines` total, not a schema shell."""
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        row.verdict = "fail"
        row.note = f"{row.note}; invalid json".strip("; ")
        return
    analysis = payload.get("analysis") or {}
    total = analysis.get("total") or analysis
    physical = total.get("physical_lines")
    if physical == 0:
        row.verdict = "warn"
        row.note = f"{row.note}; physical_lines=0".strip("; ")


def phase_watch(runner: Runner) -> None:
    watch_root = Path(tempfile.mkdtemp(prefix="fdu-qa-watch-"))
    try:
        (watch_root / "seed.txt").write_text("seed\n", encoding="utf-8")
        cache_home = Path(tempfile.mkdtemp(prefix="fdu-qa-watch-cache-"))
        env = cache_env(runner.base_env, cache_home)
        dest = runner.out_dir / "watch"
        dest.mkdir(parents=True, exist_ok=True)
        out_path = dest / "stdout.txt"
        err_path = dest / "stderr.txt"
        argv = runner.fdu_argv(
            str(watch_root), "--watch", "--view=summary", f"--interval={WATCH_INTERVAL}"
        )
        print("\n== watch ==", flush=True)
        print(f"  {' '.join(shlex.quote(a) for a in argv)}", flush=True)
        with out_path.open("wb") as out, err_path.open("wb") as err:
            started = time.monotonic()
            proc = subprocess.Popen(argv, stdout=out, stderr=err, env=env)
            time.sleep(WATCH_SETTLE_BEFORE_CREATE_S)
            (watch_root / "probe.txt").write_text("probe\n", encoding="utf-8")
            time.sleep(WATCH_SETTLE_AFTER_CREATE_S)
            proc.send_signal(signal.SIGINT)
            try:
                proc.communicate(timeout=SIGINT_GRACE_S)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.communicate()
            elapsed = time.monotonic() - started
        exit_code = proc.returncode
        note = f"SIGINT after file create; elapsed={elapsed:.2f}s"
        verdict = "ok"
        if exit_code not in WATCH_OK_EXIT_CODES:
            # 130 = 128+SIGINT; -2 = SIGINT on some Pythons
            verdict = "fail"
            note += f"; unexpected exit {exit_code}"
        row = Row(
            phase=Phase.watch.value,
            name="watch-sigint",
            argv=" ".join(argv),
            exit=exit_code if exit_code is not None else -1,
            real_s=elapsed,
            rss_mib=None,
            out_bytes=out_path.stat().st_size,
            note=note,
            verdict=verdict,
        )
        runner.suite.rows.append(row)
        print(f"  exit={row.exit} {elapsed:.3f}s {verdict} {note}", flush=True)
    finally:
        for child in watch_root.iterdir():
            child.unlink(missing_ok=True)
        watch_root.rmdir()


def phase_medium(runner: Runner, tree: Path, analyze_root: Path | None) -> None:
    cache_home = Path(tempfile.mkdtemp(prefix="fdu-qa-medium-cache-"))
    env = cache_env(runner.base_env, cache_home)
    root = str(tree)
    timeout = runner.args.medium_timeout
    commands = [
        ("med-tree-cold", [root]),
        ("med-tree-warm", [root]),
        ("med-summary", [root, "--view=summary"]),
        ("med-languages", [root, "--view=languages"]),
        ("med-combo-kinds", [root, "--view=families,types,extensions"]),
        ("med-recent", [root, "--view=recent", "--limit=10"]),
        ("med-json-summary", [root, "--view=summary", "--format=json"]),
    ]
    for name, parts in commands:
        runner.run(Phase.medium, name, runner.fdu_argv(*parts), env=env, timeout=timeout)
    if analyze_root and analyze_root.is_dir():
        runner.run(
            Phase.medium,
            "med-analyze-code-subdir",
            runner.fdu_argv(str(analyze_root), "--analyze=code"),
            env=env,
            timeout=timeout,
        )
        runner.run(
            Phase.medium,
            "med-analyze-code-subdir-warm",
            runner.fdu_argv(str(analyze_root), "--analyze=code"),
            env=env,
            timeout=timeout,
        )


def phase_large(runner: Runner, tree: Path) -> None:
    cache_home = Path(tempfile.mkdtemp(prefix="fdu-qa-large-cache-"))
    env = cache_env(runner.base_env, cache_home)
    root = str(tree)
    timeout = runner.args.large_timeout
    # Exit 2 is a documented partial/TCC result.
    expect = set(LARGE_OK_EXIT_CODES)
    steps = [
        ("large-summary-depth1", [root, "--view=summary", "--scan-depth=1", "--limit=20"]),
        ("large-summary-depth2", [root, "--view=summary", "--scan-depth=2", "--limit=20"]),
    ]
    for name, parts in steps:
        runner.run(
            Phase.large,
            name,
            runner.fdu_argv(*parts),
            env=env,
            timeout=timeout,
            expect=expect,
        )
        if runner.stopped:
            return
    for leaf in ("Preferences", "Logs"):
        sub = tree / leaf
        if not sub.is_dir():
            continue
        runner.run(
            Phase.large,
            f"large-tree-{leaf.lower()}",
            runner.fdu_argv(str(sub), "--view=tree", "--scan-depth=2", "--depth=1", "--limit=10"),
            env=env,
            timeout=timeout,
            expect=expect,
        )
        if runner.stopped:
            return


def main() -> int:
    args = parse_args()
    fdu = resolve_fdu(args.fdu)
    small_phases = {Phase.views, Phase.cache_analyze, Phase.analyze_extra}
    if args.phases & small_phases and args.small is None:
        sys.exit("FDU_QA_SMALL / --small is required for the small-tree phases")
    out_dir = (
        args.out_dir if args.out_dir is not None else Path(tempfile.mkdtemp(prefix="fdu-qa-out-"))
    )
    out_dir.mkdir(parents=True, exist_ok=True)
    print(f"out-dir: {out_dir}", flush=True)
    runner = Runner(args, out_dir, fdu)
    runner.record_version()

    if Phase.sanity in args.phases:
        phase_sanity(runner)
    small = args.small.expanduser().resolve() if args.small is not None else None
    views_cache: Path | None = None
    if small is not None and Phase.views in args.phases:
        views_home = Path(tempfile.mkdtemp(prefix="fdu-qa-views-cache-"))
        views_cache = views_home
        phase_views(runner, small, views_home)
    on_home: Path | None = None
    if small is not None and Phase.cache_analyze in args.phases:
        on_home = phase_cache_analyze(runner, small)
    extra_home = on_home or views_cache
    if small is not None and Phase.analyze_extra in args.phases:
        if extra_home is None:
            extra_home = Path(tempfile.mkdtemp(prefix="fdu-qa-extra-cache-"))
        phase_analyze_extra(runner, small, extra_home)
    if Phase.watch in args.phases:
        phase_watch(runner)
    if Phase.medium in args.phases and args.medium is not None:
        medium = args.medium.expanduser().resolve()
        analyze = (
            args.medium_analyze.expanduser().resolve()
            if args.medium_analyze is not None
            else medium / "docs"
        )
        if not analyze.is_dir():
            analyze = None
        phase_medium(runner, medium, analyze)
    elif Phase.medium in args.phases:
        print("skip medium: FDU_QA_MEDIUM / --medium not set", flush=True)
    if Phase.large in args.phases and args.large is not None:
        phase_large(runner, args.large.expanduser().resolve())
    elif Phase.large in args.phases:
        print("skip large: FDU_QA_LARGE / --large not set", flush=True)

    write_outputs(out_dir, runner.suite)
    print(f"\nWrote {out_dir / 'results.md'}", flush=True)
    print(f"{out_dir / 'results.tsv'}", flush=True)
    fails = sum(1 for row in runner.suite.rows if row.verdict == "fail")
    warns = sum(1 for row in runner.suite.rows if row.verdict == "warn")
    print(f"verdicts: fail={fails} warn={warns} total={len(runner.suite.rows)}", flush=True)
    if runner.suite.stopped_reason:
        print(f"stopped: {runner.suite.stopped_reason}", flush=True)
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())
