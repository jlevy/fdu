"""Unit tests for the dependency-free public Python value model."""

from __future__ import annotations

import json
import os
import re
import sys
from dataclasses import FrozenInstanceError
from datetime import UTC, datetime
from pathlib import Path

import fdu
import pytest
from fdu import (
    Analysis,
    AnalysisOptions,
    Bound,
    CachePolicy,
    EntryKind,
    ExtensionsSection,
    ExtensionTally,
    FilesSection,
    IgnoredEntries,
    IgnoredTally,
    Query,
    ScanOptions,
    Selection,
    SizeMetric,
    SummarySection,
    TreeSection,
    View,
    WatchOptions,
    _native,
    opened,
)
from fdu._api import (
    FduError,
    FilesystemError,
    InvalidArgumentError,
    _call,
    _loads_json,
    _loads_object,
    _query_kwargs,
)
from fdu._models import _wire_path, cache_status_from_dict, report_from_dict, status_from_dict
from fdu.opened import _opened_call, _projection_wire


def test_public_options_are_typed_immutable_values() -> None:
    scan = ScanOptions(max_depth=2, one_filesystem=True)
    assert scan.max_depth == 2
    assert scan.one_filesystem is True
    with pytest.raises(FrozenInstanceError):
        scan.max_depth = 3  # type: ignore[misc]

    query = Query(
        views=(View.SUMMARY, View.TYPES),
        selection=Selection(limit=10, size=SizeMetric.APPARENT),
    )
    assert query.views == (View.SUMMARY, View.TYPES)
    assert query.selection.limit == 10


def test_public_defaults_match_cli_semantics() -> None:
    assert CachePolicy.AUTO.value == "auto"
    assert ScanOptions() == ScanOptions(max_depth=None, one_filesystem=False)
    # Every surface observes `.gitignore` by default, as the command line does unless
    # `--no-gitignore`, and selects every entry whatever its classification.
    assert ScanOptions().read_controls is True
    assert Selection().ignored is IgnoredEntries.INCLUDE
    assert _query_kwargs(Query())["ignored"] == "include"
    only = Query(selection=Selection(ignored=IgnoredEntries.ONLY))
    assert _query_kwargs(only)["ignored"] == "only"
    assert AnalysisOptions().analyze == Analysis.NONE
    # Empty means "let the analyzers choose", which is the CLI semantics this test is
    # named for: `--analyze code` with no `--view` reports languages, not tree.
    assert Query().views == ()
    # A watch answers the same request a report of the same index answers, so it brings no
    # view of its own: this named `files`, and `--watch` never has.
    assert WatchOptions().query == Query()
    # The page denominator and the size metric come from the request model's defaults
    # table, so this package states neither on its own.
    assert Query().words_per_page == _native.DEFAULT_WORDS_PER_PAGE
    assert Selection().size.value == _native.DEFAULT_SIZE


def test_invalid_option_values_fail_before_crossing_native_boundary() -> None:
    with pytest.raises(ValueError, match="max_depth"):
        ScanOptions(max_depth=-1)
    with pytest.raises(ValueError, match="control_budget"):
        ScanOptions(control_budget=-1)
    with pytest.raises(ValueError, match="control_budget"):
        opened.OpenedOptions(control_budget=-1)
    with pytest.raises(ValueError, match="control_line_limit"):
        ScanOptions(control_line_limit=-1)
    with pytest.raises(ValueError, match="control_line_limit"):
        opened.OpenedOptions(control_line_limit=-1)
    with pytest.raises(ValueError, match="workers"):
        AnalysisOptions(workers=-1)
    with pytest.raises(ValueError, match="max_size"):
        opened.EntrySelection(max_size=-1)
    for interval in (float("nan"), float("inf"), float("-inf"), 0.0, -1.0, 1e-300, 0.1e-9):
        with pytest.raises(ValueError, match="interval"):
            WatchOptions(interval=interval)


def test_opened_entry_selection_composes_the_stable_query_selection() -> None:
    selection = opened.EntrySelection(
        query=Selection(kinds=(EntryKind.FILE,)),
        max_size=100,
        exclude_ignored=True,
        logical_extensions=(".js.map",),
        exact_names=("makefile",),
        terminal_extensions=(".rs",),
        ancestor_names=("src",),
    )
    projection = opened.Flat(selection=selection)
    assert projection.selection.query.kinds == (EntryKind.FILE,)
    assert projection.selection.max_size == 100
    assert projection.selection.exact_names == ("makefile",)


@pytest.mark.parametrize(
    ("arguments", "message"),
    [
        ({"terminal_extensions": (".rs", ".rs")}, "terminal_extensions entries must be unique"),
        # Wrong twice: the same fault as the Rust `validate` names, in CatalogQuery's order.
        ({"terminal_extensions": ("rs", "rs")}, "terminal_extensions entries must be unique"),
        ({"terminal_extensions": (".RS", "rs")}, "must start with a dot"),
        ({"terminal_extensions": (".tar.gz", ".RS")}, "must be lowercase"),
        ({"terminal_extensions": ("rs",)}, "must start with a dot"),
        ({"terminal_extensions": (".RS",)}, "must be lowercase"),
        ({"terminal_extensions": (".\u00c9e",)}, "must be lowercase"),
        ({"terminal_extensions": (".",)}, "canonical terminal suffixes"),
        ({"terminal_extensions": (".tar.gz",)}, "canonical terminal suffixes"),
        ({"terminal_extensions": (".a/b",)}, "canonical terminal suffixes"),
        ({"terminal_extensions": (".a\\b",)}, "canonical terminal suffixes"),
        ({"ancestor_names": ("src", "src")}, "ancestor_names entries must be unique"),
        ({"ancestor_names": ("..", "..")}, "ancestor_names entries must be unique"),
        ({"ancestor_names": ("",)}, "exact path-component names"),
        ({"ancestor_names": (".",)}, "exact path-component names"),
        ({"ancestor_names": ("..",)}, "exact path-component names"),
        ({"ancestor_names": ("a/b",)}, "exact path-component names"),
        ({"ancestor_names": ("a\\b",)}, "exact path-component names"),
    ],
)
def test_opened_entry_selection_refuses_what_could_never_match(
    arguments: dict[str, tuple[str, ...]], message: str
) -> None:
    with pytest.raises(ValueError, match=re.escape(message)):
        opened.EntrySelection(**arguments)  # pyright: ignore[reportArgumentType]


def test_opened_entry_selection_admits_canonical_suffixes_and_escaped_components() -> None:
    selection = opened.EntrySelection(
        terminal_extensions=(".rs", ".c++"), ancestor_names=("x%FF", "..foo")
    )
    assert selection.ancestor_names == ("x%FF", "..foo")
    with pytest.raises(TypeError, match="tuple of strings"):
        opened.EntrySelection(ancestor_names="src")  # pyright: ignore[reportArgumentType]


def test_opened_tree_defaults_to_one_visible_level_and_encodes_its_shape() -> None:
    # The binding's defaults when a field is absent, so a caller who says nothing gets
    # the same page with or without these fields.
    default = opened.Tree()
    assert (default.depth, default.include_ignored) == (1, True)
    assert _projection_wire(default) == {
        "kind": "tree",
        "path": "",
        "depth": 1,
        "include_ignored": True,
        "page": {"limit": 256, "max_work": 100_000},
    }

    deep = _projection_wire(opened.Tree("src", depth=Bound.ALL, include_ignored=False))
    # The native grammar spells an unbounded depth the way `--depth all` does.
    assert (deep["depth"], deep["include_ignored"]) == ("all", False)
    assert _projection_wire(opened.Tree(depth=3))["depth"] == 3


@pytest.mark.parametrize(
    ("depth", "error"),
    [
        (0, ValueError),
        (-1, ValueError),
        # `bool` is an `int`, so without the guard `True` would silently mean one level.
        (True, TypeError),
        ("all", TypeError),
        (1.5, TypeError),
    ],
)
def test_opened_tree_depth_fails_before_crossing_native_boundary(
    depth: object,
    error: type[Exception],
) -> None:
    with pytest.raises(error, match="depth"):
        opened.Tree(depth=depth)  # type: ignore[arg-type]


def test_opened_tree_include_ignored_must_be_a_bool() -> None:
    with pytest.raises(TypeError, match="include_ignored"):
        opened.Tree(include_ignored="no")  # type: ignore[arg-type]


def test_opened_options_take_the_registry_document_not_its_path() -> None:
    assert opened.OpenedOptions().type_rules is None
    document = '[[kind]]\nid = "notes"\nfamily = "prose"\nextensions = ["rs"]\n'
    assert opened.OpenedOptions(type_rules=document).type_rules == document
    with pytest.raises(TypeError, match="type_rules"):
        opened.OpenedOptions(type_rules=Path("registry.toml"))  # type: ignore[arg-type]


def test_bare_strings_are_rejected_for_sequence_fields() -> None:
    # A str is iterable, so without the guard include="*.rs" would silently run as the
    # per-character patterns "*", ".", "r", "s" and match nearly everything.
    with pytest.raises(TypeError, match="include"):
        Selection(include="*.rs")  # type: ignore[arg-type]
    with pytest.raises(TypeError, match="kinds"):
        Selection(kinds="file")  # type: ignore[arg-type]
    with pytest.raises(TypeError, match="views"):
        Query(views=View.TREE)  # type: ignore[arg-type]


def test_naive_datetimes_gain_an_explicit_local_offset() -> None:
    naive = datetime(2026, 8, 1, 12, 30)
    aware = datetime(2026, 8, 1, 12, 30, tzinfo=UTC)
    kwargs = _query_kwargs(Query(selection=Selection(modified_since=naive, modified_before=aware)))
    since = datetime.fromisoformat(str(kwargs["modified_since"]))
    assert since.tzinfo is not None
    assert since == naive.astimezone()
    assert kwargs["modified_before"] == "2026-08-01T12:30:00+00:00"


def test_malformed_wire_reports_fail_loudly() -> None:
    # Wire validation must be real raises, not asserts, so it survives python -O.
    with pytest.raises(TypeError, match="sections"):
        report_from_dict({"reports": "nope"})


@pytest.mark.skipif(os.name != "posix", reason="Unix byte paths require a POSIX filesystem")
def test_wire_paths_prefer_lossless_raw_identity() -> None:
    raw = {"path": "n�", "path_raw": {"encoding": "unix-bytes", "hex": "6e80"}}
    assert os.fsencode(_wire_path(raw)) == b"n\x80"

    wire = _envelope(
        [
            {
                "view": "files",
                "bound": None,
                "files": [
                    {
                        **raw,
                        "kind": "file",
                        "bytes": 1,
                        "allocated": 1,
                        "mtime_ns": 0,
                        "ignored": None,
                    }
                ],
            }
        ]
    )
    wire.update({"root": "/�", "root_raw": {"encoding": "unix-bytes", "hex": "2f80"}})
    report = report_from_dict(wire)
    assert os.fsencode(report.root) == b"/\x80"
    section = report.sections[0]
    assert isinstance(section, FilesSection)
    assert os.fsencode(section.files[0].path) == b"n\x80"

    class NativeIndex:
        def since(self, _clock: int) -> dict[str, object]:
            return {
                "truncated": False,
                "clock": 1,
                "ops": [
                    {
                        "op": "upsert",
                        "clock": 1,
                        "path": "n�",
                        "path_raw": {"encoding": "unix-bytes", "hex": "6e80"},
                        "kind": "file",
                        "bytes": 1,
                        "allocated": 1,
                        "mtime_ns": 0,
                        "ignored": False,
                    }
                ],
            }

    changed = fdu.Index(NativeIndex()).since(0)  # type: ignore[arg-type]
    assert os.fsencode(changed.changes[0].path) == b"n\x80"

    status = status_from_dict(
        {
            "complete": False,
            "coverage": {"kind": "partial", "reason": "inaccessible"},
            "errors": [
                {
                    "path": "n�",
                    "path_raw": {"encoding": "unix-bytes", "hex": "6e80"},
                    "kind": "permission",
                    "message": "denied",
                }
            ],
            "errors_omitted": 0,
            "ignore_rules": None,
        }
    )
    assert status.errors[0].path is not None
    assert os.fsencode(status.errors[0].path) == b"n\x80"

    cache = cache_status_from_dict(
        {
            **raw,
            "bytes": 1,
            "state": "unrecognized",
            "stale_reason": None,
            "format_version": None,
            "leftover_kind": None,
            "root": None,
            "entries": None,
            "identity": None,
            "content": None,
        }
    )
    assert os.fsencode(cache.path) == b"n\x80"


def test_tree_parser_is_iterative_at_filesystem_depth() -> None:
    depth = 4_000
    node: dict[str, object] = {
        "name": "leaf",
        "path": "leaf",
        "kind": "dir",
        "bytes": 0,
        "allocated": 0,
        "files": 0,
        "dirs": 0,
        "ignored": None,
        "newest_mtime_ns": None,
        "truncated": False,
        "children": [],
    }
    for level in range(depth):
        node = {**node, "name": str(level), "path": str(level), "children": [node]}

    wire = _envelope([{"view": "tree", "tree": node}])
    report = report_from_dict(wire)
    section = report.sections[0]
    assert isinstance(section, TreeSection)
    parsed = section.tree
    visited = 0
    while parsed.children:
        parsed = parsed.children[0]
        visited += 1
    assert visited == depth
    copied = report.as_dict()
    assert copied is not wire
    assert copied["reports"] is not wire["reports"]


def test_native_json_fallback_parses_a_deep_rendered_report_end_to_end() -> None:
    depth = 4_000
    leaf = {
        "name": "leaf",
        "path": "leaf",
        "kind": "dir",
        "bytes": 0,
        "allocated": 0,
        "files": 0,
        "dirs": 0,
        "ignored": None,
        "newest_mtime_ns": None,
        "truncated": False,
        "children": [],
    }
    node = json.dumps(leaf, separators=(",", ":"))
    for level in range(depth):
        parent = {key: value for key, value in leaf.items() if key != "children"}
        parent.update(name=str(level), path=str(level))
        fields = json.dumps(parent, separators=(",", ":"))
        node = f'{fields[:-1]},"children":[{node}]}}'
    marker = "__DEEP_TREE__"
    rendered = json.dumps(_envelope([{"view": "tree", "tree": marker}])).replace(
        f'"{marker}"', node
    )

    report = report_from_dict(_loads_object(rendered))
    copied = report.as_dict()
    parsed: object = copied["reports"]
    assert isinstance(parsed, list)
    tree = parsed[0]["tree"]
    visited = 0
    while tree["children"]:
        tree = tree["children"][0]
        visited += 1
    assert visited == depth


@pytest.mark.parametrize(
    "document",
    ["[1,]", '{"a":1,}', '{"a" 1}', "{} trailing", "[", '{"a":}'],
)
def test_iterative_json_fallback_rejects_malformed_framing(
    monkeypatch: pytest.MonkeyPatch, document: str
) -> None:
    def recurse(_document: str) -> object:
        raise RecursionError

    monkeypatch.setattr(json, "loads", recurse)
    with pytest.raises(json.JSONDecodeError):
        _loads_json(document)


def test_iterative_json_fallback_preserves_exact_integers(monkeypatch: pytest.MonkeyPatch) -> None:
    def recurse(_document: str) -> object:
        raise RecursionError

    monkeypatch.setattr(json, "loads", recurse)
    value = _loads_json('{"n":18446744073709551615,"nested":[true,null,"x"]}')
    assert value == {"n": 18_446_744_073_709_551_615, "nested": [True, None, "x"]}


def test_deep_malformed_map_key_reports_json_error_without_recursing() -> None:
    depth = 4_000
    nested_key = "[" * depth + "0" + "]" * depth
    document = "[" * depth + "{" + nested_key + ":1}" + "]" * depth
    with pytest.raises(json.JSONDecodeError):
        _loads_json(document)


def _envelope(sections: list[dict[str, object]]) -> dict[str, object]:
    return {
        "schema": "fdu.report/7",
        "generator": "fdu 0.1.0",
        "root": "/root",
        "request": {
            "scope": {
                "max_depth": None,
                "follow_symlinks": False,
                "one_filesystem": False,
                "exclude_special": False,
                "read_controls": True,
            },
            "analyze": [],
            "size": "allocated",
            "views": [str(section["view"]) for section in sections],
            "omitted_views": [],
        },
        "status": {
            "complete": True,
            "coverage": {"kind": "complete"},
            "errors": [],
            "errors_omitted": 0,
        },
        "provenance": {
            "source": "cold_scan",
            "freshness": "fresh",
            "scan_started_at": None,
            "generated_at": "2026-09-15T00:00:00.000000000Z",
            "tiers": {
                "entries": {
                    "source": "scanned",
                    "freshness": "fresh",
                    "observed_at_ns": 0,
                },
                "content": None,
            },
        },
        "ignore_rules": None,
        "analysis": None,
        "reports": sections,
    }


@pytest.mark.parametrize("positive", [True, False])
def test_directory_age_extremes_preserve_exact_signed_integers(positive: bool) -> None:
    reference = (2**63 - 1) if positive else -(2**63)
    modified = -(2**63) if positive else (2**63 - 1)
    age = reference - modified
    wire = _envelope(
        [
            {
                "view": "list",
                "bound": None,
                "files": [
                    {
                        "path": "extreme.txt",
                        "kind": "file",
                        "bytes": 0,
                        "allocated": 0,
                        "mtime_ns": modified,
                        "age_ns": age,
                        "ignored": None,
                    }
                ],
            }
        ]
    )
    wire["age_reference_ns"] = reference
    report = report_from_dict(wire)
    section = report.sections[0]
    assert isinstance(section, FilesSection)
    assert report.age_reference_ns == reference
    assert section.files[0].mtime_ns == modified
    assert section.files[0].age_ns == age
    assert report.as_dict()["reports"][0]["files"][0]["age_ns"] == age


def test_every_row_parses_its_ignored_share_and_keeps_null_distinct_from_zero() -> None:
    share = {"files": 1, "dirs": 1, "bytes": 128, "allocated": 4096}
    leaf = {
        "name": "dist",
        "path": "dist",
        "kind": "dir",
        "bytes": 128,
        "allocated": 4096,
        "files": 1,
        "dirs": 0,
        "ignored": {**share, "dirs": 0},
        "newest_mtime_ns": 1,
        "truncated": False,
        "children": [],
    }
    root = {**leaf, "name": ".", "path": "", "dirs": 1, "ignored": share, "children": [leaf]}
    summary = {"files": 2, "dirs": 1, "bytes": 164, "allocated": 8192, "ignored": share}
    report = report_from_dict(
        _envelope(
            [
                {"view": "summary", "summary": {**summary, "newest_mtime_ns": 2}},
                {"view": "tree", "tree": root},
                {
                    "view": "extensions",
                    "bound": None,
                    "extensions": [
                        {
                            "extension": ".gz",
                            "files": 1,
                            "bytes": 128,
                            "allocated": 4096,
                            "ignored": {"files": 1, "bytes": 128, "allocated": 4096},
                        },
                        {
                            "extension": ".rs",
                            "files": 1,
                            "bytes": 36,
                            "allocated": 4096,
                            "ignored": {"files": 0, "bytes": 0, "allocated": 0},
                        },
                    ],
                },
                {
                    "view": "files",
                    "bound": None,
                    "files": [
                        {
                            "path": "dist",
                            "kind": "dir",
                            "bytes": 0,
                            "allocated": 0,
                            "mtime_ns": 0,
                            "ignored": True,
                        },
                        {
                            "path": "src",
                            "kind": "dir",
                            "bytes": 0,
                            "allocated": 0,
                            "mtime_ns": 0,
                            "ignored": None,
                        },
                    ],
                },
            ]
        )
    )
    summary_section, tree_section, extensions_section, files_section = report.sections
    assert isinstance(summary_section, SummarySection)
    assert summary_section.summary.ignored == IgnoredTally(1, 1, 128, 4096)
    assert isinstance(tree_section, TreeSection)
    assert tree_section.tree.ignored == IgnoredTally(1, 1, 128, 4096)
    assert tree_section.tree.children[0].ignored == IgnoredTally(1, 0, 128, 4096)
    assert isinstance(extensions_section, ExtensionsSection)
    gz, rs = extensions_section.extensions
    assert gz.ignored == ExtensionTally(1, 128, 4096)
    assert rs.ignored == ExtensionTally(0, 0, 0), "a zero share is not null"
    assert isinstance(files_section, FilesSection)
    assert [row.ignored for row in files_section.files] == [True, None]

    malformed = {**summary, "newest_mtime_ns": 2, "ignored": 1}
    with pytest.raises(TypeError, match="ignored"):
        report_from_dict(_envelope([{"view": "summary", "summary": malformed}]))


def test_metadata_only_metric_rows_keep_unrequested_units_absent() -> None:
    row = {
        "id": "total",
        "family": "unknown",
        "files": 1,
        "bytes": 1,
        "allocated": 1,
        "share": {"numerator": 1, "denominator": 1},
        "metrics": {},
        "coverage": {},
        "detection": {
            "sources": {"unknown": 1},
            "confidence": {"unknown": 1},
            "flags": {"generated": 0, "vendored": 0, "documentation": 0},
        },
    }
    report = report_from_dict(
        _envelope(
            [
                {
                    "view": "types",
                    "metrics": {
                        "group": "type",
                        "share_metric": "allocated_bytes",
                        "bound": None,
                        "total": row,
                        "rows": [],
                    },
                }
            ]
        )
    )
    section = report.sections[0]
    assert isinstance(section, fdu.MetricsSection)
    assert section.total.metrics == fdu.MetricValues()
    assert section.total.lines_coverage is None
    assert section.total.code_coverage is None
    assert section.total.words_coverage is None
    assert section.total.pages is None


@pytest.mark.parametrize(
    ("native_error", "public_error"),
    [
        (ValueError("bad option"), InvalidArgumentError),
        (OSError(2, "missing", "root"), FilesystemError),
        (RuntimeError("native failure"), FduError),
    ],
)
def test_native_failures_use_the_public_exception_hierarchy(
    native_error: Exception,
    public_error: type[Exception],
) -> None:
    def fail() -> None:
        raise native_error

    with pytest.raises(public_error):
        _call(fail)


@pytest.mark.parametrize(
    ("native_error", "public_error"),
    [
        (ValueError("bad option"), InvalidArgumentError),
        (OverflowError("can't convert negative int to unsigned"), InvalidArgumentError),
        (OSError(2, "missing", "root"), FilesystemError),
        # A poisoned index, or any other operational failure the binding does not type,
        # is still an opened-root failure a caller can catch as one.
        (RuntimeError("index lock poisoned"), opened.OpenedIndexError),
    ],
)
def test_opened_failures_use_the_opened_exception_hierarchy(
    native_error: Exception,
    public_error: type[Exception],
) -> None:
    def fail() -> None:
        raise native_error

    with pytest.raises(public_error):
        _opened_call(fail)


def test_a_scope_this_build_cannot_honour_is_a_refused_request(tmp_path: Path) -> None:
    """An unsupported scan scope is an argument error, not the engine failing.

    The kind is what this pins. The same refusal exits 2 on the command line, and a caller
    here catches it as ``InvalidArgumentError``, which is a ``ValueError``; reporting it as
    an engine error on one surface and a refused request on the other made one request
    have two kinds of outcome, which is what the path-independence matrix measured for
    ``--one-filesystem`` on Windows.

    ``follow_symlinks`` is refused on every platform and ``one_filesystem`` only where the
    platform has no device identity, so the first case is how this is checked anywhere.
    """
    (tmp_path / "file.txt").write_text("contents", encoding="utf-8")

    with pytest.raises(InvalidArgumentError, match="follow_symlinks"):
        opened.OpenedIndex.open(tmp_path, opened.OpenedOptions(follow_symlinks=True))

    if sys.platform != "win32":
        return
    scope = ScanOptions(one_filesystem=True)
    for route in (fdu.report, fdu.open, fdu.scan):
        with pytest.raises(InvalidArgumentError, match="one_filesystem"):
            route(tmp_path, scan=scope)


def test_cache_models_read_wire_presence_without_native_padding() -> None:
    absent = cache_status_from_dict(
        {"path": "missing", "bytes": 0, "state": "absent", "content": None}
    )
    assert absent.root is None and absent.entries is None and absent.identity is None
    assert absent.stale_reason is None and absent.leftover_kind is None
    stale = cache_status_from_dict(
        {
            "path": "stale",
            "bytes": 9,
            "state": "stale",
            "stale_reason": "other_engine",
            "format_version": None,
            "content": {
                "bytes": 4,
                "state": "stale",
                "stale_reason": "older_format",
                "format_version": 1,
            },
        }
    )
    assert stale.stale_reason is fdu.StaleReason.OTHER_ENGINE
    assert stale.content is not None
    assert stale.content.format_version == 1 and stale.content.identity is None


def test_directory_listing_withdrawal_transition_is_typed() -> None:
    transition = opened._transition({"kind": "directory_incomplete", "path": "dir"})
    assert transition.kind is opened.StateTransitionKind.DIRECTORY_INCOMPLETE
    assert transition.path == Path("dir")
    assert transition.previous_freshness is None
    assert transition.current_freshness is None
    assert transition.previous_state is None
    assert transition.current_state is None
