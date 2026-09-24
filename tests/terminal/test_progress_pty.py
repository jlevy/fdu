"""The progress indicator in a real pseudo-terminal.

Unit tests pin the frames and the gating with injected terminal facts; this runs the
built binary where a person would, and checks what only a terminal shows: a frame is
drawn, the line is erased before the report, Ctrl-C erases it and the process dies by
the signal, and nothing is drawn when stderr is not a terminal.

The indicator waits 500 ms before its first frame, so the tree must take longer than
that to answer. Content analysis over many small files does on every CI runner seen so
far; if a run still finishes inside the delay, the tree doubles and the run repeats.
"""

from __future__ import annotations

import os
import select
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FDU = os.environ.get("FDU_BIN") or str(REPO / "target" / "debug" / "fdu")
ERASE = b"\r\x1b[2K"
SPINNER_LEAD = "⠋".encode()[:2]  # every braille spinner cell starts with these bytes
INTERRUPTED = b"fdu: interrupted"
TIMEOUT_S = 120.0


def environment(cache: Path) -> dict[str, str]:
    env = dict(os.environ, TERM="xterm-256color", NO_COLOR="1", XDG_CACHE_HOME=str(cache))
    env.pop("CI", None)
    return env


def grow_tree(root: Path, directories: int, files: int) -> None:
    """Add `directories` directories of `files` small text files for the analyzer to read."""
    start = sum(1 for _ in root.iterdir())
    for d in range(start, start + directories):
        directory = root / f"d{d:05}"
        directory.mkdir()
        for f in range(files):
            with open(directory / f"f{f:04}.txt", "w", encoding="utf-8") as handle:
                handle.write("one line of words for the analyzer\n")


def run_in_pty(args: list[str], env: dict[str, str], interrupt_after_frame: bool = False):
    """Run fdu with a pseudo-terminal as stdout and stderr; return (output, wait status)."""
    import pty

    pid, fd = pty.fork()
    if pid == 0:  # the child: fdu, with the terminal as its controlling terminal
        try:
            os.execve(FDU, [FDU, *args], env)
        finally:
            os._exit(127)
    output = bytearray()
    deadline = time.monotonic() + TIMEOUT_S
    interrupted = False
    while time.monotonic() < deadline:
        ready, _, _ = select.select([fd], [], [], 0.05)
        if ready:
            try:
                chunk = os.read(fd, 65536)
            except OSError:  # EIO once the child has exited and the terminal is gone
                break
            if not chunk:
                break
            output += chunk
        if interrupt_after_frame and not interrupted and ERASE + SPINNER_LEAD in output:
            os.kill(pid, signal.SIGINT)
            interrupted = True
    else:
        os.kill(pid, signal.SIGKILL)
        raise AssertionError(f"fdu did not finish within {TIMEOUT_S} s")
    _, status = os.waitpid(pid, 0)
    os.close(fd)
    return bytes(output), status


@unittest.skipIf(sys.platform == "win32", "Python has no pty on Windows")
class ProgressInATerminal(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        if not Path(FDU).is_file():
            raise unittest.SkipTest(f"no fdu binary at {FDU}; run `make build` first")
        cls._tmp = tempfile.TemporaryDirectory(prefix="fdu-progress-pty-")
        base = Path(cls._tmp.name)
        cls.tree = base / "tree"
        cls.tree.mkdir()
        cls.env = environment(base / "cache")
        cls.args = ["--cache", "off", "--analyze", "all", str(cls.tree)]
        # Grow the tree until one run draws a frame, so a fast runner still exercises
        # the indicator rather than passing on a run that never drew.
        directories = 40
        for _ in range(5):
            grow_tree(cls.tree, directories, 250)
            cls.output, cls.status = run_in_pty(cls.args, cls.env)
            if ERASE + SPINNER_LEAD in cls.output:
                return
            directories *= 2
        raise AssertionError("no tree finished slowly enough to draw a frame")

    @classmethod
    def tearDownClass(cls) -> None:
        cls._tmp.cleanup()

    def test_a_frame_is_drawn_and_erased_before_the_report(self) -> None:
        self.assertTrue(os.WIFEXITED(self.status), self.status)
        self.assertEqual(os.WEXITSTATUS(self.status), 0, self.output[-400:])
        first_frame = self.output.index(ERASE + SPINNER_LEAD)
        report = self.output.index(b"Performance:")
        last_erase = self.output.rindex(ERASE)
        self.assertLess(first_frame, last_erase)
        self.assertLess(last_erase, report, "the line is erased before the report")
        # Nothing but the erase itself sits between the last erase and the report's
        # first line: no frame was left behind for the report to overwrite.
        self.assertNotIn(SPINNER_LEAD, self.output[last_erase + len(ERASE) : report])
        self.assertIn(b"Analyzing", self.output[:last_erase], "the analysis phase was shown")

    def test_ctrl_c_erases_the_line_and_dies_by_the_signal(self) -> None:
        output, status = run_in_pty(self.args, self.env, interrupt_after_frame=True)
        self.assertTrue(os.WIFSIGNALED(status), f"status {status}: {output[-400:]!r}")
        self.assertEqual(os.WTERMSIG(status), signal.SIGINT)
        message = output.rindex(INTERRUPTED)
        self.assertLess(output.rindex(ERASE), message, "erased before the message")
        self.assertNotIn(SPINNER_LEAD, output[message:], "no frame after the message")

    def test_nothing_is_drawn_when_stderr_is_not_a_terminal(self) -> None:
        run = subprocess.run(
            [FDU, *self.args],
            env=self.env,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            timeout=TIMEOUT_S,
            check=False,
        )
        self.assertEqual(run.returncode, 0, run.stderr[-400:])
        self.assertNotIn(ERASE, run.stderr)
        self.assertNotIn(SPINNER_LEAD, run.stderr)


if __name__ == "__main__":
    unittest.main()
