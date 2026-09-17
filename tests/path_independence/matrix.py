"""What the path-independence harness asks, and in which tier.

A request is a surface-neutral spec: scope, selection, views, and analyzers. The command
line renders it as flags (`cli_args`) and the Python route rebuilds it from the same
dict (`pyrun.py`), so both surfaces ask the identical question.

A warmer is an earlier request whose run leaves stored state behind. A mutation changes
the tree after warming. A route is how the measured answer is obtained.
"""

from __future__ import annotations

import os
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import Any

Spec = dict[str, Any]

SCOPE_KEYS = frozenset({"no_gitignore", "budget", "line_limit", "scan_depth", "one_fs"})

POLICIES = ("auto", "read-only", "only")


def spec(views: list[str] | None = None, analyze: str | None = None, **fields: Any) -> Spec:
    """Build a request spec, sorting each field into scope or selection."""
    built: Spec = {"scope": {}, "sel": {}}
    for key, value in fields.items():
        built["scope" if key in SCOPE_KEYS else "sel"][key] = value
    if views:
        built["views"] = views
    if analyze:
        built["analyze"] = analyze
    return built


def cli_args(request: Spec) -> list[str]:
    """The command-line flags that ask `request`."""
    args: list[str] = []
    scope, selection = request.get("scope", {}), request.get("sel", {})
    if scope.get("no_gitignore"):
        args.append("--no-gitignore")
    if "budget" in scope:
        args += ["--gitignore-budget", str(scope["budget"])]
    if "line_limit" in scope:
        args += ["--gitignore-line-limit", str(scope["line_limit"])]
    if "scan_depth" in scope:
        args += ["--scan-depth", str(scope["scan_depth"])]
    if scope.get("one_fs"):
        args.append("--one-filesystem")
    for glob in selection.get("include", []):
        args += ["--include", glob]
    for glob in selection.get("exclude", []):
        args += ["--exclude", glob]
    if "min_size" in selection:
        args += ["--min-size", str(selection["min_size"])]
    if "modified_since" in selection:
        args += ["--modified-since", selection["modified_since"]]
    if "kind" in selection:
        args += ["--kind", ",".join(selection["kind"])]
    if selection.get("ignored") == "exclude":
        args.append("--exclude-ignored")
    if selection.get("ignored") == "only":
        args.append("--only-ignored")
    if "depth" in selection:
        args += ["--depth", str(selection["depth"])]
    if "limit" in selection:
        args += ["--limit", str(selection["limit"])]
    if "sort" in selection:
        args += ["--sort", selection["sort"]]
    if selection.get("reverse"):
        args.append("--reverse")
    if "size" in selection:
        args += ["--size", selection["size"]]
    if request.get("views"):
        args += ["--view", ",".join(request["views"])]
    if request.get("analyze"):
        args += ["--analyze", request["analyze"]]
    return args


REQUESTS: dict[str, Spec] = {
    # scope
    "default": spec(),
    "nogi": spec(no_gitignore=True),
    "budget1k": spec(budget="1KiB"),
    "budgetall": spec(budget="all"),
    "linelim20": spec(line_limit="20"),
    "scandepth1": spec(scan_depth=1),
    "scandepth2": spec(scan_depth=2),
    "onefs": spec(one_fs=True),
    # selection
    "exclign": spec(ignored="exclude"),
    "onlyign": spec(ignored="only"),
    "incl_rs": spec(include=["*.rs"]),
    "excl_src": spec(exclude=["src/**"]),
    "minsize100": spec(min_size="100"),
    "modsince": spec(modified_since="2023-03-15T00:00:00Z", views=["files"]),
    "kind_file": spec(kind=["file"]),
    "kind_dirsym": spec(kind=["dir", "symlink"], views=["files"]),
    "limit2": spec(limit=2),
    "sort_name": spec(sort="name"),
    "sort_mtime_rev": spec(sort="mtime", reverse=True, views=["files"]),
    "depth1": spec(depth=1),
    "depthall": spec(depth="all"),
    "sizeapp": spec(size="apparent"),
    # views, metadata only
    "v_summary": spec(views=["summary"]),
    "v_tree": spec(views=["tree"]),
    "v_families": spec(views=["families"]),
    "v_types": spec(views=["types"]),
    "v_extensions": spec(views=["extensions"]),
    "v_languages": spec(views=["languages"]),
    "v_largest": spec(views=["largest"]),
    "v_recent": spec(views=["recent"]),
    "v_files": spec(views=["files"]),
    "v_full": spec(views=["full"]),
    "v_documents": spec(views=["documents"]),
    "v_summary_types": spec(views=["summary", "types"]),
    # analysis with implied views
    "a_none": spec(analyze="none"),
    "a_lines": spec(analyze="lines"),
    "a_code": spec(analyze="code"),
    "a_words": spec(analyze="words"),
    "a_all": spec(analyze="all"),
    "a_codewords": spec(analyze="code,words"),
    # analysis with explicit views
    "a_lines_v_languages": spec(analyze="lines", views=["languages"]),
    "a_lines_v_documents": spec(analyze="lines", views=["documents"]),
    "a_lines_v_files": spec(analyze="lines", views=["files"]),
    "a_code_v_documents": spec(analyze="code", views=["documents"]),
    "a_code_v_summary": spec(analyze="code", views=["summary"]),
    "a_code_v_full": spec(analyze="code", views=["full"]),
    "a_words_v_tree": spec(analyze="words", views=["tree"]),
    "a_all_v_full": spec(analyze="all", views=["full"]),
    "a_all_v_files": spec(analyze="all", views=["files"]),
    # analysis with scope and selection
    "a_code_exclign": spec(analyze="code", ignored="exclude"),
    "a_all_nogi": spec(analyze="all", no_gitignore=True),
    "a_lines_budget1k": spec(analyze="lines", budget="1KiB"),
    "a_words_onlyign": spec(analyze="words", ignored="only"),
    "a_code_scandepth1": spec(analyze="code", scan_depth=1),
    "a_lines_incl_rs": spec(analyze="lines", include=["*.rs"]),
    "a_all_sizeapp": spec(analyze="all", size="apparent"),
    "a_code_langs_name_lim1": spec(analyze="code", views=["languages"], sort="name", limit=1),
    # the summary tier
    "v_summary_exclign": spec(views=["summary"], ignored="exclude"),
    "v_summary_nogi": spec(views=["summary"], no_gitignore=True),
    "v_summary_scandepth1": spec(views=["summary"], scan_depth=1),
    "v_summary_minsize": spec(views=["summary"], min_size="100"),
    "v_summary_a_all": spec(views=["summary"], analyze="all"),
    "v_summary_a_lines": spec(views=["summary"], analyze="lines"),
}

# A warmer's spec and the cache policy its own run uses.
WARMERS: dict[str, tuple[Spec, str]] = {
    "W_default": (spec(), "auto"),
    "W_nogi": (spec(no_gitignore=True), "auto"),
    "W_all": (spec(analyze="all"), "auto"),
    "W_code": (spec(analyze="code"), "auto"),
    "W_lines": (spec(analyze="lines"), "auto"),
    "W_words": (spec(analyze="words"), "auto"),
    "W_budget1k": (spec(budget="1KiB"), "auto"),
    "W_summary": (spec(views=["summary"]), "auto"),
    "W_scandepth1": (spec(scan_depth=1), "auto"),
    "W_refresh": (spec(), "refresh"),
}


# Every mutation stamps what it changes with this instant, including the parent directory
# of an added or removed entry, so two copies mutated at different moments stay identical.
MUTATION_NS = 1_704_070_860_000_000_000  # 2024-01-01T01:01:00Z


def _stamp(*paths: Path) -> None:
    for path in paths:
        os.utime(path, ns=(MUTATION_NS, MUTATION_NS), follow_symlinks=not path.is_symlink())


def _rewrite_same_size(path: Path, unit: bytes, *, keep_mtime: bool) -> None:
    """Replace a file's bytes with different content of the same length."""
    before = path.stat()
    old = path.read_bytes()
    new = unit * (len(old) // len(unit)) + unit[:1] * (len(old) % len(unit))
    assert len(new) == len(old) and new != old
    path.write_bytes(new)
    if keep_mtime:
        os.utime(path, ns=(before.st_atime_ns, before.st_mtime_ns))
    else:
        _stamp(path)


def _same_size(tree: Path, *, keep_mtime: bool) -> None:
    _rewrite_same_size(tree / "docs/notes.txt", b"x\n", keep_mtime=keep_mtime)
    _rewrite_same_size(tree / "src/main.rs", b"// c\n", keep_mtime=keep_mtime)


def _touch(tree: Path) -> None:
    _stamp(tree / "src/lib.py")


def _add(tree: Path) -> None:
    (tree / "src/new_module.py").write_bytes(b"# new\nprint(1)\n")
    _stamp(tree / "src/new_module.py", tree / "src")


def _delete(tree: Path) -> None:
    (tree / "docs/notes.txt").unlink()
    _stamp(tree / "docs")


def _write(path: Path, content: bytes) -> None:
    path.write_bytes(content)
    _stamp(path)


def _retarget_symlink(tree: Path) -> None:
    link = tree / "link_to_main"
    link.unlink()
    os.symlink("src/lib.py", link)
    if os.utime in os.supports_follow_symlinks:
        _stamp(link)
    _stamp(tree)


UNREADABLE = "src/nested"


def _make_unreadable(tree: Path) -> None:
    os.chmod(tree / UNREADABLE, 0)


def _restore_readable(tree: Path) -> None:
    os.chmod(tree / UNREADABLE, 0o755)


@dataclass(frozen=True)
class Mutation:
    """A change to the tree after warming.

    `restore` undoes what would stop the tree from being copied or deleted afterwards.
    """

    apply: Callable[[Path], None]
    needs_symlinks: bool = False
    needs_permissions: bool = False
    restore: Callable[[Path], None] | None = None


MUTATIONS: dict[str, Mutation] = {
    "samesize": Mutation(lambda tree: _same_size(tree, keep_mtime=False)),
    # Only content and ctime change; no timestamp a caller can set distinguishes it.
    "samesize_keepmtime": Mutation(lambda tree: _same_size(tree, keep_mtime=True)),
    "touch": Mutation(_touch),
    "add": Mutation(_add),
    "delete": Mutation(_delete),
    "gitignore_root": Mutation(
        lambda tree: _write(tree / ".gitignore", (tree / ".gitignore").read_bytes() + b"*.txt\n")
    ),
    # Same length as the original `*.tmp\n`, so only content distinguishes the rule.
    "gitignore_samesize": Mutation(lambda tree: _write(tree / "src/.gitignore", b"*.py*\n")),
    "gitignore_big_samesize": Mutation(
        lambda tree: _write(
            tree / "big/.gitignore",
            (tree / "big/.gitignore").read_bytes().replace(b"*.bak\n", b"*.txt\n"),
        )
    ),
    "symlink": Mutation(_retarget_symlink, needs_symlinks=True),
    # A directory the walk cannot list: a cold run is partial, and a warm run must not
    # serve the facts it retained from before.
    "unreadable": Mutation(_make_unreadable, needs_permissions=True, restore=_restore_readable),
}

# Routes that obtain the measured answer. `cli-report` is the command line; the others
# are the Python package: `fdu.report`, `fdu.open(...).report`, and `fdu.scan(...).report`.
CLI_ROUTE = "cli-report"
PY_ROUTES = ("py-report", "py-open", "py-scan")
ROUTES = (CLI_ROUTE, *PY_ROUTES)


@dataclass(frozen=True)
class Tier:
    """Which requests, histories, mutations, and routes one run covers."""

    name: str
    requests: tuple[str, ...]
    warmers: tuple[str, ...]
    selfwarm: bool
    mutations: tuple[str, ...]
    mutation_warmers: tuple[str, ...]
    cross_warmers: tuple[str, ...]


FULL = Tier(
    name="full",
    requests=tuple(REQUESTS),
    warmers=tuple(WARMERS),
    selfwarm=True,
    mutations=tuple(MUTATIONS),
    mutation_warmers=("W_default", "W_all", "W_code", "W_nogi", "W_budget1k"),
    cross_warmers=("W_default", "W_all", "W_lines", "W_summary"),
)

SUBSET = Tier(
    name="subset",
    requests=(
        "default",
        "nogi",
        "budget1k",
        "scandepth1",
        "exclign",
        "onlyign",
        "v_summary",
        "v_summary_nogi",
        "v_types",
        "a_lines",
        "a_code",
        "a_words",
        "a_all",
        "a_lines_v_documents",
        "a_code_langs_name_lim1",
        "a_all_nogi",
        "onefs",
    ),
    warmers=("W_default", "W_nogi", "W_all", "W_code", "W_words"),
    selfwarm=False,
    # One mutation per way a change can be detected: a visible mtime, content alone,
    # entries added and removed, a rule change, a link retarget, and an unlistable tree.
    mutations=(
        "samesize",
        "samesize_keepmtime",
        "add",
        "delete",
        "gitignore_root",
        "symlink",
        "unreadable",
    ),
    mutation_warmers=("W_default", "W_all"),
    cross_warmers=("W_default", "W_all"),
)

TIERS = {tier.name: tier for tier in (SUBSET, FULL)}
