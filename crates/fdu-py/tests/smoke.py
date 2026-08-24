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

from fdu import _native as fdu_py


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

    # Bulk child listing: one call returns a page of children with their subtree totals.
    page = index.children("")
    assert page is not None
    by_name = {c["name"]: c for c in page["rows"]}
    assert set(by_name) == {"a.txt", "src"}, by_name
    assert by_name["src"]["kind"] == "dir"
    assert by_name["src"]["files"] == 1, by_name
    assert by_name["a.txt"]["kind"] == "file"
    assert by_name["a.txt"]["bytes"] == 5
    assert page["remainder"] is None and page["next"] is None, page

    # Every bundled read reports what it cost, beside what it answered. The count is the
    # sum of what its projections did: the listing walks the root and its two rows (3),
    # the totals read the root (1), and the "src" roll-up walks root and src (2).
    bundled = index.read(children_of="", rollups=["src"], total=True)
    work = bundled["work"]
    assert work["rows"] == 2, work
    assert work["entries_visited"] == 3 + 1 + 2, work
    assert work["dirs_visited"] == 5, work
    assert work["lock_wait_ns"] <= work["wall_ns"], work

    # And it pages: a bound on the rows, a cursor to resume from, and what was left out.
    first = index.children("", None, 1)
    assert first is not None and len(first["rows"]) == 1, first
    assert first["next"] == first["rows"][0]["name"], first
    assert first["remainder"]["rows"] == 1, first
    rest = index.children("", first["next"], 1)
    assert rest is not None and rest["next"] is None, rest
    assert {c["name"] for c in first["rows"]} | {c["name"] for c in rest["rows"]} == {
        "a.txt",
        "src",
    }

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
    assert cli_data["schema"] == "fdu.report/4", cli_data
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
            raw_names = {os.fsencode(child["name"]) for child in raw_children["rows"]}
            assert raw_name in raw_names, raw_children
            assert raw_index.total()["by_extension"][".rs"]["files"] == 1

            raw_mark = raw_index.cursor()
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
    mark = index.cursor()
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

    # The cursor comes back with the ops, from the same read, and is the position to
    # resume from. Sampling the clock separately let a commit land in between, so the
    # next resume would start past a commit nobody had seen.
    assert changed["cursor"]["session"] == mark["session"], changed
    assert changed["cursor"]["clock"] >= mark["clock"], changed

    # A cursor this index cannot place is refused, not answered. Both shapes below used
    # to return an empty op list -- indistinguishable from "you are up to date" about a
    # position this index has never been at.
    for bad, why in (
        ({"session": mark["session"] ^ 0xFFFF, "clock": 0}, "another opened index"),
        ({"session": mark["session"], "clock": mark["clock"] + 10_000}, "a future position"),
    ):
        try:
            index.since(bad)
        except RuntimeError:  # the engine refuses; FduError derives from RuntimeError
            pass
        else:
            raise AssertionError(f"{why} must be refused rather than answered")

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
    kind_page = fdu_py.scan(str(kind_root)).children("")
    kinds = {child["name"]: child["kind"] for child in kind_page["rows"]}
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
        seen.extend(next(feed)["changes"])
        if any(change["path"].endswith("created.rs") for change in seen):
            break
    assert any(change["path"].endswith("created.rs") for change in seen), seen
    created = next(c for c in seen if c["path"].endswith("created.rs"))
    assert created["op"] == "upsert", created
    assert created["bytes"] == 12, created

    # One opened root is one authority. A watch used to deep-clone the index into a
    # private handle, so the object the caller kept went stale at the first event and
    # only a refresh() brought it back -- a server holding that index would serve numbers
    # that stopped being true, with nothing in the answer saying so. The session now
    # shares the opened handle, so consuming a mutation on the feed is consuming it on
    # the index.
    before = watch_index.read(total=True)
    # The `created.rs` event above was consumed on the feed, so the shared index has
    # already advanced. Under the old private clone this read still said clock 0, because
    # the mutation landed somewhere the caller could not see.
    assert before["clock"] > 0, (
        "events consumed on the feed must have advanced the opened index: "
        f"clock is still {before['clock']}"
    )
    (watch_root / "second.rs").write_text("fn second() {}")
    for _ in range(40):
        next(feed)
        after = watch_index.read(total=True)
        if after["clock"] != before["clock"]:
            break
    assert after["clock"] != before["clock"], (
        "a watch mutation must be visible from the index it was opened on, "
        f"without refresh: {before['clock']} == {after['clock']}"
    )
    assert after["total"]["files"] > before["total"]["files"], (before, after)

    # A closed feed is exhausted rather than an error, so a for-loop ends cleanly
    # instead of raising something a caller has to special-case.
    feed.close()

    # Closing the feed drops a reference, not the index. This is the fear the old deep
    # clone was defending against, and sharing the handle is what makes it a non-event.
    still_usable = watch_index.read(total=True)
    assert still_usable["total"]["files"] == after["total"]["files"], still_usable
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

    print(f"fdu._native {fdu_py.__version__} ok")


if __name__ == "__main__":
    main()
