"""Smoke test for the fdu Python extension module.

Runs against an installed wheel, so it checks what a user actually gets rather than what
the build tree contains. Deliberately dependency-free: it is executed by CI in a bare
virtualenv holding nothing but the wheel.

Run manually with:

    uv venv --clear .venv-smoke
    uv pip install --python .venv-smoke dist/*.whl
    uv run --no-project --python .venv-smoke python tests/smoke.py
"""

from __future__ import annotations

import os
import pathlib
import tempfile

import fdu_py


def main() -> None:
    root = pathlib.Path(tempfile.mkdtemp())
    (root / "a.txt").write_text("hello")
    (root / "src").mkdir()
    (root / "src" / "main.rs").write_text("fn main() {}")

    index = fdu_py.scan(str(root))

    assert index.root == os.path.realpath(root), index.root
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
