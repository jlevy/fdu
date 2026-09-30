"""Tests for the stability-pass driver: arguments, prerequisites, redaction, the report,
and the steps' verdicts, against a host that answers instead of building or running fdu."""

from __future__ import annotations

import io
import json
import os
import tempfile
import unittest
from collections.abc import Callable, Mapping, Sequence
from contextlib import redirect_stderr, redirect_stdout
from dataclasses import replace
from pathlib import Path
from typing import Any
from unittest import mock

from scripts.release import semver_check
from scripts.release import stability_pass as sp

ROOT = Path(__file__).resolve().parents[2]
SHA = "e808f96042b9b92b722f03bd4fce8d8e2fca7efe"
TREE = "df899f7da7fd2fabd908b1ec6e893f392233e273"
VERSION = "fdu 0.3.0-dev+ge808f9604"
ALL_TOOLS = {"git", "make", "cargo", "du", "setpriv", *sp.PEERS}

Script = Callable[[list[str], Mapping[str, str]], tuple[int, str]]


class FakeHost(sp.Host):
    """A host with every prerequisite, unless a test takes one away."""

    platform = "linux"

    def __init__(
        self,
        *,
        tools: set[str] | None = None,
        answers: dict[tuple[str, ...], tuple[int, str]] | None = None,
        euid: int = 1000,
        accounts: dict[str, tuple[int, int]] | None = None,
        free: int = 100 * sp.GB,
        script: Script | None = None,
    ) -> None:
        self.tools = ALL_TOOLS if tools is None else tools
        pinned = semver_check.pinned_tool_version(ROOT)
        self.answers = {
            ("uv", "--version"): (0, "uv 0.12.1 (x86_64-unknown-linux-gnu)"),
            ("cargo", "semver-checks", "--version"): (0, f"cargo-semver-checks {pinned}\n"),
            (sp.GNU_TIME, "--version"): (0, "time (GNU Time) UNKNOWN\n"),
            ("du", "--version"): (0, "du (GNU coreutils) 9.4\n"),
            **(answers or {}),
        }
        self._euid = euid
        self.accounts = {"nobody": (65534, 65534)} if accounts is None else accounts
        self.free = free
        self.script = script
        self.executed: list[tuple[list[str], dict[str, str]]] = []

    def which(self, name: str) -> str | None:
        return f"/usr/bin/{name}" if name in self.tools else None

    def capture(
        self,
        argv: Sequence[str | Path],
        cwd: Path | None = None,
        env: Mapping[str, str] | None = None,
    ) -> tuple[int, str]:
        args = tuple(str(arg) for arg in argv)
        for prefix, answer in self.answers.items():
            if args[: len(prefix)] == prefix:
                return answer
        if args[0] == "git":
            command = args[3:]
            if command[:2] == ("rev-parse", "--verify") or command == ("rev-parse", "HEAD"):
                return 0, SHA + "\n"
            if command[:1] == ("rev-parse",) and command[-1].endswith("^{tree}"):
                return 0, TREE + "\n"
            if command[:1] == ("show",):
                return 0, '[package]\nname = "fdu"\nversion = "0.3.0"\n'
            return 0, ""
        if args[0].endswith("/fdu") and args[1:] == ("--version",):
            return 0, VERSION + "\n"
        return 127, f"{args[0]}: not found"

    def euid(self) -> int:
        return self._euid

    def account(self, name: str) -> tuple[int, int] | None:
        return self.accounts.get(name)

    def free_bytes(self, path: Path) -> int:
        return self.free

    def execute(
        self,
        argv: Sequence[str | Path],
        *,
        cwd: Path,
        env: Mapping[str, str],
        log: Path,
        header: Sequence[str],
    ) -> int:
        args = [str(arg) for arg in argv]
        self.executed.append((args, dict(env)))
        status, output = self.script(args, env) if self.script else (0, "")
        log.parent.mkdir(parents=True, exist_ok=True)
        log.write_text(
            "\n".join(
                [*header, "== start: now", output, "== end: now", f"== exit status: {status}"]
            )
            + "\n",
            encoding="utf-8",
        )
        return status


def quietly(function: Callable[[], int]) -> int:
    with redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
        return function()


class Scratch(unittest.TestCase):
    """A temporary directory with a tree for every role."""

    def setUp(self) -> None:
        self.scratch = tempfile.TemporaryDirectory()
        self.base = Path(self.scratch.name)
        for name in ("small", "medium", "large", "slow", "analyze", "peer-a", "peer-b"):
            (self.base / name).mkdir()

    def tearDown(self) -> None:
        self.scratch.cleanup()

    def trees(self, **changes: Any) -> sp.Trees:
        trees = sp.Trees(
            small=self.base / "small",
            medium=self.base / "medium",
            large=self.base / "large",
            progress=self.base / "slow",
            progress_analyze=self.base / "analyze",
            peers=(self.base / "peer-a", self.base / "peer-b"),
        )
        return replace(trees, **changes)

    def config(self, steps: Sequence[str] = sp.STEPS, **changes: Any) -> sp.Config:
        trees = changes.pop("trees", None) or self.trees()
        work = changes.pop("work", self.base / "work")
        return sp.Config(commit=SHA, work=work, trees=trees, steps=tuple(steps), **changes)


class StepSelectionTests(unittest.TestCase):
    def test_no_selection_runs_every_step_in_order(self) -> None:
        self.assertEqual(sp.select_steps([]), sp.STEPS)
        self.assertEqual(sp.STEPS[:5], (*sp.GATES, "candidate"))

    def test_stages_aliases_and_steps_combine_in_pass_order(self) -> None:
        self.assertEqual(sp.select_steps(["gates"]), sp.GATES)
        self.assertEqual(
            sp.select_steps(["served,peers", "candidate"]),
            ("candidate", "peer-self-test", "peer-trees", "served"),
        )
        self.assertEqual(sp.select_steps(["breaks"]), ("break-no-snapshot", "break-partial-stored"))

    def test_report_alone_selects_no_step(self) -> None:
        self.assertEqual(sp.select_steps(["report"]), ())

    def test_an_unknown_name_is_a_usage_error_that_lists_the_names(self) -> None:
        with self.assertRaisesRegex(sp.UsageError, "--only bogus: .*gates, candidate, qa"):
            sp.select_steps(["gates,bogus"])


class ConfigureTests(Scratch):
    def configure(self, argv: Sequence[str], environ: Mapping[str, str]) -> sp.Config:
        return sp.configure(sp.parser().parse_args(list(argv)), environ)

    def test_the_release_identity_comes_from_the_checklist_variables(self) -> None:
        config = self.configure([], {"COMMIT": SHA, "RELEASE": str(self.base / "0.3.0")})
        self.assertEqual(config.commit, SHA)
        self.assertEqual(config.work, self.base / "0.3.0" / "stability")
        self.assertEqual(config.steps, sp.STEPS)

    def test_arguments_outrank_the_environment(self) -> None:
        config = self.configure(
            ["--commit", "abc", "--work-dir", str(self.base / "w"), "--small", str(self.base)],
            {"COMMIT": SHA, "RELEASE": "/elsewhere", "FDU_QA_SMALL": "/other"},
        )
        self.assertEqual((config.commit, config.work), ("abc", self.base / "w"))
        self.assertEqual(config.trees.small, self.base)

    def test_without_a_commit_or_a_work_directory_nothing_runs(self) -> None:
        with self.assertRaisesRegex(sp.UsageError, "set COMMIT"):
            self.configure([], {"RELEASE": "/r"})
        with self.assertRaisesRegex(sp.UsageError, "set RELEASE"):
            self.configure([], {"COMMIT": SHA})

    def test_trees_come_from_the_playbook_variables(self) -> None:
        environ = {
            "COMMIT": SHA,
            "RELEASE": "/r",
            "FDU_QA_SMALL": "/s",
            "FDU_QA_MEDIUM": "/m",
            "FDU_QA_MEDIUM_ANALYZE": "/m/docs",
            "FDU_QA_LARGE": "/l",
            "FDU_QA_PEER_TREES": os.pathsep.join(["/p1", "", "/p2"]),
        }
        trees = self.configure([], environ).trees
        self.assertEqual(trees.small, Path("/s"))
        self.assertEqual(trees.large, Path("/l"))
        self.assertEqual(trees.peers, (Path("/p1"), Path("/p2")))
        # The pty probe falls back on the medium tree and its analyze subtree.
        self.assertEqual((trees.progress, trees.progress_analyze), (Path("/m"), Path("/m/docs")))

    def test_the_probe_trees_can_be_named_on_their_own(self) -> None:
        environ = {
            "COMMIT": SHA,
            "RELEASE": "/r",
            "FDU_QA_MEDIUM": "/m",
            "FDU_QA_PROGRESS_TREE": "/home",
            "FDU_QA_PROGRESS_ANALYZE": "/src/drivers",
        }
        trees = self.configure([], environ).trees
        self.assertEqual(
            (trees.progress, trees.progress_analyze), (Path("/home"), Path("/src/drivers"))
        )
        without = self.configure([], {"COMMIT": SHA, "RELEASE": "/r", "FDU_QA_MEDIUM": "/m"}).trees
        self.assertEqual(without.progress_analyze, Path("/m"))

    def test_repeated_peer_trees_labels_and_a_wrapper(self) -> None:
        config = self.configure(
            [
                *("--peer-tree", "/a", "--peer-tree", "/b", "--label", "/a=tree a"),
                *("--wrap", "flock -s /lock", "--only", "qa"),
            ],
            {"COMMIT": SHA, "RELEASE": "/r", "FDU_QA_PEER_TREES": "/ignored"},
        )
        self.assertEqual(config.trees.peers, (Path("/a"), Path("/b")))
        self.assertEqual(config.labels, (("/a", "tree a"),))
        self.assertEqual(config.wrap, ("flock", "-s", "/lock"))
        self.assertEqual(config.steps, sp.STAGES["qa"])
        with self.assertRaisesRegex(sp.UsageError, "PATH=LABEL"):
            self.configure(["--label", "nolabel"], {"COMMIT": SHA, "RELEASE": "/r"})


class PrerequisiteTests(Scratch):
    def test_a_host_with_everything_needs_nothing(self) -> None:
        self.assertEqual(sp.prerequisites(self.config(), FakeHost()), ([], {}))

    def test_each_missing_prerequisite_is_one_clear_message(self) -> None:
        pinned = semver_check.pinned_tool_version(ROOT)
        cases = [
            ({(sp.GNU_TIME, "--version"): (127, "")}, "GNU time is not at /usr/bin/time"),
            ({(sp.GNU_TIME, "--version"): (0, "BSD time")}, "GNU time is not at /usr/bin/time"),
            ({("uv", "--version"): (127, "")}, "uv is not installed: install the reviewed"),
            ({("uv", "--version"): (0, "uv 0.11.9")}, "uv 0.11.9 is older than the reviewed"),
            (
                {("cargo", "semver-checks", "--version"): (0, "cargo-semver-checks 0.1.0")},
                f"is 0.1.0, not the reviewed {pinned} that make semver-check needs: "
                f"cargo install --locked cargo-semver-checks --version {pinned}",
            ),
            (
                {("cargo", "semver-checks", "--version"): (101, "no such command")},
                "cargo-semver-checks is not installed",
            ),
        ]
        for answers, message in cases:
            with self.subTest(message=message):
                problems, _ = sp.prerequisites(self.config(), FakeHost(answers=answers))
                self.assertEqual(len(problems), 1, problems)
                self.assertIn(message, problems[0])

    def test_too_little_space_for_the_builds(self) -> None:
        problems, _ = sp.prerequisites(self.config(), FakeHost(free=3 * sp.GB))
        self.assertEqual(len(problems), 1)
        self.assertIn("3.0 GB free", problems[0])
        self.assertIn("--min-free-gb", problems[0])

    def test_only_the_selected_steps_prerequisites_are_checked(self) -> None:
        bare = FakeHost(
            tools={"git", "du", *sp.PEERS},
            answers={("uv", "--version"): (127, ""), (sp.GNU_TIME, "--version"): (127, "")},
            free=0,
        )
        self.assertEqual(sp.prerequisites(self.config(["served", "cross-warm"]), bare), ([], {}))
        # Installing from given wheels builds nothing, so it needs no cargo or space.
        wheels = self.config(["candidate"], wheels=self.base / "small")
        problems, _ = sp.prerequisites(wheels, FakeHost(tools={"git"}, free=0))
        self.assertEqual(problems, [])

    def test_the_harness_needs_a_small_tree_and_every_tree_must_be_a_directory(self) -> None:
        (self.base / "file").write_text("x", encoding="utf-8")
        trees = self.trees(small=None, medium=self.base / "file", peers=(self.base / "gone",))
        problems, _ = sp.prerequisites(self.config(trees=trees), FakeHost())
        self.assertEqual(len(problems), 3, problems)
        self.assertIn("FDU_QA_SMALL (--small) is not set", problems[0])
        self.assertIn(
            f"FDU_QA_MEDIUM names {self.base / 'file'}, which is not a directory", problems[1]
        )
        self.assertIn("FDU_QA_PEER_TREES names", problems[2])

    def test_a_missing_peer_skips_peer_agreement_and_names_it(self) -> None:
        host = FakeHost(tools=ALL_TOOLS - {"dua", "diskus"})
        problems, skips = sp.prerequisites(self.config(), host)
        self.assertEqual(problems, [])
        self.assertEqual(set(skips), {"peer-self-test", "peer-trees"})
        self.assertIn("not installed: dua, diskus", skips["peer-self-test"])

    def test_without_gnu_du_peer_agreement_is_skipped(self) -> None:
        host = FakeHost(answers={("du", "--version"): (1, "usage: du")})
        _, skips = sp.prerequisites(self.config(), host)
        self.assertIn("GNU du, the reference, is not installed", skips["peer-trees"])
        # macOS's coreutils name counts, as it does for the peer script.
        host = FakeHost(
            tools=ALL_TOOLS | {"gdu"},
            answers={("du", "--version"): (1, ""), ("gdu", "--version"): (0, "(GNU coreutils)")},
        )
        self.assertEqual(sp.prerequisites(self.config(), host)[1], {})

    def test_unset_trees_skip_only_the_steps_that_need_them(self) -> None:
        trees = self.trees(peers=(), progress=None, progress_analyze=None)
        _, skips = sp.prerequisites(self.config(trees=trees), FakeHost())
        self.assertEqual(set(skips), {"peer-trees", "pty-probe"})
        self.assertIn("FDU_QA_PEER_TREES is not set", skips["peer-trees"])

    def test_as_root_without_setpriv_or_nobody_the_refusal_steps_are_skipped(self) -> None:
        for host, reason in (
            (FakeHost(euid=0, tools=ALL_TOOLS - {"setpriv"}), "without setpriv"),
            (FakeHost(euid=0, accounts={}), "no nobody account"),
        ):
            with self.subTest(reason=reason):
                problems, skips = sp.prerequisites(self.config(), host)
                self.assertEqual(problems, [])
                self.assertEqual(set(skips), set(sp.AS_NOBODY))
                self.assertIn(reason, skips["refusals"])

    def test_as_root_nobody_must_reach_the_work_directory(self) -> None:
        private = self.base / "private"
        private.mkdir(mode=0o700)
        config = self.config(work=private / "work")
        problems, skips = sp.prerequisites(config, FakeHost(euid=0))
        self.assertEqual(skips, {})
        # The checkout's own reachability depends on where the test runs; the work
        # directory's does not.
        self.assertTrue(all("is not reachable by nobody" in problem for problem in problems))
        self.assertTrue(any(f"the work directory, {private}" in problem for problem in problems))

    def test_an_unprivileged_run_needs_no_nobody(self) -> None:
        private = self.base / "private"
        private.mkdir(mode=0o700)
        host = FakeHost(tools=ALL_TOOLS - {"setpriv"}, accounts={})
        self.assertEqual(sp.prerequisites(self.config(work=private), host), ([], {}))


class VersionTests(unittest.TestCase):
    IDENTITY = sp.Identity(SHA, TREE, "0.3.0", tagged=False)

    def test_the_version_must_name_the_commit_and_be_clean(self) -> None:
        self.assertTrue(sp.version_names_commit(VERSION, self.IDENTITY, bare_ok=False))
        self.assertFalse(sp.version_names_commit(VERSION + ".dirty", self.IDENTITY, bare_ok=False))
        other = "fdu 0.3.0-dev+g123456789"
        self.assertFalse(sp.version_names_commit(other, self.IDENTITY, bare_ok=True))

    def test_a_bare_version_is_the_commit_only_when_stamped_as_the_release(self) -> None:
        self.assertFalse(sp.version_names_commit("fdu 0.3.0", self.IDENTITY, bare_ok=False))
        self.assertTrue(sp.version_names_commit("fdu 0.3.0", self.IDENTITY, bare_ok=True))
        tagged = replace(self.IDENTITY, tagged=True)
        self.assertTrue(sp.version_names_commit("fdu 0.3.0", tagged, bare_ok=False))
        self.assertFalse(sp.version_names_commit("fdu 0.2.9", tagged, bare_ok=True))


class RedactionTests(unittest.TestCase):
    def test_default_labels(self) -> None:
        home = "/home/alice"
        self.assertEqual(sp.default_label("/", home), "/")
        self.assertEqual(sp.default_label("/usr", home), "/usr")
        self.assertEqual(sp.default_label("/Applications/", home), "/Applications")
        self.assertEqual(sp.default_label("/home/alice/.rustup", home), "~/.rustup")
        self.assertEqual(sp.default_label("/home/alice", home), "~")
        # Another home directory's name is a user's name.
        self.assertEqual(sp.default_label("/home/bob", home), "<home>")
        self.assertEqual(sp.default_label("/root", home), "<home>")
        self.assertEqual(sp.default_label("/home/bob/subjects/linux-v6.12", home), "linux-v6.12")
        self.assertEqual(sp.default_label("/tmp/fdu/tree", home), "tree")

    def test_known_paths_become_their_labels_longest_first(self) -> None:
        state = {
            "paths": {
                "trees": {"small": "/data/w/worktree", "medium": "/srv/src/linux"},
                "work": "/data/w",
                "worktree": "/data/w/worktree",
                "root": "/data/checkout",
                "tmp": "/tmp",
                "labels": [["/srv/src/linux", "linux tree"]],
            },
            "trees_base": "/tmp/fdu-trees-abc",
        }
        pairs = sp.label_pairs(state, "/home/alice")
        text = (
            "/data/w/worktree/crates /data/w/logs/x.log `/srv/src/linux` /srv/src/linux2 "
            "/tmp/fdu-trees-abc/served /tmp/other /data/checkout/scripts"
        )
        self.assertEqual(
            sp.redact(text, pairs, "/home/alice"),
            "<fdu checkout>/crates <work>/logs/x.log `linux tree` /srv/src/linux2 "
            "<trees>/served <tmp>/other <fdu checkout>/scripts",
        )

    def test_anything_left_that_names_a_home_is_replaced(self) -> None:
        text = (
            "### `/tmp/x/fdu-peer-agreement-k3j2_x/probe root`\n"
            "⠴ /home/alice/src  Scanning\n/Users/bob/Library /root/.cache /rootless"
        )
        self.assertEqual(
            sp.redact(text, [], "/home/alice"),
            "### `<scratch>/probe root`\n⠴ ~/src  Scanning\n<home>/Library <home>/.cache /rootless",
        )


PEER_OUTPUT = """## Peer agreement (fdu 0.3.0-dev+ge808f9604, 2026-09-30)

### `{root}`

fdu read 44.0 KiB allocated.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 68.0 KiB | +24.0 KiB | +24.0 KiB | directories | agrees exactly | none | 0.0 s |
| dust | apparent | 1.0 GiB | +21.0 KiB | +21.0 KiB | directories | agrees exactly | none | 0.0 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 0 of 4.

Every reading is explained."""

REFUSALS = """case                     cold   warm   only  warm source      only source  fresh   verdict
--------------------------------------------------------------------------------------------------------
default                     2      2      1  cold_scan        -            -       withheld
summary                     2      2      1  cold_scan        -            -       withheld

answer mismatches: 0
mechanism failures (cache did not serve): 0
stale reference instants: 0
cases the snapshot served: 0 of 2"""

SERVED = REFUSALS.replace("2      2      1  cold_scan        -            -       withheld",
                          "0      0      0  cold_scan        cache_only   stale   ok").replace(
    "served: 0 of 2", "served: 2 of 2")  # fmt: skip

NO_SNAPSHOT = REFUSALS.replace("withheld", "NO-SNAPSHOT(rc=1)").replace(
    "mechanism failures (cache did not serve): 0", "mechanism failures (cache did not serve): 2"
)

PARTIAL_STORED = REFUSALS.replace("withheld", "PARTIAL-STORED")

CROSS = """warmer   ask                   rc  analysis.analyze           verdict
----------------------------------------------------------------------------------
W_none   a_lines                0  ['lines']                  ok
W_lines  a_lines                0  ['lines']                  {verdict}

cross-warm violations: {violations}"""


class SummaryTests(unittest.TestCase):
    def test_warm_cold_tables_are_counted(self) -> None:
        self.assertEqual(
            sp.warm_cold_summary(REFUSALS, refusals=True),
            "2 of 2 cases partial and withheld; 0 answer mismatches, 0 mechanism failures, "
            "0 stale reference instants",
        )
        self.assertTrue(
            sp.warm_cold_summary(SERVED, refusals=False).startswith(
                "2 of 2 served `cache_only` and labelled `stale`; 0 answer mismatches"
            )
        )

    def test_cross_warm_and_peer_outputs_are_counted(self) -> None:
        cross = CROSS.format(verdict="ok", violations=0)
        self.assertEqual(
            sp.cross_warm_summary(cross), "2 of 2 pairs matched the cold answer; 0 violations"
        )
        self.assertEqual(
            sp.peer_summary(PEER_OUTPUT.format(root="/t"), 0),
            "2 of 2 readings on 1 trees agree exactly, 0 `UNEXPLAINED`; Every reading is "
            "explained; exit status 0",
        )

    def test_a_log_body_drops_only_the_pass_header_and_footer(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            log = Path(temporary) / "x.log"
            log.write_text(
                "== commit: x\n== start: y\nline one\n== inside\nlast\n== exit status: 0\n"
            )
            self.assertEqual(sp.body(log), "line one\n== inside\nlast")
            self.assertEqual(sp.body(Path(temporary) / "missing.log"), "")


def correctness_script(broken_exit: int = 1) -> Script:
    """Canned output for each command the correctness steps run."""

    def answer(args: list[str], env: Mapping[str, str]) -> tuple[int, str]:
        script = Path(next(arg for arg in args if arg.endswith(".py"))).name
        wrapper = Path(env.get("FDU_BIN", "")).name
        if script == "build_tree.py":
            return 0, "running as uid 1000\n\nkinds present: 15, absent: 1\nabsent: chardev"
        if wrapper == "break_no_snapshot.py":
            if script == "cross_warm.py":
                return broken_exit, CROSS.format(verdict="NOT-WARM(scanned)", violations=1)
            return broken_exit, NO_SNAPSHOT
        if wrapper == "break_partial_stored.py":
            return broken_exit, PARTIAL_STORED
        if script == "cross_warm.py":
            return 0, CROSS.format(verdict="ok", violations=0)
        return 0, REFUSALS if "--refusals-only" in args else SERVED

    return answer


class RunTests(Scratch):
    """The steps' verdicts, the state, the exit status, and the report, end to end."""

    def install(self, work: Path) -> None:
        """A candidate a previous run installed and verified."""
        work.mkdir(parents=True, exist_ok=True)
        (work / "bin").mkdir()
        state = {
            "commit": SHA,
            "tree": TREE,
            "version": "0.3.0",
            "steps": {"candidate": {"status": "passed", "detail": "installed", "logs": []}},
            "candidate": {"fdu": str(work / "bin" / "fdu"), "version": VERSION, "wheels": []},
        }
        (work / "state.json").write_text(json.dumps(state), encoding="utf-8")

    def run_pass(self, host: FakeHost, *only: str) -> tuple[int, dict[str, Any]]:
        work = self.base / "work"
        argv = ["--commit", SHA, "--work-dir", str(work), "--date", "2026-10-01"]
        for item in only:
            argv += ["--only", item]
        status = quietly(lambda: sp.main(argv, host))
        return status, json.loads((work / "state.json").read_text(encoding="utf-8"))

    def test_the_correctness_stage_and_both_breaks_pass(self) -> None:
        self.install(self.base / "work")
        host = FakeHost(script=correctness_script())
        status, state = self.run_pass(host, "correctness")
        self.assertEqual(status, 0, state["steps"])
        for step in sp.STAGES["correctness"]:
            self.assertEqual(state["steps"][step]["status"], "passed", step)
        self.assertEqual(
            state["steps"]["break-no-snapshot"]["detail"],
            "`warm_cold.py` exited 1, 2 rows `NO-SNAPSHOT`; "
            "`cross_warm.py` exited 1, 1 rows `NOT-WARM(scanned)`",
        )
        self.assertEqual(state["kinds"]["tree"], "15 kinds present, 1 absent (chardev)")
        # Each tree is built once, and removed once the pass is done with it.
        builds = [args for args, _ in host.executed if args[1].endswith("build_tree.py")]
        self.assertEqual(len(builds), 2)
        self.assertFalse(Path(state["trees_base"]).exists())

    def test_a_break_that_does_not_fail_its_script_fails_the_pass(self) -> None:
        self.install(self.base / "work")
        status, state = self.run_pass(FakeHost(script=correctness_script(broken_exit=0)), "breaks")
        self.assertEqual(status, sp.EXIT_FAILED)
        self.assertEqual(state["steps"]["break-no-snapshot"]["status"], "failed")
        self.assertEqual(state["steps"]["break-partial-stored"]["status"], "failed")

    def test_the_refusal_steps_run_as_nobody_when_running_as_root(self) -> None:
        self.install(self.base / "work")
        host = FakeHost(euid=0, answers={("setpriv",): (0, "")}, script=correctness_script())
        # Whether nobody can reach these paths is the prerequisite tests' question.
        with mock.patch.object(sp, "traversable", return_value=True):
            status, _ = self.run_pass(host, "refusals")
        self.assertEqual(status, 0)
        (refusal,) = [args for args, _ in host.executed if "--refusals-only" in args]
        self.assertEqual(
            refusal[:4], ["setpriv", "--reuid=65534", "--regid=65534", "--clear-groups"]
        )

    def test_a_skipped_step_leaves_the_pass_incomplete(self) -> None:
        self.install(self.base / "work")
        host = FakeHost(euid=0, tools=ALL_TOOLS - {"setpriv"}, script=correctness_script())
        status, state = self.run_pass(host, "refusals,served")
        self.assertEqual(status, sp.EXIT_INCOMPLETE)
        self.assertEqual(state["steps"]["refusals"]["status"], "skipped")
        self.assertEqual(state["steps"]["served"]["status"], "passed")

    def test_a_missing_prerequisite_runs_nothing(self) -> None:
        self.install(self.base / "work")
        host = FakeHost(answers={(sp.GNU_TIME, "--version"): (127, "")})
        status, state = self.run_pass(host, "harness")
        self.assertEqual(status, sp.EXIT_USAGE)
        self.assertNotIn("harness", state["steps"])
        self.assertEqual(host.executed, [])

    def test_a_qa_step_without_a_candidate_installs_one_first(self) -> None:
        def build(args: list[str], env: Mapping[str, str]) -> tuple[int, str]:
            if args[:3] == ["uv", "tool", "install"]:
                bin_dir = Path(env["UV_TOOL_BIN_DIR"])
                (bin_dir / "fdu").write_text("", encoding="utf-8")
            return 0, ""

        wheels = self.base / "wheels"
        wheels.mkdir()
        host = FakeHost(script=build)
        work = self.base / "work"
        argv = ["--commit", SHA, "--work-dir", str(work), "--wheels", str(wheels)]
        status = quietly(lambda: sp.main([*argv, "--only", "terminal"], host))
        state = json.loads((work / "state.json").read_text(encoding="utf-8"))
        self.assertEqual(status, 0, state["steps"])
        self.assertEqual(list(state["steps"]), ["candidate", "terminal"])
        self.assertEqual(state["candidate"]["version"], VERSION)
        install = next(args for args, _ in host.executed if args[:2] == ["uv", "tool"])
        self.assertIn("--no-index", install)
        self.assertEqual(install[-1], "fdu")

    def run_gates(self, check_status: int) -> tuple[int, dict[str, Any], FakeHost, Path]:
        def make(args: list[str], env: Mapping[str, str]) -> tuple[int, str]:
            if args[-1] == "semver-check":
                return 0, "fdu 0.3.0: starts a new compatibility series; nothing to check"
            return (check_status if args[-1] == "check" else 0), "OK"

        work = self.base / "work"
        # The worktree `git worktree add` would have made.
        (work / "worktree").mkdir(parents=True)
        (work / "target").mkdir()
        host = FakeHost(script=make)
        argv = ["--commit", SHA, "--work-dir", str(work), "--only", "gates"]
        status = quietly(lambda: sp.main([*argv, "--wrap", "flock -s /lock"], host))
        state = json.loads((work / "state.json").read_text(encoding="utf-8"))
        return status, state, host, work

    def test_the_gates_run_in_the_worktree_with_their_own_target(self) -> None:
        status, state, host, work = self.run_gates(check_status=0)
        self.assertEqual(status, 0, state["steps"])
        self.assertEqual([args for args, _ in host.executed], [
            ["flock", "-s", "/lock", "make", gate] for gate in sp.GATES
        ])  # fmt: skip
        for _, env in host.executed:
            self.assertEqual(env["CARGO_TARGET_DIR"], str(work / "target"))
        self.assertEqual(
            state["steps"]["semver-check"]["detail"],
            "`make semver-check` exited 0: fdu 0.3.0: starts a new compatibility series; "
            "nothing to check",
        )
        # Nothing failed, so the worktree and target are gone; the records stay.
        self.assertFalse((work / "worktree").exists())
        self.assertFalse((work / "target").exists())
        self.assertEqual(len(list(work.glob("report-*-release-0.3.0-stability-pass.md"))), 1)
        self.assertTrue((work / "summary-tables.md").exists())
        log = (work / "logs" / "gate-check.log").read_text(encoding="utf-8")
        self.assertIn(f"== commit: {SHA} tree: {TREE}", log)
        self.assertIn("== gate: make check", log)

    def test_a_failed_gate_keeps_the_worktree_and_the_other_gates_still_run(self) -> None:
        status, state, host, work = self.run_gates(check_status=2)
        self.assertEqual(status, sp.EXIT_FAILED)
        self.assertEqual(state["steps"]["check"]["status"], "failed")
        self.assertEqual(state["steps"]["check"]["exits"], [2])
        self.assertEqual(len(host.executed), len(sp.GATES))
        self.assertTrue((work / "worktree").exists())
        self.assertTrue((work / "target").exists())

    def test_a_work_directory_belongs_to_one_commit(self) -> None:
        work = self.base / "work"
        work.mkdir()
        (work / "state.json").write_text(json.dumps({"commit": "0" * 40}), encoding="utf-8")
        stderr = io.StringIO()
        with redirect_stdout(io.StringIO()), redirect_stderr(stderr):
            status = sp.main(["--commit", SHA, "--work-dir", str(work)], FakeHost())
        self.assertEqual(status, sp.EXIT_USAGE)
        self.assertIn("holds the pass for commit 000000000", stderr.getvalue())


class ReportTests(Scratch):
    """The dated report and the summary tables, in the 0.3.0 record's shapes."""

    def write_pass(self, failed: str | None = None, skipped: str | None = None) -> Path:
        work = self.base / "work"
        logs = work / "logs"
        logs.mkdir(parents=True)
        private = str(self.base / "small")
        outputs = {
            "gate-check": ("check", "OK"),
            "gate-semver-check": ("semver-check", "fdu 0.3.0: starts a new compatibility series"),
            "peer-self-test": (
                "peer-self-test",
                PEER_OUTPUT.format(root=f"{work}/scratch/fdu-peer-agreement-ab12/probe root"),
            ),
            "peer-trees": ("peer-trees", PEER_OUTPUT.format(root=private)),
            "pty-probe": ("pty-probe", f"ok   a frame\n     sample frame: ⠴ {private}  Scanning"),
            "correctness-refusals": ("refusals", REFUSALS),
        }
        steps: dict[str, Any] = {}
        for log, (step, output) in outputs.items():
            path = logs / f"{log}.log"
            path.write_text(f"== commit: {SHA}\n{output}\n== exit status: 0\n", encoding="utf-8")
            steps[step] = {"status": "passed", "detail": f"{step} detail", "logs": [str(path)]}
        for step in sp.STEPS:
            steps.setdefault(step, {"status": "passed", "detail": f"{step} detail", "logs": []})
        if failed:
            steps[failed] = {"status": "failed", "detail": "it broke", "logs": []}
        if skipped:
            steps[skipped] = {"status": "skipped", "detail": "not installed: dua", "logs": []}
        rows = [
            {"phase": "sanity", "name": "help", "verdict": "ok"},
            {"phase": "views", "name": "documents-no-analyze", "verdict": "warn"},
            {"phase": "cache-analyze", "name": "on-code-2", "verdict": "ok"},
            {"phase": "large", "name": "large-summary-depth1", "verdict": "ok"},
        ]
        (work / "qa").mkdir()
        (work / "qa" / "results.json").write_text(json.dumps({"rows": rows}), encoding="utf-8")
        (work / "qa" / "results.md").write_text(
            "| Phase | Name |\n| --- | --- |\n| sanity | help |\n", encoding="utf-8"
        )
        state: dict[str, Any] = {
            "commit": SHA,
            "tree": TREE,
            "version": "0.3.0",
            "steps": steps,
            "candidate": {"fdu": f"{work}/bin/fdu", "version": VERSION, "wheels": ["fdu.whl"]},
            "paths": {"trees": {"small": private}, "work": str(work), "tmp": "/tmp"},
            "labels": {"small": "small"},
            "variables": {"FDU_QA_SMALL": "small"},
            "regime": {"system": "Linux x86_64", "user": "root", "peers": {"GNU du": "du 9.4"}},
        }
        (work / "state.json").write_text(json.dumps(state), encoding="utf-8")
        return work

    def written(self, work: Path) -> tuple[str, str]:
        state = json.loads((work / "state.json").read_text(encoding="utf-8"))
        report, tables = sp.write_report(work, state, "2026-10-01", "/home/nobody-here")
        self.assertEqual(report.name, "report-2026-10-01-release-0.3.0-stability-pass.md")
        return report.read_text(encoding="utf-8"), tables.read_text(encoding="utf-8")

    def test_the_report_has_the_records_sections_and_no_private_path(self) -> None:
        report, tables = self.written(self.write_pass())
        self.assertTrue(report.startswith("# 0.3.0 Stability Pass — 2026-10-01\n"))
        for heading in ("## Verdict", "## Regime", "## Reproduce", "## Full Tables"):
            self.assertIn(f"\n{heading}\n", report)
        self.assertIn(f"The pass tested commit `{SHA}`, tree `{TREE}`.", report)
        self.assertIn("Nothing failed, and nothing was skipped.", report)
        self.assertIn("| `make check` | Passed: check detail |", report)
        self.assertIn("| Correctness, refusal tree as `nobody` | Passed: refusals detail |", report)
        self.assertIn("#### `<scratch>/probe root`", report)
        self.assertIn("#### `small`", report)
        self.assertIn("sample frame: ⠴ small  Scanning", report)
        self.assertIn("```\ncase ", report)
        self.assertIn("export FDU_QA_SMALL=small", report)
        self.assertTrue(report.endswith("-->\n"))
        for text in (report, tables):
            self.assertNotIn(str(self.base), text)

    def test_the_summary_tables_take_the_playbook_and_runbook_shapes(self) -> None:
        _, tables = self.written(self.write_pass())
        self.assertIn("| Phase | Status | Notes |", tables)
        self.assertIn("| Phase 1: Setup | ✅ Passed | candidate detail |", tables)
        self.assertIn(
            "| Phase 2: Small-tree views | ✅ Passed | 2 checks: 1 ok, 1 warn "
            "(`documents-no-analyze`) |",
            tables,
        )
        self.assertIn("| Phase 4: Medium tree | ⏸️ Blocked | No rows; the harness skipped", tables)
        self.assertIn("| Phase 6: Terminal Progress | ⏳ Pending |", tables)
        self.assertIn("| Phase 7: Peer agreement | ✅ Passed | du 9.4.", tables)
        self.assertIn("| Pass | Result |", tables)
        self.assertIn("| `cross_warm.py`, complete tree | cross-warm detail |", tables)
        self.assertIn("- **Partial answer stored.**", tables)

    def test_failures_skips_and_steps_not_run_are_named(self) -> None:
        work = self.write_pass(failed="check", skipped="peer-trees")
        state = json.loads((work / "state.json").read_text(encoding="utf-8"))
        del state["steps"]["served"]
        (work / "state.json").write_text(json.dumps(state), encoding="utf-8")
        report, tables = self.written(work)
        self.assertIn("Failed: check. Skipped: peer-trees. Not run: served.", report)
        self.assertIn("| `make check` | Failed: it broke |", report)
        self.assertIn("| Correctness, served tree | Not run |", report)
        self.assertIn("| Phase 7: Peer agreement | ⏸️ Blocked |", tables)


if __name__ == "__main__":
    unittest.main()
