"""End-user smoke test for an installed fdu wheel.

This file intentionally has no third-party imports. CI runs it in a virtual environment
that contains only the built wheel, which catches accidental runtime dependencies and
source-tree imports.
"""

from __future__ import annotations

import ast
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

import fdu
from fdu import _native


def main() -> None:
    root = Path(tempfile.mkdtemp(prefix="fdu-public-api-"))
    (root / "src").mkdir()
    (root / "src" / "main.rs").write_text("fn main() {}", encoding="utf-8")
    (root / "notes.md").write_text("release notes", encoding="utf-8")

    index = fdu.scan(root, scan=fdu.ScanOptions(max_depth=3))
    assert os.path.samefile(index.root, root)
    assert index.status.complete is True
    assert index.status.freshness is fdu.Freshness.FRESH
    assert not index.status.errors

    try:
        fdu.scan(root / "missing")
    except fdu.FilesystemError as error:
        assert isinstance(error, OSError)
        assert isinstance(error, fdu.FduError)
        assert Path(error.filename).name == "missing"
    else:
        raise AssertionError("missing roots must raise fdu.FilesystemError")

    total = index.total()
    assert total.files == 2
    assert total.bytes == 25
    assert total.by_extension[".rs"].files == 1

    children = index.children()
    assert children is not None
    assert {child.name for child in children} == {"notes.md", "src"}
    assert index.rollup("src") is not None
    assert index.rollup("missing") is None

    report = index.report(
        fdu.Query(
            views=(fdu.View.SUMMARY, fdu.View.EXTENSIONS, fdu.View.FILES),
            selection=fdu.Selection(size=fdu.SizeMetric.APPARENT),
        )
    )
    assert report.status.complete is True
    assert report.status.freshness is fdu.Freshness.FRESH
    assert [section.view for section in report.sections] == [
        fdu.View.SUMMARY,
        fdu.View.EXTENSIONS,
        fdu.View.FILES,
    ]
    wire = report.as_dict()
    assert wire["schema"] == "fdu.report/1"
    assert wire["generator"] == f"fdu {fdu.__version__}"
    assert json.loads(json.dumps(wire)) == wire

    package_dir = Path(fdu.__file__).parent
    public_names = {name for name in dir(fdu) if not name.startswith("_") or name == "__version__"}
    assert public_names == set(fdu.__all__), (public_names, set(fdu.__all__))
    assert (package_dir / "py.typed").is_file()
    stub_path = package_dir / "_native.pyi"
    assert stub_path.is_file()
    stub_tree = ast.parse(stub_path.read_text(encoding="utf-8"))
    stub_exports = {
        node.name for node in stub_tree.body if isinstance(node, (ast.ClassDef, ast.FunctionDef))
    }
    stub_exports.add("__version__")
    runtime_exports = {name for name in dir(_native) if not name.startswith("__")}
    runtime_exports.add("__version__")
    assert runtime_exports == stub_exports, (runtime_exports, stub_exports)
    assert _native.Index.__module__ == "fdu._native"
    assert _native.Watch.__module__ == "fdu._native"
    assert importlib.util.find_spec("fdu_py") is None
    contract = _native.contract()
    assert contract["cache_policies"] == [value.value for value in fdu.CachePolicy]
    assert contract["analysis_profiles"] == [value.value for value in fdu.AnalysisProfile]
    assert contract["views"] == [value.value for value in fdu.View]
    assert contract["entry_kinds"] == [value.value for value in fdu.EntryKind]
    assert contract["size_metrics"] == [value.value for value in fdu.SizeMetric]
    assert contract["sort_keys"] == [value.value for value in fdu.SortKey]

    provenance = index.provenance("src")
    assert provenance is not None
    assert provenance.status is fdu.Coverage.COMPLETE
    assert provenance.source is fdu.ValueSource.SCANNED

    mark = index.clock
    (root / "new.txt").write_text("new", encoding="utf-8")
    refresh = index.refresh()
    assert refresh.inserted == 1
    assert refresh.status.complete is True
    changes = index.since(mark)
    assert changes.truncated is False
    assert any(change.path == Path("new.txt") for change in changes.changes)

    cache_root = Path(tempfile.mkdtemp(prefix="fdu-public-cache-"))
    (cache_root / "cached.txt").write_text("cached", encoding="utf-8")
    fdu.open(cache_root, cache=fdu.CachePolicy.AUTO)
    cached = fdu.open(cache_root, cache=fdu.CachePolicy.ONLY)
    # Coverage and currency are independent: a snapshot can cover the complete scope
    # while remaining deliberately stale until revalidation.
    assert cached.status.complete is True
    assert cached.status.freshness is fdu.Freshness.STALE
    status = fdu.cache_status(cache_root)
    assert status is not None and status.recognized

    entrypoint = Path(sys.executable).with_name("fdu.exe" if os.name == "nt" else "fdu")
    version = subprocess.run([entrypoint, "--version"], check=False, capture_output=True, text=True)
    assert version.returncode == 0, version
    assert version.stdout.startswith(f"fdu {fdu.__version__}"), version.stdout
    assert version.stderr == "", version.stderr

    # Rebuild the API report after the refresh above so both sides observe the same
    # filesystem state. The independently scanned CLI document must otherwise match.
    wire = index.report(
        fdu.Query(
            views=(fdu.View.SUMMARY, fdu.View.EXTENSIONS, fdu.View.FILES),
            selection=fdu.Selection(size=fdu.SizeMetric.APPARENT),
        )
    ).as_dict()
    cli_report = subprocess.run(
        [
            entrypoint,
            "--cache",
            "off",
            "--format",
            "json",
            "--view",
            "summary,extensions,files",
            "--size",
            "apparent",
            str(root),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert cli_report.returncode == 0, cli_report
    cli_wire = json.loads(cli_report.stdout)
    assert wire["source"] == "warm_revalidate"
    assert cli_wire["source"] == "cold_scan"
    for volatile in ("scan_started_at", "generated_at", "source"):
        cli_wire.pop(volatile)
        wire.pop(volatile)
    assert wire == cli_wire, (wire, cli_wire)

    print(f"fdu {fdu.__version__} public API ok")


if __name__ == "__main__":
    main()
