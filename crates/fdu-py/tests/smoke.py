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
    assert total["by_extension"][".txt"] == {"files": 1, "bytes": 5}, total
    assert total["by_extension"][".rs"] == {"files": 1, "bytes": 12}, total

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
            "--no-cache",
            "--json",
            "--apparent-size",
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
    assert cli_data["schema"] == "fdu.tree/2", cli_data
    assert cli_data["complete"] is True, cli_data
    assert cli_data["tree_truncated"] is True, cli_data
    assert cli_data["tree"]["bytes"] == 17, cli_data
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
                [os.fsencode(entrypoint), b"--no-cache", b"--json", raw_root],
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
                [os.fsencode(entrypoint), b"--no-cache", b"--json", raw_root],
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

    print(f"fdu_py {fdu_py.__version__} ok")


if __name__ == "__main__":
    main()
