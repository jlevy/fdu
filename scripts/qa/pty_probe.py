#!/usr/bin/env python3
"""Run the checks of the QA playbook's Phase 6 that a pseudo-terminal can judge.

    python3 scripts/qa/pty_probe.py --fdu "$(command -v fdu)" --tree SLOW --small SMALL \\
        --analyze-tree ANALYZE

The progress indicator is drawn only on a terminal, so each check runs the installed fdu
with a pseudo-terminal as its stderr (and, for most, its stdout) and reads back what was
drawn: frames that appear after about half a second, fit the terminal's width, climb,
hold their columns, and are erased before the report; nothing when stderr is a file,
stdout is JSON, `--progress never`, `CI=1`, or `TERM=dumb`; no color under `NO_COLOR`
but the animation kept; an erased line, `fdu: interrupted`, and death by SIGINT on
Ctrl-C; and frames that shrink when the terminal narrows mid-run.

What it cannot judge is what a person sees in a real window, and Windows, which has no
pty here: Phase 6 stays pending until someone watches one. `make test-terminal` covers
drawing, erasing, and Ctrl-C against a small fixture; this probe covers the rest of the
phase against real trees.

`--tree` must take well over half a second to scan, metadata only, or no frame is drawn;
`--analyze-tree` must take several seconds under `--analyze all`, and is the long run for
the width, narrowing, and Ctrl-C checks; `--small` must scan in well under half a second.
Each defaults to the playbook's variable (`FDU_QA_PROGRESS_TREE`,
`FDU_QA_PROGRESS_ANALYZE`, `FDU_QA_SMALL`), and the probe times the two slow trees first
and stops with a message if either is too fast, rather than failing every check.

Exit status: 0 when every check passed, 1 when any failed, 2 when the trees or the
binary cannot support the checks. Unix only; stdlib only, like the other QA scripts.
"""

from __future__ import annotations

import argparse
import fcntl
import os
import re
import select
import shutil
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import time
import unicodedata
from collections.abc import Sequence
from dataclasses import dataclass, field

ERASE = b"\r\x1b[2K"
SPIN = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"
# Every spinner glyph is a Braille pattern, and all share their first two UTF-8 bytes.
SPIN_LEAD = "⠋".encode()[:2]
SGR = re.compile(rb"\x1b\[[0-9;]*m")
RESIZED = b"<<RESIZED>>"
# The slow tree must take longer than this, warm, for the first frame to be judged; the
# analyze tree longer than the second, for the width and narrowing checks to see frames.
MIN_SLOW_SECONDS = 1.0
MIN_ANALYZE_SECONDS = 2.0
# The small tree must finish before any frame can be drawn.
MAX_SMALL_SECONDS = 0.3


@dataclass
class Run:
    """What one fdu run drew on the terminal, wrote to a redirected stream, and exited with."""

    out: bytes
    redirected: bytes | None
    status: int
    first_frame: float | None


@dataclass
class Probe:
    fdu: str
    cache: str
    results: list[tuple[bool, str]] = field(default_factory=list)

    def check(self, ok: object, what: str) -> None:
        self.results.append((bool(ok), what))
        print(f"{'ok  ' if ok else 'FAIL'} {what}", flush=True)

    def env(self, **extra: str | None) -> dict[str, str]:
        env = dict(os.environ, TERM="xterm-256color", XDG_CACHE_HOME=self.cache)
        for key in ("CI", "FDU_CACHE_DIR", "NO_COLOR"):
            env.pop(key, None)
        for key, value in extra.items():
            if value is None:
                env.pop(key, None)
            else:
                env[key] = value
        return env

    def run(
        self,
        args: Sequence[str],
        env: dict[str, str],
        cols: int = 100,
        stdout: str = "pty",
        stderr: str = "pty",
        interrupt_after_frame: bool = False,
        resize: Sequence[tuple[int, int]] = (),
        timeout: float = 180.0,
    ) -> Run:
        """Run fdu on a pty; `stdout` and `stderr` are "pty", "null", or a file path.

        `resize` lists (frames seen, columns): after that many frames the terminal narrows,
        and a marker is recorded in the output where it did."""
        master, slave = os.openpty()
        set_size(master, cols)
        pid = os.fork()
        if pid == 0:
            os.setsid()
            fcntl.ioctl(slave, termios.TIOCSCTTY, 0)
            os.dup2(slave, 0)
            for fd, how in ((1, stdout), (2, stderr)):
                if how == "pty":
                    os.dup2(slave, fd)
                    continue
                path = os.devnull if how == "null" else how
                os.dup2(os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC), fd)
            os.close(master)
            try:
                os.execve(self.fdu, [self.fdu, *args], env)
            finally:
                os._exit(127)
        os.close(slave)
        out = bytearray()
        started = time.monotonic()
        first_frame = None
        interrupted = False
        pending = list(resize)
        deadline = started + timeout
        while True:
            if time.monotonic() >= deadline:
                os.kill(pid, signal.SIGKILL)
                os.waitpid(pid, 0)
                raise SystemExit(f"timed out after {timeout:.0f} s: fdu {' '.join(args)}")
            ready, _, _ = select.select([master], [], [], 0.02)
            if ready:
                try:
                    chunk = os.read(master, 65536)
                except OSError:
                    break
                if not chunk:
                    break
                out += chunk
            drawn = SGR.sub(b"", bytes(out)).count(ERASE + SPIN_LEAD)
            if drawn and first_frame is None:
                first_frame = time.monotonic() - started
            while pending and drawn >= pending[0][0]:
                set_size(master, pending.pop(0)[1])
                out += RESIZED
            if interrupt_after_frame and drawn and not interrupted:
                os.kill(pid, signal.SIGINT)
                interrupted = True
        _, status = os.waitpid(pid, 0)
        os.close(master)
        redirected = None
        for how in (stdout, stderr):
            if how not in ("pty", "null"):
                with open(how, "rb") as captured:
                    redirected = captured.read()
        return Run(bytes(out), redirected, status, first_frame)


def set_size(fd: int, cols: int, rows: int = 40) -> None:
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))


def width(text: str) -> int:
    """Terminal columns: wide East Asian characters take two, combining marks none."""
    return sum(
        0 if unicodedata.combining(c) else 2 if unicodedata.east_asian_width(c) in "WF" else 1
        for c in text
    )


def frames(out: bytes) -> list[str]:
    """Every drawn frame's visible text, in order; a resize marker ends the frame it is in."""
    found = []
    for piece in SGR.sub(b"", out).split(ERASE)[1:]:
        if piece.startswith(SPIN_LEAD):
            text = piece.split(RESIZED)[0].decode("utf-8", "replace")
            found.append(text)
    return found


def after_last_frame_is_erased(out: bytes) -> bool:
    """The last frame is followed by an erase before anything else is printed."""
    out = SGR.sub(b"", out)
    last = out.rfind(ERASE + SPIN_LEAD)
    if last < 0:
        return False
    rest = out[last + len(ERASE) :]
    following = rest.find(ERASE)
    if following < 0:
        return False
    # Between the frame and its erase: only frame text, no newline (no report line).
    return b"\n" not in rest[:following]


def exit_code(status: int) -> int | None:
    return os.WEXITSTATUS(status) if os.WIFEXITED(status) else None


def minimal(frame: str) -> bool:
    """Below 20 columns only the spinner and the phase word remain."""
    return re.fullmatch(rf"[{SPIN}] [A-Z][a-z]+", frame) is not None


def timed(fdu: str, args: Sequence[str]) -> tuple[float, int]:
    """Wall time and exit status of a run with no terminal, after one run to warm caches."""
    command = [fdu, "--progress", "never", *args]
    subprocess.run(command, capture_output=True, check=False)
    started = time.monotonic()
    result = subprocess.run(command, capture_output=True, check=False)
    return time.monotonic() - started, result.returncode


def preflight(fdu: str, slow: str, small: str, analyze: str) -> list[str]:
    """Why these trees cannot support the checks, if they cannot: each is timed warm."""
    problems = []
    for label, args, least in (
        ("--tree", ["--cache", "off", slow], MIN_SLOW_SECONDS),
        ("--analyze-tree", ["--cache", "off", "--analyze", "all", analyze], MIN_ANALYZE_SECONDS),
    ):
        seconds, status = timed(fdu, args)
        if status != 0:
            problems.append(f"{label}: fdu {' '.join(args)} exited {status}")
        elif seconds < least:
            problems.append(
                f"{label} took {seconds:.2f} s, and needs more than {least:.1f} s for frames "
                "to be drawn and judged: choose a larger tree"
            )
    seconds, status = timed(fdu, ["--cache", "off", small])
    if status != 0 or seconds > MAX_SMALL_SECONDS:
        problems.append(
            f"--small took {seconds:.2f} s (exit {status}), and must finish in under "
            f"{MAX_SMALL_SECONDS:.1f} s so that no frame is drawn: choose a smaller tree"
        )
    return problems


def probe(p: Probe, slow: str, small: str, analyze: str) -> None:
    long_run = ["--cache", "off", "--analyze", "all", analyze]

    # 1. The slow metadata scan at 100 columns.
    started = time.monotonic()
    run = p.run(["--cache", "off", slow], p.env())
    elapsed = time.monotonic() - started
    fs = frames(run.out)
    first = run.first_frame
    p.check(exit_code(run.status) == 0, f"slow scan exits 0 ({elapsed:.1f} s)")
    p.check(len(fs) > 0, f"slow scan draws frames ({len(fs)} frames)")
    p.check(
        first is not None and first >= 0.45,
        f"first frame after about half a second ({first and round(first, 2)} s)",
    )
    p.check(any("Scanning" in f for f in fs), "a frame says Scanning")
    words = ("Scanning", "Indexing", "Summarizing", "Loading", "Saving")
    print(f"     phases seen: {sorted({w for f in fs for w in words if w in f})}")
    p.check(after_last_frame_is_erased(run.out), "the last frame is erased before the report")
    plain = SGR.sub(b"", run.out)
    report = plain[plain.rfind(ERASE) + len(ERASE) :]
    p.check(
        SPIN_LEAD not in report and len(report.strip()) > 0,
        "the report follows the erase, with no frame left in it",
    )
    p.check(all(width(f) <= 99 for f in fs), "every frame fits 100 columns (at most 99)")
    p.check(all("\n" not in f for f in fs), "no frame contains a newline")
    counts = [re.search(r"([\d,]+) files", f) for f in fs]
    seq = [int(m.group(1).replace(",", "")) for m in counts if m]
    p.check(
        seq == sorted(seq) and len(seq) > 1 and seq[-1] > seq[0],
        f"file counts climb ({seq[:1]} to {seq[-1:]})",
    )
    starts = {f.find("files") for f in fs if "Scanning" in f}
    p.check(len(starts) == 1, f"the counts hold their column while Scanning ({sorted(starts)})")
    print("     sample frame:", fs[len(fs) // 2] if fs else None)

    # 2. A small tree draws nothing.
    run = p.run(["--cache", "off", small], p.env())
    p.check(exit_code(run.status) == 0 and ERASE not in run.out, "a small tree shows no indicator")

    # 3. Stderr redirected to a file: no progress written there.
    errfile = os.path.join(p.cache, "stderr.txt")
    run = p.run(["--cache", "off", slow], p.env(), stderr=errfile)
    err = run.redirected or b""
    lines = [line for line in err.decode("utf-8", "replace").splitlines() if line.strip()]
    bad = [line for line in lines if not re.match(r"^(note|warn|tip|perf):", line)]
    p.check(
        exit_code(run.status) == 0 and b"\r" not in err and b"\x1b" not in err,
        "redirected stderr holds no carriage return or escape",
    )
    p.check(
        not bad,
        f"redirected stderr holds only note/warn/tip/perf lines ({len(lines)} lines"
        f"{', bad: ' + repr(bad[:2]) if bad else ''})",
    )
    p.check(ERASE not in run.out, "nothing drawn on the terminal when stderr is a file")

    # 4. JSON to /dev/null draws nothing; --progress always draws; never draws nothing.
    run = p.run(["--cache", "off", "--format", "json", slow], p.env(), stdout="null")
    p.check(exit_code(run.status) == 0 and ERASE not in run.out, "--format json shows nothing")
    always = ["--cache", "off", "--progress", "always", "--format", "json", slow]
    run = p.run(always, p.env(), stdout="null")
    p.check(
        exit_code(run.status) == 0 and frames(run.out) and after_last_frame_is_erased(run.out),
        "--progress always --format json draws and erases",
    )
    for fmt in ("text", "json", "yaml"):
        extra = [] if fmt == "text" else ["--format", fmt]
        never = ["--cache", "off", "--progress", "never", *extra, slow]
        run = p.run(never, p.env(), stdout="null")
        p.check(
            exit_code(run.status) == 0 and ERASE not in run.out,
            f"--progress never shows nothing ({fmt})",
        )

    # 5. CI=1 and TERM=dumb draw nothing.
    run = p.run(["--cache", "off", slow], p.env(CI="1"))
    p.check(exit_code(run.status) == 0 and ERASE not in run.out, "CI=1 shows nothing")
    run = p.run(["--cache", "off", slow], p.env(TERM="dumb"))
    p.check(exit_code(run.status) == 0 and ERASE not in run.out, "TERM=dumb shows nothing")

    # 6. NO_COLOR keeps the animation, without color.
    run = p.run(["--cache", "off", slow], p.env(NO_COLOR="1"))
    raw = [piece for piece in run.out.split(ERASE)[1:] if piece.startswith(SPIN_LEAD)]
    p.check(exit_code(run.status) == 0 and len(raw) > 0, "NO_COLOR=1 keeps the animation")
    p.check(
        all(not SGR.search(piece.split(b"\r\n")[0]) for piece in raw),
        "NO_COLOR=1 frames carry no color",
    )
    run = p.run(["--cache", "off", slow], p.env())
    raw = [piece for piece in run.out.split(ERASE)[1:] if SGR.sub(b"", piece).startswith(SPIN_LEAD)]
    p.check(
        len(raw) > 0 and all(SGR.search(piece) for piece in raw),
        "without NO_COLOR the frames are colored",
    )

    # 7. Ctrl-C erases the line, prints fdu: interrupted, and dies by SIGINT.
    run = p.run(long_run, p.env(), interrupt_after_frame=True)
    p.check(
        os.WIFSIGNALED(run.status) and os.WTERMSIG(run.status) == signal.SIGINT,
        f"Ctrl-C: dies by SIGINT (status {run.status}; shell $? would be 130)",
    )
    out = SGR.sub(b"", run.out)
    at = out.find(b"fdu: interrupted")
    p.check(at >= 0, "Ctrl-C: prints fdu: interrupted")
    p.check(
        at >= 0 and out.rfind(ERASE, 0, at) > out.rfind(ERASE + SPIN_LEAD, 0, at),
        "Ctrl-C: the line is erased before the message",
    )
    p.check(SPIN_LEAD not in out[at:], "Ctrl-C: no frame after the message")

    # 8. Widths: every frame fits, and below 20 columns only the spinner and phase word.
    for cols in (60, 40, 30, 20, 19, 12):
        run = p.run(long_run, p.env(), cols=cols)
        fs = frames(run.out)
        fits = bool(fs) and all(width(f) <= cols - 1 and "\n" not in f for f in fs)
        widest = max(fs, key=width) if fs else None
        if cols < 20:
            p.check(
                fits and all(minimal(f) for f in fs),
                f"{cols} columns: {len(fs)} frames fit, spinner and phase word only ({widest!r})",
            )
        else:
            p.check(
                fits,
                f"{cols} columns: {len(fs)} frames fit, widest {width(widest or '')} ({widest!r})",
            )

    # 9. Narrowing mid-run.
    run = p.run(long_run, p.env(), cols=100, resize=[(5, 45), (15, 16)])
    segments = run.out.split(RESIZED)
    ok = len(segments) == 3
    for segment, cols in zip(segments[1:], (45, 16), strict=False):
        # Frames drawn after the resize; the first may predate the new size by one tick.
        fs = frames(segment)[1:]
        ok = ok and all(width(f) <= cols - 1 for f in fs)
        if cols < 20:
            ok = ok and all(minimal(f) for f in fs)
    p.check(
        ok and exit_code(run.status) == 0,
        "narrowing mid-run shrinks every later frame without wrapping",
    )

    # 10. Analyzing shows a climbing percentage.
    run = p.run(long_run, p.env())
    fs = frames(run.out)
    pct = [int(m.group(1)) for f in fs if "Analyzing" in f for m in [re.search(r"(\d+)%", f)] if m]
    p.check(
        exit_code(run.status) == 0 and len(pct) > 0,
        f"--analyze all shows Analyzing with a percentage ({pct[:1]} to {pct[-1:]})",
    )
    p.check(pct == sorted(pct), "the percentage climbs")
    p.check(
        after_last_frame_is_erased(run.out), "the analyze indicator is erased before the report"
    )


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=(__doc__ or "").split("\n\n")[0])
    env = os.environ.get
    result.add_argument("--fdu", default=env("FDU", "fdu"), help="the binary under test")
    result.add_argument(
        "--tree",
        default=env("FDU_QA_PROGRESS_TREE"),
        help="a tree whose metadata scan takes well over half a second",
    )
    result.add_argument(
        "--small", default=env("FDU_QA_SMALL"), help="a tree that scans too fast to draw"
    )
    result.add_argument(
        "--analyze-tree",
        default=env("FDU_QA_PROGRESS_ANALYZE"),
        help="a tree that takes several seconds under --analyze all",
    )
    return result


def main(argv: Sequence[str] | None = None) -> int:
    args = parser().parse_args(argv)
    given = {"--tree": args.tree, "--small": args.small, "--analyze-tree": args.analyze_tree}
    missing = [flag for flag, value in given.items() if not value]
    if missing:
        print(f"error: {', '.join(missing)} not given, nor set in the environment", file=sys.stderr)
        return 2
    fdu = shutil.which(args.fdu) or args.fdu
    if not os.access(fdu, os.X_OK):
        print(f"error: fdu binary not found: {args.fdu}", file=sys.stderr)
        return 2
    print(f"fdu: {fdu}\nslow tree: {args.tree}\nsmall tree: {args.small}")
    print(f"analyze tree: {args.analyze_tree}\n", flush=True)
    problems = preflight(fdu, args.tree, args.small, args.analyze_tree)
    if problems:
        for problem in problems:
            print(f"error: {problem}", file=sys.stderr)
        return 2
    cache = tempfile.mkdtemp(prefix="fdu-pty-cache-")
    p = Probe(fdu, cache)
    try:
        probe(p, args.tree, args.small, args.analyze_tree)
    finally:
        shutil.rmtree(cache, ignore_errors=True)
    passed = sum(ok for ok, _ in p.results)
    print(f"\n{passed} of {len(p.results)} checks passed")
    return 0 if passed == len(p.results) else 1


if __name__ == "__main__":
    sys.exit(main())
