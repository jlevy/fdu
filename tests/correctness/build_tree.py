#!/usr/bin/env python3
"""Build a tree holding every file kind this platform supports.

The point is the kinds a fixture author does not think of. Each builder reports what it
actually managed to create, because a kind the platform refuses must be recorded as
absent rather than silently skipped -- a run that quietly built fewer kinds looks
identical to a run that passed.
"""

from __future__ import annotations

import os
import shutil
import socket
import stat
import sys
from contextlib import suppress
from pathlib import Path

made: dict[str, bool] = {}


def note(kind: str, ok: bool) -> None:
    made[kind] = ok


def build(root: Path, *, refusals: bool = True) -> dict[str, bool]:
    # A fresh directory, always. Re-running into an existing tree raises FileExistsError
    # from every `os.link`/`os.symlink`/`os.mkfifo`/`os.mknod`, which is an OSError, so a
    # second run used to report six kinds as "platform refused" and print a truthful-
    # looking 11-of-17. A report that degrades on rerun is worse than no report.
    if root.exists():
        # `denied-dir` is 0o000, so make the tree traversable before removing it.
        for path in root.rglob("*"):
            if path.is_dir() and not path.is_symlink():
                with suppress(OSError):
                    os.chmod(path, 0o755)
        shutil.rmtree(root)
    root.mkdir(parents=True)

    # --- regular files across sizes -------------------------------------------------
    (root / "empty.txt").write_bytes(b"")
    (root / "small.txt").write_text("one line\n")
    (root / "medium.rs").write_text("fn main() {}\n" * 500)
    (root / "prose.md").write_text("# Title\n\nSome words here.\n" * 50)
    (root / "binary.bin").write_bytes(bytes(range(256)) * 64)
    note("regular", True)

    # A sparse file: apparent size far exceeds allocated blocks, which is exactly where
    # `--size apparent` and `--size allocated` must disagree.
    try:
        with open(root / "sparse.img", "wb") as handle:
            handle.truncate(1 << 30)  # 1 GiB apparent, ~0 allocated
        note("sparse", True)
    except OSError:
        note("sparse", False)

    # --- hard links ------------------------------------------------------------------
    # Two names for one inode, and a second pair split across directories, so attribution
    # can be wrong in two distinguishable ways.
    links = root / "links"
    links.mkdir(exist_ok=True)
    elsewhere = root / "links-elsewhere"
    elsewhere.mkdir(exist_ok=True)
    try:
        target = links / "original.dat"
        target.write_bytes(b"x" * 4096)
        os.link(target, links / "same-dir-alias.dat")
        os.link(target, elsewhere / "other-dir-alias.dat")
        note("hardlink", True)
    except OSError:
        note("hardlink", False)

    # --- symlinks --------------------------------------------------------------------
    syms = root / "symlinks"
    syms.mkdir(exist_ok=True)
    try:
        os.symlink("../small.txt", syms / "relative")
        os.symlink(str(root / "small.txt"), syms / "absolute")
        os.symlink("../links", syms / "to-directory")
        os.symlink("nowhere-at-all", syms / "dangling")
        os.symlink("cycle", syms / "cycle")  # points at itself
        os.symlink("mutual-b", syms / "mutual-a")
        os.symlink("mutual-a", syms / "mutual-b")
        note("symlink", True)
    except OSError:
        note("symlink", False)

    # --- FIFOs and sockets -----------------------------------------------------------
    special = root / "special"
    special.mkdir(exist_ok=True)
    try:
        os.mkfifo(special / "a-fifo")
        note("fifo", True)
    except (OSError, AttributeError):
        note("fifo", False)
    try:
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.bind(str(special / "a-socket"))
        sock.close()
        note("socket", True)
    except OSError:
        note("socket", False)

    # --- device nodes ----------------------------------------------------------------
    try:
        os.mknod(special / "a-char-device", 0o600 | stat.S_IFCHR, os.makedev(1, 3))
        note("chardev", True)
    except OSError:
        note("chardev", False)
    try:
        os.mknod(special / "a-block-device", 0o600 | stat.S_IFBLK, os.makedev(7, 0))
        note("blockdev", True)
    except OSError:
        note("blockdev", False)

    # --- permission bits -------------------------------------------------------------
    modes = root / "modes"
    modes.mkdir(exist_ok=True)
    for name, bit in (("setuid", stat.S_ISUID), ("setgid", stat.S_ISGID), ("sticky", stat.S_ISVTX)):
        path = modes / f"{name}.bin"
        path.write_bytes(b"\x7fELF")
        os.chmod(path, 0o755 | bit)
    note("mode-bits", True)

    # Unreadable as a non-root user. Root ignores these, so the run that exercises them
    # has to drop privileges; recorded here either way.
    # Root ignores mode bits, so these entries exist but exercise nothing. Recorded as a
    # fact about the run rather than printed as a plain "yes".
    # A tree built without them is complete, which is what the serving proof needs: a
    # partial scan never writes the entry tier, so nothing it answers is ever served.
    note(
        "permission-denied-effective",
        os.geteuid() != 0 if refusals else "skipped (--without-refusals)",
    )
    denied = root / "denied"
    denied.mkdir(exist_ok=True)
    (denied / "secret.txt").write_text("hidden\n")
    (denied / "readable.txt").write_text("visible\n")
    nolist = root / "denied-dir"
    nolist.mkdir(exist_ok=True)
    (nolist / "inside.txt").write_text("unreachable\n")
    if refusals:
        os.chmod(denied / "secret.txt", 0o000)
        os.chmod(nolist, 0o000)

    # --- awkward names ---------------------------------------------------------------
    names = root / "names"
    names.mkdir(exist_ok=True)
    awkward = [
        "with space.txt",
        "with'quote.txt",
        'with"doublequote.txt',
        "with[bracket].txt",  # JSON Lines once rewrote bracketed paths
        "with{brace}.txt",
        "with\\backslash.txt",
        "with:colon.txt",
        "with,comma.txt",
        "with\ttab.txt",
        "with\nnewline.txt",
        "-leading-dash.txt",
        "--looks-like-a-flag.txt",
        "unicode-éà中文-\U0001f600.txt",
        "CaseCollide.txt",
        "casecollide.txt",  # distinct on Linux, collapses on APFS/NTFS
        ".hidden",
        "trailing.space .txt",
    ]
    created = 0
    for name in awkward:
        try:
            (names / name).write_text("n\n")
            created += 1
        except OSError:
            pass
    note("awkward-names", created == len(awkward))
    made["awkward-names-created"] = created  # type: ignore[assignment]

    # A name that is not valid UTF-8 at all.
    try:
        with open(os.path.join(os.fsencode(names), b"invalid-\xff\xfe-utf8.txt"), "wb") as handle:
            handle.write(b"b\n")
        note("non-utf8-name", True)
    except OSError:
        note("non-utf8-name", False)

    # --- depth and breadth -----------------------------------------------------------
    deep = root / "deep"
    current = deep
    for level in range(40):
        current = current / f"level-{level:02d}"
    current.mkdir(parents=True, exist_ok=True)
    (current / "bottom.txt").write_text("deep\n")
    note("deep-nesting", True)

    # A path close to the per-component and total limits.
    try:
        long_dir = root / ("l" * 200)
        long_dir.mkdir(exist_ok=True)
        (long_dir / ("f" * 200 + ".txt")).write_text("long\n")
        note("long-names", True)
    except OSError:
        note("long-names", False)

    wide = root / "wide"
    wide.mkdir(exist_ok=True)
    for index in range(500):
        (wide / f"file-{index:04d}.txt").write_text(f"{index}\n")
    note("wide-directory", True)

    (root / "empty-dir").mkdir(exist_ok=True)
    note("empty-dir", True)

    # --- ignore controls -------------------------------------------------------------
    ignored = root / "ignored"
    ignored.mkdir(exist_ok=True)
    (ignored / "keep.txt").write_text("keep\n")
    (ignored / "skip.log").write_text("skip\n")
    (ignored / ".gitignore").write_text("*.log\n")
    (root / ".gitignore").write_text("ignored-at-root/\n")
    root_ignored = root / "ignored-at-root"
    root_ignored.mkdir(exist_ok=True)
    (root_ignored / "inside.txt").write_text("inside\n")
    note("gitignore", True)

    return made


if __name__ == "__main__":
    arguments = sys.argv[1:]
    refusals = "--without-refusals" not in arguments
    target = Path(next(arg for arg in arguments if not arg.startswith("--")))
    facts = build(target, refusals=refusals)
    print(f"running as uid {os.geteuid()}\n")
    width = max(len(k) for k in facts)
    for kind, ok in facts.items():
        if isinstance(ok, bool):
            print(f"{kind.ljust(width)}  {'yes' if ok else 'NO (platform refused)'}")
        else:
            print(f"{kind.ljust(width)}  {ok}")
    if os.geteuid() == 0:
        print("\nNote: running as root. Device nodes need CAP_MKNOD and so need root,")
        print("but mode bits are ignored by root, so the unreadable file and unlistable")
        print("directory exercise nothing. No single run can honestly report all kinds:")
        print("build as root for devices, then run the comparison as an unprivileged user.")
    absent = [k for k, v in facts.items() if v is False]
    print(f"\nkinds present: {sum(1 for v in facts.values() if v is True)}, absent: {len(absent)}")
    if absent:
        print("absent: " + ", ".join(absent))
