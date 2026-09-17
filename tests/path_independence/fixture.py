"""Build the path-independence fixture tree, identically on every platform.

The tree is small but covers what makes an fdu answer depend on its inputs: nested
`.gitignore` files (one larger than a 1 KiB budget), code in languages with and without
a line-of-code counter, prose, a binary, hidden and empty entries, symlinks, and mtimes
that straddle the selection windows the matrix requests.

Every byte and timestamp is fixed, so two builds are identical apart from what the
platform cannot represent: where symbolic links cannot be created, they are skipped and
`FixtureFacts.symlinks` records that.
"""

from __future__ import annotations

import os
import random
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path

BLOB_SEED = 0x5EED
BLOB_BYTES = 9000

FILES: dict[str, bytes] = {
    ".gitignore": b"build/\n*.log\n",
    "src/.gitignore": b"*.tmp\n",
    "big/.gitignore": b"".join(f"generated_pattern_number_{i}/\n".encode() for i in range(1, 121))
    + b"*.bak\n",
    "src/main.rs": (
        b"// Entry point\n"
        b"fn main() {\n"
        b"    // say hello\n"
        b'    println!("hello world");\n'
        b"\n"
        b"    let x = 1 + 2;\n"
        b'    println!("{}", x);\n'
        b"}\n"
    ),
    "src/nested/deep/util.rs": (
        b"/// Adds numbers.\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n"
    ),
    "src/lib.py": (
        b'"""Module docstring."""\n'
        b"# comment line\n"
        b"\n"
        b"def f(x):\n"
        b"    return x * 2  # trailing\n"
        b"\n"
        b"\n"
        b"class A:\n"
        b"    pass\n"
    ),
    "src/Main.hs": (
        b'-- Haskell comment\nmodule Main where\n\nmain :: IO ()\nmain = putStrLn "hi"\n'
    ),
    "src/scratch.tmp": b"scratch data that is ignored by pattern\n",
    "docs/readme.md": (
        b"# Title\n"
        b"\n"
        b"Some prose words here, with *emphasis* and a [link](http://example.com).\n"
        b"\n"
        b"- list item one\n"
        b"- list item two\n"
        b"\n"
        b"```rust\n"
        b"fn code_in_doc() {}\n"
        b"```\n"
    ),
    "docs/notes.txt": (
        b"Plain text notes. These are several words of prose\n"
        b"spread over two lines.\n"
        b"\n"
        b"A second paragraph follows.\n"
    ),
    "build/obj/out.o": b"\x00\x01\x02binary\x00",
    "app.log": b"build log line\nanother\n",
    ".hidden": b"hidden secret config\n",
    "big/keep.txt": b"keep me\n",
    "big/old.bak": b"backup content that is ignored only if big/.gitignore is read\n",
    "vendor/thing.c": b"vendored code\n",
}

DIRECTORIES = ("src/nested/deep", "docs", "data", "build/obj", "empty", "big", "vendor")

# (link path, target relative to the link's directory, target is a directory)
SYMLINKS = (("link_to_main", "src/main.rs", False), ("link_to_src", "src", True))


def _instant(year: int, month: int) -> int:
    """Nanoseconds since the epoch for 01:01 UTC on the first of the month."""
    return int(datetime(year, month, 1, 1, 1, tzinfo=UTC).timestamp()) * 1_000_000_000


# Grouped by instant. Directories come last, because creating an entry changes its
# parent's mtime, and the root is last of all.
MTIMES: tuple[tuple[int, tuple[str, ...]], ...] = (
    (_instant(2021, 1), ("src/main.rs",)),
    (_instant(2022, 1), ("src/lib.py",)),
    (_instant(2023, 1), ("src/Main.hs",)),
    (_instant(2023, 2), ("src/nested/deep/util.rs",)),
    (_instant(2023, 3), ("docs/readme.md",)),
    (_instant(2023, 4), ("docs/notes.txt",)),
    (_instant(2023, 5), ("data/blob.bin",)),
    (_instant(2023, 6), ("build/obj/out.o", "app.log", ".hidden")),
    (
        _instant(2023, 7),
        ("big/keep.txt", "big/old.bak", "big/.gitignore", "src/scratch.tmp", "vendor/thing.c"),
    ),
    (_instant(2023, 8), (".gitignore", "src/.gitignore")),
    (
        _instant(2023, 9),
        (
            "src/nested/deep",
            "src/nested",
            "docs",
            "data",
            "build/obj",
            "build",
            "empty",
            "big",
            "vendor",
            "src",
        ),
    ),
    (_instant(2023, 10), (".",)),
)
SYMLINK_MTIME = _instant(2020, 1)


@dataclass(frozen=True)
class FixtureFacts:
    """What a built fixture contains beyond its fixed bytes, and what the host enforces."""

    root: Path
    symlinks: bool
    permissions: bool


def build_fixture(root: Path) -> FixtureFacts:
    """Create the fixture at `root`, which must not exist yet."""
    root.mkdir(parents=True)
    for directory in DIRECTORIES:
        (root / directory).mkdir(parents=True, exist_ok=True)
    for relative, content in FILES.items():
        (root / relative).write_bytes(content)
    (root / "data/blob.bin").write_bytes(random.Random(BLOB_SEED).randbytes(BLOB_BYTES))

    symlinks = _create_symlinks(root)
    for instant, paths in MTIMES:
        for relative in paths:
            os.utime(root / relative, ns=(instant, instant))
    return FixtureFacts(root=root, symlinks=symlinks, permissions=_permissions_enforced(root))


def _permissions_enforced(root: Path) -> bool:
    """Whether a directory with no permissions is unlistable here.

    Windows ignores POSIX mode bits and root bypasses them, so an unreadable-subtree case
    cannot be built on either. Probed beside the fixture, never inside it.
    """
    probe = root.parent / f".{root.name}-permission-probe"
    probe.mkdir()
    try:
        os.chmod(probe, 0)
        try:
            os.listdir(probe)
        except PermissionError:
            return True
        return False
    finally:
        os.chmod(probe, 0o755)
        probe.rmdir()


def _create_symlinks(root: Path) -> bool:
    """Create the fixture's symlinks, or none where the platform refuses them."""
    created: list[Path] = []
    try:
        for link, target, is_directory in SYMLINKS:
            path = root / link
            os.symlink(target, path, target_is_directory=is_directory)
            created.append(path)
    except OSError:
        # Windows without Developer Mode refuses symlink creation. Remove any partial
        # set so a tree has all of its links or none.
        for path in created:
            path.unlink()
        return False
    if os.utime in os.supports_follow_symlinks:
        for link, _, _ in SYMLINKS:
            os.utime(root / link, ns=(SYMLINK_MTIME, SYMLINK_MTIME), follow_symlinks=False)
    return True
