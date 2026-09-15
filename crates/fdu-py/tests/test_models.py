"""Unit tests for the dependency-free public Python value model."""

from __future__ import annotations

import re
from dataclasses import FrozenInstanceError
from datetime import UTC, datetime
from pathlib import Path

import pytest
from fdu import (
    Analysis,
    AnalysisOptions,
    Bound,
    CachePolicy,
    EntryKind,
    Query,
    ScanOptions,
    Selection,
    SizeMetric,
    View,
    opened,
)
from fdu._api import FduError, FilesystemError, InvalidArgumentError, _call, _query_kwargs
from fdu._models import report_from_dict
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
    # The one deliberate departure: an index observes `.gitignore` control state by
    # default, while the command line turns it off because no command-line view reads it.
    assert ScanOptions().read_controls is True
    assert AnalysisOptions().analyze == Analysis.NONE
    # Empty means "let the analyzers choose", which is the CLI semantics this test is
    # named for: `--analyze code` with no `--view` reports languages, not tree.
    assert Query().views == ()


def test_invalid_option_values_fail_before_crossing_native_boundary() -> None:
    with pytest.raises(ValueError, match="max_depth"):
        ScanOptions(max_depth=-1)
    with pytest.raises(ValueError, match="control_budget"):
        ScanOptions(control_budget=-1)
    with pytest.raises(ValueError, match="control_budget"):
        opened.OpenedOptions(control_budget=-1)
    with pytest.raises(ValueError, match="workers"):
        AnalysisOptions(workers=-1)
    with pytest.raises(ValueError, match="words_per_page"):
        Query(words_per_page=0)
    with pytest.raises(ValueError, match="max_size"):
        opened.EntrySelection(max_size=-1)


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
