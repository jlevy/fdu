#!/usr/bin/env python3
"""
Run a release's stability pass on its commit, from a clean worktree to a written report.

Step 2 of the [release checklist](../../docs/project/guides/release-process.md#stability-pass)
as one command. With the checklist's `COMMIT` and `RELEASE` set, and the QA playbook's
`FDU_QA_*` variables naming the trees:

    make release-stability
    make release-stability ARGS="--only qa"     # one stage again, after a host failure

It creates a worktree of the commit with its own Cargo target directory and runs the four
gates there; builds the candidate wheel from it, installs it into a uv tool directory of
its own, and requires `fdu --version` to name the commit; runs the installed-CLI QA
harness, peer agreement (its self-test, then the real trees), the Phase 6 pty probe, and
the terminal tests against that candidate; builds the correctness runbook's two trees and
runs its three passes, then both deliberate breaks, each of which must make its script
exit 1; and writes a dated report and the two summary tables the QA playbook and the
correctness runbook record, with every private path replaced by a label.

Every path comes from arguments or the environment. Each step's log, and `state.json`,
which records every result, are under the work directory (default `$RELEASE/stability`),
so `--only` can rerun a stage or a step and the report still covers the whole pass. A
work directory belongs to one commit, and is outside any checkout.

Prerequisites are checked before anything runs, each with one message: GNU time at
`/usr/bin/time`, uv, the reviewed `cargo-semver-checks`, every rustup target
`make cross-lint` checks, `FDU_QA_SMALL`, and free space. A missing peer tool, or, when
running as root, a missing `setpriv` or `nobody` account, is reported and the part that
needs it is marked skipped, never passed. So is a step that ran and left part of itself
out: a harness without its medium or large tree, or one that stopped at its memory
limit, and a cross-lint that skipped a target.

Exit status, which describes the whole recorded pass and not only the steps this run
ran: 0 when every step has passed, 1 when one has failed, and 3 when none has failed but
one was skipped or has not run. So a rerun with `--only` exits 0 only once the pass is
whole. 2 means a prerequisite or an argument is wrong and nothing ran. Unless a step in
the record has failed, the worktree and the target directory the pass made are removed;
a `--target-dir` is the caller's and is kept, as are the candidate, the logs, and the
report.
"""

from __future__ import annotations

import argparse
import contextlib
import dataclasses
import hashlib
import json
import os
import platform
import pwd
import re
import shlex
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import threading
import time
import tomllib
import traceback
from collections import Counter
from collections.abc import Callable, Iterable, Mapping, Sequence
from dataclasses import dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

if __package__ in (None, ""):
    # Run as a script: make the repository root importable, as the tests have it.
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from scripts.atomic_write import write_text_atomic
from scripts.release import semver_check

ROOT = Path(__file__).resolve().parents[2]

GATES = ("check", "cross-lint", "semver-check", "release-rehearse")
STAGES: dict[str, tuple[str, ...]] = {
    "gates": GATES,
    "candidate": ("candidate",),
    "qa": ("harness", "peer-self-test", "peer-trees", "pty-probe", "terminal"),
    "correctness": (
        "refusals",
        "served",
        "cross-warm",
        "break-no-snapshot",
        "break-partial-stored",
    ),
}
STEPS = tuple(step for steps in STAGES.values() for step in steps)
# Names `--only` also accepts. `report` selects no step: the report is written after any run.
ALIASES: dict[str, tuple[str, ...]] = {
    "peers": ("peer-self-test", "peer-trees"),
    "breaks": ("break-no-snapshot", "break-partial-stored"),
    "report": (),
}
NEEDS_CANDIDATE = frozenset(STAGES["qa"] + STAGES["correctness"])
# The refusal tree's unreadable entries refuse only a process that mode bits bind.
AS_NOBODY = frozenset({"refusals", "break-partial-stored"})
PEERS = ("dust", "pdu", "dua", "diskus")
GNU_TIME = "/usr/bin/time"
NOBODY = "nobody"
EXIT_FAILED, EXIT_USAGE, EXIT_INCOMPLETE = 1, 2, 3
GB = 10**9
# An atomically rewritten log shows a long step's output so far this often.
LOG_REFRESH_SECONDS = 15.0
# Host preconditions the Makefile reads, and the interpreter pin; the report states each.
DECLARED = ("FDU_TEST_ALLOW_NO_PERMISSION_BITS", "FDU_TEST_ALLOW_NO_NATIVE_WATCH", "UV_PYTHON")
# The harness phases that run only when their tree is named, and the variable that names it.
HARNESS_TREE_PHASES = {"medium": "FDU_QA_MEDIUM", "large": "FDU_QA_LARGE"}

# The QA playbook's variables, and the tree each names.
TREE_ROLES = {
    "small": "a small typed tree for the harness, such as this repository's checkout",
    "medium": "a larger working tree for the harness",
    "medium_analyze": "the medium tree's subtree for --analyze=code",
    "large": "a hostile wide tree, bounded, such as ~/Library",
    "progress": "for the pty probe, a tree whose scan takes well over half a second "
    "(else the medium tree)",
    "progress_analyze": "for the pty probe, a tree that takes seconds under --analyze all "
    "(else the medium tree's subtree, or the medium tree)",
}
TREE_VARIABLES = {
    "small": "FDU_QA_SMALL",
    "medium": "FDU_QA_MEDIUM",
    "medium_analyze": "FDU_QA_MEDIUM_ANALYZE",
    "large": "FDU_QA_LARGE",
    "progress": "FDU_QA_PROGRESS_TREE",
    "progress_analyze": "FDU_QA_PROGRESS_ANALYZE",
}
PEER_TREES_VARIABLE = "FDU_QA_PEER_TREES"

# Where fixture trees are left as they are in a report: nobody's name is in them.
SYSTEM_ROOTS = (
    "/usr",
    "/Applications",
    "/System",
    "/Library",
    "/opt/homebrew",
    "/bin",
    "/sbin",
    "/lib",
    "/etc",
    "/nix/store",
)
HOME_DIRECTORY = re.compile(r"^/(?:home|Users)/[^/]+$")
BOUNDARY = r"(?=$|[/\s`'\"|),:;\]])"
PEER_SCRATCH = re.compile(r"[^\s`'\"]*fdu-peer-agreement-[A-Za-z0-9_]+")
ANY_HOME = re.compile(r"/(?:home|Users)/[^/\s`'\"|),:;\]]+")
ROOT_HOME = re.compile(r"/root" + BOUNDARY)
TOOL_ROW = re.compile(
    r"^\| (?:GNU du -l|GNU du|BSD du|dust|pdu|dua|diskus) \| (?:allocated|apparent) \|"
)


class UsageError(Exception):
    """An argument or the work directory is wrong; nothing ran."""


class StepError(Exception):
    """A step could not run to a verdict; the message says why."""

    def __init__(self, message: str, logs: Sequence[Path] = ()) -> None:
        super().__init__(message)
        self.logs = list(logs)


@dataclass(frozen=True)
class Trees:
    small: Path | None = None
    medium: Path | None = None
    medium_analyze: Path | None = None
    large: Path | None = None
    progress: Path | None = None
    progress_analyze: Path | None = None
    peers: tuple[Path, ...] = ()

    def named(self) -> dict[str, Path]:
        """Every tree that is set, by the name the report gives it."""
        found = {
            name: value for name in TREE_VARIABLES if isinstance(value := getattr(self, name), Path)
        }
        found.update({f"peer {index + 1}": path for index, path in enumerate(self.peers)})
        return found


@dataclass(frozen=True)
class Config:
    commit: str
    work: Path
    trees: Trees
    steps: tuple[str, ...]
    root: Path = ROOT
    target: Path | None = None
    python: str = "3.12"
    wrap: tuple[str, ...] = ()
    wheels: Path | None = None
    keep: bool = False
    min_free_gb: float = 12.0
    labels: tuple[tuple[str, str], ...] = ()
    date: str = ""
    timeout: float | None = None


@dataclass(frozen=True)
class Identity:
    """The release commit, its tree, and the version its manifest declares."""

    sha: str
    tree: str
    version: str
    tagged: bool


@dataclass
class Outcome:
    status: str  # passed, failed, or skipped
    detail: str
    logs: list[Path] = field(default_factory=list)
    exits: list[int] = field(default_factory=list)

    @classmethod
    def of(cls, ok: bool, detail: str, logs: Sequence[Path], exits: Sequence[int]) -> Outcome:
        return cls("passed" if ok else "failed", detail, list(logs), list(exits))


def utc_now() -> str:
    return datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ")


class Host:
    """Everything the pass asks of the machine, so a test can answer instead."""

    platform = sys.platform

    def which(self, name: str) -> str | None:
        return shutil.which(name)

    def capture(
        self,
        argv: Sequence[str | Path],
        cwd: Path | None = None,
        env: Mapping[str, str] | None = None,
    ) -> tuple[int, str]:
        """A command's exit status and its output, stdout then stderr; 127 if it cannot start."""
        try:
            result = subprocess.run(
                [str(arg) for arg in argv],
                cwd=cwd,
                env=None if env is None else dict(env),
                capture_output=True,
                text=True,
                errors="replace",
                check=False,
            )
        except OSError as error:
            return 127, str(error)
        return result.returncode, result.stdout + result.stderr

    def euid(self) -> int:
        return os.geteuid()

    def account(self, name: str) -> tuple[int, int] | None:
        try:
            entry = pwd.getpwnam(name)
        except KeyError:
            return None
        return entry.pw_uid, entry.pw_gid

    def free_bytes(self, path: Path) -> int:
        existing = path
        while not existing.exists() and existing != existing.parent:
            existing = existing.parent
        return shutil.disk_usage(existing).free

    def execute(
        self,
        argv: Sequence[str | Path],
        *,
        cwd: Path,
        env: Mapping[str, str],
        log: Path,
        header: Sequence[str],
        timeout: float | None = None,
    ) -> int:
        """Run a command to completion, its stdout and stderr together into `log`.

        The log is rewritten whole every few seconds while the command runs, so it can be
        followed, and once more with the exit status when it ends. A command still running
        after `timeout` seconds is stopped with everything it started, and that is a
        `StepError`: a hung step fails, and the rest of the pass still runs."""
        command = [str(arg) for arg in argv]
        head = "\n".join([*header, f"== command: {shlex.join(command)}", f"== start: {utc_now()}"])
        write_text_atomic(log, head + "\n", encoding="utf-8")
        try:
            process = subprocess.Popen(
                command,
                cwd=cwd,
                env=dict(env),
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                # A group of its own, so the whole step can be stopped, not only `make`.
                process_group=0,
            )
        except OSError as error:
            tail = f"== end: {utc_now()}\n== could not start: {error}\n== exit status: 127\n"
            write_text_atomic(log, f"{head}\n{tail}", encoding="utf-8")
            return 127
        expired = threading.Event()

        def stop() -> None:
            with contextlib.suppress(OSError):
                os.killpg(process.pid, signal.SIGKILL)

        def expire() -> None:
            expired.set()
            stop()

        timer = threading.Timer(timeout, expire) if timeout else None
        if timer is not None:
            timer.start()
        chunks: list[bytes] = []
        written = time.monotonic()
        assert process.stdout is not None
        try:
            for line in iter(process.stdout.readline, b""):
                chunks.append(line)
                if time.monotonic() - written > LOG_REFRESH_SECONDS:
                    output = b"".join(chunks).decode("utf-8", "replace")
                    write_text_atomic(log, f"{head}\n{output}", encoding="utf-8")
                    written = time.monotonic()
            status = process.wait()
        except BaseException:
            # Ctrl-C reaches this process but not the step's group, so stop that too.
            stop()
            process.wait()
            raise
        finally:
            if timer is not None:
                timer.cancel()
            process.stdout.close()
        output = b"".join(chunks).decode("utf-8", "replace")
        if output and not output.endswith("\n"):
            output += "\n"
        ended = f"== end: {utc_now()}"
        if expired.is_set():
            assert timeout is not None
            ended += f" (stopped after {timeout / 60:g} minutes)"
        write_text_atomic(
            log, f"{head}\n{output}{ended}\n== exit status: {status}\n", encoding="utf-8"
        )
        if expired.is_set():
            assert timeout is not None
            raise StepError(
                f"`{shlex.join(command[:3])}` was still running after {timeout / 60:g} minutes "
                "and was stopped (--timeout-minutes)",
                [log],
            )
        return status


# --- Arguments --------------------------------------------------------------------------


def select_steps(only: Sequence[str]) -> tuple[str, ...]:
    """The steps `--only` names, in pass order; every step when it names none."""
    if not only:
        return STEPS
    chosen: set[str] = set()
    for item in (part.strip() for value in only for part in value.split(",")):
        if not item:
            continue
        if item in STAGES:
            chosen.update(STAGES[item])
        elif item in ALIASES:
            chosen.update(ALIASES[item])
        elif item in STEPS:
            chosen.add(item)
        else:
            names = ", ".join(dict.fromkeys([*STAGES, *ALIASES, *STEPS]))
            raise UsageError(f"--only {item}: not a stage or step; choose from {names}")
    return tuple(step for step in STEPS if step in chosen)


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=(__doc__ or "").strip().split("\n\n")[0])
    result.add_argument("--commit", help="the release commit (default: $COMMIT)")
    result.add_argument(
        "--work-dir",
        type=Path,
        help="where the pass works and writes (default: $RELEASE/stability)",
    )
    for name, variable in TREE_VARIABLES.items():
        result.add_argument(
            f"--{name.replace('_', '-')}", type=Path, help=f"{TREE_ROLES[name]} (${variable})"
        )
    result.add_argument(
        "--peer-tree",
        type=Path,
        action="append",
        help=f"a real tree for peer agreement; repeat for more (default: ${PEER_TREES_VARIABLE},"
        " separated like PATH)",
    )
    result.add_argument(
        "--only",
        action="append",
        default=[],
        help="stages or steps to run, comma-separated: "
        + ", ".join([*STAGES, *ALIASES])
        + ", or a step name",
    )
    result.add_argument(
        "--target-dir",
        type=Path,
        help="Cargo target, which is then kept (default: WORK/target, removed after a pass)",
    )
    result.add_argument(
        "--wheels", type=Path, help="install the candidate from these wheels instead of building"
    )
    result.add_argument("--python", default="3.12", help="the interpreter the candidate runs on")
    result.add_argument(
        "--wrap", default="", help="a command prefix for the gates and the build, such as a lock"
    )
    result.add_argument(
        "--label",
        action="append",
        default=[],
        metavar="PATH=LABEL",
        help="how the report names a path",
    )
    result.add_argument("--keep", action="store_true", help="keep the worktree and target")
    result.add_argument("--min-free-gb", type=float, default=12.0)
    result.add_argument(
        "--timeout-minutes",
        type=float,
        default=360.0,
        help="stop a command still running after this long and fail its step; 0 waits forever",
    )
    result.add_argument("--date", help="the report's date (default: today, UTC)")
    return result


def configure(args: argparse.Namespace, environ: Mapping[str, str]) -> Config:
    """The pass's configuration from its arguments, falling back on the environment."""
    commit = args.commit or environ.get("COMMIT")
    if not commit:
        raise UsageError(
            "no release commit: set COMMIT, as the release checklist does, or pass --commit"
        )
    work = args.work_dir
    if work is None and environ.get("RELEASE"):
        work = Path(environ["RELEASE"]).expanduser() / "stability"
    if work is None:
        raise UsageError(
            "no work directory: set RELEASE, whose stability/ it then uses, or pass --work-dir"
        )

    def tree(name: str) -> Path | None:
        given = getattr(args, name)
        raw = str(given) if given is not None else environ.get(TREE_VARIABLES[name], "")
        return Path(raw).expanduser().absolute() if raw else None

    small, medium = tree("small"), tree("medium")
    medium_analyze = tree("medium_analyze")
    if args.peer_tree:
        peers = tuple(path.expanduser().absolute() for path in args.peer_tree)
    else:
        raw = environ.get(PEER_TREES_VARIABLE, "")
        peers = tuple(Path(part).expanduser().absolute() for part in raw.split(os.pathsep) if part)
    progress = tree("progress") or medium
    progress_analyze = tree("progress_analyze") or medium_analyze or medium
    labels = []
    for pair in args.label:
        path, separator, label = pair.partition("=")
        if not separator or not path or not label:
            raise UsageError(f"--label {pair}: expected PATH=LABEL")
        labels.append((str(Path(path).expanduser().absolute()), label))
    return Config(
        commit=commit,
        work=work.expanduser().absolute(),
        trees=Trees(
            small=small,
            medium=medium,
            medium_analyze=medium_analyze,
            large=tree("large"),
            progress=progress,
            progress_analyze=progress_analyze,
            peers=peers,
        ),
        steps=select_steps(args.only),
        target=args.target_dir.expanduser().absolute() if args.target_dir else None,
        python=args.python,
        wrap=tuple(shlex.split(args.wrap)),
        wheels=args.wheels.expanduser().absolute() if args.wheels else None,
        keep=args.keep,
        min_free_gb=args.min_free_gb,
        labels=tuple(labels),
        date=args.date or datetime.now(UTC).strftime("%Y-%m-%d"),
        timeout=args.timeout_minutes * 60 if args.timeout_minutes > 0 else None,
    )


# --- Identity and prerequisites ---------------------------------------------------------


def identify(config: Config, host: Host) -> Identity:
    """Resolve the commit in this clone, with its tree and declared version."""
    git = ["git", "-C", str(config.root)]
    status, sha = host.capture(
        [*git, "rev-parse", "--verify", "--quiet", f"{config.commit}^{{commit}}"]
    )
    if status != 0 or not sha.strip():
        raise UsageError(
            f"commit {config.commit} is not in this clone: fetch it (git fetch origin) and rerun"
        )
    sha = sha.strip()
    _, tree = host.capture([*git, "rev-parse", f"{sha}^{{tree}}"])
    status, manifest = host.capture([*git, "show", f"{sha}:crates/fdu/Cargo.toml"])
    try:
        version = tomllib.loads(manifest)["package"]["version"] if status == 0 else ""
    except (tomllib.TOMLDecodeError, KeyError):
        version = ""
    if not isinstance(version, str) or not version:
        raise UsageError(f"commit {sha[:9]} has no readable crates/fdu/Cargo.toml version")
    _, tags = host.capture([*git, "tag", "--points-at", sha])
    return Identity(sha, tree.strip(), version, f"v{version}" in tags.split())


def version_key(text: str) -> tuple[int, int, int] | None:
    match = re.search(r"(\d+)\.(\d+)\.(\d+)", text)
    return None if match is None else (int(match[1]), int(match[2]), int(match[3]))


def uv_floor(root: Path) -> str:
    """The reviewed uv version the Makefile pins, which CI pins too."""
    makefile = (root / "Makefile").read_text(encoding="utf-8")
    match = re.search(r"^UV_MIN_VERSION\s*:?=\s*(\S+)", makefile, re.M)
    if match is None:
        raise UsageError("the Makefile pins no UV_MIN_VERSION")
    return match[1]


def declared(name: str) -> bool:
    """Whether the environment declares a host precondition, as the Makefile reads it."""
    return os.environ.get(name) == "1"


def cross_targets(root: Path) -> tuple[str, ...]:
    """The targets `make cross-lint` checks, each of which it skips when not installed."""
    makefile = (root / "Makefile").read_text(encoding="utf-8")
    match = re.search(r"^CROSS_TARGETS\s*:?=\s*((?:.*\\\n)*.*)$", makefile, re.M)
    if match is None or not match[1].replace("\\\n", " ").split():
        raise UsageError("the Makefile names no CROSS_TARGETS")
    return tuple(match[1].replace("\\\n", " ").split())


def traversable(path: Path) -> bool:
    """Whether every user can reach `path`: each existing directory on it is searchable."""
    for part in [path, *path.parents]:
        try:
            mode = part.stat().st_mode
        except OSError:
            continue
        if stat.S_ISDIR(mode) and not mode & stat.S_IXOTH:
            return False
    return True


def prerequisites(config: Config, host: Host) -> tuple[list[str], dict[str, str]]:
    """What must be fixed before anything runs, one message each, and the steps that are
    skipped for want of an optional tool or tree, with the reason."""
    steps = set(config.steps)
    problems: list[str] = []
    skips: dict[str, str] = {}
    builds = bool(steps & set(GATES)) or ("candidate" in steps and config.wheels is None)

    if steps & {*GATES, "candidate"}:
        floor = uv_floor(config.root)
        status, output = host.capture(["uv", "--version"])
        installed = version_key(output) if status == 0 else None
        remedy = f"curl -LsSf https://astral.sh/uv/{floor}/install.sh | sh"
        if installed is None:
            problems.append(f"uv is not installed: install the reviewed {floor} with {remedy}")
        elif installed < (version_key(floor) or (0, 0, 0)):
            problems.append(
                f"uv {'.'.join(map(str, installed))} is older than the reviewed {floor} the "
                f"Makefile pins: install it with {remedy}"
            )
    if steps & set(GATES):
        if host.which("make") is None:
            problems.append("make is not on PATH, and the gates are Make targets")
        pinned = semver_check.pinned_tool_version(config.root)
        status, output = host.capture(["cargo", "semver-checks", "--version"])
        match = re.fullmatch(rf"{semver_check.TOOL} (\S+)\s*", output) if status == 0 else None
        if match is None or match[1] != pinned:
            found = "is not installed" if match is None else f"is {match[1]}"
            problems.append(
                f"{semver_check.TOOL} {found}, not the reviewed {pinned} that make semver-check "
                f"needs: {semver_check.install_command(pinned)}"
            )
    if "cross-lint" in steps:
        wanted = cross_targets(config.root)
        status, output = host.capture(["rustup", "target", "list", "--installed"], cwd=config.root)
        installed = set(output.split()) if status == 0 else set()
        if missing := [target for target in wanted if target not in installed]:
            problems.append(
                f"make cross-lint skips a target rustup has not installed, and {len(missing)} of "
                f"its {len(wanted)} are missing: rustup target add {' '.join(missing)}"
            )
    if "check" in steps and host.euid() == 0 and not declared("FDU_TEST_ALLOW_NO_PERMISSION_BITS"):
        problems.append(
            "running as root, where mode bits bind no one and make check stops at its "
            "permission-bits preflight: run as an unprivileged user, or declare the host "
            "unable with FDU_TEST_ALLOW_NO_PERMISSION_BITS=1, as AGENTS.md describes"
        )
    if builds:
        if host.which("cargo") is None:
            problems.append("cargo is not on PATH, and the gates and the wheel build need it")
        free = host.free_bytes(config.work)
        if free < config.min_free_gb * GB:
            problems.append(
                f"{free / GB:.1f} GB free for the work directory, and the gates and the release "
                f"build need about {config.min_free_gb:g} GB (--min-free-gb)"
            )
    if config.wheels is not None and "candidate" in steps and not config.wheels.is_dir():
        problems.append(f"--wheels {config.wheels} is not a directory")

    if "harness" in steps:
        if host.platform == "darwin":
            ok = host.capture([GNU_TIME, "true"])[0] == 0
        else:
            status, output = host.capture([GNU_TIME, "--version"])
            ok = status == 0 and "GNU" in output
        if not ok:
            problems.append(
                f"GNU time is not at {GNU_TIME}: the QA harness reads wall time and peak RSS "
                "from it (Debian and Ubuntu: apt install time)"
            )
        if config.trees.small is None:
            problems.append(
                "FDU_QA_SMALL (--small) is not set: the QA harness needs a small typed tree, "
                "such as a checkout of this repository"
            )
    for name, path in config.trees.named().items():
        if not path.is_dir():
            variable = TREE_VARIABLES.get(name, PEER_TREES_VARIABLE)
            problems.append(f"{variable} names {path}, which is not a directory")

    peer_steps = steps & {"peer-self-test", "peer-trees"}
    if peer_steps:
        if gnu_du(host) is None:
            reason = "GNU du, the reference, is not installed (macOS: brew install coreutils)"
            skips.update(dict.fromkeys(peer_steps, reason))
        elif missing := [tool for tool in PEERS if host.which(tool) is None]:
            reason = f"not installed: {', '.join(missing)}; the self-test requires every peer"
            skips.update(dict.fromkeys(peer_steps, reason))
    if "peer-trees" in steps and not config.trees.peers:
        skips.setdefault("peer-trees", f"{PEER_TREES_VARIABLE} is not set")
    if "pty-probe" in steps:
        if config.trees.progress is None or config.trees.progress_analyze is None:
            skips["pty-probe"] = "neither FDU_QA_PROGRESS_TREE nor FDU_QA_MEDIUM is set"
        elif config.trees.small is None:
            skips["pty-probe"] = "FDU_QA_SMALL is not set"

    if steps & AS_NOBODY and host.euid() == 0:
        reason = None
        if host.which("setpriv") is None:
            reason = "running as root without setpriv, so the refusals cannot take effect"
        elif host.account(NOBODY) is None:
            reason = f"running as root with no {NOBODY} account, so the refusals cannot take effect"
        if reason:
            skips.update(dict.fromkeys(steps & AS_NOBODY, reason))
        else:
            for label, path in (
                ("the work directory", config.work),
                ("this checkout", config.root),
            ):
                if not traversable(path):
                    problems.append(
                        f"{label}, {path}, is not reachable by {NOBODY}, who runs the refusal "
                        "pass: choose a work directory, and run from a checkout, that every "
                        "user can reach"
                    )
    return problems, skips


def notices(config: Config) -> list[str]:
    """What will leave the pass incomplete though nothing is wrong, said before it runs."""
    found = []
    if "harness" in config.steps:
        for phase, variable in HARNESS_TREE_PHASES.items():
            if getattr(config.trees, phase) is None:
                found.append(
                    f"{variable} is not set, so the harness skips its `{phase}` phase and is "
                    "recorded as skipped: the pass will be incomplete (exit 3)"
                )
    return found


def gnu_du(host: Host) -> str | None:
    """GNU du, which macOS installs from coreutils as `gdu`, found as the peer script does."""
    for name in ("gdu", "du"):
        if host.which(name) is None:
            continue
        status, output = host.capture([name, "--version"])
        if status == 0 and "GNU coreutils" in output:
            return name
    return None


def version_names_commit(version: str, identity: Identity, *, bare_ok: bool) -> bool:
    """`fdu --version` names the commit: its nine-digit revision, not dirty, or the bare
    version where the build was stamped as the release (a tag, or the rehearsal's own)."""
    text = version.strip()
    if ".dirty" in text:
        return False
    if f"+g{identity.sha[:9]}" in text:
        return True
    return (bare_ok or identity.tagged) and text == f"fdu {identity.version}"


# --- State ------------------------------------------------------------------------------


def load_state(work: Path, identity: Identity) -> dict[str, Any]:
    """The work directory's record, which must be for this commit."""
    path = work / "state.json"
    if not path.exists():
        return {
            "commit": identity.sha,
            "tree": identity.tree,
            "version": identity.version,
            "steps": {},
        }
    try:
        state = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise UsageError(f"{path} cannot be read ({error}); use a new work directory") from error
    if state.get("commit") != identity.sha:
        raise UsageError(
            f"{work} holds the pass for commit {str(state.get('commit'))[:9]}, not "
            f"{identity.sha[:9]}: use a new work directory for this commit"
        )
    state.setdefault("steps", {})
    return state


def save_state(work: Path, state: Mapping[str, Any]) -> None:
    write_text_atomic(work / "state.json", json.dumps(state, indent=1) + "\n", encoding="utf-8")


# --- Running ----------------------------------------------------------------------------


@dataclass
class Context:
    config: Config
    identity: Identity
    host: Host
    state: dict[str, Any]
    trees_base: Path | None = None
    built: set[str] = field(default_factory=set)
    # Recorded with each step, so a pass assembled from several runs says which was which.
    tooling: str = ""
    user: str = ""

    @property
    def work(self) -> Path:
        return self.config.work

    @property
    def worktree(self) -> Path:
        return self.work / "worktree"

    @property
    def target(self) -> Path:
        return self.config.target or self.work / "target"

    @property
    def fdu(self) -> Path:
        return Path(self.state["candidate"]["fdu"])

    def script(self, *parts: str) -> Path:
        return self.config.root.joinpath(*parts)

    def log(self, name: str) -> Path:
        return self.work / "logs" / f"{name}.log"

    def env(self, **extra: str) -> dict[str, str]:
        return {**os.environ, **extra}

    def execute(
        self,
        argv: Sequence[str | Path],
        *,
        cwd: Path,
        log: Path,
        env: Mapping[str, str] | None = None,
        header: Sequence[str] = (),
    ) -> int:
        log.parent.mkdir(parents=True, exist_ok=True)
        print(f"      log: {log}", flush=True)
        lines = [
            f"== commit: {self.identity.sha} tree: {self.identity.tree}",
            *header,
            f"== free: {self.host.free_bytes(self.work) / GB:.1f} GB",
        ]
        return self.host.execute(
            argv,
            cwd=cwd,
            env=env or self.env(),
            log=log,
            header=lines,
            timeout=self.config.timeout,
        )


def body(log: Path) -> str:
    """A step's output: its log without the pass's own `==` header and footer lines.

    The header ends at its `== start:` line and the footer begins at `== end:`, so output
    that itself opens or closes with `== ` lines, as `make cross-lint`'s does, is kept."""
    try:
        lines = log.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return ""
    start, end = 0, len(lines)
    while start < end and lines[start].startswith("== "):
        start += 1
        if lines[start - 1].startswith("== start:"):
            break
    while end > start and lines[end - 1].startswith("== "):
        end -= 1
        if lines[end].startswith("== end:"):
            break
    return "\n".join(lines[start:end]).strip("\n")


def ensure_worktree(ctx: Context) -> Path:
    """A clean worktree at the commit, created once and reused while it stays clean."""
    tree = ctx.worktree
    if tree.exists():
        _, head = ctx.host.capture(["git", "-C", str(tree), "rev-parse", "HEAD"])
        _, dirty = ctx.host.capture(
            ["git", "-C", str(tree), "status", "--porcelain", "--untracked-files=no"]
        )
        if head.strip() != ctx.identity.sha or dirty.strip():
            raise StepError(
                f"{tree} is not a clean checkout of {ctx.identity.sha[:9]}: remove it with "
                f"`git worktree remove --force {tree}` and rerun"
            )
        return tree
    git = ["git", "-C", str(ctx.config.root), "worktree", "add", "--detach", str(tree)]
    status, output = ctx.host.capture([*git, ctx.identity.sha])
    if status != 0:
        raise StepError(f"git worktree add failed: {output.strip()}")
    made(ctx, "worktree")
    return tree


def made(ctx: Context, what: str) -> None:
    """Record that this pass created the worktree or the target, so `cleanup` may remove it."""
    ctx.state.setdefault("created", {})[what] = True
    save_state(ctx.work, ctx.state)


def build_env(ctx: Context) -> dict[str, str]:
    """The environment of a build, whose target directory the pass makes when it is its own."""
    if ctx.config.target is None and not ctx.target.exists():
        ctx.target.mkdir(parents=True)
        made(ctx, "target")
    return ctx.env(CARGO_TARGET_DIR=str(ctx.target))


def gate(ctx: Context, name: str) -> Outcome:
    tree = ensure_worktree(ctx)
    log = ctx.log(f"gate-{name}")
    _, dirty = ctx.host.capture(["git", "-C", str(tree), "status", "--porcelain"])
    header = [f"== gate: make {name}", f"== status: {len(dirty.splitlines())} dirty paths"]
    status = ctx.execute(
        [*ctx.config.wrap, "make", name], cwd=tree, env=build_env(ctx), log=log, header=header
    )
    detail = f"`make {name}` exited {status}"
    if name == "semver-check":
        said = [
            line for line in body(log).splitlines() if re.match(r"^fdu(?:-core)? \S+[: ]", line)
        ]
        if said:
            detail += ": " + "; ".join(said)
    if name == "cross-lint":
        # It exits 0 for a target rustup has not installed, having linted nothing there.
        linted = re.findall(r"^== clippy: (\S+)", body(log), re.M)
        skipped = re.findall(r"^== skipping (\S+)", body(log), re.M)
        detail += f": linted {', '.join(linted) or 'no target'}"
        if skipped:
            detail += (
                f"; skipped {', '.join(skipped)}, not installed "
                f"(rustup target add {' '.join(skipped)})"
            )
        if status == 0 and (skipped or not linted):
            return Outcome("skipped", detail, [log], [status])
    return Outcome.of(status == 0, detail, [log], [status])


def candidate(ctx: Context) -> Outcome:
    """Build the wheel from the worktree, or take the given ones, and install it alone."""
    logs: list[Path] = []
    source = ctx.config.wheels
    if source is None:
        tree = ensure_worktree(ctx)
        source = ctx.work / "wheel"
        source.mkdir(parents=True, exist_ok=True)
        for old in source.glob("*.whl"):
            old.unlink()
        # The build is a cargo command outside Make's guard against a target directory
        # that another checkout built in, so run that guard first, as each gate does.
        log = ctx.log("candidate-target-owner")
        logs.append(log)
        guard = [*ctx.config.wrap, "make", "target-owner"]
        status = ctx.execute(guard, cwd=tree, env=build_env(ctx), log=log)
        if status != 0:
            return Outcome("failed", f"`make target-owner` exited {status}", logs, [status])
        log = ctx.log("candidate-build")
        logs.append(log)
        build = [
            *ctx.config.wrap,
            *("uv", "run", "--frozen", "--only-group", "dev"),
            *("maturin", "build", "--locked", "--release", "--out", str(source)),
        ]
        status = ctx.execute(build, cwd=tree / "crates" / "fdu-py", env=build_env(ctx), log=log)
        if status != 0:
            return Outcome("failed", f"the wheel build exited {status}", logs, [status])
    tools, binaries = ctx.work / "tools", ctx.work / "bin"
    for directory in (tools, binaries):
        directory.mkdir(parents=True, exist_ok=True)
        directory.chmod(0o755)
    log = ctx.log("candidate-install")
    logs.append(log)
    # `--no-build`: a wheel or nothing. Given files may hold a source distribution too,
    # and one built here would be a candidate nobody published.
    install = [
        *("uv", "tool", "install", "--force", "--python", ctx.config.python),
        *("--no-index", "--no-build", "--find-links", str(source), "fdu"),
    ]
    env = ctx.env(UV_TOOL_DIR=str(tools), UV_TOOL_BIN_DIR=str(binaries))
    status = ctx.execute(install, cwd=ctx.work, env=env, log=log)
    if status != 0:
        return Outcome("failed", f"`uv tool install` exited {status}", logs, [status])
    fdu = binaries / "fdu"
    status, version = ctx.host.capture([str(fdu), "--version"])
    version = version.strip()
    given = ctx.config.wheels is not None
    wheel = installed_wheel(tools, source)
    # A given wheel is stamped as the release and names no commit, so only the rehearsal's
    # own record can say which commit it is.
    proven, proof = rehearsal_proof(wheel, source, ctx.identity) if given else (False, "")
    ctx.state["candidate"] = {
        "fdu": str(fdu),
        "version": version,
        "wheel": wheel.name if wheel else None,
        "built": not given,
        "proof": proof if proven else "",
    }
    if not (status == 0 and version_names_commit(version, ctx.identity, bare_ok=proven)):
        detail = f"`fdu --version` printed `{version}`, which does not name {ctx.identity.sha[:9]}"
        if given and not proven and version == f"fdu {ctx.identity.version}":
            detail += f", and {proof}"
        return Outcome("failed", detail, logs, [status])
    detail = f"`fdu --version` printed `{version}`"
    if wheel:
        detail += f", from `{wheel.name}`"
    return Outcome("passed", detail, logs, [0])


def wheel_tags(name: str) -> set[str] | None:
    """The compatibility tags a wheel's filename declares, as its `WHEEL` file lists them."""
    parts = name.removesuffix(".whl").split("-")
    if len(parts) < 5:
        return None
    python, abi, platforms = parts[-3:]
    return {
        f"{p}-{a}-{m}"
        for p in python.split(".")
        for a in abi.split(".")
        for m in platforms.split(".")
    }


def installed_wheel(tools: Path, source: Path) -> Path | None:
    """Which of the wheels in `source` uv installed: the one whose tags the installed
    metadata repeats, or the only one there."""
    wheels = sorted(source.glob("fdu-*.whl"))
    for metadata in sorted(tools.glob("fdu/lib/python*/site-packages/fdu-*.dist-info/WHEEL")):
        try:
            lines = metadata.read_text(encoding="utf-8").splitlines()
        except OSError:
            continue
        tags = {line.partition(":")[2].strip() for line in lines if line.startswith("Tag:")}
        matches = [wheel for wheel in wheels if wheel_tags(wheel.name) == tags]
        if len(matches) == 1:
            return matches[0]
    return wheels[0] if len(wheels) == 1 else None


def rehearsal_proof(wheel: Path | None, source: Path, identity: Identity) -> tuple[bool, str]:
    """Whether a given wheel is the rehearsal's wheel of this commit, and why or why not.

    The release checklist keeps a rehearsal's files in `$RELEASE/rehearsal/files`, their
    checksums beside them in `evidence/SHA256SUMS`, and the commit in `$RELEASE/state.json`."""
    if wheel is None:
        return False, "the installed wheel could not be told from the others given"
    try:
        release = json.loads((source.parent.parent / "state.json").read_text(encoding="utf-8"))
        sums = (source.parent / "evidence" / "SHA256SUMS").read_text(encoding="utf-8")
        recorded = release.get("commit")
    except (OSError, json.JSONDecodeError, AttributeError):
        return False, (
            f"`{wheel.name}` is not in a release directory's rehearsal/files, whose record "
            "would say which commit it was built from"
        )
    if recorded != identity.sha:
        return False, f"the release directory of `{wheel.name}` records commit {str(recorded)[:9]}"
    listed = {
        match[2]: match[1]
        for line in sums.splitlines()
        if (match := re.fullmatch(r"([0-9a-f]{64})  (\S+)", line))
    }
    digest = hashlib.sha256(wheel.read_bytes()).hexdigest()
    if listed.get(wheel.name) != digest:
        return False, f"the rehearsal's SHA256SUMS does not list `{wheel.name}` with its digest"
    return (
        True,
        "which the rehearsal's SHA256SUMS lists and whose release directory records the commit",
    )


def candidate_ready(ctx: Context) -> bool:
    """The candidate a previous run installed is still there and still the commit's."""
    recorded = ctx.state.get("candidate")
    if not recorded or ctx.state["steps"].get("candidate", {}).get("status") != "passed":
        return False
    status, version = ctx.host.capture([recorded["fdu"], "--version"])
    return status == 0 and version.strip() == recorded["version"]


def harness(ctx: Context) -> Outcome:
    out = ctx.work / "qa"
    shutil.rmtree(out, ignore_errors=True)
    env = {k: v for k, v in os.environ.items() if not k.startswith("FDU_QA_")}
    env.update(FDU=str(ctx.fdu), FDU_QA_OUT=str(out))
    for name, variable in TREE_VARIABLES.items():
        if name in ("small", "medium", "medium_analyze", "large"):
            path = getattr(ctx.config.trees, name)
            if path is not None:
                env[variable] = str(path)
    log = ctx.log("harness")
    script = ctx.script("scripts", "run_installed_cli_qa.py")
    status = ctx.execute([sys.executable, script], cwd=ctx.config.root, env=env, log=log)
    counts = harness_counts(out / "results.json")
    detail = f"the harness exited {status}"
    if counts:
        total = sum(counts.values())
        shown = ["ok", "warn", "fail", *(["skip"] if counts.get("skip") else [])]
        detail = f"{total} checks: " + ", ".join(
            f"{counts[verdict]} {verdict}" for verdict in shown
        )
    # The harness exits 0 when nothing it ran failed, whatever it left out.
    gaps = harness_gaps(out / "results.json")
    if status == 0 and gaps:
        return Outcome("skipped", f"{detail}; {'; '.join(gaps)}", [log], [status])
    return Outcome.of(status == 0, detail, [log], [status])


def harness_results(path: Path) -> dict[str, Any]:
    try:
        results = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    return results if isinstance(results, dict) else {}


def harness_rows(path: Path) -> list[dict[str, Any]]:
    rows = harness_results(path).get("rows", [])
    return [row for row in rows if isinstance(row, dict)] if isinstance(rows, list) else []


def harness_gaps(path: Path) -> list[str]:
    """What the harness left out of a run it exited 0 from: a stop on SIGKILL or the RSS
    limit, after which every later check is a `skip` row, or a phase whose tree is unset."""
    rows = harness_rows(path)
    gaps = []
    if reason := harness_results(path).get("stopped_reason"):
        gaps.append(f"it stopped early ({reason})")
    if skipped := sum(row.get("verdict") == "skip" for row in rows):
        gaps.append(f"{skipped} checks did not run")
    phases = {row.get("phase") for row in rows}
    for phase, variable in HARNESS_TREE_PHASES.items():
        if phase not in phases:
            gaps.append(f"its `{phase}` phase did not run, which needs {variable}")
    return gaps


def harness_counts(path: Path, phases: Iterable[str] | None = None) -> Counter[str]:
    wanted = None if phases is None else set(phases)
    return Counter(
        str(row.get("verdict"))
        for row in harness_rows(path)
        if wanted is None or row.get("phase") in wanted
    )


def peer_self_test(ctx: Context) -> Outcome:
    scratch = ctx.work / "scratch"
    scratch.mkdir(parents=True, exist_ok=True)
    log = ctx.log("peer-self-test")
    script = ctx.script("scripts", "qa_peer_agreement.py")
    argv = [sys.executable, script, "--self-test", "--fdu", ctx.fdu, "--scratch", scratch]
    status = ctx.execute(argv, cwd=ctx.config.root, log=log)
    said = re.search(r"^Self-test: (.*?)\.?$", body(log), re.M)
    detail = said[1] if said else "no summary"
    return Outcome.of(status == 0, f"{detail}; exit status {status}", [log], [status])


def peer_trees(ctx: Context) -> Outcome:
    record = ctx.work / "peers" / "peer-agreement.json"
    record.parent.mkdir(parents=True, exist_ok=True)
    log = ctx.log("peer-trees")
    script = ctx.script("scripts", "qa_peer_agreement.py")
    argv = [sys.executable, script, *ctx.config.trees.peers, "--fdu", ctx.fdu, "--json", record]
    status = ctx.execute(argv, cwd=ctx.config.root, log=log)
    return Outcome.of(status == 0, peer_summary(body(log), status), [log], [status])


def peer_summary(text: str, status: int) -> str:
    rows = [line for line in text.splitlines() if TOOL_ROW.match(line)]
    exact = sum("| agrees exactly |" in line for line in rows)
    trees = len(re.findall(r"^### `", text, re.M))
    unexplained = text.count("UNEXPLAINED")
    last = text.strip().splitlines()[-1].rstrip(".") if text.strip() else "no summary"
    return (
        f"{exact} of {len(rows)} readings on {trees} trees agree exactly, "
        f"{unexplained} `UNEXPLAINED`; {last}; exit status {status}"
    )


def pty_probe(ctx: Context) -> Outcome:
    trees = ctx.config.trees
    assert trees.progress and trees.small and trees.progress_analyze
    log = ctx.log("pty-probe")
    argv = [
        *(sys.executable, ctx.script("scripts", "qa", "pty_probe.py"), "--fdu", ctx.fdu),
        *("--tree", trees.progress, "--small", trees.small),
        *("--analyze-tree", trees.progress_analyze),
    ]
    status = ctx.execute(argv, cwd=ctx.config.root, log=log)
    said = re.search(r"^(\d+) of (\d+) checks passed", body(log), re.M)
    if said:
        detail = f"{said[0]}"
    elif status == 2:
        detail = "the trees cannot support the probe; its log says why"
    else:
        detail = f"the probe exited {status}"
    return Outcome.of(status == 0, detail, [log], [status])


def terminal(ctx: Context) -> Outcome:
    log = ctx.log("terminal")
    argv = [sys.executable, "-m", "unittest", "discover", "-s", "tests/terminal", "-p", "test_*.py"]
    env = ctx.env(FDU_BIN=str(ctx.fdu))
    status = ctx.execute(argv, cwd=ctx.config.root, env=env, log=log)
    ran = re.search(r"^Ran (\d+) tests?", body(log), re.M)
    detail = f"{ran[1]} tests ran against the candidate" if ran else f"the tests exited {status}"
    return Outcome.of(status == 0, f"{detail}; exit status {status}", [log], [status])


# --- Correctness ------------------------------------------------------------------------


def trees_base(ctx: Context) -> Path:
    """A short directory every user can reach: a Unix socket path is limited to about 104
    bytes, and the refusal pass runs as nobody."""
    if ctx.trees_base is None:
        parent = "/tmp" if Path("/tmp").is_dir() else None
        base = Path(tempfile.mkdtemp(prefix="fdu-trees-", dir=parent))
        base.chmod(0o755)
        scratch = base / "tmp"
        scratch.mkdir()
        scratch.chmod(0o1777)
        ctx.trees_base = base
        ctx.state["trees_base"] = str(base)
    return ctx.trees_base


def remove_trees(base: Path) -> None:
    """Remove the trees, making the unlistable entries traversable first."""
    for directory, names, _ in os.walk(base):
        for name in names:
            path = Path(directory, name)
            if not path.is_symlink():
                with contextlib.suppress(OSError):
                    path.chmod(0o755)
    shutil.rmtree(base, ignore_errors=True)


def correctness_tree(ctx: Context, name: str) -> Path:
    """The refusal tree (`tree`) or the tree without refusals (`served`), built once."""
    base = trees_base(ctx)
    path = base / name
    if name not in ctx.built:
        script = ctx.script("tests", "correctness", "build_tree.py")
        flags = ["--without-refusals"] if name == "served" else []
        log = ctx.log(f"build-{name}")
        status = ctx.execute([sys.executable, script, *flags, path], cwd=base, log=log)
        if status != 0:
            raise StepError(f"building the {name} tree exited {status}; see {log}")
        ctx.built.add(name)
        ctx.state.setdefault("kinds", {})[name] = kinds_summary(body(log))
    return path


def kinds_summary(text: str) -> str:
    """What the tree builder managed to create, as its last lines report it."""
    counts = re.search(r"^kinds present: (\d+), absent: (\d+)", text, re.M)
    if counts is None:
        return "kinds not reported"
    absent = re.search(r"^absent: (.*)$", text, re.M)
    names = f" ({absent[1]})" if absent else ""
    return f"{counts[1]} kinds present, {counts[2]} absent{names}"


def nobody_python(ctx: Context) -> str:
    """An interpreter nobody can run: this one if every user can reach it, else python3."""
    for candidate_path in (sys.executable, shutil.which("python3")):
        if candidate_path and traversable(Path(os.path.realpath(candidate_path)).parent):
            return candidate_path
    return sys.executable


def as_nobody(ctx: Context, argv: Sequence[str | Path]) -> list[str | Path]:
    """The command as nobody when running as root, where mode bits bind no one."""
    if ctx.host.euid() != 0:
        return list(argv)
    account = ctx.host.account(NOBODY)
    assert account is not None
    uid, gid = account
    return ["setpriv", f"--reuid={uid}", f"--regid={gid}", "--clear-groups", *argv]


def ready_for_nobody(ctx: Context, env: Mapping[str, str], python: str) -> None:
    """Nobody can read the scripts and run the candidate, or the pass says why not."""
    if ctx.host.euid() != 0:
        return
    probe = (
        "import subprocess, sys; open(sys.argv[1]).close(); "
        "subprocess.run([sys.argv[2], '--version'], check=True, capture_output=True)"
    )
    script = ctx.script("tests", "correctness", "warm_cold.py")
    argv = as_nobody(ctx, [python, "-c", probe, script, ctx.fdu])
    status, output = ctx.host.capture(argv, cwd=trees_base(ctx), env=env)
    if status != 0:
        last = output.strip().splitlines()[-1] if output.strip() else f"exit status {status}"
        raise StepError(
            f"{NOBODY} cannot read the correctness scripts or run the candidate ({last}): keep "
            "the work directory, this checkout, and the interpreter where every user can reach"
        )


def table_rows(text: str) -> list[str]:
    """The rows of a correctness script's table: from its dashes to the first blank line."""
    lines = text.splitlines()
    start = next((i + 1 for i, line in enumerate(lines) if line and set(line) == {"-"}), None)
    if start is None:
        return []
    rows = []
    for line in lines[start:]:
        if not line.strip():
            break
        rows.append(line)
    return rows


def count(text: str, label: str) -> int | None:
    match = re.search(rf"^{re.escape(label)}: (\d+)", text, re.M)
    return None if match is None else int(match[1])


def warm_cold_summary(text: str, *, refusals: bool) -> str:
    rows = table_rows(text)
    verdicts = [row.split()[7:] for row in rows]
    mismatches = count(text, "answer mismatches")
    mechanism = count(text, "mechanism failures (cache did not serve)")
    stale = count(text, "stale reference instants")
    served = re.search(r"^cases the snapshot served: (\d+) of (\d+)", text, re.M)
    if refusals:
        withheld = sum(verdict == ["withheld"] for verdict in verdicts)
        head = f"{withheld} of {len(rows)} cases partial and withheld"
    else:
        head = (
            f"{served[1]} of {served[2]} served `cache_only` and labelled `stale`"
            if served
            else f"{len(rows)} cases"
        )
    return (
        f"{head}; {mismatches} answer mismatches, {mechanism} mechanism failures, "
        f"{stale} stale reference instants"
    )


def cross_warm_summary(text: str) -> str:
    rows = table_rows(text)
    ok = sum(row.rstrip().endswith(" ok") for row in rows)
    return (
        f"{ok} of {len(rows)} pairs matched the cold answer; "
        f"{count(text, 'cross-warm violations')} violations"
    )


def refusals(ctx: Context) -> Outcome:
    tree = correctness_tree(ctx, "tree")
    python = nobody_python(ctx)
    env = ctx.env(FDU_BIN=str(ctx.fdu), TMPDIR=str(trees_base(ctx) / "tmp"))
    ready_for_nobody(ctx, env, python)
    script = ctx.script("tests", "correctness", "warm_cold.py")
    log = ctx.log("correctness-refusals")
    argv = as_nobody(ctx, [python, script, "--refusals-only", tree])
    status = ctx.execute(argv, cwd=trees_base(ctx), env=env, log=log)
    detail = warm_cold_summary(body(log), refusals=True)
    return Outcome.of(status == 0, f"{detail}; exit status {status}", [log], [status])


def served(ctx: Context) -> Outcome:
    tree = correctness_tree(ctx, "served")
    script = ctx.script("tests", "correctness", "warm_cold.py")
    log = ctx.log("correctness-served")
    env = ctx.env(FDU_BIN=str(ctx.fdu))
    status = ctx.execute([sys.executable, script, tree], cwd=trees_base(ctx), env=env, log=log)
    detail = warm_cold_summary(body(log), refusals=False)
    return Outcome.of(status == 0, f"{detail}; exit status {status}", [log], [status])


def cross_warm(ctx: Context) -> Outcome:
    tree = correctness_tree(ctx, "served")
    script = ctx.script("tests", "correctness", "cross_warm.py")
    log = ctx.log("correctness-cross-warm")
    env = ctx.env(FDU_BIN=str(ctx.fdu))
    status = ctx.execute([sys.executable, script, tree], cwd=trees_base(ctx), env=env, log=log)
    detail = cross_warm_summary(body(log))
    return Outcome.of(status == 0, f"{detail}; exit status {status}", [log], [status])


def no_snapshot_caught(text: str) -> tuple[bool, str]:
    """Whether `warm_cold.py` caught every case of a cache that stored nothing, and how
    many of each kind: a metadata case `NO-SNAPSHOT`, an analysis case `NOT-WARM`."""
    rows = table_rows(text)
    missing = sum("NO-SNAPSHOT(" in row for row in rows)
    cold = sum("NOT-WARM(" in row and "NO-SNAPSHOT(" not in row for row in rows)
    said = f"{missing + cold} of {len(rows)} cases caught"
    if rows:
        said += f" ({missing} `NO-SNAPSHOT`, {cold} `NOT-WARM`)"
    return bool(rows) and missing + cold == len(rows), said


def not_warm_caught(text: str) -> tuple[bool, str]:
    """Whether `cross_warm.py` caught every pair it holds to serving, which it counts."""
    held = count(text, "pairs held to serving")
    caught = sum("NOT-WARM(" in row for row in table_rows(text))
    if held is None:
        return False, f"{caught} pairs `NOT-WARM`, and no count of the pairs held to serving"
    return held > 0 and caught == held, f"{caught} of {held} same-analyzer pairs `NOT-WARM`"


def break_no_snapshot(ctx: Context) -> Outcome:
    """With nothing ever stored, both scripts over the served tree must exit 1, and each
    must catch every case it holds to serving. A script exits 1 on its first failing
    case, so the exit status alone would pass a check that had gone blind to the rest."""
    tree = correctness_tree(ctx, "served")
    wrapper = ctx.script("tests", "correctness", "break_no_snapshot.py")
    env = ctx.env(FDU_BIN=str(wrapper), FDU_REAL=str(ctx.fdu))
    logs, exits, parts, ok = [], [], [], True
    for name, script, judge in (
        ("warm-cold", "warm_cold.py", no_snapshot_caught),
        ("cross-warm", "cross_warm.py", not_warm_caught),
    ):
        log = ctx.log(f"break-no-snapshot-{name}")
        argv = [sys.executable, ctx.script("tests", "correctness", script), tree]
        status = ctx.execute(argv, cwd=trees_base(ctx), env=env, log=log)
        logs.append(log)
        exits.append(status)
        every, said = judge(body(log))
        ok = ok and status == 1 and every
        parts.append(f"`{script}` exited {status}, {said}")
    return Outcome.of(ok, "; ".join(parts), logs, exits)


def break_partial_stored(ctx: Context) -> Outcome:
    """With a partial answer served, the refusal pass must exit 1, every case caught."""
    tree = correctness_tree(ctx, "tree")
    python = nobody_python(ctx)
    wrapper = ctx.script("tests", "correctness", "break_partial_stored.py")
    env = ctx.env(FDU_BIN=str(wrapper), FDU_REAL=str(ctx.fdu), TMPDIR=str(trees_base(ctx) / "tmp"))
    ready_for_nobody(ctx, env, python)
    script = ctx.script("tests", "correctness", "warm_cold.py")
    log = ctx.log("break-partial-stored")
    argv = as_nobody(ctx, [python, script, "--refusals-only", tree])
    status = ctx.execute(argv, cwd=trees_base(ctx), env=env, log=log)
    rows = table_rows(body(log))
    caught = sum("PARTIAL-STORED" in row for row in rows)
    detail = f"`warm_cold.py --refusals-only` exited {status}, {caught} of {len(rows)} cases `PARTIAL-STORED`"
    return Outcome.of(status == 1 and bool(rows) and caught == len(rows), detail, [log], [status])


RUNNERS: dict[str, Callable[[Context], Outcome]] = {
    **{name: (lambda ctx, name=name: gate(ctx, name)) for name in GATES},
    "candidate": candidate,
    "harness": harness,
    "peer-self-test": peer_self_test,
    "peer-trees": peer_trees,
    "pty-probe": pty_probe,
    "terminal": terminal,
    "refusals": refusals,
    "served": served,
    "cross-warm": cross_warm,
    "break-no-snapshot": break_no_snapshot,
    "break-partial-stored": break_partial_stored,
}


def record(ctx: Context, step: str, outcome: Outcome, seconds: float) -> None:
    ctx.state["steps"][step] = {
        "status": outcome.status,
        "detail": outcome.detail,
        "logs": [str(log) for log in outcome.logs],
        "exits": outcome.exits,
        "finished": utc_now(),
        "seconds": round(seconds, 1),
        "tooling": ctx.tooling,
        "user": ctx.user,
    }
    save_state(ctx.work, ctx.state)


def run_steps(ctx: Context, steps: Sequence[str], skips: Mapping[str, str]) -> list[Outcome]:
    outcomes = []
    for index, step in enumerate(steps, 1):
        print(f"[{index}/{len(steps)}] {step}", flush=True)
        started = time.monotonic()
        if step in skips:
            outcome = Outcome("skipped", skips[step])
        elif step in NEEDS_CANDIDATE and ctx.state.get("candidate") is None:
            outcome = Outcome("skipped", "no candidate is installed")
        elif (
            step in NEEDS_CANDIDATE
            and ctx.state["steps"].get("candidate", {}).get("status") != "passed"
        ):
            outcome = Outcome("skipped", "the candidate step did not pass")
        else:
            try:
                outcome = RUNNERS[step](ctx)
            except StepError as error:
                outcome = Outcome("failed", str(error), error.logs)
            except Exception as error:  # recorded, so the rest of the pass still runs
                traceback.print_exc()
                outcome = Outcome("failed", f"internal error: {error!r}")
        record(ctx, step, outcome, time.monotonic() - started)
        print(f"      {outcome.status}: {outcome.detail}", flush=True)
        outcomes.append(outcome)
    return outcomes


def cleanup(ctx: Context) -> None:
    """Remove the worktree and the target directory this pass made. The candidate and the
    records stay, and so does a directory the pass found there or was pointed at: a
    `--target-dir` is the caller's, whatever is in it."""
    created = ctx.state.get("created", {})
    if created.get("worktree") and ctx.worktree.exists():
        ctx.host.capture(
            ["git", "-C", str(ctx.config.root), "worktree", "remove", "--force", str(ctx.worktree)]
        )
        shutil.rmtree(ctx.worktree, ignore_errors=True)
        ctx.host.capture(["git", "-C", str(ctx.config.root), "worktree", "prune"])
    own_target = ctx.work / "target"
    if created.get("target") and own_target.exists():
        shutil.rmtree(own_target, ignore_errors=True)
    ctx.state["created"] = {}
    save_state(ctx.work, ctx.state)


def enclosing_checkout(work: Path, host: Host) -> str | None:
    """The checkout the work directory is inside, if it is inside one."""
    existing = work
    while not existing.exists() and existing != existing.parent:
        existing = existing.parent
    status, output = host.capture(["git", "-C", str(existing), "rev-parse", "--show-toplevel"])
    return output.strip() if status == 0 and output.strip() else None


# --- Regime -----------------------------------------------------------------------------


def first_line(host: Host, argv: Sequence[str]) -> str | None:
    status, output = host.capture(argv)
    lines = output.strip().splitlines()
    return lines[0].strip() if status == 0 and lines else None


def tooling_revision(config: Config, host: Host) -> str:
    """The checkout the QA and correctness scripts run from: its commit, and whether its
    tracked files differ from it."""
    git = ["git", "-C", str(config.root)]
    _, head = host.capture([*git, "rev-parse", "HEAD"])
    _, changed = host.capture([*git, "status", "--porcelain", "--untracked-files=no"])
    return head.strip()[:9] + ("+dirty" if changed.strip() else "")


def user_kind(host: Host) -> str:
    """Root or not, which is what decides the refusal pass; never the login name."""
    return "root" if host.euid() == 0 else "unprivileged"


def regime(config: Config, host: Host) -> dict[str, Any]:
    """The host a result is evidence about, as far as it can be read rather than known."""
    filesystem = None
    if host.platform == "linux":
        filesystem = first_line(host, ["findmnt", "-no", "FSTYPE", "-T", str(config.work)])
        filesystem = filesystem or first_line(host, ["stat", "-f", "-c", "%T", str(config.work)])
    virtualization = (
        first_line(host, ["systemd-detect-virt"]) if host.which("systemd-detect-virt") else None
    )
    du = gnu_du(host)
    peers = {"GNU du": first_line(host, [du, "--version"]) if du else None}
    for tool in PEERS:
        peers[tool] = first_line(host, [tool, "--version"]) if host.which(tool) else None
    stated = {name: os.environ[name] for name in DECLARED if name in os.environ}
    return {
        "system": f"{platform.system()} {platform.machine()}",
        "kernel": platform.release(),
        "cpus": os.cpu_count(),
        "virtualization": virtualization,
        "filesystem": filesystem,
        "user": user_kind(host),
        "declared": stated,
        "peers": peers,
        "tooling": tooling_revision(config, host),
    }


# --- Redaction --------------------------------------------------------------------------


def default_label(path: str, home: str) -> str:
    """How a report names a tree: relative to home, a system path as it is, or its name."""
    path = os.path.normpath(path)
    if path == "/":
        return "/"
    if home and home != "/" and (path == home or path.startswith(home + "/")):
        return "~" + path[len(home) :]
    if HOME_DIRECTORY.match(path) or path == "/root":
        return "<home>"
    if any(path == root or path.startswith(root + "/") for root in SYSTEM_ROOTS):
        return path
    return os.path.basename(path)


def label_pairs(state: Mapping[str, Any], home: str) -> list[tuple[str, str]]:
    """Every path the pass knows, with its label, longest first so none is cut short."""
    paths = state.get("paths", {})
    labels = {path: default_label(path, home) for path in paths.get("trees", {}).values()}
    for key, label in (
        ("tmp", "<tmp>"),
        ("work", "<work>"),
        ("root", "<fdu checkout>"),
        ("worktree", "<fdu checkout>"),
    ):
        if paths.get(key):
            labels[paths[key]] = label
    if state.get("trees_base"):
        labels[state["trees_base"]] = "<trees>"
    labels.update(dict(paths.get("labels", [])))
    pairs: dict[str, str] = {}
    for path, label in labels.items():
        for form in {os.path.normpath(path), os.path.realpath(path)}:
            if form not in ("/", label):
                pairs.setdefault(form, label)
    return sorted(pairs.items(), key=lambda item: -len(item[0]))


def redact(text: str, pairs: Sequence[tuple[str, str]], home: str) -> str:
    """The text with each known path replaced by its label, then anything left that names
    a home directory replaced too."""
    for path, label in pairs:
        text = re.sub(re.escape(path) + BOUNDARY, lambda _, label=label: label, text)
    text = PEER_SCRATCH.sub("<scratch>", text)
    if home and home != "/":
        text = re.sub(re.escape(home) + BOUNDARY, "~", text)
    text = ANY_HOME.sub("<home>", text)
    return ROOT_HOME.sub("<home>", text)


# --- Report -----------------------------------------------------------------------------

# The playbook's own row names and the record's ranges use these; the report is ASCII
# otherwise, and `make docs-format` makes its quotes typographic once it is committed.
EN_DASH, TIMES = "\u2013", "\u00d7"
STATUS_WORDS = {"passed": "Passed", "failed": "Failed", "skipped": "Skipped"}
PLAYBOOK_STATUS = {"passed": "✅ Passed", "failed": "❌ Failed", "skipped": "⏸️ Blocked"}
FOOTER = (
    "<!-- This document follows common-doc-guidelines.md.\n"
    "See github.com/jlevy/practical-prose and review guidelines before editing.\n-->\n"
)


def verdict(step: Mapping[str, Any] | None) -> str:
    if not step:
        return "Not run"
    word = STATUS_WORDS.get(step.get("status", ""), str(step.get("status")))
    return f"{word}: {step['detail']}" if step.get("detail") else word


def fenced(text: str) -> str:
    return f"```\n{text.replace('```', "'''")}\n```"


def demoted(text: str) -> str:
    """The peer script's Markdown, nested one level deeper, without its own title."""
    lines = [line for line in text.splitlines() if not line.startswith("## Peer agreement")]
    return "\n".join(re.sub(r"^(#{3,}) ", r"#\1 ", line) for line in lines).strip("\n")


@dataclass
class Evidence:
    """What the report is written from: the state, and each log's output. None of it is
    redacted yet; `write_report` redacts what is rendered from it, whole."""

    state: Mapping[str, Any]
    bodies: dict[str, str]
    harness_table: str | None
    harness_rows: list[dict[str, Any]]
    harness_stopped: str = ""

    def step(self, name: str) -> Mapping[str, Any] | None:
        return self.state.get("steps", {}).get(name)

    def text(self, log: str) -> str | None:
        return self.bodies.get(log)


def gather(work: Path, state: Mapping[str, Any]) -> Evidence:
    bodies = {}
    for step in state.get("steps", {}).values():
        for log in step.get("logs", []):
            bodies[Path(log).stem] = body(Path(log))
    table_path = work / "qa" / "results.md"
    table = table_path.read_text(encoding="utf-8").strip() if table_path.exists() else None
    results = work / "qa" / "results.json"
    stopped = str(harness_results(results).get("stopped_reason") or "")
    return Evidence(state, bodies, table, harness_rows(results), stopped)


def tally(state: Mapping[str, Any]) -> tuple[list[str], list[str], list[str]]:
    """The recorded pass's steps that failed, were skipped, and have not run."""
    steps = state.get("steps", {})
    failed = [name for name in STEPS if steps.get(name, {}).get("status") == "failed"]
    skipped = [name for name in STEPS if steps.get(name, {}).get("status") == "skipped"]
    missing = [name for name in STEPS if name not in steps]
    return failed, skipped, missing


def pass_status(state: Mapping[str, Any]) -> int:
    """The exit status of the whole recorded pass, whichever steps this run ran: a rerun
    of one stage must not read as the pass when another stage has failed or not run."""
    failed, skipped, missing = tally(state)
    if failed:
        return EXIT_FAILED
    return EXIT_INCOMPLETE if skipped or missing else 0


def overall(state: Mapping[str, Any]) -> str:
    failed, skipped, missing = tally(state)
    parts = []
    if failed:
        parts.append(f"Failed: {', '.join(failed)}.")
    if skipped:
        parts.append(f"Skipped: {', '.join(skipped)}.")
    if missing:
        parts.append(f"Not run: {', '.join(missing)}.")
    return " ".join(parts) or "Nothing failed, and nothing was skipped."


def lowered(text: str) -> str:
    return text[:1].lower() + text[1:]


def note(step: Mapping[str, Any] | None) -> str:
    """A step's detail, which already says it passed; otherwise its verdict."""
    if step and step.get("status") == "passed" and step.get("detail"):
        return str(step["detail"])
    return verdict(step)


def step_fact(state: Mapping[str, Any], step: str, key: str) -> str:
    """What a step recorded of the run it was part of, else what the last run recorded."""
    recorded = state.get("steps", {}).get(step, {}).get(key)
    return str(recorded or state.get("regime", {}).get(key) or "")


def refusal_user(state: Mapping[str, Any], step: str) -> str:
    """Who the refusals had to bind: `nobody` when the step's run was root's."""
    return "`nobody`" if step_fact(state, step, "user") == "root" else "an unprivileged user"


def grouped(state: Mapping[str, Any], key: str) -> dict[str, list[str]]:
    """Each distinct value the steps recorded for `key`, with the steps that recorded it."""
    found: dict[str, list[str]] = {}
    for name in STEPS:
        if name in state.get("steps", {}) and (value := step_fact(state, name, key)):
            found.setdefault(value, []).append(name)
    return found


def phase_six(evidence: Evidence) -> str:
    return (
        "Pending: a person has not watched a real window, and Windows has not run. "
        f"Terminal tests {lowered(verdict(evidence.step('terminal')))}; "
        f"pty probe {lowered(verdict(evidence.step('pty-probe')))}"
    )


def render_report(evidence: Evidence, date: str) -> str:
    state = evidence.state
    version, sha, tree = state["version"], state["commit"], state["tree"]
    as_user = refusal_user(state, "refusals")
    rows = [
        ("`make check`", "check"),
        ("`make cross-lint`", "cross-lint"),
        ("`make semver-check`", "semver-check"),
        ("`make release-rehearse`", "release-rehearse"),
        ("Candidate", "candidate"),
        (f"QA harness, phases 1{EN_DASH}5 and 8", "harness"),
        ("QA phase 7, peer agreement self-test", "peer-self-test"),
        ("QA phase 7, peer agreement on real trees", "peer-trees"),
        (f"Correctness, refusal tree as {as_user}", "refusals"),
        ("Correctness, served tree", "served"),
        ("Correctness, cross-warm matrix", "cross-warm"),
        ("Correctness, break: no snapshot stored", "break-no-snapshot"),
        ("Correctness, break: partial answer stored", "break-partial-stored"),
    ]
    table = ["| Gate or phase | Verdict |", "| --- | --- |"]
    for label, step in rows:
        table.append(f"| {label} | {verdict(evidence.step(step))} |")
        if step == "harness":
            table.append(f"| QA phase 6, terminal progress | {phase_six(evidence)} |")
    lines = [
        f"# {version} Stability Pass — {date}",
        "",
        f"This is the full record of the {version} release checklist's stability pass (step 2",
        "of the [release process](../guides/release-process.md#the-steps)), written by",
        "`make release-stability`. The summaries belong beside the procedures, in the",
        "[QA playbook's Current Status](../../../tests/qa/cli-installed-e2e.qa.md) and the",
        "[correctness runbook's Last Recorded Run](../guides/correctness-runbook.md#last-recorded-run);",
        "this report keeps every table they summarize.",
        "",
        "## Verdict",
        "",
        f"The pass tested commit `{sha}`, tree `{tree}`.",
        "A release commit with that tree inherits every result here by tree identity; check it",
        "with `git rev-parse <commit>^{tree}` before tagging.",
        overall(state),
        "",
        *table,
        "",
        "## Regime",
        "",
        *regime_lines(state),
        "",
        "## Reproduce",
        "",
        *reproduce_lines(state),
        "",
        "## Full Tables",
        "",
        "Paths are replaced by the labels above.",
        *full_tables(evidence),
        "",
        FOOTER,
    ]
    return "\n".join(lines)


def regime_lines(state: Mapping[str, Any]) -> list[str]:
    facts = state.get("regime", {})
    host = f"{facts.get('system', 'unknown')}, {facts.get('cpus', '?')} CPUs, kernel "
    host += str(facts.get("kernel", "unknown"))
    virtualization = facts.get("virtualization")
    if virtualization == "none":
        host += ", bare metal"
    elif virtualization:
        host += f", virtualized ({virtualization})"
    if facts.get("filesystem"):
        host += f", the work directory on {facts['filesystem']}"
    users = grouped(state, "user") or {str(facts.get("user", "")): []}
    spoken = {"root": "root", "unprivileged": "an unprivileged user"}
    if len(users) == 1:
        # Anything else is an older record's login name, which a report does not repeat.
        host += f", running as {spoken.get(next(iter(users)), 'an unprivileged user')}."
    else:
        host += ", running as " + " and as ".join(
            f"{spoken.get(user, 'an unprivileged user')} for {', '.join(steps)}"
            for user, steps in users.items()
        )
        host += "."
    stated = ", ".join(f"`{k}={v}`" for k, v in facts.get("declared", {}).items()) or "none"
    candidate_facts = state.get("candidate", {})
    wheel = candidate_facts.get("wheel") or next(iter(candidate_facts.get("wheels", [])), None)
    named = f"`{wheel}` wheel" if wheel else "wheel"
    if candidate_facts.get("built", True):
        source = f"The {named} built from the commit"
    else:
        source = f"The given {named}"
        if candidate_facts.get("proof"):
            source += f", {candidate_facts['proof']}"
    peers = ", ".join(
        f"{version}" if version else f"{tool} not installed"
        for tool, version in facts.get("peers", {}).items()
    )
    labels = state.get("labels", {})
    trees = (
        "; ".join(f"{name.replace('_', ' ')}: `{label}`" for name, label in labels.items())
        or "none recorded"
    )
    kinds = "; ".join(f"{name}: {text}" for name, text in state.get("kinds", {}).items())
    lines = [
        f"- **Host.** {host}"
        + ("" if virtualization else " Say whether it is bare metal or virtualized.")
        + " Say whether anything else ran.",
        f"- **Declared preconditions.** {stated}.",
        f"- **Candidate.** {source}, installed with `uv tool install` into an isolated tool "
        f"directory; `fdu --version` printed `{candidate_facts.get('version', 'nothing')}`.",
        f"- **Peers.** {peers or 'none recorded'}.",
        f"- **Trees.** {trees}.",
    ]
    if kinds:
        lines.append(f"- **Correctness trees.** {kinds}.")
    lines.append(tooling_line(state))
    return [line for line in lines if line]


def tooling_line(state: Mapping[str, Any]) -> str:
    """The revision of the QA and correctness scripts, per step when the runs differed."""

    def named(revision: str) -> str:
        commit, _, dirty = revision.partition("+")
        return f"`{commit[:9]}`" + (" with uncommitted changes" if dirty else "")

    revisions = grouped(state, "tooling")
    if not revisions:
        return ""
    if len(revisions) == 1:
        return f"- **Tooling.** The QA and correctness scripts at {named(next(iter(revisions)))}."
    parts = "; ".join(f"{named(rev)} for {', '.join(steps)}" for rev, steps in revisions.items())
    return f"- **Tooling.** The runs used more than one revision of the scripts: {parts}."


def reproduce_lines(state: Mapping[str, Any]) -> list[str]:
    variables = state.get("variables", {})
    exports = [f"export {name}={shlex.quote(value)}" for name, value in variables.items()]
    return [
        "```shell",
        f"export COMMIT={state['commit']}",
        "export RELEASE=<a directory outside any checkout>",
        *exports,
        "make release-stability",
        "```",
    ]


def full_tables(evidence: Evidence) -> list[str]:
    candidate_facts = evidence.state.get("candidate", {})
    sections: list[tuple[str, str | None]] = [
        (
            f"Installed-CLI harness ({candidate_facts.get('version', 'fdu')})",
            evidence.harness_table,
        ),
        ("Peer agreement self-test", demoted(evidence.text("peer-self-test") or "") or None),
        ("Peer agreement on real trees", demoted(evidence.text("peer-trees") or "") or None),
    ]
    for title, log in (
        ("Phase 6 pty probe", "pty-probe"),
        ("Terminal tests", "terminal"),
        ("Correctness: refusals", "correctness-refusals"),
        ("Correctness: served", "correctness-served"),
        ("Correctness: cross-warm", "correctness-cross-warm"),
        ("Correctness: break, no snapshot, warm against cold", "break-no-snapshot-warm-cold"),
        ("Correctness: break, no snapshot, cross-warm", "break-no-snapshot-cross-warm"),
        ("Correctness: break, partial answer stored", "break-partial-stored"),
    ):
        text = evidence.text(log)
        sections.append((title, fenced(text) if text else None))
    lines = []
    for title, text in sections:
        lines += ["", f"### {title}", "", text or "Not run."]
    return lines


def harness_note(evidence: Evidence, phases: Sequence[str]) -> tuple[str, str]:
    """A playbook row's status and notes from the harness rows of its phases."""
    rows = [row for row in evidence.harness_rows if row.get("phase") in phases]
    if not rows:
        step = evidence.step("harness")
        reason = "not run" if step is None else "no rows; the harness skipped this phase"
        return "⏸️ Blocked", reason.capitalize()
    counts = Counter(str(row.get("verdict")) for row in rows)
    # A phase the harness stopped in has not passed, whatever its earlier rows say.
    status = (
        "❌ Failed" if counts.get("fail") else "⏸️ Blocked" if counts.get("skip") else "✅ Passed"
    )
    notes = f"{len(rows)} checks: " + ", ".join(
        f"{counts[key]} {key}" for key in ("ok", "warn", "fail", "skip") if counts.get(key)
    )
    warned = [str(row.get("name")) for row in rows if row.get("verdict") in ("warn", "fail")]
    if warned:
        notes += f" ({', '.join(f'`{name}`' for name in warned)})"
    if counts.get("skip") and evidence.harness_stopped:
        notes += f"; the harness stopped at {evidence.harness_stopped}"
    return status, notes


def playbook_status(step: Mapping[str, Any] | None) -> str:
    return "⏸️ Blocked" if not step else PLAYBOOK_STATUS.get(step["status"], step["status"])


def render_tables(evidence: Evidence) -> str:
    state = evidence.state
    labels = state.get("labels", {})
    candidate_step = evidence.step("candidate")
    peer_steps = [evidence.step("peer-self-test"), evidence.step("peer-trees")]
    peer_status = (
        "❌ Failed"
        if any(step and step["status"] == "failed" for step in peer_steps)
        else "✅ Passed"
        if all(step and step["status"] == "passed" for step in peer_steps)
        else "⏸️ Blocked"
    )
    versions = ", ".join(v for v in state.get("regime", {}).get("peers", {}).values() if v)
    phases = [
        ("Phase 1: Setup", playbook_status(candidate_step), note(candidate_step)),
        ("Phase 2: Small-tree views", *harness_note(evidence, ("sanity", "views"))),
        (
            f"Phase 3: Cache {TIMES} analyze",
            *harness_note(evidence, ("cache-analyze", "analyze-extra", "watch")),
        ),
        ("Phase 4: Medium tree", *harness_note(evidence, ("medium",))),
        ("Phase 5: Bounded large tree", *harness_note(evidence, ("large",))),
        ("Phase 6: Terminal Progress", "⏳ Pending", phase_six(evidence)),
        (
            "Phase 7: Peer agreement",
            peer_status,
            f"{versions}. Self-test: {note(peer_steps[0])}. Real trees: {note(peer_steps[1])}",
        ),
        (
            "Phase 8: Results",
            playbook_status(evidence.step("harness")),
            note(evidence.step("harness")),
        ),
    ]
    trees = "; ".join(f"{name.replace('_', ' ')} `{label}`" for name, label in labels.items())
    as_user = refusal_user(state, "refusals")
    passes = [
        (f"`--refusals-only`, refusal tree, as {as_user}", "refusals"),
        ("`warm_cold.py`, complete tree", "served"),
        ("`cross_warm.py`, complete tree", "cross-warm"),
    ]
    lines = [
        f"# {state['version']} Stability Pass: Summary Tables",
        "",
        f"Commit `{state['commit']}`, tree `{state['tree']}`. Trees: {trees or 'none recorded'}.",
        "",
        "## QA Playbook Current Status",
        "",
        "| Phase | Status | Notes |",
        "| --- | --- | --- |",
        *(f"| {phase} | {status} | {notes} |" for phase, status, notes in phases),
        "",
        "## Correctness Runbook Last Recorded Run",
        "",
        "| Pass | Result |",
        "| --- | --- |",
        *(f"| {label} | {note(evidence.step(step))} |" for label, step in passes),
        "",
        "Each pass was checked by breaking it:",
        "",
        "- **No snapshot stored.** A wrapper turned `--cache on` into `--cache off`: "
        f"{note(evidence.step('break-no-snapshot'))}.",
        "- **Partial answer stored.** A wrapper answered `--stale-ok` with the cold output "
        f"relabeled `cache_only`: {note(evidence.step('break-partial-stored'))}.",
        "",
    ]
    return "\n".join(lines)


def report_paths(work: Path, state: Mapping[str, Any], date: str) -> tuple[Path, Path]:
    name = f"report-{date}-release-{state['version']}-stability-pass.md"
    return work / name, work / "summary-tables.md"


def write_report(work: Path, state: Mapping[str, Any], date: str, home: str) -> tuple[Path, Path]:
    """Write the report and the tables, each redacted whole: a step's detail, a declared
    value, and a log all reach the page, and any of them can carry a private path."""
    evidence = gather(work, state)
    pairs = label_pairs(state, home)
    report, tables = report_paths(work, state, date)
    for path, text in ((report, render_report(evidence, date)), (tables, render_tables(evidence))):
        write_text_atomic(path, redact(text, pairs, home), encoding="utf-8")
    return report, tables


def remember(config: Config, host: Host, state: dict[str, Any]) -> None:
    """Record this run's paths, their labels, and the host, for the report."""
    home = os.path.expanduser("~")
    paths = state.setdefault("paths", {})
    named = {name: str(path) for name, path in config.trees.named().items()}
    paths["trees"] = {**paths.get("trees", {}), **named}
    paths.update(
        root=str(config.root),
        work=str(config.work),
        worktree=str(config.work / "worktree"),
        tmp=os.path.realpath(tempfile.gettempdir()),
        labels=[*paths.get("labels", []), *map(list, config.labels)],
    )
    pairs = label_pairs(state, home)
    state["labels"] = {name: redact(path, pairs, home) for name, path in paths["trees"].items()}
    variables = {}
    for name, path in paths["trees"].items():
        variable = TREE_VARIABLES.get(name)
        if variable:
            variables[variable] = redact(path, pairs, home)
    peers = [
        redact(path, pairs, home)
        for name, path in paths["trees"].items()
        if name.startswith("peer ")
    ]
    if peers:
        variables[PEER_TREES_VARIABLE] = os.pathsep.join(peers)
    state["variables"] = variables
    if config.steps or "regime" not in state:
        state["regime"] = regime(config, host)


def main(argv: Sequence[str] | None = None, host: Host | None = None) -> int:
    host = host or Host()
    try:
        config = configure(parser().parse_args(argv), os.environ)
        if host.which("git") is None:
            raise UsageError("git is not on PATH")
        if host.platform not in ("linux", "darwin"):
            raise UsageError(f"the stability pass runs on Linux or macOS, not {host.platform}")
        identity = identify(config, host)
        if checkout := enclosing_checkout(config.work, host):
            raise UsageError(
                f"the work directory {config.work} is inside the checkout {checkout}: the pass "
                "makes a worktree and builds there, so use a directory outside any checkout, "
                "as RELEASE is"
            )
        state = load_state(config.work, identity)
        ctx = Context(config, identity, host, state)
        steps = list(config.steps)
        if set(steps) & NEEDS_CANDIDATE and "candidate" not in steps and not candidate_ready(ctx):
            steps.insert(next(i for i, s in enumerate(steps) if s in NEEDS_CANDIDATE), "candidate")
            config = dataclasses.replace(config, steps=tuple(steps))
            ctx.config = config
        problems, skips = prerequisites(config, host)
    except UsageError as error:
        print(f"error: {error}", file=sys.stderr)
        return EXIT_USAGE
    if problems:
        for problem in problems:
            print(f"error: {problem}", file=sys.stderr)
        return EXIT_USAGE
    for notice in notices(config):
        print(f"note: {notice}", flush=True)
    config.work.mkdir(parents=True, exist_ok=True)
    if set(steps) & AS_NOBODY and host.euid() == 0:
        # Nobody runs the refusal pass, and the candidate it runs is installed here.
        config.work.chmod(config.work.stat().st_mode | stat.S_IXOTH)
    remember(config, host, state)
    ctx.tooling, ctx.user = tooling_revision(config, host), user_kind(host)
    save_state(config.work, state)
    print(f"stability pass of {identity.sha[:9]} ({identity.version}) in {config.work}", flush=True)
    try:
        outcomes = run_steps(ctx, steps, skips)
    finally:
        if ctx.trees_base is not None:
            remove_trees(ctx.trees_base)
        save_state(config.work, state)
    report, tables = write_report(config.work, state, config.date, os.path.expanduser("~"))
    print(f"\nreport: {report}\ntables: {tables}")
    if outcomes:
        ran = Counter(outcome.status for outcome in outcomes)
        counts = ", ".join(f"{ran[word]} {word}" for word in STATUS_WORDS if ran[word])
        print(f"this run: {counts}")
    # The status is the whole record's, not this run's: after `--only`, a step another run
    # failed, skipped, or never reached still decides whether the pass has passed.
    status = pass_status(state)
    verdicts = {
        0: "every step has passed.",
        EXIT_FAILED: "failed.",
        EXIT_INCOMPLETE: "nothing has failed, but it is incomplete.",
    }
    summary = "" if status == 0 else f" {overall(state)}"
    print(f"the pass: {verdicts[status]}{summary}")
    if status != EXIT_FAILED and not config.keep:
        # A failure anywhere in the record keeps the worktree and target it was built in.
        cleanup(ctx)
    return status


if __name__ == "__main__":
    sys.exit(main())
