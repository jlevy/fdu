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
import re
import subprocess
import sys
import tempfile
from datetime import UTC, datetime, timedelta
from pathlib import Path

import fdu
from fdu import _native


def _stable(text: str) -> str:
    """Blank the fields that differ between any two runs, and only those."""

    return re.sub(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{9}Z", "[TIME]", text)


def check_watch_reports_its_own_index(root: Path) -> None:
    """A repaint must come from the session's index, not the one it was opened from.

    The aggregates stop being true at the first event, so reporting the opened index
    repaints numbers that never change while claiming to be live -- a display that looks
    like it works and does not (fdu-m66a).
    """

    index = fdu.open(root)
    with index.watch(fdu.WatchOptions(interval=0.2)) as watch:
        live = watch.report()
        # A snapshot: rendering twice gives the same answer both times.
        assert live.render(fdu.Format.TEXT) == live.render(fdu.Format.TEXT)
        assert live.status.source is not None

    # And a change record renders as the CLI streams it, rather than as repr().
    record = fdu.Change(clock=1, path=Path("a.txt"), kind=fdu.ChangeKind.UPSERT)
    line = record.render(fdu.Format.JSONL)
    assert '"schema": "fdu.stream/1"' in line, line
    assert '"op": "upsert"' in line, line
    assert "\t" in record.render(fdu.Format.TEXT)


def check_the_one_shot_retains_nothing(root: Path) -> None:
    """`fdu.report` runs the contract the command line runs, not a session.

    `open` retains an index and writes a snapshot, which is right for a caller asking many
    questions and wrong for one asking a single question -- an unfiltered summary is
    answered by a transient tier that retains nothing, so a session cached state the walk
    never saved and a later cache-only read could see it (fdu-4msv).
    """

    report = fdu.report(root, fdu.Query(views=(fdu.View.SUMMARY,)))
    assert report.status.source is not None

    # Rendering twice must not cost a second walk: the handle owns the finished report.
    text = report.render(fdu.Format.TEXT)
    assert text == report.render(fdu.Format.TEXT)
    assert report.render(fdu.Format.JSON) != text

    # An unusable cache is the operation failing, not the caller asking wrongly. Calling it
    # an argument error sent a caller looking in the wrong place, and made the CLI shim
    # exit 2 as a usage error where the command line exits 1.
    try:
        fdu.report(root, fdu.Query(views=(fdu.View.SUMMARY,)), cache=fdu.CachePolicy.ONLY)
    except fdu.InvalidArgumentError as error:  # pragma: no cover - the regression
        raise AssertionError(f"an unusable snapshot is not an argument error: {error}") from None
    except fdu.FduError:
        pass


def check_the_list_grammar_reaches_python(root: Path) -> None:
    """A view spec is parsed by the one grammar, not by whichever surface got it first.

    Duplicate and empty-entry rejection lived in the CLI, so `views="tree,tree"` was a
    typo there and a silent no-op here -- one request meaning two things depending on
    which door it came through.
    """

    index = fdu.scan(root)
    for spec, expected in [
        ("tree,tree", "appears more than once"),
        ("tree,,types", "empty entry in the list"),
        ("full,tree", "cannot be combined"),
        ("bogus", "expected one of"),
    ]:
        try:
            index.report(fdu.Query(views=spec))
        except fdu.InvalidArgumentError as error:
            assert expected in str(error), (spec, str(error))
        else:
            raise AssertionError(f"{spec!r} must be rejected")

    # And a spec the grammar accepts still works, including `full` expansion.
    assert len(index.report(fdu.Query(views="full")).sections) > 1
    assert len(index.report(fdu.Query(views="tree,types")).sections) == 2


def check_render_matches_the_cli(root: Path, binary: str) -> None:
    """The package renders what the command line prints.

    Until this existed a Python caller wanting fdu's own output had to shell out to the
    binary -- the same admission the console script makes, since `fdu:_main` calls
    `_native.main()` and the `fdu` the wheel installs has never exercised a line of the
    Python API.

    The comparison is against the real CLI rather than a recorded string, because a
    recording drifts and the point is that the two agree today.
    """

    index = fdu.scan(str(root))
    for view in (fdu.View.TREE, fdu.View.LARGEST, fdu.View.SUMMARY):
        report = index.report(fdu.Query(views=(view,)))
        for fmt in fdu.Format:
            rendered = report.render(fmt)
            cli = subprocess.run(
                [
                    binary,
                    "--cache",
                    "off",
                    "--color",
                    "never",
                    "--format",
                    str(fmt),
                    "--view",
                    str(view),
                    str(root),
                ],
                capture_output=True,
                text=True,
                check=True,
            ).stdout
            # The CLI appends a performance footer; the schema excludes that telemetry and
            # a Report does not carry the counts behind it, so it is the one difference.
            body = "\n".join(
                line for line in cli.splitlines() if not line.startswith("Performance:")
            ).rstrip()
            # Two separate runs, so the walk timestamps differ. Normalised the same way
            # the golden corpus masks them: the values are unstable, the shape is not.
            assert _stable(rendered.rstrip()) == _stable(body), (
                view,
                fmt,
                rendered[:200],
                body[:200],
            )

    # A report built by hand has no index to render through, and says so.
    from dataclasses import replace as _replace

    detached = _replace(index.report(fdu.Query()), _renderer=None)
    try:
        detached.render()
        raise SystemExit("a report with no index behind it must refuse to render")
    except fdu.InvalidArgumentError as error:
        # Every producer that does bind one, so the message does not send a caller who
        # used fdu.report or Watch.report looking at the wrong call.
        for producer in ("Index.report", "fdu.report", "Watch.report"):
            assert producer in str(error), (producer, str(error))


def check_a_report_is_a_snapshot(root: Path) -> None:
    """One report must not answer differently each time it is asked.

    `Index.report` used to bind a renderer to the *query* and re-project the retained index
    per format, so `as_dict` held the values the call was answered with while `render`
    quietly returned newer ones once the index moved (fdu-4gno). All three producers now
    bind to the finished report.
    """

    index = fdu.open(root)
    report = index.report(fdu.Query(views=(fdu.View.SUMMARY,)))
    before = report.render(fdu.Format.JSON)

    (root / "snapshot-probe.bin").write_bytes(b"x" * 100_000)
    index.refresh()

    assert report.render(fdu.Format.JSON) == before, "render must not follow the live index"
    assert json.loads(before)["reports"][0] == report.as_dict()["reports"][0], (
        "as_dict and render must serialize one value, not two"
    )

    # And the index really did move, or this proves nothing.
    assert index.report(fdu.Query(views=(fdu.View.SUMMARY,))).render(fdu.Format.JSON) != before
    (root / "snapshot-probe.bin").unlink()
    index.refresh()


def check_a_report_states_its_own_omissions(root: Path) -> None:
    """A dropped view must be readable as a value, not only inside rendered text.

    `full` without analyzers cannot answer `documents`. The report says so, and a caller
    reading `sections` needs that as a note rather than having to scrape the text rendering
    to find out why a section is absent (fdu-7wd1).
    """

    report = fdu.report(root, fdu.Query(views=(fdu.View.FULL,)))
    assert report.notes, "a dropped view must be stated on the report"
    assert any("documents" in note for note in report.notes), report.notes

    # Named in this surface's vocabulary: there is no --analyze in Python (fdu-4apt).
    for note in report.notes:
        assert "--analyze" not in note, note
        assert "--view" not in note, note
    assert any("add analyze " in note for note in report.notes), report.notes

    # The same rule as a hard error, in the same vocabulary.
    try:
        fdu.report(root, fdu.Query(views=(fdu.View.DOCUMENTS,)))
        raise SystemExit("documents without analyzers must be rejected")
    except fdu.InvalidArgumentError as error:
        assert str(error).startswith("view documents"), error
        assert "add analyze " in str(error), error
        assert "--analyze" not in str(error), error

    # Nothing dropped, nothing said.
    assert not fdu.report(root, fdu.Query(views=(fdu.View.SUMMARY,))).notes


def check_every_failure_is_an_fdu_error(root: Path) -> None:
    """`except FduError` must be enough to catch what this package raises.

    `Change.render` reached the extension directly and so raised pyo3's bare `ValueError`,
    which that clause does not catch (fdu-dygl).
    """

    change = fdu.Change(path=root / "x", kind=fdu.ChangeKind.UPSERT, clock=1)
    assert json.loads(change.render(fdu.Format.JSONL))["op"] == "upsert"
    try:
        change.render("xml")  # pyright: ignore[reportArgumentType]
        raise SystemExit("an unknown format must be rejected")
    except fdu.FduError as error:
        assert isinstance(error, fdu.InvalidArgumentError), type(error)


def check_the_watch_rule_names_an_instant(root: Path) -> None:
    """A repaint separator must render the instant it was given, exactly.

    A naive datetime names no instant; reading it as local time moved the rule by the
    machine's UTC offset while still printing `Z`. And nanoseconds cannot survive a float,
    so `Change.mtime_ns` -- which is what a caller repainting after a batch actually holds
    -- goes in as an int (fdu-uwv0).
    """

    del root
    aware = datetime(2026, 8, 10, 18, 22, 31, tzinfo=UTC)
    assert fdu.watch_rule(aware) == "──── 2026-08-10T18:22:31.000000000Z ────"

    # An int carries the full nanosecond; a float cannot, and quantized this to ...457024.
    exact_ns = 1_786_386_151_123_456_789
    assert fdu.watch_rule(exact_ns) == "──── 2026-08-10T18:22:31.123456789Z ────"

    try:
        fdu.watch_rule(datetime(2026, 8, 10, 18, 22, 31))
        raise SystemExit("a naive datetime must be refused, not read as local time")
    except fdu.InvalidArgumentError as error:
        assert "aware" in str(error), error


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
    check_a_report_is_a_snapshot(root)
    check_a_report_states_its_own_omissions(root)
    check_every_failure_is_an_fdu_error(root)
    check_the_watch_rule_names_an_instant(root)
    check_the_list_grammar_reaches_python(root)
    check_the_one_shot_retains_nothing(root)
    check_watch_reports_its_own_index(root)
    check_render_matches_the_cli(
        root, str(Path(sys.executable).with_name("fdu.exe" if os.name == "nt" else "fdu"))
    )

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
    assert contract["cache_scopes"] == [value.value for value in fdu.CacheScope]
    assert contract["formats"] == [value.value for value in fdu.Format]

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

    # A scoped refresh sees a change inside its subtree and reaches the same state a
    # whole-tree refresh would. This is the hint-ingestion primitive: a caller running
    # its own watcher pushes each hint through here rather than through a second path
    # into the index, and it pays for the subtree rather than the tree.
    (root / "src" / "scoped.txt").write_text("scoped", encoding="utf-8")
    scoped = index.refresh("src")
    assert scoped.inserted == 1, "a scoped refresh must observe a change inside its scope"
    assert index.rollup("src") is not None
    scoped_total = index.total().files
    # The same tree refreshed whole must agree, so scoping changed the cost and not the
    # answer.
    assert index.refresh().inserted == 0, "the scoped refresh already applied the change"
    assert index.total().files == scoped_total

    # A change outside the scope is not observed by a refresh scoped elsewhere.
    (root / "outside.txt").write_text("outside", encoding="utf-8")
    assert index.refresh("src").inserted == 0, "a scoped refresh must not reach outside it"
    assert index.refresh().inserted == 1, "the whole-tree refresh still finds it"

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
