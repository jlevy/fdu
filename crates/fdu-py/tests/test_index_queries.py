"""Retained-index results stay coherent across Python conversion and refresh."""

from __future__ import annotations

import os
import runpy
import subprocess
import sys
from pathlib import Path
from typing import cast

import fdu
import pytest
from fdu import _native


@pytest.mark.parametrize("path", [Path(), Path("src")])
def test_rollup_keeps_provenance_when_refresh_runs_before_conversion(
    tmp_path: Path, path: Path
) -> None:
    root = tmp_path / "tree"
    directory = root / "src"
    directory.mkdir(parents=True)
    payload = directory / "main.rs"
    payload.write_bytes(b"payload")
    index = fdu.open(root, cache=fdu.CachePolicy.OFF)
    old = index.total() if path == Path() else index.rollup(path)
    assert old is not None and old.provenance is not None

    def refresh_after_native_read() -> None:
        if path == Path():
            (root / "added.txt").write_bytes(b"new")
        else:
            payload.unlink()
            directory.rmdir()
        index.refresh()

    class NativeRead:
        def total(self) -> dict[str, object]:
            raw = index._native.total()
            refresh_after_native_read()
            return raw

        def rollup(self, relative: str | Path) -> dict[str, object] | None:
            raw = index._native.rollup(relative)
            refresh_after_native_read()
            return raw

        def provenance(self, relative: str | Path) -> dict[str, object] | None:
            if path == Path():
                raise AssertionError("roll-up provenance must come from the same native response")
            return index._native.provenance(relative)

    interleaved = fdu.Index(cast(_native.Index, NativeRead()))
    result = interleaved.total() if path == Path() else interleaved.rollup(path)
    assert result == old
    if path != Path():
        assert index.provenance(path) is None
        assert index.rollup(path) is None
    else:
        assert index.total().files == old.files + 1


def test_adapter_uses_typed_freshness_without_constructing_a_report(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    (tmp_path / "main.rs").write_bytes(b"payload")
    index = fdu.scan(tmp_path)

    def refuse_report(*_args: object, **_kwargs: object) -> fdu.Report:
        raise AssertionError("reading freshness must not construct a report")

    monkeypatch.setattr(fdu.Index, "report", refuse_report)
    assert index.freshness is fdu.Freshness.FRESH
    example = Path(__file__).resolve().parents[1] / "examples" / "rollup_adapter.py"
    namespace = runpy.run_path(str(example))
    summary = namespace["summarize"](index)
    assert summary is not None
    assert summary.freshness == fdu.Freshness.FRESH.value
    assert (summary.files, summary.apparent_bytes, summary.complete) == (1, 7, True)


def test_since_gives_each_upsert_its_change_time(tmp_path: Path) -> None:
    """``Index.since()`` carries each upsert's change time, the clock a default window
    admits it by, as a watch's records and the report's own rows do (review I4 on #209).
    """
    root = tmp_path / "tree"
    root.mkdir()
    (root / "kept.txt").write_bytes(b"kept")
    index = fdu.open(root, cache=fdu.CachePolicy.OFF)
    mark = index.clock
    added = root / "added.txt"
    added.write_bytes(b"new")
    assert index.refresh().inserted == 1
    changes = index.since(mark)
    assert changes.truncated is False
    upserts = {
        change.path: change for change in changes.changes if change.kind is fdu.ChangeKind.UPSERT
    }
    change = upserts[Path("added.txt")]
    files = index.report(fdu.Query(views=(fdu.View.FILES,))).sections[0]
    assert isinstance(files, fdu.FilesSection)
    (row,) = (row for row in files.files if row.path == Path("added.txt"))
    assert change.changed_ns is not None
    assert (change.mtime_ns, change.changed_ns) == (row.mtime_ns, row.changed_ns)
    if os.name == "posix":
        status = added.stat()
        assert change.changed_ns == max(status.st_mtime_ns, status.st_ctime_ns)
    assert all(change.path != Path() for change in changes.changes), changes


@pytest.mark.skipif(
    sys.platform != "linux" or os.geteuid() != 0,
    reason="remounting the root's filesystem needs root on Linux",
)
def test_since_never_lists_the_root_after_its_filesystem_is_renumbered(tmp_path: Path) -> None:
    """A whole-root refresh after the root's filesystem was mounted again under a new
    number records the new device in the root's own entry, which holds nothing else and
    which no report counts. No change record carries it: it read as an upsert of ``.``
    with zero bytes and a 1970 modification time (review I4 on #209).
    """
    root, hold = tmp_path / "tree", tmp_path / "hold"
    root.mkdir()
    hold.mkdir()

    def mount(at: Path) -> None:
        subprocess.run(["mount", "-t", "tmpfs", "-o", "size=4m", "tmpfs", str(at)], check=True)

    mount(root)
    try:
        (root / "sub").mkdir()
        (root / "sub" / "one.bin").write_bytes(b"x" * 100)
        index = fdu.open(root, cache=fdu.CachePolicy.OFF)
        before = os.stat(root).st_dev
        subprocess.run(["umount", str(root)], check=True)
        # The kernel deals the lowest free number, so the freed one is held elsewhere and
        # the root's next mount is numbered afresh.
        mount(hold)
        mount(root)
        (root / "sub").mkdir()
        (root / "sub" / "one.bin").write_bytes(b"x" * 100)
        assert os.stat(root).st_dev != before, "the remount must number the root afresh"
        mark = index.clock
        refreshed = index.refresh()
        assert refreshed.updated >= 1, refreshed
        changes = index.since(mark)
        assert changes.truncated is False
        assert changes.changes, "the refresh recorded the remount"
        assert all(change.path != Path() for change in changes.changes), changes
    finally:
        for at in (root, hold):
            subprocess.run(["umount", str(at)], check=False, capture_output=True)
