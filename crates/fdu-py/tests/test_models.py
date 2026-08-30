"""Unit tests for the dependency-free public Python value model."""

from __future__ import annotations

from dataclasses import FrozenInstanceError
from datetime import UTC, datetime

import pytest
from fdu import (
    Analysis,
    AnalysisOptions,
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
    assert AnalysisOptions().analyze == Analysis.NONE
    # Empty means "let the analyzers choose", which is the CLI semantics this test is
    # named for: `--analyze code` with no `--view` reports languages, not tree.
    assert Query().views == ()


def test_invalid_option_values_fail_before_crossing_native_boundary() -> None:
    with pytest.raises(ValueError, match="max_depth"):
        ScanOptions(max_depth=-1)
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
