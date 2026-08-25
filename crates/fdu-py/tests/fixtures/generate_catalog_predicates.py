"""Regenerate `catalog-predicates.json` from the consumer contract's own code.

The consuming contract's catalog query is a dataclass that validates its own predicates and
five lines of Python that apply them. Transcribing either here would produce a fixture that
agrees with a *reading* of that code and not necessarily with the code, which is the
failure the fixture exists to prevent. So this runs both: the query type is imported from
the consumer's contract module, and the matcher is lifted out of its provider module by AST
and executed. Only free names are supplied, which is a binding rather than a re-reading.

The consumer is not part of this repository. Point this at a checkout of it:

    python3 generate_catalog_predicates.py PATH_TO_CHECKOUT/src

The recorded revision is written into the fixture, so a fixture built against an older
contract can be told from one built against the current contract without rerunning
anything. Regenerating needs Python 3.12 or newer: the consumer's provider module uses
`type` statements, which older parsers reject before reaching the function this reads.
"""

from __future__ import annotations

import ast
import importlib.util
import json
import pathlib
import subprocess
import sys
import types
from pathlib import PurePosixPath

DEFAULT_SOURCE_ROOT = pathlib.Path("attic/metabrowser/src")
CONTRACT = "metabrowser/inventory_engine/contract.py"
PROVIDER = "metabrowser/inventory_engine/providers/python_inventory.py"
FIXTURE = pathlib.Path(__file__).resolve().parent / "catalog-predicates.json"

#: path, type, content, gitignored -- the tree both engines are asked about.
#:
#: Chosen so every clause of the predicate separates at least one pair of entries: a
#: compound tail beside a plain one, an uppercase suffix beside its lowercase spelling, a
#: leading-dot name with no suffix at all, a nested ancestor, a directory and a symlink that
#: are not files, and an ignored entry.
CORPUS = [
    (".gitignore", "file", "*.log\n", False),
    ("Makefile", "file", "all:\n", False),
    ("notes.txt", "file", "plain", False),
    ("link.rs", "symlink", "", False),
    ("docs", "dir", "", False),
    ("docs/guide.rs", "file", "fn guide() {}", False),
    ("docs/readme.md", "file", "hi", False),
    ("src", "dir", "", False),
    ("src/NOTES.TXT", "file", "loud!!", False),
    ("src/archive.tar.gz", "file", "tarball", False),
    ("src/build.log", "file", "noisy...", True),
    ("src/main.rs", "file", "fn main() {}", False),
    ("src/nested", "dir", "", False),
    ("src/nested/deep.rs", "file", "fn deep() {}", False),
    ("tests", "dir", "", False),
    ("tests/case.rs", "file", "x", False),
]

#: Queries both engines are expected to answer identically.
CASES = [
    ("everything-a-file", {"include_ignored": True}),
    ("ignored-entries-excluded", {"include_ignored": False}),
    ("terminal-rs", {"include_ignored": True, "terminal_extensions": [".rs"]}),
    ("terminal-txt-case-folded", {"include_ignored": True, "terminal_extensions": [".txt"]}),
    ("terminal-is-the-last-suffix", {"include_ignored": True, "terminal_extensions": [".gz"]}),
    ("terminal-any-of", {"include_ignored": True, "terminal_extensions": [".md", ".rs"]}),
    ("ancestor-src", {"include_ignored": True, "ancestor_names": ["src"]}),
    ("ancestor-at-depth", {"include_ignored": True, "ancestor_names": ["nested"]}),
    ("ancestor-any-of", {"include_ignored": True, "ancestor_names": ["src", "tests"]}),
    ("ancestor-is-case-sensitive", {"include_ignored": True, "ancestor_names": ["SRC"]}),
    (
        "both-predicates",
        {"include_ignored": True, "terminal_extensions": [".rs"], "ancestor_names": ["src"]},
    ),
    ("size-strictly-below", {"include_ignored": True, "size_less_than": 6}),
    ("size-of-one-admits-nothing-smaller", {"include_ignored": True, "size_less_than": 1}),
    (
        "every-predicate-at-once",
        {
            "include_ignored": False,
            "terminal_extensions": [".rs", ".txt"],
            "ancestor_names": ["src", "docs"],
            "size_less_than": 13,
        },
    ),
]

#: Values expected to be refused where they are written, by both engines.
#:
#: Each is a predicate that could only ever match nothing, which is the argument for
#: refusing rather than answering: a caller who wrote one reads an empty page as a fact
#: about their tree.
REFUSALS = [
    ("terminal-without-its-dot", {"terminal_extensions": ["rs"]}),
    ("terminal-uppercase", {"terminal_extensions": [".RS"]}),
    ("terminal-compound-tail", {"terminal_extensions": [".tar.gz"]}),
    ("terminal-bare-dot", {"terminal_extensions": ["."]}),
    ("ancestor-empty", {"ancestor_names": [""]}),
    ("ancestor-with-a-separator", {"ancestor_names": ["src/lib"]}),
    ("size-not-positive", {"size_less_than": 0}),
]

#: Values the two engines judge differently, recorded rather than hidden.
#:
#: Neither difference can change an answer, and `consumer_admits` below is the check of
#: that: where fdu refuses and the contract accepts, the contract's own answer is empty.
ASYMMETRIES = [
    ("ancestor-current-directory", {"ancestor_names": ["."]}),
    ("ancestor-parent-directory", {"ancestor_names": [".."]}),
]

ABOUT = [
    "Expected catalog-predicate behaviour for the consumer contract this repository's",
    "reference provider adapts to. Nothing here was transcribed: `admits` comes from",
    "executing the consumer's own `_catalog_entry_matches`, and `refused` from constructing",
    "its own `CatalogQuery`, which validates its predicates. Regenerate with",
    "`generate_catalog_predicates.py` beside this file.",
    "",
    "The predicate, in five clauses: an entry must be a file; it must be unignored unless",
    "include_ignored; its size in bytes must be strictly below size_less_than; its terminal",
    "suffix -- the last one, lowered -- must appear in terminal_extensions; and one of its",
    "ancestor components must appear in ancestor_names, matched exactly and",
    "case-sensitively. An empty list does not constrain.",
    "",
    "`refusals` are values both engines refuse where they are written rather than answering",
    "with an empty page, because each could only ever match nothing. `asymmetries` are the",
    "values they judge differently: fdu's rule for an ancestor name is one whole path",
    "component, so it refuses `.` and `..`, which the contract accepts and its own matcher",
    "then admits nothing for -- recorded in `consumer_admits`, so the claim that nothing is",
    "lost is checked rather than asserted. The reverse asymmetry is not listed because it",
    "is platform-dependent: a backslash is a legal directory name on unix and fdu accepts",
    "one there, while the contract refuses it everywhere.",
    "",
    "`admits` is sorted, because the contract's predicate says which entries match and",
    "nothing about their order. fdu answers in the tree's own path key, where a directory's",
    "children come before a later sibling file, so the two orders differ while the sets do",
    "not. Ordering and paging are pinned where they are the subject; here they are not.",
    "",
    "`consumer_source` is the matcher as it stood at `consumer.revision`, kept so a reader",
    "can see what these answers came from. It is a record rather than an input: nothing",
    "reads it back, and the way to trust it is to regenerate.",
]


def revision(source_root: pathlib.Path) -> dict[str, str]:
    """The consumer checkout's commit, or an explicit absence."""

    try:
        found = subprocess.run(
            ["git", "-C", str(source_root), "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            check=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return {"revision": "unknown", "note": "generated outside a checkout of the consumer"}
    return {"revision": found.stdout.strip()}


def load_contract(source_root: pathlib.Path) -> object:
    """The consumer's real `CatalogQuery`, without importing its whole package.

    Two of its imports are stubbed. Importing `metabrowser` itself pulls in the application,
    third-party dependencies and all, for two constants and two type names that this
    dataclass never touches.
    """

    package = types.ModuleType("metabrowser")
    package.__path__ = [str(source_root / "metabrowser")]  # type: ignore[attr-defined]
    sys.modules["metabrowser"] = package
    engine = types.ModuleType("metabrowser.inventory_engine")
    engine.__path__ = [str(source_root / "metabrowser" / "inventory_engine")]  # type: ignore[attr-defined]
    sys.modules["metabrowser.inventory_engine"] = engine
    for name, attributes in [
        ("metabrowser.constants", {"LOGS_DIR": "logs", "STATE_DIR": "state"}),
        ("metabrowser.wire_models", {"NavigationTallies": object, "RollupResult": object}),
    ]:
        stub = types.ModuleType(name)
        for key, value in attributes.items():
            setattr(stub, key, value)
        sys.modules[name] = stub

    spec = importlib.util.spec_from_file_location(
        "metabrowser.inventory_engine.contract", source_root / CONTRACT
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module.CatalogQuery


def load_matcher(provider: pathlib.Path) -> tuple[object, str]:
    """The consumer's `_catalog_entry_matches`, lifted out by AST and executed."""

    source = provider.read_text(encoding="utf-8")
    wanted = next(
        node
        for node in ast.parse(source).body
        if isinstance(node, ast.FunctionDef) and node.name == "_catalog_entry_matches"
    )
    segment = ast.get_source_segment(source, wanted)
    assert segment is not None
    namespace: dict[str, object] = {"PurePosixPath": PurePosixPath}
    # The provider module carries `from __future__ import annotations`, so its annotations
    # are strings there; compiled alone this segment would evaluate them and fail on names
    # imported only for typing.
    # Executing the contract under test is the point of this file.
    exec(
        compile("from __future__ import annotations\n\n" + segment, str(provider), "exec"),
        namespace,
    )
    return namespace["_catalog_entry_matches"], segment


def entry(path: str, kind: str, content: str, gitignored: bool) -> types.SimpleNamespace:
    return types.SimpleNamespace(
        path=path,
        name=PurePosixPath(path).name,
        type=kind,
        size=len(content.encode("utf-8")),
        gitignored=gitignored,
    )


def written(overrides: dict[str, object]) -> dict[str, object]:
    """One case's four predicate fields, defaults filled in."""

    fields: dict[str, object] = {
        "include_ignored": False,
        "terminal_extensions": [],
        "ancestor_names": [],
        "size_less_than": None,
    }
    fields.update(overrides)
    return fields


def build(catalog_query: object, spelled: dict[str, object]) -> object:
    return catalog_query(  # type: ignore[operator]
        query_id="fixture",
        max_rows=1_000,
        include_ignored=spelled["include_ignored"],
        terminal_extensions=tuple(spelled["terminal_extensions"]),  # type: ignore[arg-type]
        ancestor_names=tuple(spelled["ancestor_names"]),  # type: ignore[arg-type]
        size_less_than=spelled["size_less_than"],
    )


def main() -> None:
    source_root = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_SOURCE_ROOT
    provider = source_root / PROVIDER
    if not provider.is_file():
        raise SystemExit(
            f"no consumer provider at {provider}; pass the checkout's src as an argument"
        )
    catalog_query = load_contract(source_root)
    matches, segment = load_matcher(provider)
    entries = [entry(*row) for row in CORPUS]

    cases = []
    for name, overrides in CASES:
        spelled = written(overrides)
        query = build(catalog_query, spelled)
        cases.append(
            {
                "name": name,
                "query": spelled,
                "admits": sorted(row.path for row in entries if matches(row, query)),
            }
        )

    refusals = []
    for name, overrides in REFUSALS:
        spelled = written({"include_ignored": True, **overrides})
        try:
            build(catalog_query, spelled)
        except ValueError as error:
            refusals.append({"name": name, "query": spelled, "refused": str(error)})
        else:
            raise SystemExit(f"{name}: the contract no longer refuses this; the fixture is stale")

    asymmetries = []
    for name, overrides in ASYMMETRIES:
        spelled = written({"include_ignored": True, **overrides})
        query = build(catalog_query, spelled)
        asymmetries.append(
            {
                "name": name,
                "query": spelled,
                "fdu_refuses": True,
                "consumer_admits": sorted(row.path for row in entries if matches(row, query)),
            }
        )

    fixture = {
        "about": ABOUT,
        "consumer": revision(source_root),
        "corpus": [
            {"path": path, "type": kind, "content": content, "gitignored": ignored}
            for path, kind, content, ignored in CORPUS
        ],
        "consumer_source": segment.splitlines(),
        "cases": cases,
        "refusals": refusals,
        "asymmetries": asymmetries,
    }
    FIXTURE.write_text(json.dumps(fixture, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"wrote {FIXTURE} from {source_root}")


if __name__ == "__main__":
    main()
