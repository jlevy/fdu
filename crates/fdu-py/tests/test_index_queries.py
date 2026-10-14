"""Retained-index results stay coherent across Python conversion and refresh."""

from __future__ import annotations

import runpy
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
