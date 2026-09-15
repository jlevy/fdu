"""Tests for the packaged-crate smoke test's resolution contract and orchestration."""

from __future__ import annotations

import io
import subprocess
import tarfile
import tempfile
import unittest
from pathlib import Path
from textwrap import dedent
from typing import Any

from scripts.release.smoke_crate import smoke_crate, verify_relock

VERSION = "0.1.0"
REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"

# The shape `cargo package` writes into the `fdu` crate: `fdu-core` pinned to crates.io.
SHIPPED_LOCK = dedent(f"""
    version = 4

    [[package]]
    name = "anstyle"
    version = "1.0.13"
    source = "{REGISTRY}"
    checksum = "{"a" * 64}"

    [[package]]
    name = "fdu"
    version = "{VERSION}"
    dependencies = [
     "fdu-core",
    ]

    [[package]]
    name = "fdu-core"
    version = "{VERSION}"
    source = "{REGISTRY}"
    checksum = "{"b" * 64}"
    dependencies = [
     "anstyle",
    ]
    """).lstrip()

# What `cargo metadata` writes once `fdu-core` is patched to the packaged sibling.
RELOCKED_LOCK = SHIPPED_LOCK.replace(f'source = "{REGISTRY}"\nchecksum = "{"b" * 64}"\n', "")


def write_crate(directory: Path, package: str, files: dict[str, str]) -> None:
    """Write one `.crate` archive holding the given package-relative files."""
    with tarfile.open(directory / f"{package}-{VERSION}.crate", "w:gz") as archive:
        for name, content in files.items():
            data = content.encode()
            info = tarfile.TarInfo(f"{package}-{VERSION}/{name}")
            info.size = len(data)
            archive.addfile(info, io.BytesIO(data))


class FakeCargo:
    """Stands in for cargo and the installed binary, recording every command."""

    def __init__(self, *, relocked: str = RELOCKED_LOCK, reported: str = f"fdu {VERSION}\n"):
        self.relocked = relocked
        self.reported = reported
        self.failing: str | None = None
        self.calls: list[tuple[list[str], dict[str, Any]]] = []

    def __call__(self, command: list[str], **options: Any) -> subprocess.CompletedProcess[str]:
        self.calls.append((command, options))
        step = command[1]
        if step == self.failing:
            return subprocess.CompletedProcess(command, 101, "")
        if step == "metadata":
            manifest = Path(command[command.index("--manifest-path") + 1])
            (manifest.parent / "Cargo.lock").write_text(self.relocked, encoding="utf-8")
        stdout = self.reported if step == "--version" else "fdu-core-0.1.0  1 KB\n"
        return subprocess.CompletedProcess(command, 0, stdout)


class VerifyRelockTests(unittest.TestCase):
    """Only fdu-core's source may change when it is patched to its sibling."""

    def test_a_source_only_change_is_accepted(self) -> None:
        verify_relock(SHIPPED_LOCK, RELOCKED_LOCK, VERSION)

    def test_any_other_difference_is_rejected(self) -> None:
        cases = {
            "the patch did not apply": SHIPPED_LOCK,
            "moved pins other than fdu-core: anstyle": RELOCKED_LOCK.replace("1.0.13", "1.0.14"),
        }
        for message, relocked in cases.items():
            with self.subTest(message), self.assertRaisesRegex(ValueError, message):
                verify_relock(SHIPPED_LOCK, relocked, VERSION)


class SmokeCrateTests(unittest.TestCase):
    """The smoke installs the packaged CLI, locked, against the packaged engine."""

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.crates = root / "crates"
        self.crates.mkdir()
        self.work = root / "work"
        write_crate(self.crates, "fdu-core", {"Cargo.toml": "", "src/lib.rs": ""})
        write_crate(self.crates, "fdu", {"Cargo.toml": "", "Cargo.lock": SHIPPED_LOCK})

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_install_is_locked_and_patched_to_the_packaged_engine(self) -> None:
        cargo = FakeCargo()
        smoke_crate(self.crates, VERSION, self.work, runner=cargo)
        steps = [command[1] for command, _ in cargo.calls]
        self.assertEqual(steps, ["metadata", "install", "--version", "--cache"])
        install, options = cargo.calls[1]
        self.assertIn("--locked", install)
        core = (self.work / f"fdu-core-{VERSION}").resolve()
        self.assertEqual(
            install[install.index("--config") + 1], f'patch.crates-io.fdu-core.path="{core}"'
        )
        self.assertEqual(install[install.index("--path") + 1], str(self.work / f"fdu-{VERSION}"))
        self.assertEqual(options["env"]["FDU_RELEASE_TAG"], f"v{VERSION}")

    def test_a_failed_install_stops_the_smoke(self) -> None:
        cargo = FakeCargo()
        cargo.failing = "install"
        with self.assertRaisesRegex(ValueError, "exited with status 101"):
            smoke_crate(self.crates, VERSION, self.work, runner=cargo)
        self.assertEqual([command[1] for command, _ in cargo.calls], ["metadata", "install"])

    def test_a_binary_reporting_another_version_is_rejected(self) -> None:
        cargo = FakeCargo(reported=f"fdu {VERSION}-dev+g0123456789\n")
        with self.assertRaisesRegex(ValueError, "reports 'fdu 0.1.0-dev"):
            smoke_crate(self.crates, VERSION, self.work, runner=cargo)

    def test_a_used_work_directory_is_refused(self) -> None:
        self.work.mkdir()
        (self.work / "leftover").write_text("", encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "must be empty"):
            smoke_crate(self.crates, VERSION, self.work, runner=FakeCargo())


if __name__ == "__main__":
    unittest.main()
