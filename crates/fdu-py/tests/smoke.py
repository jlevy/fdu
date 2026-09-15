"""Internal-boundary smoke test for the private fdu native extension.

Runs against an installed wheel, so it checks what a user actually gets rather than what
the build tree contains. Deliberately dependency-free: it is executed by CI in a bare
virtualenv holding nothing but the wheel.

Run manually with:

    uv venv --clear .venv-smoke
    uv pip install --python .venv-smoke --no-index --find-links dist fdu
    uv run --no-project --python .venv-smoke python tests/smoke.py
    uvx --isolated --no-index --find-links dist --from fdu fdu --version
"""

from __future__ import annotations

import errno
import json
import os
import pathlib
import re
import subprocess
import sys
import tempfile
import threading
from concurrent.futures import ThreadPoolExecutor

from fdu import Bound, EntryKind, FilesSection, Format, InvalidArgumentError, Query, Selection, View
from fdu import _native as fdu_py
from fdu.opened import (
    Aggregate,
    ChangeCursorUnavailableError,
    ChangeOutcomeKind,
    Continue,
    CoverageKind,
    Diagnostics,
    DirectoryRollUp,
    EntrySelection,
    Flat,
    KnowledgeKind,
    Lookup,
    OpenedIndex,
    OpenedIndexClosedError,
    OpenedOptions,
    Page,
    ReadResponse,
    RefusalReason,
    ReportProjection,
    Tree,
    VersionUnavailableError,
)


def main() -> None:
    root = pathlib.Path(tempfile.mkdtemp())
    (root / "a.txt").write_text("hello")
    (root / "src").mkdir()
    (root / "src" / "main.rs").write_text("fn main() {}")

    index = fdu_py.scan(root)

    # Rust preserves Windows canonical verbatim paths (`\\?\`), while Python's
    # realpath commonly returns the conventional spelling. Compare the filesystem
    # identity rather than weakening native long-path behavior to satisfy a string.
    assert os.path.samefile(index.root, root), (index.root, root)
    assert index.complete is True, index.errors
    assert index.freshness == "fresh", index.freshness
    assert index.errors == [], index.errors
    assert len(index) == 4, f"root + 2 files + 1 dir, got {len(index)}"

    missing = root / "missing"
    try:
        fdu_py.scan(missing)
    except OSError as error:
        assert error.errno is not None, error
        assert os.path.samefile(pathlib.Path(error.filename).parent, root), error
        assert pathlib.Path(error.filename).name == missing.name, error
    else:
        raise AssertionError("a missing scan root must raise OSError with its path")

    total = index.total()
    assert total["files"] == 2, total
    assert total["dirs"] == 1, total
    assert total["bytes"] == 17, total
    # Allocated rides alongside apparent bytes per type, so a report asked for allocated
    # sizes keeps its per-type breakdown instead of switching metrics.
    txt = total["by_extension"][".txt"]
    rs = total["by_extension"][".rs"]
    assert (txt["files"], txt["bytes"]) == (1, 5), total
    assert (rs["files"], rs["bytes"]) == (1, 12), total
    assert txt["allocated"] >= txt["bytes"], total
    assert rs["allocated"] >= rs["bytes"], total

    # Per-directory roll-ups, which is the thing no surveyed tool provides.
    src = index.rollup("src")
    assert src is not None and src["files"] == 1 and src["bytes"] == 12, src
    assert index.rollup("does-not-exist") is None
    assert index.rollup("a.txt") is None, "a file has no roll-up"

    # Bulk child listing: one call returns every child with its roll-up.
    children = index.children("")
    assert children is not None
    by_name = {c["name"]: c for c in children}
    assert set(by_name) == {"a.txt", "src"}, by_name
    assert by_name["src"]["kind"] == "dir"
    assert by_name["a.txt"]["kind"] == "file"
    assert by_name["a.txt"]["bytes"] == 5

    # The installed wheel is also the zero-install CLI artifact used by uvx.
    entrypoint = pathlib.Path(sys.executable).with_name("fdu.exe" if os.name == "nt" else "fdu")
    assert entrypoint.is_file(), entrypoint

    version = subprocess.run([entrypoint, "--version"], check=False, capture_output=True, text=True)
    assert version.returncode == 0, version
    # A wheel built from a checkout carries the git revision as semver build metadata;
    # one built without git metadata reports the bare semver. Either way the semver
    # itself must match the module's exactly.
    version_pattern = rf"fdu {re.escape(fdu_py.__version__)}(-dev\+g[0-9a-f]{{7,12}}(\.dirty)?)?\n"
    assert re.fullmatch(version_pattern, version.stdout), version.stdout
    assert version.stderr == "", version.stderr

    help_result = subprocess.run(
        [entrypoint, "--help"], check=False, capture_output=True, text=True
    )
    assert help_result.returncode == 0, help_result
    # Help is the flag reference. The prose it used to carry now lives behind --docs, so
    # help states where to find it rather than opening with a page of it.
    assert "--color <WHEN>" in help_result.stdout, help_result.stdout
    assert "--skill" in help_result.stdout, help_result.stdout
    assert "--docs" in help_result.stdout, help_result.stdout
    assert "Run `fdu --docs`" in help_result.stdout, help_result.stdout
    assert help_result.stderr == "", help_result.stderr

    # The guide answers without a PATH and without scanning, from the installed wheel's
    # own entry point.
    docs_result = subprocess.run(
        [entrypoint, "--docs"], check=False, capture_output=True, text=True
    )
    assert docs_result.returncode == 0, docs_result
    for section in ("THE LADDER", "SIX AXES", "CONTENT ANALYSIS", "OUTPUT AND AUTOMATION"):
        assert section in docs_result.stdout, (section, docs_result.stdout[:400])
    assert docs_result.stderr == "", docs_result.stderr

    cli_scan = subprocess.run(
        [
            entrypoint,
            "--cache",
            "off",
            "--format",
            "json",
            "--size",
            "apparent",
            "--depth",
            "1",
            str(root),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert cli_scan.returncode == 0, cli_scan
    cli_data = json.loads(cli_scan.stdout)
    assert cli_data["schema"] == "fdu.report/5", cli_data
    assert cli_data["complete"] is True, cli_data
    tree = cli_data["reports"][0]["tree"]
    assert tree["bytes"] == 17, cli_data
    # Truncation describes omitted tree rows. A file is already represented in its
    # directory's totals, so reaching the depth bound at a file-only leaf omits nothing.
    assert tree["truncated"] is False, cli_data
    assert tree["children"][0]["truncated"] is False, cli_data
    assert cli_scan.stderr == "", cli_scan.stderr

    usage = subprocess.run(
        [entrypoint, "--definitely-not-an-option"],
        check=False,
        capture_output=True,
        text=True,
    )
    assert usage.returncode == 2, usage
    assert usage.stdout == "", usage.stdout
    assert "unexpected argument" in usage.stderr, usage.stderr
    assert "Traceback" not in usage.stderr, usage.stderr

    if os.name != "nt":
        # Python stores undecodable argv bytes with surrogateescape. The wheel entry
        # point must recover the native bytes rather than narrowing them to UTF-8. Keep
        # this fixture outside `root`: Linux accepts the byte name, and adding it beneath
        # the already indexed API fixture would couple this check to the refresh test.
        raw_parent = tempfile.mkdtemp(prefix="fdu-native-argv-")
        raw_root = os.fsencode(raw_parent) + b"/raw-\xff"
        try:
            os.mkdir(raw_root)
        except OSError as error:
            if error.errno != errno.EILSEQ:
                raise
            # APFS rejects this fixture, but passing the same bytes to fdu still proves
            # that Python argv reached Rust losslessly instead of raising in PyO3.
            raw_scan = subprocess.run(
                [os.fsencode(entrypoint), b"--cache", b"off", b"--format", b"json", raw_root],
                check=False,
                capture_output=True,
            )
            assert raw_scan.returncode == 1, raw_scan
            assert raw_scan.stderr.startswith(b"fdu:"), raw_scan.stderr
            assert b"Traceback" not in raw_scan.stderr, raw_scan.stderr
        else:
            with open(raw_root + b"/data.bin", "wb") as raw_file:
                raw_file.write(b"raw")
            raw_name = b"source-\xfe.RS"
            with open(raw_root + b"/" + raw_name, "wb") as raw_file:
                raw_file.write(b"fn main() {}")

            # PathBuf follows Python's path-string protocol. Decode through the native
            # filesystem codec so undecodable bytes become surrogateescape code points
            # that PyO3 can round-trip to the original Unix OsString.
            raw_index = fdu_py.scan(os.fsdecode(raw_root))
            assert os.fsencode(raw_index.root) == os.path.realpath(raw_root), raw_index.root
            raw_children = raw_index.children()
            assert raw_children is not None
            assert raw_name in {os.fsencode(child["name"]) for child in raw_children}, raw_children
            assert raw_index.total()["by_extension"][".rs"]["files"] == 1

            raw_mark = raw_index.clock
            added_name = b"notes-\xfd.md"
            with open(raw_root + b"/" + added_name, "wb") as raw_file:
                raw_file.write(b"notes")
            raw_index.refresh()
            raw_ops = raw_index.since(raw_mark)["ops"]
            assert added_name in {os.fsencode(op["path"]) for op in raw_ops}, raw_ops

            raw_scan = subprocess.run(
                [os.fsencode(entrypoint), b"--cache", b"off", b"--format", b"json", raw_root],
                check=False,
                capture_output=True,
            )
            assert raw_scan.returncode == 0, raw_scan
            raw_data = json.loads(raw_scan.stdout)
            assert raw_data["root_raw"] == {
                "encoding": "unix-bytes",
                "hex": os.path.realpath(raw_root).hex(),
            }, raw_data
            assert raw_scan.stderr == b"", raw_scan.stderr

    # Revalidation reconciles against the filesystem and reports what moved.
    mark = index.clock
    (root / "added.md").write_text("new")
    os.remove(root / "a.txt")

    stats = index.refresh()
    assert stats["inserted"] == 1, stats
    assert stats["removed"] == 1, stats
    assert stats["unchanged"] == 2, stats
    assert stats["complete"] is True, stats
    assert stats["freshness"] == "fresh", stats
    assert stats["error_count"] == 0 and stats["errors"] == [], stats

    after = index.total()
    assert after["files"] == 2, after
    assert after["bytes"] == 15, after
    assert ".txt" not in after["by_extension"], after

    # since() reports the change feed, and flags a consumer that fell too far behind.
    changed = index.since(mark)
    assert changed["truncated"] is False, changed
    ops = {(op["op"], op["path"]) for op in changed["ops"]}
    assert ("remove", "a.txt") in ops, ops
    assert ("upsert", "added.md") in ops, ops

    # refresh() retains the semantic scan scope used to create the index.
    scoped_root = pathlib.Path(tempfile.mkdtemp())
    (scoped_root / "nested").mkdir()
    (scoped_root / "nested" / "before.txt").write_text("before")
    scoped = fdu_py.scan(str(scoped_root), max_depth=1)
    assert scoped.total()["files"] == 0, scoped.total()
    (scoped_root / "nested" / "after.txt").write_text("after")
    scoped.refresh()
    assert scoped.total()["files"] == 0, "refresh widened max_depth"

    # Kind labels remain lossless at the language boundary.
    kind_root = pathlib.Path(tempfile.mkdtemp())
    (kind_root / "directory").mkdir()
    (kind_root / "file").write_text("file")
    expected_kinds = {"directory": "dir", "file": "file"}
    if os.name != "nt":
        os.symlink("file", kind_root / "link")
        os.mkfifo(kind_root / "fifo")
        expected_kinds.update({"link": "symlink", "fifo": "other"})
    kinds = {child["name"]: child["kind"] for child in fdu_py.scan(str(kind_root)).children("")}
    assert kinds == expected_kinds, kinds

    # The query surface: the same five axes the CLI exposes, as one typed call.
    query_root = pathlib.Path(tempfile.mkdtemp())
    (query_root / "src").mkdir()
    (query_root / "src" / "main.rs").write_text("fn main() {}")
    (query_root / "src" / "lib.rs").write_text("pub fn lib() {}")
    (query_root / "notes.md").write_text("notes")
    index = fdu_py.scan(str(query_root))

    summary = index.report(views=["summary"])["reports"][0]["summary"]
    assert summary["files"] == 3, summary
    assert summary["dirs"] == 1, summary

    # Selection narrows without rescanning, and every view is reachable.
    rust_only = index.report(views=["files"], include=["*.rs"], kind=["file"])
    # Reported paths carry native separators, so compare in a separator-agnostic way
    # rather than narrowing what the engine reports to satisfy a string.
    paths = sorted(row["path"].replace(os.sep, "/") for row in rust_only["reports"][0]["files"])
    assert paths == ["src/lib.rs", "src/main.rs"], paths

    extension_rows = index.report(views=["extensions"])["reports"][0]["extensions"]
    extensions = sorted(row["extension"] for row in extension_rows)
    assert extensions == [".md", ".rs"], extensions

    types = index.report(views=["types"])["reports"][0]["metrics"]
    assert sorted(row["id"] for row in types["rows"]) == ["markdown", "rust"], types
    assert types["total"]["detection"] == {
        "sources": {"extension": 3},
        "confidence": {"certain": 3},
        "flags": {"generated": 0, "vendored": 0, "documentation": 0},
    }, types

    languages_report = index.report(views=["languages"])
    assert languages_report["analysis"] is None, languages_report
    languages = languages_report["reports"][0]["metrics"]
    assert languages["share_metric"] == "allocated_bytes", languages
    assert [(row["id"], row["files"]) for row in languages["rows"]] == [("rust", 2)], languages

    analyzed = fdu_py.scan(str(query_root), analyze="lines")
    documents = analyzed.report(views=["documents"], words_per_page=250)
    assert documents["analysis"]["analyze"] == ["lines"], documents
    document_metrics = documents["reports"][0]["metrics"]
    markdown = document_metrics["rows"][0]
    assert markdown["physical_lines"] == 1, markdown
    assert markdown["raw_words"] == 1, markdown
    assert markdown["words_per_page"] == 250, markdown

    tree = index.report(views=["tree"], depth="all")["reports"][0]["tree"]
    assert tree["name"] == ".", tree
    assert any(child["name"] == "src" for child in tree["children"]), tree

    # Several views come back in request order, from one index.
    ordered = index.report(views=["extensions", "types", "summary"])["reports"]
    assert [section["view"] for section in ordered] == [
        "extensions",
        "types",
        "summary",
    ], ordered

    # Value grammars are shared, so a bad value is rejected the same way everywhere.
    for bad in [
        {"min_size": "10X"},
        {"modified_since": "1.5h"},
        {"views": ["bogus"]},
        {"views": ["documents"]},
        {"sort": "sideways"},
    ]:
        try:
            index.report(**bad)
        except ValueError:
            pass
        else:
            raise AssertionError(f"expected {bad} to be rejected")

    # Cache accessors mirror the library functions.
    cache_root = pathlib.Path(tempfile.mkdtemp())
    (cache_root / "a.txt").write_text("hello")
    fdu_py.open(str(cache_root), cache="auto")
    status = fdu_py.cache_status(str(cache_root))
    assert status is not None and status["recognized"], status
    cached_index = fdu_py.open(str(cache_root), cache="only")
    # Coverage and currency are independent: the cache represents the full scope even
    # though this cache-only open deliberately did not revalidate it.
    assert cached_index.complete is True, cached_index.freshness
    assert cached_index.freshness == "stale", cached_index.freshness
    assert cached_index.report(views=["summary"])["complete"] is True
    # Rust keeps Windows verbatim paths (\\?\); compare filesystem identity rather than
    # weakening native long-path behavior to satisfy a string.
    assert os.path.samefile(status["root"], cache_root), status
    assert fdu_py.clear_cache(str(cache_root)) is True
    assert fdu_py.cache_status(str(cache_root))["recognized"] is False

    # Expected coverage exclusions remain queryable without becoming operational errors.
    partial_root = pathlib.Path(tempfile.mkdtemp())
    (partial_root / "invalid.txt").write_bytes(b"valid prefix\xff")
    partial = fdu_py.open(str(partial_root), cache="auto", analyze="lines")
    assert partial.complete is True, partial.errors
    assert partial.errors == [], partial.errors

    cached_partial = fdu_py.open(str(partial_root), cache="only", analyze="lines")
    assert cached_partial.complete is True, cached_partial.errors
    assert cached_partial.freshness == "stale", cached_partial.freshness
    assert cached_partial.errors == [], cached_partial.errors
    partial_report = cached_partial.report(views=["types"], size="apparent")
    assert partial_report["complete"] is True, partial_report
    assert partial_report["errors"] == [], partial_report
    coverage = partial_report["reports"][0]["metrics"]["total"]["coverage"]
    assert coverage == {"invalid_utf8": 1}, coverage
    refreshed = cached_partial.refresh()
    assert refreshed["complete"] is True, refreshed
    assert refreshed["errors"] == [], refreshed

    # Cache policy is the same closed vocabulary the CLI accepts.
    try:
        fdu_py.open(str(cache_root), cache="sometimes")
    except ValueError:
        pass
    else:
        raise AssertionError("expected an invalid cache policy to be rejected")

    # The watch feed: event-driven, and closable without hanging the interpreter.
    watch_root = pathlib.Path(tempfile.mkdtemp())
    (watch_root / "seed.txt").write_text("seed")
    watch_index = fdu_py.scan(str(watch_root))
    feed = watch_index.watch(interval=0.25, views=["files"])

    (watch_root / "created.rs").write_text("fn main() {}")
    seen = []
    for _ in range(40):
        seen.extend(next(feed))
        if any(change["path"].endswith("created.rs") for change in seen):
            break
    assert any(change["path"].endswith("created.rs") for change in seen), seen
    created = next(c for c in seen if c["path"].endswith("created.rs"))
    assert created["op"] == "upsert", created
    assert created["bytes"] == 12, created

    # A closed feed is exhausted rather than an error, so a for-loop ends cleanly
    # instead of raising something a caller has to special-case.
    feed.close()
    try:
        next(feed)
    except StopIteration:
        pass
    else:
        raise AssertionError("a closed feed must stop iterating")
    assert list(feed) == [], "iterating a closed feed yields nothing"

    # And it works as a context manager.
    with watch_index.watch(interval=0.1) as scoped:
        assert next(scoped) is not None

    # The long-lived surface owns one native engine and returns complete immutable
    # values. Drive one lifecycle through the installed wheel rather than importing a
    # sibling checkout, because that is the artifact MetaBrowser and other clients use.
    opened_root = pathlib.Path(tempfile.mkdtemp(prefix="fdu-opened-wheel-"))
    (opened_root / "alpha.txt").write_text("alpha")
    (opened_root / "src").mkdir()
    (opened_root / "src" / "main.rs").write_text("fn main() {}")
    try:
        OpenedOptions(max_files=0)
    except ValueError:
        pass
    else:
        raise AssertionError("a zero discovery budget must fail at the public boundary")

    opened = OpenedIndex.open(opened_root)
    opened.prioritize(("src",))

    state = opened.state()
    cursor = state.change_cursor
    for invalid_timeout in (float("inf"), float(2**64)):
        try:
            opened.changes(cursor, timeout=invalid_timeout)
        except ValueError:
            pass
        else:
            raise AssertionError("an unrepresentable timeout must fail before native polling")
    for _ in range(40):
        if state.state.coverage.kind is CoverageKind.COMPLETE:
            break
        polled = opened.changes(cursor, timeout=0.25)
        cursor = polled.cursor
        state = opened.state()
    assert state.state.coverage.kind is CoverageKind.COMPLETE, state

    response = opened.read(
        Lookup("alpha.txt"),
        DirectoryRollUp(""),
        Tree("", page=Page(limit=1, max_work=100_000)),
        Flat(page=Page(limit=2, max_work=100_000)),
        Aggregate(),
        ReportProjection(query=Query(views=(View.SUMMARY,))),
        Diagnostics(),
    )
    assert len(response.results) == 7, response
    lookup = response.results[0]
    assert lookup.kind == "lookup" and lookup.value.kind is KnowledgeKind.PRESENT, lookup
    tree_result = response.results[2]
    assert tree_result.kind == "tree", tree_result
    assert tree_result.value.kind is KnowledgeKind.PRESENT, tree_result
    page = tree_result.value.value
    assert page is not None and len(page.rows) == 1 and page.next is not None, page
    continued = opened.read(Continue(page.next))
    assert continued.results[0].kind == "tree", continued
    replayed = opened.read(Continue(page.next), Lookup("alpha.txt"))
    assert replayed.results[0].kind == "refused", replayed
    assert replayed.results[0].reason is RefusalReason.CONTINUATION_UNAVAILABLE, replayed
    assert replayed.results[1].kind == "lookup", replayed
    assert response.results[5].kind == "report", response.results[5]
    opened_report = response.results[5].value
    assert json.loads(opened_report.render(Format.JSON)) == opened_report.as_dict()
    assert response.results[6].kind == "diagnostics", response.results[6]

    # Version and cursor identities are scoped to one opened session. Crossing them
    # between roots must remain a typed recovery condition rather than a generic native
    # exception or an accidentally accepted read.
    foreign = OpenedIndex.open(opened_root)
    foreign_version = foreign.state().version
    try:
        opened.read(Diagnostics(), expected=foreign_version)
    except VersionUnavailableError:
        pass
    else:
        raise AssertionError("a foreign expected version must raise its typed error")
    try:
        opened.changes(foreign_version)
    except ChangeCursorUnavailableError:
        pass
    else:
        raise AssertionError("a foreign change cursor must raise its typed error")
    foreign.close()

    # A projection that meets a path of the wrong kind refuses alone. A directory a caller
    # paged earlier can become a file between reads; its tree page refuses, a roll-up of a
    # file refuses, and the lookup beside them still answers in the same read.
    kinds_root = pathlib.Path(tempfile.mkdtemp(prefix="fdu-opened-kinds-"))
    (kinds_root / "a").write_text("a")
    (kinds_root / "README.md").write_text("readme")
    (kinds_root / "dir").mkdir()
    (kinds_root / "dir" / "inner.txt").write_text("inner")
    with OpenedIndex.open(kinds_root) as kinds:
        for _ in range(40):
            if kinds.state().state.coverage.kind is CoverageKind.COMPLETE:
                break
            kinds.changes(kinds.state().change_cursor, timeout=0.25)
        assert kinds.read(Tree("dir")).results[0].kind == "tree"
        (kinds_root / "dir" / "inner.txt").unlink()
        (kinds_root / "dir").rmdir()
        (kinds_root / "dir").write_text("now a file")
        kinds.refresh(("dir",))
        mixed = kinds.read(Lookup("a"), Tree("dir"), DirectoryRollUp("README.md"))
        lookup_a, tree_dir, rollup_readme = mixed.results
        assert lookup_a.kind == "lookup" and lookup_a.value.kind is KnowledgeKind.PRESENT, mixed
        assert tree_dir.kind == "refused", tree_dir
        assert tree_dir.reason is RefusalReason.NOT_A_DIRECTORY, tree_dir
        assert tree_dir.path == pathlib.Path("dir"), tree_dir
        assert rollup_readme.kind == "refused", rollup_readme
        assert rollup_readme.reason is RefusalReason.NOT_A_DIRECTORY, rollup_readme
        assert rollup_readme.path == pathlib.Path("README.md"), rollup_readme

    # Every axis of an opened read's selection, a report projection's globs included,
    # matches the portable path a page returns: a name shown escaped is a filter a caller
    # can write back unchanged, and its native spelling matches nothing.
    escaped_root = pathlib.Path(tempfile.mkdtemp(prefix="fdu-opened-escaped-"))
    (escaped_root / "100%.txt").write_text("x")
    (escaped_root / "50%").mkdir()
    (escaped_root / "50%" / "inner.txt").write_text("y")
    with OpenedIndex.open(escaped_root) as escaped:
        for _ in range(40):
            if escaped.state().state.coverage.kind is CoverageKind.COMPLETE:
                break
            escaped.changes(escaped.state().change_cursor, timeout=0.25)
        wide = Page(limit=64, max_work=100_000)

        def admitted(selection: EntrySelection) -> set[str]:
            result = escaped.read(Flat(selection=selection, page=wide)).results[0]
            assert result.kind == "flat" and result.value.next is None, result
            return {row.portable_path for row in result.value.rows if row.kind is EntryKind.FILE}

        shown = admitted(EntrySelection())
        assert shown == {"100%25.txt", "50%25/inner.txt"}, shown
        for path in shown:
            parent, _, name = path.rpartition("/")
            assert admitted(EntrySelection(exact_names=(name,))) == {path}, path
            assert admitted(EntrySelection(query=Selection(include=(f"**/{path}",)))) == {path}
            if parent:
                assert admitted(EntrySelection(ancestor_names=(parent,))) == {path}, path
        assert admitted(EntrySelection(exact_names=("100%.txt",))) == set()
        report = escaped.read(
            ReportProjection(
                query=Query(views=(View.FILES,), selection=Selection(include=("50%25/*",)))
            )
        ).results[0]
        assert report.kind == "report", report
        files = report.value.sections[0]
        assert isinstance(files, FilesSection), files
        assert [row.path for row in files.files] == [pathlib.Path("50%") / "inner.txt"], files

    before_refresh = response.change_cursor
    (opened_root / "added.md").write_text("added")
    receipt = opened.refresh(("added.md",))
    assert receipt.accepted == (pathlib.Path("added.md"),), receipt
    changed = opened.changes(before_refresh)
    assert changed.outcome.kind is ChangeOutcomeKind.CHANGES, changed
    assert any(
        change.path == pathlib.Path("added.md")
        for commit in changed.outcome.commits
        for change in commit.changes
    ), changed

    # A tree page's depth and ignore pruning, and the registry a root classifies with,
    # reach the engine only through these typed values: the MetaBrowser adapter has no
    # other way to ask for a deeper page, a pruned one, or its own registry.
    shaped_root = pathlib.Path(tempfile.mkdtemp(prefix="fdu-opened-shape-"))
    (shaped_root / ".gitignore").write_text("build/\n")
    (shaped_root / "build").mkdir()
    (shaped_root / "build" / "out.o").write_text("object")
    (shaped_root / "src").mkdir()
    (shaped_root / "src" / "main.rs").write_text("fn main() {}")

    def settled(handle: OpenedIndex) -> ReadResponse:
        current = handle.state()
        cursor = current.change_cursor
        for _ in range(40):
            if current.state.coverage.kind is CoverageKind.COMPLETE:
                return current
            cursor = handle.changes(cursor, timeout=0.25).cursor
            current = handle.state()
        raise AssertionError(f"discovery did not complete: {current}")

    def tree_rows(handle: OpenedIndex, projection: Tree) -> set[str]:
        result = handle.read(projection).results[0]
        assert result.kind == "tree", result
        assert result.value.kind is KnowledgeKind.PRESENT, result
        page = result.value.value
        assert page is not None and page.next is None, page
        return {row.portable_path for row in page.rows}

    registry = '[[kind]]\nid = "notes"\nfamily = "prose"\nextensions = ["rs"]\n'
    with (
        OpenedIndex.open(shaped_root) as compiled_rules,
        OpenedIndex.open(shaped_root, OpenedOptions(type_rules=registry)) as custom_rules,
    ):
        compiled_state = settled(compiled_rules)
        custom_state = settled(custom_rules)

        full = Page(limit=100, max_work=100_000)
        one_level = tree_rows(compiled_rules, Tree("", page=full))
        assert {"build", "src"} <= one_level and "src/main.rs" not in one_level, one_level
        two_levels = tree_rows(compiled_rules, Tree("", page=full, depth=2))
        assert {"build/out.o", "src/main.rs"} <= two_levels, two_levels
        assert tree_rows(compiled_rules, Tree("", page=full, depth=Bound.ALL)) == two_levels
        pruned = tree_rows(
            compiled_rules, Tree("", page=full, depth=Bound.ALL, include_ignored=False)
        )
        # Pruning drops the ignored directory and everything under it, not only its row.
        assert "src/main.rs" in pruned, pruned
        assert not {"build", "build/out.o"} & pruned, pruned

        assert (
            custom_state.version.semantics.type_rules_fingerprint
            != compiled_state.version.semantics.type_rules_fingerprint
        ), "the reported identity must describe the registry the root was opened with"
        kinds = []
        for handle in (compiled_rules, custom_rules):
            lookup = handle.read(Lookup("src/main.rs")).results[0]
            assert lookup.kind == "lookup" and lookup.value.value is not None, lookup
            classification = lookup.value.value.classification
            assert classification is not None, lookup
            kinds.append(classification.kind_id)
        assert kinds[1] == "notes" and kinds[0] != "notes", kinds

    # A registry that does not parse is the caller's argument, rejected before discovery.
    try:
        OpenedIndex.open(shaped_root, OpenedOptions(type_rules="[[kind]]\nid = \n"))
    except InvalidArgumentError:
        pass
    else:
        raise AssertionError("an unparseable registry must raise InvalidArgumentError")

    # A journal budget read as an item count holds almost no history; the engine refuses it.
    try:
        OpenedIndex.open(shaped_root, OpenedOptions(journal_capacity_bytes=4096))
    except InvalidArgumentError as error:
        assert "journal_capacity_bytes is 4096 bytes" in str(error), error
    else:
        raise AssertionError("a journal budget below the minimum must raise InvalidArgumentError")

    opened.close()
    opened.close()
    try:
        opened.state()
    except OpenedIndexClosedError:
        pass
    else:
        raise AssertionError("an opened-root read after close must raise its typed error")

    # Python aliases share the native close authority. Exercise close concurrently from
    # two threads so idempotence is proven at the installed-wheel boundary, not inferred
    # from a Rust-only test.
    closable = OpenedIndex.open(opened_root)
    alias = closable
    close_barrier = threading.Barrier(3)

    def close_after_barrier(handle: OpenedIndex) -> None:
        close_barrier.wait()
        handle.close()

    with ThreadPoolExecutor(max_workers=2) as executor:
        closes = [executor.submit(close_after_barrier, handle) for handle in (closable, alias)]
        close_barrier.wait()
        for closing in closes:
            closing.result()
    try:
        closable.state()
    except OpenedIndexClosedError:
        pass
    else:
        raise AssertionError("all aliases must observe the shared closed lifecycle")

    print(f"fdu._native {fdu_py.__version__} ok")


if __name__ == "__main__":
    main()
