"""Smoke test for the fdu Python extension module.

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
import subprocess
import sys
import tempfile

import fdu_py


def main() -> None:
    root = pathlib.Path(tempfile.mkdtemp())
    (root / "a.txt").write_text("hello")
    (root / "src").mkdir()
    (root / "src" / "main.rs").write_text("fn main() {}")

    index = fdu_py.scan(str(root))

    # Rust preserves Windows canonical verbatim paths (`\\?\`), while Python's
    # realpath commonly returns the conventional spelling. Compare the filesystem
    # identity rather than weakening native long-path behavior to satisfy a string.
    assert os.path.samefile(index.root, root), (index.root, root)
    assert index.complete is True, index.errors
    assert index.freshness == "fresh", index.freshness
    assert index.errors == [], index.errors
    assert len(index) == 4, f"root + 2 files + 1 dir, got {len(index)}"

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
    assert version.stdout == f"fdu {fdu_py.__version__}\n", version.stdout
    assert version.stderr == "", version.stderr

    help_result = subprocess.run(
        [entrypoint, "--help"], check=False, capture_output=True, text=True
    )
    assert help_result.returncode == 0, help_result
    assert "Output and automation:" in help_result.stdout, help_result.stdout
    assert "--color <WHEN>" in help_result.stdout, help_result.stdout
    assert "--skill" in help_result.stdout, help_result.stdout
    assert help_result.stderr == "", help_result.stderr

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
    assert cli_data["schema"] == "fdu.report/1", cli_data
    assert cli_data["complete"] is True, cli_data
    tree = cli_data["reports"][0]["tree"]
    assert tree["bytes"] == 17, cli_data
    # Truncation is per node rather than one whole-tree flag: the root expanded its own
    # level, and the child that was not expanded is the one that says so.
    assert tree["truncated"] is False, cli_data
    assert tree["children"][0]["truncated"] is True, cli_data
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
    paths = sorted(row["path"] for row in rust_only["reports"][0]["files"])
    assert paths == ["src/lib.rs", "src/main.rs"], paths

    types = index.report(views=["types"])["reports"][0]["types"]
    extensions = sorted(row["extension"] for row in types)
    assert extensions == [".md", ".rs"], extensions

    tree = index.report(views=["tree"], depth="all")["reports"][0]["tree"]
    assert tree["name"] == ".", tree
    assert any(child["name"] == "src" for child in tree["children"]), tree

    # Several views come back in request order, from one index.
    ordered = index.report(views=["types", "summary"])["reports"]
    assert [section["view"] for section in ordered] == ["types", "summary"], ordered

    # Value grammars are shared, so a bad value is rejected the same way everywhere.
    for bad in [
        {"min_size": "10X"},
        {"modified_since": "1.5h"},
        {"views": ["bogus"]},
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
    assert status["root"] == str(cache_root.resolve()), status
    assert fdu_py.clear_cache(str(cache_root)) is True
    assert fdu_py.cache_status(str(cache_root))["recognized"] is False

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

    print(f"fdu_py {fdu_py.__version__} ok")


if __name__ == "__main__":
    main()
