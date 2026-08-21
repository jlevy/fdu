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
from datetime import datetime, timedelta
from pathlib import Path

import fdu
from fdu import _native


def check_every_view(root: Path) -> None:
    """Every view must reach the typed surface, and say what it bounded.

    This exists because the CLI and the binding held separate view vocabularies: `largest`
    and `recent` shipped, the CLI accepted them, the binding rejected them, and
    `make check` stayed green throughout because nothing in the Python tests named a new
    view. The loop is over `fdu.View` rather than a written list, so a view added later
    cannot be left out of it.
    """

    index = fdu.scan(str(root))
    analyzed = fdu.scan(str(root), analysis=fdu.AnalysisOptions(analyze=fdu.Analysis.ALL))
    for view in fdu.View:
        if view is fdu.View.FULL:
            continue
        # `documents` has no metadata-only projection, so it needs the analysed index.
        source = analyzed if view is fdu.View.DOCUMENTS else index
        report = source.report(fdu.Query(views=(view,)))
        assert len(report.sections) == 1, (view, report.sections)
        assert report.sections[0].view is view, (view, report.sections[0].view)

    # A bounded section reports what it dropped; an unbounded one reports nothing.
    bounded = index.report(fdu.Query(views=(fdu.View.FILES,), selection=fdu.Selection(limit=1)))
    section = bounded.sections[0]
    assert section.bound is not None, "a bounded section must say so"
    assert section.bound.shown == 1, section.bound
    assert section.bound.total > 1, section.bound

    complete = index.report(fdu.Query(views=(fdu.View.FILES,)))
    assert complete.sections[0].bound is None, "an unbounded section reports no bound"

    # Naming no view derives one from the analyzers, exactly as the command line does.
    # Python defaulted to `tree` regardless, so a caller who asked to read every file got
    # a directory tree containing none of the results -- the defect the content axis
    # removed from the CLI, still live here because nothing tested the two together.
    for analyze, expected in (
        (fdu.Analysis.NONE, fdu.View.TREE),
        (fdu.Analysis.LINES, fdu.View.FAMILIES),
        (fdu.Analysis.CODE, fdu.View.LANGUAGES),
        (fdu.Analysis.WORDS, fdu.View.DOCUMENTS),
        (fdu.Analysis.ALL, fdu.View.FAMILIES),
    ):
        derived = fdu.scan(str(root), analysis=fdu.AnalysisOptions(analyze=analyze))
        section = derived.report(fdu.Query()).sections[0]
        assert section.view is expected, (analyze, section.view, expected)

    # `full` is a total the enum offers, so the binding must honour it: it once listed
    # `full` as valid in its own error message while rejecting it.
    full = index.report(fdu.Query(views=(fdu.View.FULL,)))
    produced = {section.view for section in full.sections}
    assert fdu.View.LARGEST in produced and fdu.View.RECENT in produced, produced
    assert fdu.View.FILES not in produced, "an unbounded enumeration is not a summary"


def main() -> None:
    root = Path(tempfile.mkdtemp(prefix="fdu-public-api-"))
    (root / "src").mkdir()
    (root / "src" / "main.rs").write_text("fn main() {}", encoding="utf-8")
    (root / "notes.md").write_text("release notes", encoding="utf-8")

    check_every_view(root)

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
    assert wire["schema"] == "fdu.report/4"
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
    assert contract["analysis"] == [value.value for value in fdu.Analysis]
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

    # A naive datetime means local time; the facade must resolve it to the explicit
    # offset the engine's time grammar requires rather than letting it be rejected.
    recent = index.report(
        fdu.Query(
            views=(fdu.View.FILES,),
            selection=fdu.Selection(modified_since=datetime.now() - timedelta(hours=1)),
        )
    )
    recent_files = recent.sections[0]
    assert isinstance(recent_files, fdu.FilesSection)
    assert {row.path.name for row in recent_files.files} >= {"new.txt"}, recent_files

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
