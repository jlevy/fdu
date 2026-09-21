"""Directory inventories exercise one-shot, retained, cache, and render boundaries."""

from __future__ import annotations

import json
import os
import shutil
import time
from dataclasses import replace
from pathlib import Path

import fdu
import pytest
from fdu import opened


@pytest.fixture
def builds(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    root = tmp_path / "projects"
    root.mkdir()
    monkeypatch.setenv("XDG_CACHE_HOME", str(tmp_path / "cache"))
    old = int(time.time()) - 60 * 86400
    for name, size in (("a/.venv", 31), ("a/node_modules", 47), ("b/target", 59)):
        directory = root / name
        directory.mkdir(parents=True)
        leaf = directory / "payload"
        leaf.write_bytes(b"x" * size)
        os.utime(leaf, (old, old))
        os.utime(directory, (old, old))
    return root


def inventory(format: fdu.Format = fdu.Format.LONG) -> fdu.Query:
    return fdu.Query(
        format=format,
        selection=fdu.Selection(
            kinds=(fdu.EntryKind.DIR,),
            include=(".venv", "node_modules", "target"),
            modified_before="30d",
            size=fdu.SizeMetric.APPARENT,
            sort=fdu.SortKey.NAME,
        ),
    )


def test_retained_directory_metrics_and_formats_need_no_filesystem(builds: Path) -> None:
    index = fdu.scan(builds)
    shutil.rmtree(builds)
    report = index.report(inventory())
    section = report.sections[0]
    assert isinstance(section, fdu.FilesSection)
    assert section.view is fdu.View.LIST
    assert [row.path.as_posix() for row in section.files] == [
        "a/.venv",
        "a/node_modules",
        "b/target",
    ]
    assert [row.bytes for row in section.files] == [31, 47, 59]
    assert report.age_reference_ns is not None
    for row in section.files:
        assert (row.files, row.dirs, row.complete) == (1, 0, True)
        assert row.age_ns == report.age_reference_ns - row.mtime_ns
    paths = report.render(fdu.Format.PATHS).splitlines()
    assert len(paths) == 3
    long = report.render()
    assert "31 B" in long and "60d" in long
    wire = json.loads(report.render(fdu.Format.JSON))
    assert wire == report.as_dict()
    assert wire["reports"][0]["files"][0]["bytes"] == 31
    lines = [json.loads(line) for line in report.render(fdu.Format.JSONL).splitlines()]
    assert len(lines) == 2
    assert lines[0]["age_reference_ns"] == report.age_reference_ns
    assert "age_ns:" in report.render(fdu.Format.YAML)
    assert report.render() == long
    with pytest.raises(fdu.InvalidArgumentError, match="projection"):
        report.render(fdu.Format.TREE)
    tree = index.report(inventory(fdu.Format.TREE))
    assert isinstance(tree.sections[0], fdu.TreeSection)
    with pytest.raises(fdu.InvalidArgumentError, match="complete flat list"):
        tree.render(fdu.Format.PATHS)


def test_a_directory_at_the_scan_depth_boundary_has_an_unknown_age(builds: Path) -> None:
    # `max_depth=2` retains `a/.venv` and its siblings but never lists them, so their
    # sizes are lower bounds and their age is unknown: no modification bound matches
    # them, and the rows say why rather than reading as old and empty.
    index = fdu.scan(builds, scan=fdu.ScanOptions(max_depth=2))
    bounded = index.report(inventory())
    assert isinstance(bounded.sections[0], fdu.FilesSection)
    assert bounded.sections[0].files == ()
    query = inventory()
    unbounded = index.report(
        replace(query, selection=replace(query.selection, modified_before=None))
    )
    section = unbounded.sections[0]
    assert isinstance(section, fdu.FilesSection)
    assert [row.path.as_posix() for row in section.files] == [
        "a/.venv",
        "a/node_modules",
        "b/target",
    ]
    for row in section.files:
        assert (row.complete, row.bytes, row.age_ns) == (False, 0, None)
    assert "unknown" in unbounded.render()
    wire = json.loads(unbounded.render(fdu.Format.JSON))
    assert wire["reports"][0]["files"][0]["complete"] is False
    assert wire["reports"][0]["files"][0]["age_ns"] is None


def test_cold_warm_and_cache_only_directory_membership_agree(builds: Path) -> None:
    query = inventory(fdu.Format.JSON)
    cold = fdu.report(builds, query, cache=fdu.CachePolicy.REFRESH)
    warm = fdu.open(builds, cache=fdu.CachePolicy.READ_ONLY).report(query)
    # A new file would change an actual scan, but cannot change an unverified cache read.
    (builds / "a/.venv/new").write_bytes(b"new")
    cached = fdu.report(builds, query, cache=fdu.CachePolicy.ONLY)
    assert cached.status.source is fdu.ReportSource.CACHE_ONLY
    for result in (cold, warm, cached):
        section = result.sections[0]
        assert isinstance(section, fdu.FilesSection)
        assert [row.bytes for row in section.files] == [31, 47, 59]
        assert all(row.age_ns is not None for row in section.files)
    # A fresh directory activity disqualifies its root under the age filter.
    current = fdu.report(builds, query, cache=fdu.CachePolicy.OFF)
    section = current.sections[0]
    assert isinstance(section, fdu.FilesSection)
    assert [row.bytes for row in section.files] == [47, 59]


def test_list_default_and_format_compatibility(builds: Path) -> None:
    index = fdu.scan(builds)
    default = index.report(fdu.Query()).render()
    assert index.report(fdu.Query(views=(fdu.View.LIST,))).render() == default
    assert index.report(fdu.Query(format=fdu.Format.TREE)).render() == default
    assert index.report(fdu.Query(views=(fdu.View.TREE,))).render() == default
    for views in ((fdu.View.SUMMARY,), (fdu.View.LIST, fdu.View.SUMMARY), "full"):
        for format in (fdu.Format.TREE, fdu.Format.PATHS, fdu.Format.LONG):
            with pytest.raises(fdu.InvalidArgumentError, match="single list"):
                index.report(fdu.Query(views=views, format=format))
    # Flat defaults are complete even though the ordinary tree is folded.
    query = replace(inventory(), selection=replace(inventory().selection, limit=1))
    report = index.report(query)
    section = report.sections[0]
    assert isinstance(section, fdu.FilesSection)
    assert len(section.files) == 1
    assert section.bound is not None


def test_opened_reports_use_the_same_directory_query_contract(builds: Path) -> None:
    with opened.OpenedIndex.open(builds) as index:
        state = index.state()
        for _ in range(40):
            if state.state.coverage.kind is opened.CoverageKind.COMPLETE:
                break
            index.changes(state.change_cursor, timeout=0.25)
            state = index.state()
        assert state.state.coverage.kind is opened.CoverageKind.COMPLETE
        response = index.read(
            opened.ReportProjection(query=inventory(fdu.Format.LONG)),
            opened.ReportProjection(query=inventory(fdu.Format.JSON)),
            opened.ReportProjection(query=inventory(fdu.Format.TREE)),
        )
        for result in response.results[:2]:
            assert result.kind == "report"
            report = result.value
            section = report.sections[0]
            assert isinstance(section, fdu.FilesSection)
            assert [row.bytes for row in section.files] == [31, 47, 59]
            assert report.age_reference_ns is not None
            assert all(
                row.age_ns == report.age_reference_ns - row.mtime_ns for row in section.files
            )
        result = response.results[2]
        assert result.kind == "report"
        assert isinstance(result.value.sections[0], fdu.TreeSection)
