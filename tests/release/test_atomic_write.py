"""The shared atomic writer: whole files or none, with the call it replaces' behaviour."""

from __future__ import annotations

import gzip
import io
import os
import stat
import tempfile
import unittest
from pathlib import Path

from scripts.atomic_write import open_atomic, write_bytes_atomic, write_text_atomic


def leftovers(directory: Path) -> list[str]:
    return sorted(entry.name for entry in directory.iterdir() if ".tmp." in entry.name)


class AtomicWriteTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = Path(tempfile.mkdtemp())
        self.addCleanup(self._remove, self.directory)

    @staticmethod
    def _remove(directory: Path) -> None:
        for entry in sorted(directory.rglob("*"), reverse=True):
            if entry.is_dir() and not entry.is_symlink():
                entry.rmdir()
            else:
                entry.unlink()
        directory.rmdir()

    def test_writes_text_and_bytes_like_pathlib(self) -> None:
        text = self.directory / "notes.md"
        write_text_atomic(text, "one\ntwo\n", encoding="utf-8")
        plain = self.directory / "plain.md"
        plain.write_text("one\ntwo\n", encoding="utf-8")
        self.assertEqual(text.read_bytes(), plain.read_bytes())

        blob = self.directory / "blob.bin"
        write_bytes_atomic(blob, b"\x00\x01")
        self.assertEqual(blob.read_bytes(), b"\x00\x01")
        self.assertEqual(leftovers(self.directory), [])

    def test_replaces_an_existing_file_whole(self) -> None:
        target = self.directory / "state.json"
        target.write_text("old", encoding="utf-8")
        write_text_atomic(target, "new", encoding="utf-8")
        self.assertEqual(target.read_text(encoding="utf-8"), "new")
        self.assertEqual(leftovers(self.directory), [])

    def test_a_failure_mid_write_leaves_the_old_file_and_no_temporary(self) -> None:
        target = self.directory / "state.json"
        target.write_text("old", encoding="utf-8")
        with self.assertRaisesRegex(RuntimeError, "interrupted"), open_atomic(target) as output:
            output.write("half of the new")
            raise RuntimeError("interrupted")
        self.assertEqual(target.read_text(encoding="utf-8"), "old")
        self.assertEqual(leftovers(self.directory), [])

    def test_the_target_is_untouched_until_the_block_completes(self) -> None:
        target = self.directory / "report.md"
        target.write_text("old", encoding="utf-8")
        with open_atomic(target, encoding="utf-8") as output:
            output.write("new")
            output.flush()
            self.assertEqual(target.read_text(encoding="utf-8"), "old")
            self.assertEqual(len(leftovers(self.directory)), 1)
        self.assertEqual(target.read_text(encoding="utf-8"), "new")

    def test_an_unencodable_text_fails_before_the_target_changes(self) -> None:
        target = self.directory / "ascii.txt"
        target.write_text("old", encoding="ascii")
        with self.assertRaises(UnicodeEncodeError):
            write_text_atomic(target, "café", encoding="ascii")
        self.assertEqual(target.read_text(encoding="ascii"), "old")
        self.assertEqual(leftovers(self.directory), [])

    def test_exclusive_mode_refuses_an_existing_target(self) -> None:
        target = self.directory / "run.json"
        with open_atomic(target, "x", encoding="utf-8") as output:
            output.write("first")
        with self.assertRaises(FileExistsError), open_atomic(target, "x") as output:
            output.write("second")
        self.assertEqual(target.read_text(encoding="utf-8"), "first")
        self.assertEqual(leftovers(self.directory), [])

    def test_exclusive_mode_loses_a_race_without_clobbering(self) -> None:
        target = self.directory / "run.json"
        with self.assertRaises(FileExistsError), open_atomic(target, "xb") as output:
            target.write_bytes(b"the other writer")
            output.write(b"ours")
        self.assertEqual(target.read_bytes(), b"the other writer")
        self.assertEqual(leftovers(self.directory), [])

    def test_a_child_process_can_write_through_the_handle(self) -> None:
        target = self.directory / "stdout.txt"
        with open_atomic(target, "wb") as output:
            os.write(output.fileno(), b"from a descriptor\n")
        self.assertEqual(target.read_bytes(), b"from a descriptor\n")

    def test_deterministic_gzip_through_the_handle(self) -> None:
        target = self.directory / "run.json.gz"
        payload = b'{"runs": []}\n'
        with (
            open_atomic(target, "wb") as raw,
            gzip.GzipFile(filename="", mode="wb", fileobj=raw, compresslevel=9, mtime=0) as out,
        ):
            out.write(payload)
        buffer = io.BytesIO()
        with gzip.GzipFile(filename="", mode="wb", fileobj=buffer, compresslevel=9, mtime=0) as out:
            out.write(payload)
        self.assertEqual(target.read_bytes(), buffer.getvalue())

    @unittest.skipUnless(os.name == "posix", "permission bits are POSIX")
    def test_permissions_match_what_open_would_give(self) -> None:
        fresh = self.directory / "fresh.txt"
        write_text_atomic(fresh, "x")
        plain = self.directory / "plain.txt"
        plain.write_text("x")
        self.assertEqual(stat.S_IMODE(fresh.stat().st_mode), stat.S_IMODE(plain.stat().st_mode))

        script = self.directory / "run.sh"
        script.write_text("#!/bin/sh\n")
        script.chmod(0o750)
        write_text_atomic(script, "#!/bin/sh\necho\n")
        self.assertEqual(stat.S_IMODE(script.stat().st_mode), 0o750)

    @unittest.skipUnless(hasattr(os, "symlink") and os.name == "posix", "needs symlinks")
    def test_a_symbolic_link_is_written_through(self) -> None:
        real = self.directory / "real.txt"
        real.write_text("old", encoding="utf-8")
        link = self.directory / "link.txt"
        link.symlink_to(real.name)
        write_text_atomic(link, "new", encoding="utf-8")
        self.assertTrue(link.is_symlink())
        self.assertEqual(real.read_text(encoding="utf-8"), "new")

    def test_refuses_modes_that_are_not_a_whole_write(self) -> None:
        for mode in ("a", "r+", "w+"):
            with self.subTest(mode=mode), self.assertRaises(ValueError):
                open_atomic(self.directory / "log.txt", mode).__enter__()
        self.assertEqual(leftovers(self.directory), [])


if __name__ == "__main__":
    unittest.main()
