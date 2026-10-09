"""Run the path-independence matrix and judge each answer against a cold run.

The invariant (see the explicit core models plan, "The Rule"): for a request, tree
state, delivery, and any history, a run returns the cold run's content and tree status,
a named failure, or, under `--stale-ok`, a labelled stale answer equal to a cold run at
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
if str(REPO) not in sys.path:
    sys.path.append(str(REPO))

import matrix  # noqa: E402
import registry  # noqa: E402
from fixture import FixtureFacts, build_fixture, copy_fixture  # noqa: E402
from scripts.atomic_write import write_text_atomic  # noqa: E402

Outcome = Literal["complete", "partial", "failure"]

# The one failure `--stale-ok` may answer with: no stored state serves the request.
# The command line exits 1 with this message, and Python raises fdu.FduError with it.
CACHE_MISS = "snapshot is not usable"

# Exit codes a Python route reports, mirroring the command line's: 1 for fdu's own
# error, 2 for a refused request (ValueError), and 3 for anything else, which is never
# a named failure.
PY_FDU_ERROR, PY_REFUSED, PY_UNEXPECTED = 1, 2, 3
# The routes that open an index and then read it, which hold their analyzers fixed.
INDEX_ROUTES = ("py-open", "py-scan")
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


def root_placeholder(position: int) -> str:
    """The placeholder for the root at `position` of a report over several roots."""
    return f"<root:{position}>"


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
    # Several roots are named in `roots`, each by a label (here the absolute path the
    # harness gave) and its canonical path; each spelling becomes that root's placeholder.
    roots = answer.get("roots")
    if isinstance(roots, list):
        encoded = json.dumps(answer)
        spellings = [
            (json.dumps(str(entry[field]))[1:-1], root_placeholder(position))
            for position, entry in enumerate(roots)
            for field in ("path", "label")
            if isinstance(entry, dict) and entry.get(field)
        ]
        # Longest first, so a root's path is never replaced inside a longer one.
        for spelling, placeholder in sorted(spellings, key=lambda pair: -len(pair[0])):
            encoded = encoded.replace(spelling, placeholder)
        answer = json.loads(encoded)
    # Keep the invocation intact for evidence while removing delivery-only facts.
    content = json.loads(json.dumps(answer))
    provenance = content.pop("provenance", {})
    if not isinstance(provenance, dict):
        raise TypeError("report provenance must be an object")
    reference = content.pop("age_reference_ns", None)
    # The same instant, rendered: it describes when the answer was read, as the reference
    # does, and each row's `modified_at` is compared as it is.
    content.pop("age_reference_at", None)

    def residual(row: dict[str, Any]) -> None:
        # Exact zero means the age agrees with its clock and mtime. Comparing this
        # residual preserves age bugs while allowing different read times.
        if "age_ns" in row and reference is not None and row["age_ns"] is not None:
            row["age_ns"] = row["age_ns"] - (reference - row["mtime_ns"])

    for section in content.get("reports", []):
        for row in section.get("files", []):
            residual(row)
        # Every tree node carries the activity it counts and its age, so the folded tier a
        # cold tree takes and the full index a warm one reads are compared row by row; over
        # several roots, so do the total row and every root's tree.
        tree = section.get("tree")
        nodes = [tree] if isinstance(tree, dict) else []
        total = section.get("total")
        if isinstance(total, dict):
            residual(total)
        nodes += [entry["tree"] for entry in section.get("trees", []) if isinstance(entry, dict)]
        while nodes:
            node = nodes.pop()
            residual(node)
            nodes.extend(node.get("children", []))
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


def _sum_ignored(shares: list[Any], keys: tuple[str, ...]) -> dict[str, int] | None:
    """The ignored share of a sum: unknown when any part's is."""
    if any(share is None for share in shares):
        return None
    return {key: sum(int(share[key]) for share in shares) for key in keys}


def _newest(values: list[Any]) -> int | None:
    known = [int(value) for value in values if value is not None]
    return max(known) if known else None


def _total_of(nodes: list[dict[str, Any]]) -> dict[str, Any]:
    """The total row several root rows sum to, without its age, which each run measures
    from its own instant."""
    return {
        "bytes": sum(int(node["bytes"]) for node in nodes),
        "allocated": sum(int(node["allocated"]) for node in nodes),
        "files": sum(int(node["files"]) for node in nodes),
        "dirs": sum(int(node["dirs"]) for node in nodes),
        "ignored": _sum_ignored(
            [node["ignored"] for node in nodes], ("files", "dirs", "bytes", "allocated")
        ),
        "newest_mtime_ns": _newest([node["newest_mtime_ns"] for node in nodes]),
        "mtime_ns": _newest([node["mtime_ns"] for node in nodes]),
        "complete": all(node["complete"] is True for node in nodes),
    }


def merged_facts(request_id: str, singles: list[dict[str, Any]]) -> Any:
    """What a report over several roots must say, merged from each root's own report.

    `singles` are the normalized single-root answers in the roots' order. Each request in
    `matrix.ROOT_REQUESTS` merges by one rule: additive values sum, maxima take the
    maximum, an unknown ignored share stays unknown, rows keep their root, and each
    root's tree is its own, named by its root.
    """
    sections = [answer["reports"][0] for answer in singles]
    if request_id == "r_summary":
        rows = [section["summary"] for section in sections]
        return {
            "files": sum(int(row["files"]) for row in rows),
            "dirs": sum(int(row["dirs"]) for row in rows),
            "bytes": sum(int(row["bytes"]) for row in rows),
            "allocated": sum(int(row["allocated"]) for row in rows),
            "ignored": _sum_ignored(
                [row["ignored"] for row in rows], ("files", "dirs", "bytes", "allocated")
            ),
            "newest_mtime_ns": _newest([row["newest_mtime_ns"] for row in rows]),
        }
    if request_id == "r_files":
        merged = [
            {"root": position, **row}
            for position, section in enumerate(sections)
            for row in section["files"]
        ]
        return sorted(merged, key=lambda row: (row["root"], row["path"]))
    if request_id == "r_extensions":
        buckets: dict[str, list[dict[str, Any]]] = defaultdict(list)
        for section in sections:
            for row in section["extensions"]:
                buckets[row["extension"]].append(row)
        # A root's ignored parts are known for every bucket or for none, so a root with no
        # rows says nothing; every root that has rows must know them.
        known = all(
            all(row["ignored"] is not None for row in section["extensions"]) for section in sections
        )
        return {
            extension: {
                "files": sum(int(row["files"]) for row in rows),
                "bytes": sum(int(row["bytes"]) for row in rows),
                "allocated": sum(int(row["allocated"]) for row in rows),
                "ignored": _sum_ignored([row["ignored"] for row in rows], ("files", "bytes"))
                if known
                else None,
            }
            for extension, rows in sorted(buckets.items())
        }
    if request_id in ("r_full_tree", "r_default"):
        nodes = [section["tree"] for section in sections]
        facts: dict[str, Any] = {"tree": None, "total": _total_of(nodes)}
        if request_id == "r_full_tree":
            facts["trees"] = {
                str(position): {**node, "name": root_placeholder(position)}
                for position, node in enumerate(nodes)
            }
        return facts
    raise KeyError(request_id)


def several_facts(request_id: str, answer: dict[str, Any]) -> Any:
    """The facts `merged_facts` derives, as a normalized report over several roots
    states them."""
    section = answer["reports"][0]
    if request_id == "r_summary":
        return section["summary"]
    if request_id == "r_files":
        return sorted(section["files"], key=lambda row: (row["root"], row["path"]))
    if request_id == "r_extensions":
        return {
            row["extension"]: {
                "files": row["files"],
                "bytes": row["bytes"],
                "allocated": row["allocated"],
                "ignored": None
                if row["ignored"] is None
                else {"files": row["ignored"]["files"], "bytes": row["ignored"]["bytes"]},
            }
            for row in section["extensions"]
        }
    if request_id in ("r_full_tree", "r_default"):
        total = section.get("total")
        facts: dict[str, Any] = {
            "tree": section.get("tree"),
            "total": None
            if total is None
            else {
                key: value for key, value in total.items() if key not in ("age_ns", "modified_at")
            },
        }
        if request_id == "r_full_tree":
            facts["trees"] = {str(entry["root"]): entry["tree"] for entry in section["trees"]}
        return facts
    raise KeyError(request_id)


def merge_verdict(request_id: str, oracle: Invocation, singles: list[Invocation]) -> Verdict:
    """Judge a report over several roots against the merge of its single-root reports."""
    if oracle.outcome == "failure" or any(single.outcome == "failure" for single in singles):
        outcomes = ",".join(invocation.outcome for invocation in (oracle, *singles))
        return Verdict("outcome_class", (f"<roots:{outcomes}>",))
    assert oracle.answer is not None
    expected = merged_facts(
        request_id, [normalize(single.answer)[0] for single in singles if single.answer]
    )
    observed = several_facts(request_id, normalize(oracle.answer)[0])
    diff = json_diff(expected, observed)
    if not diff:
        return Verdict("same")
    sample = tuple(f"{p}: {json.dumps(a)} -> {json.dumps(b)}" for p, a, b in diff[:12])
    return Verdict("differs", tuple(sorted({generalize(p) for p, _, _ in diff})), sample)


def generalize(path: str) -> str:
    """A diff path with list positions and identities erased."""
    return _ELEMENT.sub("[]", path)


def compare(
    oracle: Invocation,
    measured: Invocation,
    *,
    policy: str,
    earlier: Invocation | None = None,
    must_serve: bool = False,
) -> Verdict:
    """Judge `measured` against the cold `oracle`.

    `earlier` is a cold run at the state before a mutation; under `--stale-ok` an answer
    equal to it is the allowed stale outcome, provided it says so.
    `must_serve` is used after an explicit complete refresh of the identical request:
    refusing that snapshot or quietly scanning instead is a broken cache contract.
    A failure on either side fails it too, even when the two failures match: a control
    that never served proves nothing about the cache.
    """
    if must_serve and "failure" in (oracle.outcome, measured.outcome):
        return Verdict("outcome_class", (f"<serve:{oracle.outcome}>{measured.outcome}>",))
    if measured.outcome == "failure":
        if oracle.outcome == "failure":
            same_surface = is_python(oracle) == is_python(measured)
            if (oracle.exit, oracle.stderr if same_surface else "") == (
                measured.exit,
                measured.stderr if same_surface else "",
            ):
                return Verdict("same")
            return Verdict("differs", ("<error>",), (oracle.stderr, measured.stderr))
        if policy == matrix.STALE_OK and is_cache_miss(measured) and not must_serve:
            return Verdict("refused")
        return Verdict("outcome_class", (f"<outcome:{oracle.outcome}>{measured.outcome}>",))
    if oracle.outcome == "failure":
        return Verdict("outcome_class", (f"<outcome:{oracle.outcome}>{measured.outcome}>",))

    assert oracle.answer is not None and measured.answer is not None
    oracle_content, _ = normalize(oracle.answer)
    measured_content, provenance = normalize(measured.answer)
    if must_serve and provenance.get("source") != "cache_only":
        return Verdict("differs", ("provenance.source",), ("expected cache_only",))
    if must_serve and provenance.get("freshness") != "stale":
        return Verdict("differs", ("provenance.freshness",), ("expected stale",))
    diff = json_diff(oracle_content, measured_content)
    if not diff:
        return Verdict("same")
    if policy == matrix.STALE_OK and earlier is not None and earlier.answer is not None:
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


def content_tier_source(invocation: Invocation) -> str | None:
    """How the content tier of an answer was produced: `scanned`, `revalidated`, ...

    The report-level source says only how the entries were produced, so a content
    request whose files were all read again still reports `warm_revalidate` once any
    snapshot exists. Whether the sidecar was reused is this tier's answer.
    """
    provenance = (invocation.answer or {}).get("provenance") or {}
    return ((provenance.get("tiers") or {}).get("content") or {}).get("source")


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
    # Each invocation gets its own cache home, so no run reads another's stored state.
    # FDU_CACHE_DIR outranks XDG_CACHE_HOME, so an operator's setting would otherwise
    # share one directory across every case; both name the same place here.
    return dict(os.environ, XDG_CACHE_HOME=str(xdg), FDU_CACHE_DIR=str(xdg / "fdu"))


def run_cli(
    surfaces: Surfaces,
    root: Path | list[Path],
    request: matrix.Spec,
    policy: str,
    xdg: Path,
) -> Invocation:
    """Ask `request` on the command line, over one root or, given a list, several."""
    roots = root if isinstance(root, list) else [root]
    argv = [str(surfaces.fdu_bin), *map(str, roots), "--format", "json"]
    argv += [*matrix.cache_args(policy), *matrix.cli_args(request)]
    done = subprocess.run(argv, capture_output=True, text=True, env=_cache_env(xdg), check=False)
    try:
        answer = json.loads(done.stdout)
    except json.JSONDecodeError:
        answer = None
    named = (
        [root_placeholder(position) for position in range(len(roots))]
        if isinstance(root, list)
        else [ROOT_PLACEHOLDER]
    )
    command = " ".join([*argv[:1], *named, *argv[1 + len(roots) :]])
    return Invocation(matrix.CLI_ROUTE, command, done.returncode, done.stderr.strip(), answer)


def watch_can_serve(request: matrix.Spec, policy: str) -> bool:
    """Compare watch answers only where Request::validate_delivery permits a watch.

    Unsupported deliveries have explicit refusal tests in the core and CLI corpus;
    they are not history-dependent differences from a one-shot answer. A content view
    is analysis as much as `--analyze` is, so a watch refuses it too.
    """
    scope = request.get("scope", {})
    return (
        policy != matrix.STALE_OK
        and request.get("analyze", "none") == "none"
        and not matrix.CONTENT_VIEWS.intersection(request.get("views", []))
        and "scan_depth" not in scope
        and not scope.get("one_fs", False)
    )


def held_spec(request: matrix.Spec, oracle: Invocation) -> matrix.Spec:
    """The request an index must be opened with to answer `request` as a one-shot does.

    A one-shot report enables the analyzers its content views imply; an index holds only
    the analyzers `fdu.open` or `fdu.scan` named, and a read never widens them. So the
    index routes are opened with the analyzers the cold answer says the request enabled,
    `request.analyze`, which is the engine's own statement of that set rather than a copy
    of its view table kept here. A request whose cold run refused is asked unchanged.
    """
    enabled = (oracle.answer or {}).get("request", {}).get("analyze")
    if not isinstance(enabled, list):
        return request
    return {**request, "analyze": ",".join(enabled) or "none"}


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
        *matrix.cache_args(policy),
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
    try:
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
        if parse_error and (timed_out or exit_code == 0 or not stderr):
            stderr = f"{stderr}\n{parse_error}".strip()
        if timed_out:
            exit_code = PY_UNEXPECTED
        command = " ".join([*argv[:1], "<root>", *argv[2:]])
        return Invocation(matrix.CLI_WATCH_ROUTE, command, exit_code, stderr, answer)
    finally:
        # A watch has no natural EOF. Reap it before joining readers, then close both
        # pipes even when parsing or an assertion leaves this route early.
        if process.poll() is None:
            process.kill()
        process.wait()
        if reader.is_alive():
            reader.join(timeout=5)
        if stderr_reader.is_alive():
            stderr_reader.join(timeout=5)
        process.stdout.close()
        process.stderr.close()


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
    root: Path | list[Path],
    request: matrix.Spec,
    policy: str,
    xdg: Path,
    route: str,
) -> Invocation:
    """Ask `request` through the Python package on `route`; several roots, given a list,
    on the one route that takes them, `fdu.report`."""
    assert surfaces.python is not None and route in matrix.PY_ROUTES
    assert not isinstance(root, list) or route == "py-report"
    job = {
        "root": [str(path) for path in root] if isinstance(root, list) else str(root),
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
        result.cases += self.phase_roots()
        result.cases += self.phase_warm()
        result.cases += self.phase_cache_contract()
        result.cases += self.phase_implied()
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
            # These display-axis requests are meant to return reports. If both routes
            # reject one after a grammar change, equality alone would pass vacuously.
            must_answer = request_id in {"depth1", "depthall", "a_code_metric_rev", "v_tree_bounds"}
            verdict = (
                Verdict("outcome_class", ("<cold-oracle:no-report>",), (oracle.stderr,))
                if must_answer and oracle.outcome == "failure"
                else compare(oracle, measured, policy="auto")
            )
            key = case_key("cold", matrix.CLI_ROUTE, "auto", "-", "-", request_id)
            return [CaseResult(key, verdict, oracle, measured)]

        return _parallel(one, self.tier.requests)

    def phase_roots(self) -> list[CaseResult]:
        """Several roots as one report, against the merge of each root's report.

        A cold several-root answer must be the merge `merged_facts` derives from the
        single-root cold answers. Then the same answer must come back from per-root
        snapshots, each left by a one-root `--cache on` run and served together by
        `--stale-ok`, and from `fdu.report` with a list of roots.
        """
        cases = [
            (set_id, request_id)
            for set_id in matrix.ROOT_SETS
            for request_id in matrix.ROOT_REQUESTS
        ]

        def one(case: tuple[str, str]) -> list[CaseResult]:
            set_id, request_id = case
            roots = [self.facts.root / name for name in matrix.ROOT_SETS[set_id]]
            request = matrix.ROOT_REQUESTS[request_id]
            label = f"{set_id}:{request_id}"
            xdg = self.ws.fresh(f"roots-cold-{set_id}-{request_id}")
            oracle = run_cli(self.surfaces, roots, request, "off", xdg)
            singles = [run_cli(self.surfaces, root, request, "off", xdg) for root in roots]
            self.ws.discard(xdg)
            results = [
                CaseResult(
                    case_key("roots", "merge", "off", "-", "-", label),
                    merge_verdict(request_id, oracle, singles),
                    oracle,
                    None,
                )
            ]

            xdg = self.ws.fresh(f"roots-serve-{set_id}-{request_id}")
            history = tuple(
                run_cli(self.surfaces, root, request, "on", xdg).command for root in roots
            )
            measured = run_cli(self.surfaces, roots, request, matrix.STALE_OK, xdg)
            self.ws.discard(xdg)
            key = case_key("roots", matrix.CLI_ROUTE, matrix.STALE_OK, "per-root-on", "-", label)
            verdict = compare(oracle, measured, policy=matrix.STALE_OK, must_serve=True)
            results.append(CaseResult(key, verdict, oracle, measured, history))

            if self.surfaces.python is not None:
                xdg = self.ws.fresh(f"roots-py-{set_id}-{request_id}")
                measured = run_py(self.surfaces, roots, request, "off", xdg, "py-report")
                self.ws.discard(xdg)
                key = case_key("roots", "py-report", "off", "-", "-", label)
                results.append(
                    CaseResult(key, compare(oracle, measured, policy="off"), oracle, measured)
                )
            return results

        return _parallel(one, cases)

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
        """Each request after itself under `auto`, then `auto`, `on`, and `stale-ok` in turn."""

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

    def phase_cache_contract(self) -> list[CaseResult]:
        """A complete forced write must serve the identical unchanged request.

        These positive controls complement answer equality: a cache that always misses
        otherwise passes by scanning cold or returning an allowed refusal. `on` requires
        indexed persistence even for a summary that `auto` answers without retaining an
        index.
        """
        requests = ("default", "nogi", "a_lines", "a_code", "a_words", "a_all")
        routes = [matrix.CLI_ROUTE]
        if self.surfaces.python is not None:
            routes += ["py-report", "py-open"]

        def one(case: tuple[str, str]) -> list[CaseResult]:
            request_id, route = case
            request = matrix.REQUESTS[request_id]
            xdg = self.ws.fresh(f"serves-{route}-{request_id}")
            try:
                seed = run_cli(self.surfaces, self.facts.root, request, "on", xdg)
                key = case_key("serves", route, matrix.STALE_OK, "on-self", "-", request_id)
                if seed.outcome != "complete":
                    verdict = Verdict("outcome_class", ("<on:not-complete>",))
                    return [CaseResult(key, verdict, self.cold[request_id], seed)]
                measured = run_route(
                    self.surfaces, route, self.facts.root, request, matrix.STALE_OK, xdg
                )
                oracle = self.cold[request_id]
                verdict = compare(oracle, measured, policy=matrix.STALE_OK, must_serve=True)
                return [CaseResult(key, verdict, oracle, measured, (seed.command,))]
            finally:
                self.ws.discard(xdg)

        return _parallel(one, ((request, route) for request in requests for route in routes))

    def phase_implied(self) -> list[CaseResult]:
        """A content view and the analyzer it implies are one request.

        Cold, the two answers are equal, request included. And each serves the other:
        after a complete `--cache on` run of one, `--stale-ok` must answer the other from
        the cache on every cache-reading route. Equal numbers alone could come from two
        separate sidecars; serving across the pair is what shows they share one basis.

        The default delivery too: after an `auto` run of one, an `auto` run of the other
        must equal the cold answer and report its content tier `revalidated`, the sidecar
        verified and reused rather than every file read again. `--stale-ok` serves without
        verifying, so it alone cannot show that the verified path reuses the pair's sidecar.
        """
        pairs = [
            (view, analyzer)
            for view, analyzer in matrix.IMPLIED
            if view in self.tier.requests and analyzer in self.tier.requests
        ]
        routes = [matrix.CLI_ROUTE]
        if self.surfaces.python is not None:
            routes += ["py-report", "py-open"]
        results: list[CaseResult] = []
        for view, analyzer in pairs:
            oracle, measured = self.cold[analyzer], self.cold[view]
            # Two identical refusals would compare equal and prove nothing.
            verdict = (
                Verdict("outcome_class", (f"<implied:{oracle.outcome}>{measured.outcome}>",))
                if "failure" in (oracle.outcome, measured.outcome)
                else compare(oracle, measured, policy="off")
            )
            key = case_key("implied", matrix.CLI_ROUTE, "off", analyzer, "-", view)
            results.append(CaseResult(key, verdict, oracle, measured))

        def one(case: tuple[str, str, str]) -> list[CaseResult]:
            seed_id, reader_id, route = case
            reader = matrix.REQUESTS[reader_id]
            oracle = self.cold[reader_id]
            xdg = self.ws.fresh(f"implied-{seed_id}-{reader_id}-{route}")
            try:
                seed = run_cli(self.surfaces, self.facts.root, matrix.REQUESTS[seed_id], "on", xdg)
                key = case_key("implied", route, matrix.STALE_OK, f"on:{seed_id}", "-", reader_id)
                if seed.outcome != "complete":
                    verdict = Verdict("outcome_class", ("<on:not-complete>",))
                    return [CaseResult(key, verdict, oracle, seed)]
                asked = held_spec(reader, oracle) if route in INDEX_ROUTES else reader
                measured = run_route(
                    self.surfaces, route, self.facts.root, asked, matrix.STALE_OK, xdg
                )
                verdict = compare(oracle, measured, policy=matrix.STALE_OK, must_serve=True)
                return [CaseResult(key, verdict, oracle, measured, (seed.command,))]
            finally:
                self.ws.discard(xdg)

        def reused(case: tuple[str, str]) -> list[CaseResult]:
            seed_id, reader_id = case
            oracle = self.cold[reader_id]
            xdg = self.ws.fresh(f"implied-auto-{seed_id}-{reader_id}")
            try:
                seed = run_cli(
                    self.surfaces, self.facts.root, matrix.REQUESTS[seed_id], "auto", xdg
                )
                key = case_key(
                    "implied", matrix.CLI_ROUTE, "auto", f"auto:{seed_id}", "-", reader_id
                )
                if seed.outcome != "complete":
                    verdict = Verdict("outcome_class", ("<auto:not-complete>",))
                    return [CaseResult(key, verdict, oracle, seed)]
                measured = run_cli(
                    self.surfaces, self.facts.root, matrix.REQUESTS[reader_id], "auto", xdg
                )
                verdict = compare(oracle, measured, policy="auto")
                source = content_tier_source(measured)
                if verdict.kind == "same" and source != "revalidated":
                    verdict = Verdict(
                        "differs",
                        ("provenance.tiers.content.source",),
                        (f"expected revalidated, got {source}",),
                    )
                return [CaseResult(key, verdict, oracle, measured, (seed.command,))]
            finally:
                self.ws.discard(xdg)

        cases = [
            (seed, reader, route)
            for view, analyzer in pairs
            for seed, reader in ((analyzer, view), (view, analyzer))
            for route in routes
        ]
        histories = [
            (seed, reader)
            for view, analyzer in pairs
            for seed, reader in ((analyzer, view), (view, analyzer))
        ]
        return results + _parallel(one, cases) + _parallel(reused, histories)

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
            policies: tuple[str, ...] = ("off",) if warmer is None else ("auto", matrix.STALE_OK)
            for policy in policies:
                for route in readers + (["py-scan"] if warmer is None else []):
                    if route == matrix.CLI_WATCH_ROUTE and not watch_can_serve(request, policy):
                        continue
                    xdg = self.ws.fresh(f"cross-{history_id}-{request_id}-{route}-{policy}")
                    history: tuple[str, ...] = ()
                    if warmer is not None and warm_route is not None:
                        history = (self._warm(self.facts.root, warmer, xdg, warm_route),)
                    asked = held_spec(request, oracle) if route in INDEX_ROUTES else request
                    measured = run_route(self.surfaces, route, self.facts.root, asked, policy, xdg)
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
        write_text_atomic(out / name, "\n".join(lines) + "\n", encoding="utf-8")


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
    # `make path-independence` names the build cargo made; CARGO_TARGET_DIR moves it.
    target = REPO / os.environ.get("CARGO_TARGET_DIR", "target")
    default_bin = target / "debug" / ("fdu.exe" if os.name == "nt" else "fdu")
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
        require_clean=True,
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
        write_text_atomic(args.record, registry.dump(recorded), encoding="utf-8")
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
