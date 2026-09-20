"""Run the path-independence matrix and judge each answer against a cold run.

The invariant (see the explicit core models plan, "The Rule"): for a request, tree
state, delivery, and any history, a run returns the cold run's content and tree status,
a named failure, or, under `--cache only`, a labelled stale answer equal to a cold run at
an earlier state. The nested `provenance` object is excluded from the comparison; the
request, status, and all answer content remain compared. Every route returns the same
kind of outcome for the same request, delivery, and history.

Usage:
    python tests/path_independence/runner.py [--tier subset|full] [--surfaces cli,python]
        [--record [PATH]] [--judged PATH] [--out DIR]
"""

from __future__ import annotations

import argparse
import difflib
import json
import os
import queue
import re
import shutil
import subprocess
import sys
import tempfile
import threading
from collections import defaultdict
from collections.abc import Callable, Iterable, Iterator
from concurrent.futures import ThreadPoolExecutor
from contextlib import contextmanager
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Literal

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
sys.path.insert(0, str(HERE))

import matrix  # noqa: E402
import registry  # noqa: E402
from fixture import FixtureFacts, build_fixture, copy_fixture  # noqa: E402

Outcome = Literal["complete", "partial", "failure"]

# The one failure `--cache only` may answer with: no stored state serves the request.
# The command line exits 1 with this message, and Python raises fdu.FduError with it.
CACHE_MISS = "snapshot is not usable"

# Exit codes a Python route reports, mirroring the command line's: 1 for fdu's own
# error, 2 for a refused request (ValueError), and 3 for anything else, which is never
# a named failure.
PY_FDU_ERROR, PY_REFUSED, PY_UNEXPECTED = 1, 2, 3
VerdictKind = Literal["same", "stale", "refused", "differs", "outcome_class"]
ALLOWED: frozenset[str] = frozenset({"same", "stale", "refused"})


@dataclass(frozen=True)
class Invocation:
    """What one fdu run returned, on either surface."""

    route: str
    command: str
    exit: int
    stderr: str
    answer: dict[str, Any] | None

    @property
    def outcome(self) -> Outcome:
        if self.answer is None:
            return "failure"
        schema = self.answer.get("schema")
        status = self.answer.get("status")
        complete = status.get("complete") if isinstance(status, dict) else None
        if not isinstance(schema, str) or not schema.startswith("fdu.report/"):
            return "failure"
        if not isinstance(complete, bool):
            return "failure"
        return "complete" if complete else "partial"


@dataclass(frozen=True)
class Verdict:
    """How a measured invocation compares with the cold oracle."""

    kind: VerdictKind
    paths: tuple[str, ...] = ()
    sample: tuple[str, ...] = ()

    @property
    def allowed(self) -> bool:
        return self.kind in ALLOWED


@dataclass(frozen=True)
class CaseResult:
    """One judged case: its key, verdict, and the invocations behind it."""

    key: str
    verdict: Verdict
    oracle: Invocation | None = None
    measured: Invocation | None = None
    history: tuple[str, ...] = ()


@dataclass
class RunResult:
    """Everything a tier run produced."""

    tier: str
    cases: list[CaseResult] = field(default_factory=list)
    cold_answers: int = 0
    # Whether this run executed every case the registry may name: the full tier, both
    # surfaces, and a fixture with symlinks. Only such a run may declare an entry dead.
    complete_matrix: bool = False


def case_key(phase: str, route: str, policy: str, history: str, mutation: str, request: str) -> str:
    """The registry key for one case: `phase/route/policy/history/mutation/request`."""
    return "/".join((phase, route, policy, history or "-", mutation or "-", request))


ROOT_PLACEHOLDER = "<root>"


def normalize(answer: dict[str, Any]) -> tuple[dict[str, Any], dict[str, Any]]:
    """Split an answer into compared content and excluded provenance.

    The root names where the tree was read, not what it contains: a copy of the tree
    lives elsewhere, and each platform spells a path its own way. So the answer's own
    root string is replaced by a placeholder wherever it appears.
    """
    root = answer.get("root")
    if isinstance(root, str) and root:
        encoded = json.dumps(answer).replace(json.dumps(root)[1:-1], ROOT_PLACEHOLDER)
        answer = json.loads(encoded)
    content = dict(answer)
    provenance = content.pop("provenance", {})
    if not isinstance(provenance, dict):
        raise TypeError("report provenance must be an object")
    return content, provenance


# Keys that identify an element of a list of objects: a file's path, a metric row's id,
# a tree node's name. Lists are aligned by them, so one inserted file reads as one
# difference rather than as every later element compared with its neighbour.
IDENTITY_KEYS = ("path", "id", "name")


def _identity_key(items: list[Any]) -> str | None:
    if not all(isinstance(item, dict) for item in items):
        return None
    for key in IDENTITY_KEYS:
        values = [item.get(key) for item in items]
        if all(isinstance(value, str | int) for value in values) and len(set(values)) == len(
            values
        ):
            return key
    return None


def json_diff(left: Any, right: Any, path: str = "") -> list[tuple[str, Any, Any]]:
    """Every leaf where two JSON values differ, as (path, left, right)."""
    if isinstance(left, dict) and isinstance(right, dict):
        out: list[tuple[str, Any, Any]] = []
        for key in sorted(set(left) | set(right)):
            child = f"{path}.{key}" if path else key
            if key not in left:
                out.append((child, "<absent>", right[key]))
            elif key not in right:
                out.append((child, left[key], "<absent>"))
            else:
                out += json_diff(left[key], right[key], child)
        return out
    if isinstance(left, list) and isinstance(right, list):
        key = _identity_key(left) if left else _identity_key(right)
        if key is not None and (not left or not right or key == _identity_key(right)):
            return _aligned_diff(left, right, key, path)
        out = []
        for index in range(max(len(left), len(right))):
            child = f"{path}[{index}]"
            if index >= len(left):
                out.append((child, "<absent>", right[index]))
            elif index >= len(right):
                out.append((child, left[index], "<absent>"))
            else:
                out += json_diff(left[index], right[index], child)
        return out
    return [] if left == right else [(path, left, right)]


def _aligned_diff(
    left: list[dict[str, Any]], right: list[dict[str, Any]], key: str, path: str
) -> list[tuple[str, Any, Any]]:
    by_left = {item[key]: item for item in left}
    by_right = {item[key]: item for item in right}
    out: list[tuple[str, Any, Any]] = []
    shared_left = [item[key] for item in left if item[key] in by_right]
    shared_right = [item[key] for item in right if item[key] in by_left]
    if shared_left != shared_right:
        out.append((f"{path}<order>", shared_left, shared_right))
    for identity in [item[key] for item in left] + [
        item[key] for item in right if item[key] not in by_left
    ]:
        child = f"{path}[{key}={json.dumps(identity)}]"
        if identity not in by_right:
            out.append((child, by_left[identity], "<absent>"))
        elif identity not in by_left:
            out.append((child, "<absent>", by_right[identity]))
        else:
            out += json_diff(by_left[identity], by_right[identity], child)
    return out


_ELEMENT = re.compile(r'\[(?:\d+|[a-z_]+=(?:-?\d+|"(?:[^"\\]|\\.)*"))\]')


def generalize(path: str) -> str:
    """A diff path with list positions and identities erased."""
    return _ELEMENT.sub("[]", path)


def compare(
    oracle: Invocation,
    measured: Invocation,
    *,
    policy: str,
    earlier: Invocation | None = None,
) -> Verdict:
    """Judge `measured` against the cold `oracle`.

    `earlier` is a cold run at the state before a mutation; under `--cache only` an answer
    equal to it is the allowed stale outcome, provided it says so.
    """
    if measured.outcome == "failure":
        if oracle.outcome == "failure":
            same_surface = is_python(oracle) == is_python(measured)
            if (oracle.exit, oracle.stderr if same_surface else "") == (
                measured.exit,
                measured.stderr if same_surface else "",
            ):
                return Verdict("same")
            return Verdict("differs", ("<error>",), (oracle.stderr, measured.stderr))
        if policy == "only" and is_cache_miss(measured):
            return Verdict("refused")
        return Verdict("outcome_class", (f"<outcome:{oracle.outcome}>{measured.outcome}>",))
    if oracle.outcome == "failure":
        return Verdict("outcome_class", (f"<outcome:{oracle.outcome}>{measured.outcome}>",))

    assert oracle.answer is not None and measured.answer is not None
    oracle_content, _ = normalize(oracle.answer)
    measured_content, provenance = normalize(measured.answer)
    diff = json_diff(oracle_content, measured_content)
    if not diff:
        return Verdict("same")
    if policy == "only" and earlier is not None and earlier.answer is not None:
        earlier_content, _ = normalize(earlier.answer)
        if earlier_content == measured_content:
            if provenance["freshness"] == "stale":
                return Verdict("stale")
            return Verdict(
                "differs",
                ("provenance.freshness",),
                (f"freshness={provenance['freshness']}",),
            )
    sample = tuple(f"{p}: {json.dumps(a)} -> {json.dumps(b)}" for p, a, b in diff[:12])
    return Verdict("differs", tuple(sorted({generalize(p) for p, _, _ in diff})), sample)


def is_python(invocation: Invocation) -> bool:
    return invocation.route in matrix.PY_ROUTES


def is_cache_miss(invocation: Invocation) -> bool:
    """Whether a failure is the named cache-only miss rather than a crash or other error."""
    if invocation.answer is not None or invocation.exit != 1:
        return False
    prefix = "FduError: " if is_python(invocation) else "fdu: "
    return invocation.stderr.startswith(prefix + CACHE_MISS)


@dataclass(frozen=True)
class Surfaces:
    """The builds under test: an fdu executable and, optionally, a Python with fdu."""

    fdu_bin: Path
    python: Path | None


def _cache_env(xdg: Path) -> dict[str, str]:
    xdg.mkdir(parents=True, exist_ok=True)
    # Each invocation gets its own cache home; `user_cache_dir` honors XDG_CACHE_HOME on
    # every platform, so no run reads another's stored state.
    return dict(os.environ, XDG_CACHE_HOME=str(xdg))


def run_cli(
    surfaces: Surfaces, root: Path, request: matrix.Spec, policy: str, xdg: Path
) -> Invocation:
    """Ask `request` on the command line."""
    argv = [str(surfaces.fdu_bin), str(root), "--format", "json", "--cache", policy]
    argv += matrix.cli_args(request)
    done = subprocess.run(argv, capture_output=True, text=True, env=_cache_env(xdg), check=False)
    try:
        answer = json.loads(done.stdout)
    except json.JSONDecodeError:
        answer = None
    command = " ".join([*argv[:1], "<root>", *argv[2:]])
    return Invocation(matrix.CLI_ROUTE, command, done.returncode, done.stderr.strip(), answer)


def run_cli_watch_initial(
    surfaces: Surfaces, root: Path, request: matrix.Spec, policy: str, xdg: Path
) -> Invocation:
    """Ask through the real watch route and retain its initial report only."""
    argv = [
        str(surfaces.fdu_bin),
        str(root),
        "--watch",
        "--format",
        "jsonl",
        "--cache",
        policy,
    ]
    argv += matrix.cli_args(request)
    process = subprocess.Popen(
        argv,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=_cache_env(xdg),
    )
    assert process.stdout is not None and process.stderr is not None
    parsed: queue.Queue[tuple[dict[str, Any] | None, str]] = queue.Queue(maxsize=1)

    def read_report() -> None:
        try:
            parsed.put((_read_jsonl_report(process.stdout), ""))
        except (EOFError, TypeError, ValueError, json.JSONDecodeError) as error:
            parsed.put((None, f"invalid watch initial report: {error}"))

    stderr_chunks: list[str] = []
    reader = threading.Thread(target=read_report, daemon=True)
    stderr_reader = threading.Thread(
        target=lambda: stderr_chunks.append(process.stderr.read()), daemon=True
    )
    reader.start()
    stderr_reader.start()
    timed_out = False
    try:
        answer, parse_error = parsed.get(timeout=30)
    except queue.Empty:
        answer, parse_error = None, "timed out waiting for watch initial report"
        timed_out = True
    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
        exit_code = 0 if answer is not None else process.returncode
    else:
        exit_code = process.returncode
    stderr_reader.join(timeout=5)
    stderr = "".join(stderr_chunks).strip()
    if parse_error:
        stderr = f"{stderr}\n{parse_error}".strip()
    if timed_out:
        exit_code = PY_UNEXPECTED
    command = " ".join([*argv[:1], "<root>", *argv[2:]])
    return Invocation(matrix.CLI_WATCH_ROUTE, command, exit_code, stderr, answer)


def _read_jsonl_report(stream: Iterable[str]) -> dict[str, Any]:
    """Read one complete JSONL report without consuming later watch records."""
    lines = iter(stream)
    first = next(lines, "")
    if not first:
        raise EOFError("watch exited before the report envelope")
    envelope = json.loads(first)
    if not isinstance(envelope, dict):
        raise TypeError("report envelope must be an object")
    request = envelope.get("request")
    views = request.get("views") if isinstance(request, dict) else None
    if not isinstance(views, list) or not all(isinstance(view, str) for view in views):
        raise ValueError("report request.views must be an array of strings")
    reports: list[dict[str, Any]] = []
    for expected_view in views:
        line = next(lines, "")
        if not line:
            raise EOFError(f"watch exited before the {expected_view} report section")
        section = json.loads(line)
        if not isinstance(section, dict) or section.get("view") != expected_view:
            raise ValueError(f"expected the {expected_view} report section")
        reports.append(section)
    return {**envelope, "reports": reports}


def run_py(
    surfaces: Surfaces,
    root: Path,
    request: matrix.Spec,
    policy: str,
    xdg: Path,
    route: str,
) -> Invocation:
    """Ask `request` through the Python package on `route`."""
    assert surfaces.python is not None and route in matrix.PY_ROUTES
    job = {
        "root": str(root),
        "mode": route.removeprefix("py-"),
        "cache": policy,
        "spec": request,
    }
    done = subprocess.run(
        [str(surfaces.python), str(HERE / "pyrun.py"), json.dumps(job)],
        capture_output=True,
        text=True,
        env=_cache_env(xdg),
        check=False,
    )
    command = f"{route} cache={policy} spec={json.dumps(request, sort_keys=True)}"
    try:
        envelope = json.loads(done.stdout)
    except json.JSONDecodeError:
        raise RuntimeError(f"pyrun.py failed ({done.returncode}): {done.stderr.strip()}") from None
    if envelope.get("refused"):
        raise RuntimeError(envelope["refused"])
    if envelope["ok"]:
        return Invocation(route, command, 0, "", envelope["answer"])
    exit_code = {"fdu": PY_FDU_ERROR, "refused": PY_REFUSED}.get(envelope["kind"], PY_UNEXPECTED)
    return Invocation(route, command, exit_code, envelope["error"], None)


def run_route(
    surfaces: Surfaces,
    route: str,
    root: Path,
    request: matrix.Spec,
    policy: str,
    xdg: Path,
) -> Invocation:
    if route == matrix.CLI_ROUTE:
        return run_cli(surfaces, root, request, policy, xdg)
    if route == matrix.CLI_WATCH_ROUTE:
        return run_cli_watch_initial(surfaces, root, request, policy, xdg)
    return run_py(surfaces, root, request, policy, xdg, route)


class Workspace:
    """Scratch space for one run: fixture copies and per-case cache homes."""

    def __init__(self, base: Path) -> None:
        self.base = base
        self._counter = 0
        self._lock = threading.Lock()

    def fresh(self, label: str) -> Path:
        with self._lock:
            self._counter += 1
            number = self._counter
        # Short names: Windows limits paths to 260 characters, and a cache home nests
        # fdu's own file names below this directory.
        path = self.base / "runs" / f"{number:06d}-{re.sub(r'[^A-Za-z0-9_.-]', '_', label)[:32]}"
        path.mkdir(parents=True)
        return path

    @staticmethod
    def discard(path: Path) -> None:
        shutil.rmtree(path, ignore_errors=True)


def _parallel(function: Callable[[Any], list[Any]], items: Iterable[Any]) -> list[Any]:
    workers = min(32, (os.cpu_count() or 4) * 2)
    with ThreadPoolExecutor(workers) as pool:
        return [result for results in pool.map(function, list(items)) for result in results]


class MatrixRun:
    """One tier over one fixture, producing judged cases."""

    def __init__(
        self,
        tier: matrix.Tier,
        surfaces: Surfaces,
        workspace: Workspace,
        facts: FixtureFacts,
    ) -> None:
        self.tier = tier
        self.surfaces = surfaces
        self.ws = workspace
        self.facts = facts
        self.cold: dict[str, Invocation] = {}

    def run(self) -> RunResult:
        result = RunResult(tier=self.tier.name)
        result.complete_matrix = (
            self.tier is matrix.FULL
            and self.surfaces.python is not None
            and self.facts.symlinks
            and self.facts.permissions
        )
        result.cases += self.phase_cold()
        result.cold_answers = sum(1 for inv in self.cold.values() if inv.outcome != "failure")
        result.cases += self.phase_warm()
        if self.tier.selfwarm:
            result.cases += self.phase_selfwarm()
        result.cases += self.phase_mutation()
        if self.surfaces.python is not None:
            result.cases += self.phase_cross()
        return result

    def _oracle(self, root: Path, request_id: str) -> Invocation:
        xdg = self.ws.fresh(f"oracle-{request_id}")
        try:
            return run_cli(self.surfaces, root, matrix.REQUESTS[request_id], "off", xdg)
        finally:
            self.ws.discard(xdg)

    def phase_cold(self) -> list[CaseResult]:
        """Cold oracles, and `auto` on an empty cache, which must equal them."""

        def one(request_id: str) -> list[CaseResult]:
            oracle = self._oracle(self.facts.root, request_id)
            self.cold[request_id] = oracle
            xdg = self.ws.fresh(f"cold-{request_id}")
            measured = run_cli(
                self.surfaces, self.facts.root, matrix.REQUESTS[request_id], "auto", xdg
            )
            self.ws.discard(xdg)
            verdict = compare(oracle, measured, policy="auto")
            key = case_key("cold", matrix.CLI_ROUTE, "auto", "-", "-", request_id)
            return [CaseResult(key, verdict, oracle, measured)]

        return _parallel(one, self.tier.requests)

    def _warm(self, root: Path, warmer_id: str, xdg: Path, route: str = matrix.CLI_ROUTE) -> str:
        request, policy = matrix.WARMERS[warmer_id]
        if route != matrix.CLI_ROUTE:
            policy = "auto"
        invocation = run_route(self.surfaces, route, root, request, policy, xdg)
        return invocation.command

    def phase_warm(self) -> list[CaseResult]:
        """Each request under each policy after each warmer."""
        cases = [
            (warmer, request, policy)
            for warmer in self.tier.warmers
            for request in self.tier.requests
            for policy in matrix.POLICIES
        ]

        def one(case: tuple[str, str, str]) -> list[CaseResult]:
            warmer, request_id, policy = case
            xdg = self.ws.fresh(f"warm-{warmer}-{request_id}-{policy}")
            warm_command = self._warm(self.facts.root, warmer, xdg)
            measured = run_cli(
                self.surfaces, self.facts.root, matrix.REQUESTS[request_id], policy, xdg
            )
            self.ws.discard(xdg)
            oracle = self.cold[request_id]
            key = case_key("warm", matrix.CLI_ROUTE, policy, warmer, "-", request_id)
            verdict = compare(oracle, measured, policy=policy)
            return [CaseResult(key, verdict, oracle, measured, (warm_command,))]

        return _parallel(one, cases)

    def phase_selfwarm(self) -> list[CaseResult]:
        """Each request after itself: `auto` twice, then `read-only`, then `only`."""

        def one(request_id: str) -> list[CaseResult]:
            request = matrix.REQUESTS[request_id]
            xdg = self.ws.fresh(f"self-{request_id}")
            history = [run_cli(self.surfaces, self.facts.root, request, "auto", xdg).command]
            results: list[CaseResult] = []
            for policy in matrix.POLICIES:
                measured = run_cli(self.surfaces, self.facts.root, request, policy, xdg)
                oracle = self.cold[request_id]
                key = case_key("selfwarm", matrix.CLI_ROUTE, policy, "self", "-", request_id)
                verdict = compare(oracle, measured, policy=policy)
                results.append(CaseResult(key, verdict, oracle, measured, tuple(history)))
                history.append(measured.command)
            self.ws.discard(xdg)
            return results

        return _parallel(one, self.tier.requests)

    def phase_mutation(self) -> list[CaseResult]:
        """Warm, change the tree, then ask every request under every policy."""
        pairs = [
            (mutation, warmer)
            for mutation in self.tier.mutations
            for warmer in self.tier.mutation_warmers
            if self._can_apply(matrix.MUTATIONS[mutation])
        ]

        # A cold answer on a mutated tree depends only on the mutation and the request,
        # so each is computed once, on a tree copy no warmer touched, and read afterwards.
        def build_oracles(mutation: str) -> list[tuple[tuple[str, str], Invocation]]:
            base = self.ws.fresh(f"oracle-{mutation}")
            tree = base / "tree"
            copy_fixture(self.facts, tree)
            with _mutated(tree, matrix.MUTATIONS[mutation]):
                built = [
                    (
                        (mutation, request_id),
                        _rerooted(self._oracle(tree, request_id), tree),
                    )
                    for request_id in self.tier.requests
                ]
            self.ws.discard(base)
            return built

        mutated_oracles = dict(
            _parallel(build_oracles, sorted({mutation for mutation, _ in pairs}))
        )

        def one(pair: tuple[str, str]) -> list[CaseResult]:
            mutation, warmer = pair
            base = self.ws.fresh(f"mutation-{mutation}-{warmer}")
            tree = base / "tree"
            copy_fixture(self.facts, tree)
            warmed = base / "xdg-warmed"
            history = (self._warm(tree, warmer, warmed),)
            results: list[CaseResult] = []
            with _mutated(tree, matrix.MUTATIONS[mutation]):
                for request_id in self.tier.requests:
                    for policy in matrix.POLICIES:
                        xdg = base / f"xdg-{request_id}-{policy}"
                        shutil.copytree(warmed, xdg)
                        invocation = run_cli(
                            self.surfaces,
                            tree,
                            matrix.REQUESTS[request_id],
                            policy,
                            xdg,
                        )
                        shutil.rmtree(xdg, ignore_errors=True)
                        measured = _rerooted(invocation, tree)
                        key = case_key(
                            "mutation",
                            matrix.CLI_ROUTE,
                            policy,
                            warmer,
                            mutation,
                            request_id,
                        )
                        oracle = mutated_oracles[(mutation, request_id)]
                        earlier = self.cold[request_id]
                        verdict = compare(oracle, measured, policy=policy, earlier=earlier)
                        results.append(CaseResult(key, verdict, oracle, measured, history))
            self.ws.discard(base)
            return results

        return _parallel(one, pairs)

    def _can_apply(self, mutation: matrix.Mutation) -> bool:
        return (self.facts.symlinks or not mutation.needs_symlinks) and (
            self.facts.permissions or not mutation.needs_permissions
        )

    def phase_cross(self) -> list[CaseResult]:
        """The Python routes against the command line, cold and after shared histories.

        Besides each answer against the cold oracle, every route that read after the
        same history under the same policy must return the same kind of outcome.
        """
        histories: list[tuple[str, str | None, str | None]] = [("cold", None, None)]
        for warmer in self.tier.cross_warmers:
            histories.append((f"{warmer}", warmer, matrix.CLI_ROUTE))
            histories.append((f"py-open:{warmer}", warmer, "py-open"))
        cases = [(history, request) for history in histories for request in self.tier.requests]

        def one(
            case: tuple[tuple[str, str | None, str | None], str],
        ) -> list[CaseResult]:
            (history_id, warmer, warm_route), request_id = case
            request = matrix.REQUESTS[request_id]
            oracle = self.cold[request_id]
            results: list[CaseResult] = []
            outcomes: dict[str, dict[str, Outcome]] = defaultdict(dict)
            readers = [matrix.CLI_ROUTE, matrix.CLI_WATCH_ROUTE, "py-report", "py-open"]
            policies: tuple[str, ...] = ("off",) if warmer is None else ("auto", "only")
            for policy in policies:
                for route in readers + (["py-scan"] if warmer is None else []):
                    xdg = self.ws.fresh(f"cross-{history_id}-{request_id}-{route}-{policy}")
                    history: tuple[str, ...] = ()
                    if warmer is not None and warm_route is not None:
                        history = (self._warm(self.facts.root, warmer, xdg, warm_route),)
                    measured = run_route(
                        self.surfaces, route, self.facts.root, request, policy, xdg
                    )
                    self.ws.discard(xdg)
                    outcomes[policy][route] = measured.outcome
                    key = case_key("cross", route, policy, history_id, "-", request_id)
                    verdict = compare(oracle, measured, policy=policy)
                    results.append(CaseResult(key, verdict, oracle, measured, history))
            for policy, by_route in outcomes.items():
                kinds = {outcome for outcome in by_route.values()}
                key = case_key("cross", "routes", policy, history_id, "-", request_id)
                if len(kinds) == 1:
                    results.append(CaseResult(key, Verdict("same")))
                else:
                    label = ",".join(f"{route}={by_route[route]}" for route in sorted(by_route))
                    results.append(
                        CaseResult(key, Verdict("outcome_class", (f"<outcomes:{label}>",)))
                    )
            return results

        return _parallel(one, cases)


@contextmanager
def _mutated(tree: Path, mutation: matrix.Mutation) -> Iterator[None]:
    """Apply `mutation` to `tree`, undoing afterwards what would block deleting it."""
    mutation.apply(tree)
    try:
        yield
    finally:
        if mutation.restore is not None:
            mutation.restore(tree)


def _rerooted(invocation: Invocation, actual: Path) -> Invocation:
    """Name a tree copy's root by placeholder in a failure message.

    Answers carry their root and `normalize` replaces it; a failure has only its message,
    so the copy's path is replaced there, in each spelling a platform may print.
    """
    if invocation.answer is not None:
        return invocation
    real = os.path.realpath(actual)
    # fdu prints the canonical root: `/private/var/...` for `/var/...` on macOS, and the
    # verbatim `\\?\C:\...` form on Windows.
    spellings = {str(actual), actual.as_posix(), real, "\\\\?\\" + real}
    stderr = invocation.stderr
    for spelling in sorted(spellings, key=len, reverse=True):
        stderr = stderr.replace(spelling, ROOT_PLACEHOLDER)
    return Invocation(invocation.route, invocation.command, invocation.exit, stderr, None)


def write_diffs(result: RunResult, keys: set[str], out: Path) -> None:
    """Write a readable diff for each named case, for local reading or CI upload."""
    out.mkdir(parents=True, exist_ok=True)
    for case in result.cases:
        if case.key not in keys:
            continue
        lines = [
            f"# {case.key}",
            f"# verdict: {case.verdict.kind} {list(case.verdict.paths)}",
        ]
        lines += [f"# history: {command}" for command in case.history]
        if case.measured is not None:
            lines.append(f"# measured: {case.measured.command}")
        lines += [f"~ {line}" for line in case.verdict.sample]
        if case.oracle is not None and case.measured is not None:
            left = _render(case.oracle)
            right = _render(case.measured)
            lines += difflib.unified_diff(left, right, "cold", "measured", lineterm="")
        name = re.sub(r"[^A-Za-z0-9_.-]", "_", case.key) + ".diff"
        (out / name).write_text("\n".join(lines) + "\n", encoding="utf-8")


def _render(invocation: Invocation) -> list[str]:
    if invocation.answer is None:
        return [f"exit {invocation.exit}", invocation.stderr]
    content, _ = normalize(invocation.answer)
    return json.dumps(content, indent=1, sort_keys=True).splitlines()


def discover_surfaces(surface_names: Iterable[str]) -> Surfaces:
    """The builds under test, from FDU_BIN and FDU_PYTHON; never resolved through PATH."""
    names = set(surface_names)
    unknown = names - {"cli", "python"}
    if unknown or "cli" not in names:
        raise SystemExit(f"surfaces must include cli and name only cli and python: {sorted(names)}")
    default_bin = REPO / "target" / "debug" / ("fdu.exe" if os.name == "nt" else "fdu")
    fdu_bin = Path(os.environ.get("FDU_BIN", default_bin))
    if not fdu_bin.is_absolute() or not fdu_bin.is_file():
        raise SystemExit(f"FDU_BIN must be an absolute path to a built fdu: {fdu_bin}")
    python: Path | None = None
    if "python" in names:
        value = os.environ.get("FDU_PYTHON")
        if not value or not Path(value).is_absolute() or not Path(value).is_file():
            raise SystemExit(f"the python surface needs FDU_PYTHON, an absolute path: {value}")
        python = Path(value)
    return Surfaces(fdu_bin, python)


def run_tier(tier: matrix.Tier, surfaces: Surfaces, base: Path) -> RunResult:
    """Build a fixture under `base` and run `tier` against it."""
    facts = build_fixture(base / "fixture")
    return MatrixRun(tier, surfaces, Workspace(base), facts).run()


def judge(
    result: RunResult, known: registry.Registry, out: Path | None = None
) -> list[registry.Failure]:
    """Verify a run against the registry, writing a diff per failing case to `out`."""
    failures = registry.verify(
        known,
        judged_cases(result),
        full=result.complete_matrix,
        platform=sys.platform,
        cold_answers=result.cold_answers,
    )
    if out is not None and failures:
        write_diffs(result, {failure.key for failure in failures if failure.key}, out)
    return failures


def judged_cases(result: RunResult) -> list[registry.Judged]:
    return [(case.key, case.verdict.allowed, case.verdict.paths) for case in result.cases]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--tier", choices=sorted(matrix.TIERS), default="subset")
    parser.add_argument("--surfaces", default="cli,python")
    parser.add_argument(
        "--record",
        nargs="?",
        type=Path,
        const=registry.DEFAULT_PATH,
        metavar="PATH",
        help="write the registry this run observes (default: rewrite the committed one)",
    )
    parser.add_argument("--out", type=Path, help="write a diff for every failing case here")
    parser.add_argument(
        "--judged",
        type=Path,
        metavar="PATH",
        help="write every judged case as JSON, for `registry.py merge` across platforms",
    )
    args = parser.parse_args(argv)

    surfaces = discover_surfaces(args.surfaces.split(","))
    tier = matrix.TIERS[args.tier]
    with tempfile.TemporaryDirectory(prefix="fdu-path-independence-") as scratch:
        result = run_tier(tier, surfaces, Path(scratch))
    known = registry.load(registry.DEFAULT_PATH)
    if args.judged is not None:
        args.judged.parent.mkdir(parents=True, exist_ok=True)
        registry.write_judged(args.judged, sys.platform, judged_cases(result))
    if args.record is not None:
        recorded = registry.record(known, judged_cases(result), platform=sys.platform)
        args.record.parent.mkdir(parents=True, exist_ok=True)
        args.record.write_text(registry.dump(recorded), encoding="utf-8")
        print(f"recorded {len(recorded.all_entries())} known violations to {args.record}")
        if args.record.resolve() == registry.DEFAULT_PATH:
            known = recorded
    failures = judge(result, known, args.out)
    print(summary(result, failures))
    return 1 if failures else 0


def summary(result: RunResult, failures: list[registry.Failure]) -> str:
    counts: dict[str, int] = defaultdict(int)
    for case in result.cases:
        counts[case.verdict.kind] += 1
    lines = [
        f"path independence ({result.tier}): {len(result.cases)} cases, "
        f"{result.cold_answers} cold answers, "
        + ", ".join(f"{kind} {count}" for kind, count in sorted(counts.items()))
    ]
    lines += [f"  FAIL {failure}" for failure in failures[:50]]
    if len(failures) > 50:
        lines.append(f"  ... and {len(failures) - 50} more")
    return "\n".join(lines)


if __name__ == "__main__":
    raise SystemExit(main())
