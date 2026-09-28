"""Create a bounded synthetic tree and a repeatable disk-growth mutation set."""

from __future__ import annotations

import argparse
import contextlib
import io
import json
import os
import stat
import tempfile
import unittest
from collections.abc import Generator
from pathlib import Path
from unittest.mock import patch

DIRECTORIES = 200
FILES_PER_DIRECTORY = 100
PAYLOAD = b"spike growth\n" * 1024
MARKER = ".fdu-spike-fixture.json"
OPEN_DIRECTORY = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC


@contextlib.contextmanager
def directory_fd(path: Path) -> Generator[int]:
    """Anchor every operation below descriptors opened without following ancestors."""
    descriptor = os.open("/", OPEN_DIRECTORY)
    try:
        for component in path.parts[1:]:
            if component in (".", ".."):
                raise ValueError("noncanonical directory component")
            child = os.open(component, OPEN_DIRECTORY, dir_fd=descriptor)
            os.close(descriptor)
            descriptor = child
        yield descriptor
    finally:
        os.close(descriptor)


@contextlib.contextmanager
def child_fd(
    parent: int, name: str, *, directory: bool = False, create: bool = False
) -> Generator[int]:
    if "/" in name or name in ("", ".", ".."):
        raise ValueError("single child name required")
    flags = (
        OPEN_DIRECTORY if directory else os.O_RDWR | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK
    )
    if create:
        flags |= os.O_CREAT | os.O_EXCL
    descriptor = os.open(name, flags, 0o644, dir_fd=parent)
    try:
        if not directory and not stat.S_ISREG(os.fstat(descriptor).st_mode):
            raise ValueError("fixture file must be regular")
        yield descriptor
    finally:
        os.close(descriptor)


def write_fd(descriptor: int, payload: bytes) -> None:
    os.lseek(descriptor, 0, os.SEEK_SET)
    os.ftruncate(descriptor, 0)
    offset = 0
    while offset < len(payload):
        written = os.write(descriptor, payload[offset:])
        if not written:
            raise OSError("short fixture write")
        offset += written


def write_new(parent: int, name: str, payload: bytes) -> None:
    with child_fd(parent, name, create=True) as descriptor:
        write_fd(descriptor, payload)


def marker_payload(mutated: bool) -> bytes:
    return json.dumps({"schema": "fdu-growth-fixture-v1", "mutated": mutated}).encode()


def prepare(root: int) -> None:
    for index in range(DIRECTORIES):
        name = f"branch-{index:03d}"
        os.mkdir(name, dir_fd=root)
        with child_fd(root, name, directory=True) as branch:
            for number in range(FILES_PER_DIRECTORY):
                write_new(branch, f"file-{number:03d}", b"baseline\n")
    write_new(root, "root-file", b"before\n")
    with (
        child_fd(root, "branch-001", directory=True) as source,
        child_fd(root, "branch-002", directory=True) as destination,
    ):
        os.link(
            "file-000", "alias", src_dir_fd=source, dst_dir_fd=destination, follow_symlinks=False
        )
    write_new(root, MARKER, marker_payload(False))


def mutate(root: int) -> None:
    # Validate all existing mutation targets before the first write. Retaining their
    # descriptors prevents later path substitutions from redirecting file writes.
    with contextlib.ExitStack() as stack:
        marker = stack.enter_context(child_fd(root, MARKER))
        if os.fstat(marker).st_size > 1024:
            raise ValueError("oversized fixture marker")
        state = json.loads(os.read(marker, 1025))
        if state != {"schema": "fdu-growth-fixture-v1", "mutated": False}:
            raise ValueError("fixture already mutated or not recognized")
        branches = {
            index: stack.enter_context(child_fd(root, f"branch-{index:03d}", directory=True))
            for index in (0, 1, 3, 4)
        }
        files = [stack.enter_context(child_fd(root, "root-file"))]
        files.extend(stack.enter_context(child_fd(branches[index], "file-000")) for index in (0, 1))
        stack.enter_context(child_fd(branches[3], "file-000"))
        for name in ("renamed-branch", "new"):
            try:
                os.stat(name, dir_fd=root, follow_symlinks=False)
            except FileNotFoundError:
                continue
            raise ValueError("fixture mutation destination already exists")
        for descriptor in files:
            write_fd(descriptor, PAYLOAD)
        os.unlink("file-000", dir_fd=branches[3])
        os.rename("branch-004", "renamed-branch", src_dir_fd=root, dst_dir_fd=root)
        os.mkdir("new", dir_fd=root)
        with child_fd(root, "new", directory=True) as new:
            os.mkdir("subtree", dir_fd=new)
            with child_fd(new, "subtree", directory=True) as subtree:
                write_new(subtree, "data", PAYLOAD)
        write_fd(marker, marker_payload(True))


def run(action: str, directory: Path) -> None:
    """Only mutate a new, explicitly marked fixture on a mounted external volume."""
    directory = directory.absolute()
    if len(directory.parts) < 5 or directory.parts[1] != "Volumes":
        raise ValueError("fixture must be below an external volume")
    volume = Path(*directory.parts[:3])
    if not os.path.ismount(volume) or directory.parent.resolve() != directory.parent:
        raise ValueError("mounted volume and nonsymlink parent required")
    if action == "prepare":
        with directory_fd(directory.parent) as parent:
            os.mkdir(directory.name, mode=0o700, dir_fd=parent)
            with child_fd(parent, directory.name, directory=True) as root:
                prepare(root)
    elif action == "mutate":
        with directory_fd(directory) as root:
            mutate(root)
    else:
        raise ValueError("unknown fixture action")
    print(
        json.dumps(
            {
                "action": action,
                "directories": DIRECTORIES,
                "files_per_directory": FILES_PER_DIRECTORY,
            }
        )
    )


class FixtureTests(unittest.TestCase):
    def test_mutation_set_and_repeat_refusal(self) -> None:
        with (
            tempfile.TemporaryDirectory() as temporary,
            patch.dict(globals(), DIRECTORIES=5, FILES_PER_DIRECTORY=2),
            contextlib.redirect_stdout(io.StringIO()),
        ):
            root = Path(temporary) / "fixture"
            run("prepare", root)
            run("mutate", root)
            self.assertEqual((root / "root-file").read_bytes(), PAYLOAD)
            self.assertEqual((root / "branch-002/alias").read_bytes(), PAYLOAD)
            self.assertFalse((root / "branch-003/file-000").exists())
            self.assertTrue((root / "renamed-branch/file-000").is_file())
            self.assertEqual((root / "new/subtree/data").read_bytes(), PAYLOAD)
            with self.assertRaises(ValueError):
                run("mutate", root)

    def test_file_and_ancestor_symlinks_refuse_before_writes(self) -> None:
        for target in ("root-file", "branch-000", MARKER):
            with (
                self.subTest(target=target),
                tempfile.TemporaryDirectory() as temporary,
                patch.dict(globals(), DIRECTORIES=5, FILES_PER_DIRECTORY=2),
                contextlib.redirect_stdout(io.StringIO()),
            ):
                base = Path(temporary)
                root, outside = base / "fixture", base / "outside"
                outside.mkdir()
                sentinel = outside / "file-000"
                sentinel.write_bytes(b"outside sentinel")
                run("prepare", root)
                original = root / target
                original.rename(root / "displaced")
                original.symlink_to(
                    outside if target == "branch-000" else sentinel,
                    target_is_directory=target == "branch-000",
                )
                with self.assertRaises(OSError):
                    run("mutate", root)
                self.assertEqual(sentinel.read_bytes(), b"outside sentinel")
                self.assertEqual((root / "branch-001/file-000").read_bytes(), b"baseline\n")
                if target != "root-file":
                    self.assertEqual((root / "root-file").read_bytes(), b"before\n")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("prepare", "mutate", "self-test"))
    parser.add_argument("directory", type=Path, nargs="?")
    args = parser.parse_args()
    if args.action == "self-test":
        unittest.main(argv=[__file__])
        return
    if args.directory is None:
        parser.error("directory required for prepare/mutate")
    run(args.action, args.directory)


if __name__ == "__main__":
    main()
