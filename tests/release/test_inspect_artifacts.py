"""Fixture tests for artifact inspection failures and evidence."""

from __future__ import annotations

import io
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path

from scripts.release.inspect_artifacts import inspect_directory

VERSION = "0.1.0"


def add_tar(archive: tarfile.TarFile, name: str, content: bytes = b"fixture") -> None:
    """Add one deterministic in-memory file to a tar archive."""
    info = tarfile.TarInfo(name)
    info.size = len(content)
    archive.addfile(info, io.BytesIO(content))


class InspectArtifactsTests(unittest.TestCase):
    """Artifact contents are contracts, not filename conventions."""

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.directory = Path(self.temporary.name)
        self._write_wheel()
        self._write_sdist()
        self._write_crate()

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def _write_wheel(self, *, typed: bool = True) -> None:
        path = self.directory / f"fdu-{VERSION}-cp312-abi3-manylinux_2_17_x86_64.whl"
        metadata = "\n".join(
            [
                "Metadata-Version: 2.4",
                "Name: fdu",
                f"Version: {VERSION}",
                "License-Expression: MIT",
                "License-File: LICENSE",
                "Classifier: Typing :: Typed",
                "",
            ]
        )
        with zipfile.ZipFile(path, "w") as archive:
            archive.writestr(f"fdu-{VERSION}.dist-info/METADATA", metadata)
            archive.writestr("fdu/__init__.py", "")
            archive.writestr("fdu/_native.pyi", "")
            if typed:
                archive.writestr("fdu/py.typed", "")
            archive.writestr("fdu/_native.abi3.so", b"native")
            archive.writestr(f"fdu-{VERSION}.dist-info/licenses/LICENSE", "MIT")
            archive.writestr(f"fdu-{VERSION}.dist-info/sboms/fdu.cyclonedx.json", "{}")

    def _write_sdist(self) -> None:
        path = self.directory / f"fdu-{VERSION}.tar.gz"
        prefix = f"fdu-{VERSION}/"
        with tarfile.open(path, "w:gz") as archive:
            for name in (
                "LICENSE",
                "README.md",
                "pyproject.toml",
                "python/fdu/__init__.py",
                "python/fdu/aio.py",
                "python/fdu/_native.pyi",
                "python/fdu/py.typed",
                "crates/fdu-py/examples/browser_provider.py",
                "crates/fdu-py/examples/rollup_adapter.py",
                "crates/fdu-py/examples/sse_resume.py",
                "crates/fdu-core/src/lib.rs",
            ):
                add_tar(archive, prefix + name)

    def _write_crate(self) -> None:
        path = self.directory / f"fdu-{VERSION}.crate"
        prefix = f"fdu-{VERSION}/"
        with tarfile.open(path, "w:gz") as archive:
            for name in ("Cargo.toml", "Cargo.toml.orig", "LICENSE", "README.md", "src/lib.rs"):
                add_tar(archive, prefix + name)

    def test_valid_artifacts_produce_hash_evidence(self) -> None:
        artifacts = inspect_directory(self.directory, VERSION)
        self.assertEqual({item.kind for item in artifacts}, {"crate", "sdist", "wheel"})
        self.assertTrue(all(len(item.sha256) == 64 for item in artifacts))

    def test_missing_typing_marker_is_rejected(self) -> None:
        self._write_wheel(typed=False)
        with self.assertRaisesRegex(ValueError, "py.typed"):
            inspect_directory(self.directory, VERSION)

    def test_release_matrix_rejects_a_host_only_wheel_set(self) -> None:
        with self.assertRaisesRegex(ValueError, "exactly five wheels"):
            inspect_directory(self.directory, VERSION, require_release_matrix=True)


if __name__ == "__main__":
    unittest.main()
