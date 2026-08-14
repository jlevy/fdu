"""Unit tests for the dependency-free public Python value model."""

from __future__ import annotations

from dataclasses import FrozenInstanceError

import pytest
from fdu import (
    AnalysisOptions,
    AnalysisProfile,
    CachePolicy,
    Query,
    ScanOptions,
    Selection,
    SizeMetric,
    View,
)
from fdu._api import FduError, FilesystemError, InvalidArgumentError, _call


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
    assert AnalysisOptions().profile is AnalysisProfile.NONE
    assert Query().views == (View.TREE,)


def test_invalid_option_values_fail_before_crossing_native_boundary() -> None:
    with pytest.raises(ValueError, match="max_depth"):
        ScanOptions(max_depth=-1)
    with pytest.raises(ValueError, match="workers"):
        AnalysisOptions(workers=-1)
    with pytest.raises(ValueError, match="words_per_page"):
        Query(words_per_page=0)


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
