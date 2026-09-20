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
from dataclasses import asdict
from datetime import UTC, datetime, timedelta
from pathlib import Path

import fdu
from fdu import _native


def _stable(text: str) -> str:
    """Blank the fields that differ between any two runs, and only those."""

    text = re.sub(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{9}Z", "[TIME]", text)
    return re.sub(r'(observed_at_ns"?:\s*)\d+', r"\1[TIME]", text)


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
        assert live.provenance.source is not None

    # And a change record renders as the CLI streams it, rather than as repr().
    record = fdu.Change(clock=1, path=Path("a.txt"), kind=fdu.ChangeKind.UPSERT)
    line = record.render(fdu.Format.JSONL)
    assert '"schema": "fdu.stream/2"' in line, line
    assert '"op": "upsert"' in line, line
    assert "\t" in record.render(fdu.Format.TEXT)


def check_the_one_shot_retains_nothing(root: Path) -> None:
    """`fdu.report` runs the contract the command line runs, not a session.

    `open` retains an index and writes a snapshot, which is right for a caller asking many
    questions and wrong for one asking a single question -- an unfiltered summary that reads
    no `.gitignore` is answered by a transient tier that retains nothing, so a session cached
    state the walk never saved and a later cache-only read could see it (fdu-4msv).
    """

    blind = fdu.ScanOptions(read_controls=False)
    report = fdu.report(root, fdu.Query(views=(fdu.View.SUMMARY,)), scan=blind)
    assert report.provenance.source is not None

    # Rendering twice must not cost a second walk: the handle owns the finished report.
    text = report.render(fdu.Format.TEXT)
    assert text == report.render(fdu.Format.TEXT)
    assert report.render(fdu.Format.JSON) != text

    # An unusable cache is the operation failing, not the caller asking wrongly. Calling it
    # an argument error sent a caller looking in the wrong place, and made the CLI shim
    # exit 2 as a usage error where the command line exits 1.
    try:
        fdu.report(
            root, fdu.Query(views=(fdu.View.SUMMARY,)), cache=fdu.CachePolicy.ONLY, scan=blind
        )
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

    # Both surfaces read `.gitignore` by default, and a report's `ignore_rules` field and
    # every row's `ignored` share say so, so the default index is the one to compare.
    index = fdu.scan(str(root))
    for view in (fdu.View.TREE, fdu.View.LARGEST, fdu.View.SUMMARY):
        report = index.report(fdu.Query(views=(view,)))
        for fmt in fdu.Format:
            rendered = report.render(fmt)
            # Rust writes UTF-8 when stdout is a pipe. Windows' locale codec can decode
            # those bytes into different code points that round-trip to the same log
            # bytes, making identical-looking output compare unequal.
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
                encoding="utf-8",
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
        answer = derived.report(fdu.Query())
        section = answer.sections[0]
        assert section.view is expected, (analyze, section.view, expected)
        wire = json.loads(answer.render(fdu.Format.JSON))
        assert answer.as_dict() == wire
        if isinstance(section, fdu.MetricsSection):
            native_rows = wire["reports"][0]["metrics"]
            for row, raw in zip(
                (section.total, *section.rows),
                (native_rows["total"], *native_rows["rows"]),
                strict=True,
            ):
                assert {
                    key: value for key, value in asdict(row.metrics).items() if value is not None
                } == raw["metrics"]
                for unit in ("lines", "code", "words"):
                    coverage = getattr(row, f"{unit}_coverage")
                    assert (None if coverage is None else dict(coverage)) == raw["coverage"].get(
                        unit
                    )
                assert (None if row.pages is None else asdict(row.pages)) == raw.get("pages")

    # `full` is a total the enum offers, so the binding must honour it: it once listed
    # `full` as valid in its own error message while rejecting it.
    full = index.report(fdu.Query(views=(fdu.View.FULL,)))
    produced = {section.view for section in full.sections}
    assert fdu.View.LARGEST in produced and fdu.View.RECENT in produced, produced
    assert fdu.View.FILES not in produced, "an unbounded enumeration is not a summary"


def check_an_index_can_opt_out_of_control_state() -> None:
    """A default open, scan, or report reads control files; one that opts out reads none.

    A request that turns ``read_controls`` off reads no control file, and its snapshot is of
    a separate scope. A watch continues its index's scope, so it inherits the same choice.
    """

    root = Path(tempfile.mkdtemp(prefix="fdu-public-controls-"))
    (root / "kept.txt").write_text("kept", encoding="utf-8")
    # One pattern longer than the engine's 16 KiB default line limit, which an index that
    # observes control state refuses.
    (root / ".gitignore").write_text("x" * (16 * 1024 + 1) + "\n", encoding="utf-8")
    opted_out = fdu.ScanOptions(read_controls=False)

    for index in (
        fdu.open(root, cache=fdu.CachePolicy.OFF, scan=opted_out),
        fdu.scan(root, scan=opted_out),
    ):
        assert index.status.complete is True, index.status.errors
        assert index.total().files == 2
        assert index.status.ignore_rules is None, "a request that read no rule says so"
        assert index.report().status.ignore_rules is None
    # A default scan reads the control file and refuses the line over the limit, without
    # ending the scan or making its sizes partial (fdu-1onj).
    observed = fdu.scan(root)
    assert observed.status.complete is True, observed.status.errors
    assert observed.total().files == 2
    defaults = fdu.ControlLimits(budget=4 * 1024 * 1024, line_limit=16 * 1024)
    refused = fdu.RefusedControl(Path(".gitignore"), fdu.ControlRefusalReason.LINE_LIMIT)
    expected = fdu.ControlObservation(limits=defaults, applied=0, refused=1, refusals=(refused,))
    assert observed.status.ignore_rules == expected, observed.status
    observed_report = observed.report(fdu.Query(views=(fdu.View.SUMMARY,)))
    assert observed_report.status.complete is True
    assert observed_report.status.ignore_rules == expected, observed_report.status
    wire = json.loads(observed_report.render(fdu.Format.JSON))
    assert wire["ignore_rules"] == {
        "limits": {"budget": 4 * 1024 * 1024, "line_limit": 16 * 1024},
        "applied": 0,
        "refused": 1,
        "refusals": [{"path": ".gitignore", "reason": "line_limit"}],
    }, wire
    # The note names the directory and the limit that fired, as this surface spells it.
    (note,) = observed_report.notes
    assert "under . are not exact" in note, note
    assert "raise control_line_limit above 16 KiB, or set it to all" in note, note
    assert "control_budget" not in note, note
    assert note in observed_report.render(fdu.Format.TEXT), note
    # Lifting the budget leaves the line limit refusing; lifting the line limit applies it.
    budget_lifted = fdu.scan(root, scan=fdu.ScanOptions(control_budget=fdu.Bound.ALL))
    assert budget_lifted.status.ignore_rules == fdu.ControlObservation(
        limits=fdu.ControlLimits(budget=None, line_limit=16 * 1024),
        applied=0,
        refused=1,
        refusals=(refused,),
    )
    lifted = fdu.scan(root, scan=fdu.ScanOptions(control_line_limit="all"))
    assert lifted.status.ignore_rules == fdu.ControlObservation(
        limits=fdu.ControlLimits(budget=4 * 1024 * 1024, line_limit=None), applied=1, refused=0
    )
    assert lifted.report().notes == ()

    # A default report and a default open share one snapshot scope. An opted-out open
    # projects that snapshot's equal entry tier into a blind index on every cache route.
    (root / ".gitignore").write_text("*.log\n", encoding="utf-8")
    assert fdu.cache_path(root) is not None
    try:
        fdu.report(root, fdu.Query(views=(fdu.View.TREE,)))
        assert fdu.open(root).report().provenance.source is fdu.ReportSource.WARM_REVALIDATE
        cached = fdu.open(root, cache=fdu.CachePolicy.ONLY)
        assert cached.report().provenance.source is fdu.ReportSource.CACHE_ONLY
        projected = fdu.open(root, scan=opted_out)
        assert projected.report().provenance.source is fdu.ReportSource.WARM_REVALIDATE
        assert projected.status.ignore_rules is None
        projected_only = fdu.open(root, cache=fdu.CachePolicy.ONLY, scan=opted_out)
        assert projected_only.report().provenance.source is fdu.ReportSource.CACHE_ONLY
        assert projected_only.status.ignore_rules is None
    finally:
        fdu.clear_cache(root)


def check_reports_carry_the_ignored_share() -> None:
    """Every row a report draws carries its ignored share, and a selection picks one side.

    ``None`` means no ``.gitignore`` was read, never that nothing is ignored, and selecting
    by ignored state without the rules is refused rather than answered.
    """

    root = Path(tempfile.mkdtemp(prefix="fdu-public-ignored-"))
    (root / ".gitignore").write_bytes(b"dist/\n")
    (root / "dist").mkdir()
    (root / "dist" / "bundle.js").write_bytes(b"x" * 100)
    (root / "src").mkdir()
    (root / "src" / "main.rs").write_bytes(b"fn main() {}")
    views = (fdu.View.SUMMARY, fdu.View.TREE, fdu.View.EXTENSIONS, fdu.View.FILES)
    apparent = fdu.SizeMetric.APPARENT

    report = fdu.report(
        root,
        fdu.Query(views=views, selection=fdu.Selection(size=apparent)),
        cache=fdu.CachePolicy.OFF,
    )
    summary, tree, extensions, files = report.sections
    assert isinstance(summary, fdu.SummarySection), summary
    share = summary.summary.ignored
    assert share is not None and (share.files, share.dirs, share.bytes) == (1, 1, 100), share
    assert isinstance(tree, fdu.TreeSection), tree
    by_name = {child.name: child.ignored for child in tree.tree.children}
    assert by_name["dist"] is not None and by_name["dist"].bytes == 100, by_name
    assert by_name["src"] == fdu.IgnoredTally(0, 0, 0, 0), by_name
    assert isinstance(extensions, fdu.ExtensionsSection), extensions
    js = next(row for row in extensions.extensions if row.extension == ".js")
    assert js.ignored is not None and js.ignored.bytes == 100, js
    assert isinstance(files, fdu.FilesSection), files
    flags = {row.path.as_posix(): row.ignored for row in files.files}
    assert (flags["dist"], flags["dist/bundle.js"], flags["src/main.rs"]) == (True, True, False)
    assert "(100 B ignored)" in report.render(fdu.Format.TEXT)

    kept_query = fdu.Query(
        views=(fdu.View.SUMMARY,),
        selection=fdu.Selection(size=apparent, ignored=fdu.IgnoredEntries.EXCLUDE),
    )
    (kept,) = fdu.report(root, kept_query, cache=fdu.CachePolicy.OFF).sections
    assert isinstance(kept, fdu.SummarySection), kept
    assert (kept.summary.files, kept.summary.bytes) == (2, 18), kept
    assert kept.summary.ignored == fdu.IgnoredTally(0, 0, 0, 0), kept
    (from_index,) = fdu.scan(root).report(kept_query).sections
    assert from_index == kept, (from_index, kept)

    blind = fdu.ScanOptions(read_controls=False)
    (unread,) = fdu.report(
        root, fdu.Query(views=(fdu.View.SUMMARY,)), cache=fdu.CachePolicy.OFF, scan=blind
    ).sections
    assert isinstance(unread, fdu.SummarySection) and unread.summary.ignored is None, unread
    only_query = fdu.Query(selection=fdu.Selection(ignored=fdu.IgnoredEntries.ONLY))
    for attempt in (
        lambda: fdu.report(root, only_query, cache=fdu.CachePolicy.OFF, scan=blind),
        lambda: fdu.scan(root, scan=blind).report(only_query),
    ):
        try:
            attempt()
        except fdu.InvalidArgumentError as error:
            expected = (
                "ignored=only needs .gitignore classification, and read_controls turned it off"
            )
            assert expected in str(error), error
        else:
            raise AssertionError("selecting by ignored state without the rules must be refused")


def check_a_one_shot_report_forwards_every_control_knob() -> None:
    """``fdu.report`` carries each ``ScanOptions`` control knob into the scan it runs.

    A one-shot report is the surface that builds its own scan, so a knob it forgets does
    nothing quietly: the report still answers, under limits its caller did not ask for.
    Each knob is pinned by an answer only that knob produces.
    """

    root = Path(tempfile.mkdtemp(prefix="fdu-public-report-knobs-"))
    # One ordinary rule, and one pattern long enough to cross a lowered line limit.
    (root / ".gitignore").write_text("*.log\n" + "a" * 2048 + "\n", encoding="utf-8")
    (root / "keep.txt").write_text("keep", encoding="utf-8")
    query = fdu.Query(views=(fdu.View.SUMMARY,))

    def rules(scan: fdu.ScanOptions | None) -> fdu.ControlObservation | None:
        options = scan if scan is not None else fdu.ScanOptions()
        return fdu.report(root, query, cache=fdu.CachePolicy.OFF, scan=options).status.ignore_rules

    defaults = fdu.ControlLimits(budget=4 * 1024 * 1024, line_limit=16 * 1024)
    assert rules(None) == fdu.ControlObservation(limits=defaults, applied=1, refused=0)

    refused = fdu.RefusedControl(Path(".gitignore"), fdu.ControlRefusalReason.LINE_LIMIT)
    assert rules(fdu.ScanOptions(control_line_limit="1KiB")) == fdu.ControlObservation(
        limits=fdu.ControlLimits(budget=4 * 1024 * 1024, line_limit=1024),
        applied=0,
        refused=1,
        refusals=(refused,),
    )

    over_budget = fdu.RefusedControl(Path(".gitignore"), fdu.ControlRefusalReason.BUDGET)
    assert rules(fdu.ScanOptions(control_budget="1KiB")) == fdu.ControlObservation(
        limits=fdu.ControlLimits(budget=1024, line_limit=16 * 1024),
        applied=0,
        refused=1,
        refusals=(over_budget,),
    )

    assert rules(fdu.ScanOptions(read_controls=False)) is None


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
    check_an_index_can_opt_out_of_control_state()
    check_reports_carry_the_ignored_share()
    check_a_one_shot_report_forwards_every_control_knob()
    check_watch_reports_its_own_index(root)
    check_render_matches_the_cli(
        root, str(Path(sys.executable).with_name("fdu.exe" if os.name == "nt" else "fdu"))
    )

    index = fdu.scan(root, scan=fdu.ScanOptions(max_depth=3))
    assert os.path.samefile(index.root, root)
    assert index.status.complete is True
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
    assert report.provenance.freshness is fdu.Freshness.FRESH
    assert [section.view for section in report.sections] == [
        fdu.View.SUMMARY,
        fdu.View.EXTENSIONS,
        fdu.View.FILES,
    ]
    wire = report.as_dict()
    assert wire["schema"] == "fdu.report/7"
    assert wire["generator"] == f"fdu {fdu.__version__}"
    assert json.loads(json.dumps(wire)) == wire

    package_dir = Path(fdu.__file__).parent
    public_names = {name for name in dir(fdu) if not name.startswith("_") or name == "__version__"}
    assert public_names == set(fdu.__all__), (public_names, set(fdu.__all__))
    assert (package_dir / "py.typed").is_file()
    stub_path = package_dir / "_native.pyi"
    assert stub_path.is_file()
    stub_tree = ast.parse(stub_path.read_text(encoding="utf-8"))
    # Classes, functions, and the module-level constants the stub declares: the native
    # module publishes the request model's defaults, so a stub that listed only callables
    # would go stale the moment one of them moved.
    stub_exports = {
        node.name for node in stub_tree.body if isinstance(node, (ast.ClassDef, ast.FunctionDef))
    } | {
        node.target.id
        for node in stub_tree.body
        if isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name)
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
    assert contract["cache_states"] == [value.value for value in fdu.CacheState]
    assert contract["content_states"] == [value.value for value in fdu.ContentState]
    assert contract["stale_reasons"] == [value.value for value in fdu.StaleReason]
    assert contract["leftover_kinds"] == [value.value for value in fdu.LeftoverKind]
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
    assert cached.report().provenance.freshness is fdu.Freshness.STALE
    status = fdu.cache_status(cache_root)
    assert status is not None and status.state is fdu.CacheState.CURRENT
    assert status.stale_reason is None and status.root is not None
    # Cache status carries the identity of every tier the store holds: a default open
    # observes .gitignore under the default limits, and wrote no content sidecar.
    assert status.identity is not None, status
    assert status.identity.ignore_rules is not None, status
    assert status.identity.entries.max_depth is None, status
    assert status.content is None, status
    # A snapshot an earlier format wrote is still fdu's: reported stale with its version,
    # and cleared, rather than stranded as a file nothing will delete. The version sits
    # after the eight-byte magic in every format.
    image = bytearray(status.path.read_bytes())
    written = int.from_bytes(image[8:12], "little")
    image[8:12] = (written - 1).to_bytes(4, "little")
    status.path.write_bytes(bytes(image))
    stale = fdu.cache_status(cache_root)
    assert stale is not None and stale.state is fdu.CacheState.STALE, stale
    assert stale.stale_reason is fdu.StaleReason.OLDER_FORMAT, stale
    assert stale.format_version == written - 1 and stale.root is None, stale
    assert stale.identity is None, stale
    assert fdu.render_cache_status([stale], scope=fdu.CacheScope.ROOT).endswith(
        "cannot be served by this build; fdu --cache-clear PATH removes it."
    )
    assert fdu.clear_cache(cache_root) is True
    absent = fdu.cache_status(cache_root)
    assert absent is not None and absent.state is fdu.CacheState.ABSENT, absent
    # A sidecar with no snapshot is fdu's own leftover, not a foreign file. Only the
    # classification is asserted here: this test shares the developer's real cache
    # directory, so it plants one file of its own and removes it, and never clears.
    orphan = status.path.with_name(status.path.name + ".content")
    orphan.write_bytes(b"FDUCTNT\0planted")
    try:
        listed = {cache.path: cache for cache in fdu.list_caches(cache_root)}
        assert listed[orphan].state is fdu.CacheState.LEFTOVER, listed
        assert listed[orphan].leftover_kind is fdu.LeftoverKind.ORPHANED_CONTENT, listed
    finally:
        orphan.unlink()

    # An analyzed open leaves a content sidecar, and status reports it as the snapshot's
    # own `content`: its records and the identity that decides which requests they serve.
    # The sidecar's entry tier is the snapshot's, because a record is only as valid as the
    # entry it was analyzed over.
    analyzed_root = Path(tempfile.mkdtemp(prefix="fdu-public-analyzed-"))
    (analyzed_root / "notes.md").write_text("one two\nthree\n", encoding="utf-8")
    fdu.open(
        analyzed_root,
        cache=fdu.CachePolicy.AUTO,
        analysis=fdu.AnalysisOptions(analyze=fdu.Analysis.LINES),
    )
    analyzed = fdu.cache_status(analyzed_root)
    assert analyzed is not None and analyzed.state is fdu.CacheState.CURRENT, analyzed
    content = analyzed.content
    assert content is not None, analyzed
    assert content.state is fdu.ContentState.CURRENT, content
    assert content.stale_reason is None and content.format_version is None, content
    assert content.records == 1, content
    assert content.identity is not None, content
    assert content.identity.analyze == (fdu.Analysis.LINES,), content
    assert content.identity.analyzers and all(
        analyzer.version > 0 for analyzer in content.identity.analyzers
    ), content
    assert analyzed.identity is not None, analyzed
    assert content.identity.entries == analyzed.identity.entries, analyzed
    assert fdu.clear_cache(analyzed_root) is True

    entrypoint = Path(sys.executable).with_name("fdu.exe" if os.name == "nt" else "fdu")
    version = subprocess.run(
        [entrypoint, "--version"], check=False, capture_output=True, encoding="utf-8"
    )
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
            "--scan-depth",
            "3",
            str(root),
        ],
        check=False,
        capture_output=True,
        encoding="utf-8",
    )
    assert cli_report.returncode == 0, cli_report
    cli_wire = json.loads(cli_report.stdout)
    assert wire["provenance"]["source"] in {
        "cold_scan",
        "warm_revalidate",
        "cache_only",
    }
    assert cli_wire["provenance"]["source"] == "cold_scan"
    for value in (wire, cli_wire):
        provenance = value["provenance"]
        for volatile in ("scan_started_at", "generated_at", "source"):
            provenance.pop(volatile)
        for tier in provenance["tiers"].values():
            if tier is not None:
                tier.pop("observed_at_ns")
    # Both surfaces read `.gitignore` control state by default, under the same limits, so
    # the envelopes agree on it as they agree on every row's ignored share.
    assert wire["ignore_rules"] == {
        "limits": {"budget": 4 * 1024 * 1024, "line_limit": 16 * 1024},
        "applied": 0,
        "refused": 0,
        "refusals": [],
    }, wire
    assert wire == cli_wire, (wire, cli_wire)

    print(f"fdu {fdu.__version__} public API ok")


if __name__ == "__main__":
    main()
