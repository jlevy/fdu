"""Tests for the guards around every registry write in the publish job."""

from __future__ import annotations

import io
import json
import shutil
import sys
import tarfile
import tempfile
import unittest
import zipfile
from contextlib import redirect_stderr, redirect_stdout
from email.message import Message
from pathlib import Path
from unittest.mock import patch
from urllib.error import HTTPError
from urllib.request import Request

from scripts.release import inspect_artifacts, publish_gate
from scripts.release.publish_gate import (
    Conflict,
    WaitTimeout,
    audit,
    check_environment,
    compare_crates,
    crate_served,
    crate_states,
    index_checksum,
    index_path,
    pypi_verdict,
    verify_files,
    wait_until,
)
from scripts.release.registry_state import RegistryError

VERSION = "0.1.0"
CRATES_IO = "https://crates.io/api/v1/crates"
INDEX = "https://index.crates.io"
PYPI = f"https://pypi.org/pypi/fdu/{VERSION}/json"
WHEEL_PLATFORMS = (
    "manylinux_2_17_x86_64.manylinux2014_x86_64",
    "manylinux_2_17_aarch64.manylinux2014_aarch64",
    "macosx_10_12_x86_64",
    "macosx_11_0_arm64",
    "win_amd64",
)


def add_tar(archive: tarfile.TarFile, name: str, content: bytes = b"fixture") -> None:
    """Add one deterministic in-memory file to a tar archive."""
    info = tarfile.TarInfo(name)
    info.size = len(content)
    archive.addfile(info, io.BytesIO(content))


def write_release_set(directory: Path) -> None:
    """Write a structurally valid release matrix: five wheels, one sdist, both crates."""
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
    for platform in WHEEL_PLATFORMS:
        path = directory / f"fdu-{VERSION}-cp312-abi3-{platform}.whl"
        with zipfile.ZipFile(path, "w") as archive:
            archive.writestr(f"fdu-{VERSION}.dist-info/METADATA", metadata)
            for member in ("fdu/__init__.py", "fdu/_native.pyi", "fdu/py.typed"):
                archive.writestr(member, "")
            archive.writestr("fdu/_native.abi3.so", platform)
            archive.writestr(f"fdu-{VERSION}.dist-info/licenses/LICENSE", "MIT")
            archive.writestr(f"fdu-{VERSION}.dist-info/sboms/fdu.cyclonedx.json", "{}")
    with tarfile.open(directory / f"fdu-{VERSION}.tar.gz", "w:gz") as archive:
        for name in (
            "LICENSE",
            "README.md",
            "pyproject.toml",
            "python/fdu/__init__.py",
            "python/fdu/_native.pyi",
            "python/fdu/py.typed",
            "crates/fdu-py/examples/rollup_adapter.py",
            "crates/fdu-core/src/lib.rs",
        ):
            add_tar(archive, f"fdu-{VERSION}/{name}")
    for package in ("fdu-core", "fdu"):
        write_crate(directory, package)


def write_crate(directory: Path, package: str, content: bytes = b"fixture") -> Path:
    """Write one structurally valid `.crate`, varying its bytes through `content`."""
    path = directory / f"{package}-{VERSION}.crate"
    with tarfile.open(path, "w:gz") as archive:
        for name in ("Cargo.toml", "Cargo.toml.orig", "LICENSE", "README.md", "src/lib.rs"):
            add_tar(archive, f"{package}-{VERSION}/{name}", content)
    return path


def record_evidence(directory: Path, evidence: Path) -> tuple[Path, Path]:
    """Produce the manifest and SHA256SUMS exactly as the evidence job does."""
    manifest = evidence / "release-manifest.json"
    checksums = evidence / "SHA256SUMS"
    argv = [
        "inspect_artifacts.py",
        str(directory),
        "--version",
        VERSION,
        "--manifest",
        str(manifest),
        "--checksums",
        str(checksums),
        "--require-release-matrix",
    ]
    with patch.object(sys, "argv", argv):
        inspect_artifacts.main()
    return manifest, checksums


def write_manifest(directory: Path, crates: dict[str, str], python: dict[str, str]) -> Path:
    """Write a manifest naming the given crate and Python digests."""
    artifacts = [
        {"filename": f"{package}-{VERSION}.crate", "kind": "crate", "sha256": digest}
        for package, digest in crates.items()
    ] + [
        {
            "filename": filename,
            "kind": "sdist" if filename.endswith(".tar.gz") else "wheel",
            "sha256": digest,
        }
        for filename, digest in python.items()
    ]
    manifest = directory / "manifest.json"
    manifest.write_text(json.dumps({"version": VERSION, "artifacts": artifacts}), encoding="utf-8")
    return manifest


def version_record(checksum: str) -> bytes:
    """A crates.io version record carrying `checksum`, as the API serves it."""
    return json.dumps({"version": {"num": VERSION, "checksum": checksum}}).encode()


def index_lines(*entries: tuple[str, str]) -> bytes:
    """A sparse-index file listing `(version, checksum)` entries, one JSON line each."""
    return "".join(
        json.dumps({"name": "fdu-core", "vers": vers, "cksum": cksum, "yanked": False}) + "\n"
        for vers, cksum in entries
    ).encode()


def pypi_record(files: dict[str, str]) -> bytes:
    """PyPI's JSON release record listing `files`."""
    urls = [{"filename": name, "digests": {"sha256": digest}} for name, digest in files.items()]
    return json.dumps({"urls": urls}).encode()


CORE = "a" * 64
CLI = "b" * 64
PYTHON = {f"fdu-{VERSION}.tar.gz": "c" * 64, f"fdu-{VERSION}-cp312-abi3-win_amd64.whl": "d" * 64}


class VerifyFilesTests(unittest.TestCase):
    """The publish job uploads only the set the evidence job recorded."""

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.files = root / "files"
        self.evidence = root / "evidence"
        self.files.mkdir()
        self.evidence.mkdir()
        write_release_set(self.files)
        self.manifest, self.checksums = record_evidence(self.files, self.evidence)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def verify(self) -> list[inspect_artifacts.Artifact]:
        return verify_files(self.files, self.manifest, self.checksums, VERSION)

    def test_the_recorded_set_verifies(self) -> None:
        self.assertEqual(len(self.verify()), 8)

    def test_bytes_changed_after_inspection_are_refused(self) -> None:
        write_crate(self.files, "fdu-core", b"different bytes")
        with self.assertRaisesRegex(ValueError, f"fdu-core-{VERSION}.crate: differs"):
            self.verify()

    def test_an_extra_file_is_refused(self) -> None:
        (self.files / "registry-state.json").write_text("{}", encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "outside the manifest: registry-state.json"):
            self.verify()

    def test_a_missing_file_is_refused(self) -> None:
        (self.files / f"fdu-{VERSION}.tar.gz").unlink()
        with self.assertRaisesRegex(ValueError, f"not downloaded: fdu-{VERSION}.tar.gz"):
            self.verify()

    def test_a_nested_directory_is_refused(self) -> None:
        wheel = next(self.files.glob("*.whl"))
        (self.files / "nested").mkdir()
        shutil.move(wheel, self.files / "nested" / wheel.name)
        with self.assertRaisesRegex(ValueError, "must contain only files"):
            self.verify()

    def test_checksums_that_disagree_with_the_manifest_are_refused(self) -> None:
        lines = self.checksums.read_text(encoding="utf-8").splitlines()
        lines[0] = "0" * 64 + lines[0][64:]
        self.checksums.write_text("\n".join(lines) + "\n", encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "SHA256SUMS and the manifest disagree"):
            self.verify()

    def test_a_manifest_for_another_version_is_refused(self) -> None:
        with self.assertRaisesRegex(ValueError, "not for version 0.1.1"):
            verify_files(self.files, self.manifest, self.checksums, "0.1.1")


class CompareCratesTests(unittest.TestCase):
    """A reproduced crate must be byte-identical to the rehearsed one."""

    def test_match_and_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            core = write_crate(directory, "fdu-core")
            digest = inspect_artifacts.digest(core)
            manifest = write_manifest(directory, {"fdu-core": digest, "fdu": CLI}, PYTHON)
            self.assertEqual(
                compare_crates(directory, manifest, VERSION, ["fdu-core"]),
                {f"fdu-core-{VERSION}.crate": digest},
            )
            write_crate(directory, "fdu-core", b"repackaged differently")
            with self.assertRaisesRegex(ValueError, f"but the rehearsal recorded {digest}"):
                compare_crates(directory, manifest, VERSION, ["fdu-core"])
            with self.assertRaisesRegex(ValueError, "was not packaged"):
                compare_crates(directory, manifest, VERSION, ["fdu"])


class CratesIoTests(unittest.TestCase):
    """crates.io is skipped only when it holds exactly the rehearsed digest."""

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.manifest = write_manifest(
            Path(self.temporary.name), {"fdu-core": CORE, "fdu": CLI}, PYTHON
        )

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_index_paths_follow_the_crates_io_layout(self) -> None:
        self.assertEqual(index_path("a"), "1/a")
        self.assertEqual(index_path("ab"), "2/ab")
        self.assertEqual(index_path("fdu"), "3/f/fdu")
        self.assertEqual(index_path("fdu-core"), "fd/u-/fdu-core")
        self.assertEqual(index_path("Serde"), "se/rd/serde")

    def test_the_index_checksum_is_read_for_exactly_the_version(self) -> None:
        url = f"{INDEX}/fd/u-/fdu-core"
        answers: dict[str, bytes | None] = {
            url: index_lines(("0.0.9", "e" * 64), (VERSION, CORE)),
        }
        self.assertEqual(index_checksum("fdu-core", VERSION, answers.__getitem__), CORE)
        answers[url] = index_lines(("0.0.9", "e" * 64))
        self.assertIsNone(index_checksum("fdu-core", VERSION, answers.__getitem__))
        answers[url] = None
        self.assertIsNone(index_checksum("fdu-core", VERSION, answers.__getitem__))
        answers[url] = index_lines((VERSION, CORE), (VERSION, CORE))
        with self.assertRaisesRegex(ValueError, "more than once"):
            index_checksum("fdu-core", VERSION, answers.__getitem__)
        answers[url] = index_lines((VERSION, "not-a-digest"))
        with self.assertRaisesRegex(ValueError, "no SHA-256 checksum"):
            index_checksum("fdu-core", VERSION, answers.__getitem__)
        answers[url] = b"<html>proxy page</html>\n"
        with self.assertRaisesRegex(RegistryError, "not JSON"):
            index_checksum("fdu-core", VERSION, answers.__getitem__)

    def registry(
        self,
        *,
        core_api: str | None = None,
        core_index: str | None = None,
        cli_api: str | None = None,
        cli_index: str | None = None,
    ) -> dict[str, bytes | None]:
        """Registry answers for both crates, `None` standing for a 404."""
        return {
            f"{CRATES_IO}/fdu-core/{VERSION}": core_api and version_record(core_api),
            f"{INDEX}/fd/u-/fdu-core": core_index and index_lines((VERSION, core_index)),
            f"{CRATES_IO}/fdu/{VERSION}": cli_api and version_record(cli_api),
            f"{INDEX}/3/f/fdu": cli_index and index_lines((VERSION, cli_index)),
        }

    def test_states_before_during_and_after_publication(self) -> None:
        cases = [
            (self.registry(), {"fdu-core": "missing", "fdu": "missing"}),
            (
                self.registry(core_api=CORE, core_index=CORE),
                {"fdu-core": "identical", "fdu": "missing"},
            ),
            # The upload landed and the index lists it, but the API record trails it.
            (self.registry(core_index=CORE), {"fdu-core": "identical", "fdu": "missing"}),
        ]
        for answers, expected in cases:
            with self.subTest(expected=expected):
                self.assertEqual(
                    crate_states(self.manifest, VERSION, answers.__getitem__), expected
                )

    def test_a_different_published_digest_is_a_conflict(self) -> None:
        for answers in (
            self.registry(core_api="f" * 64),
            self.registry(core_index="f" * 64),
            self.registry(core_api=CORE, cli_api="f" * 64),
        ):
            with self.subTest(answers=answers), self.assertRaises(Conflict):
                crate_states(self.manifest, VERSION, answers.__getitem__)

    def test_a_crate_is_served_only_when_the_api_and_the_index_both_carry_it(self) -> None:
        served = self.registry(core_api=CORE, core_index=CORE)
        self.assertTrue(crate_served(self.manifest, VERSION, "fdu-core", served.__getitem__))
        for partial in (self.registry(core_api=CORE), self.registry(core_index=CORE)):
            with self.subTest(partial=partial):
                self.assertFalse(
                    crate_served(self.manifest, VERSION, "fdu-core", partial.__getitem__)
                )
        for wrong in (
            self.registry(core_api="f" * 64, core_index=CORE),
            self.registry(core_api=CORE, core_index="f" * 64),
        ):
            with self.subTest(wrong=wrong), self.assertRaises(Conflict):
                crate_served(self.manifest, VERSION, "fdu-core", wrong.__getitem__)


class PypiTests(unittest.TestCase):
    """PyPI receives only what it lacks, and never a file it holds differently."""

    def test_verdicts(self) -> None:
        sdist, wheel = PYTHON
        self.assertEqual(pypi_verdict(PYTHON, None), "missing")
        self.assertEqual(pypi_verdict(PYTHON, {sdist: PYTHON[sdist]}), "partial")
        self.assertEqual(pypi_verdict(PYTHON, dict(PYTHON)), "identical")
        with self.assertRaisesRegex(Conflict, f"hash mismatch: {wheel}"):
            pypi_verdict(PYTHON, {**PYTHON, wheel: "0" * 64})
        with self.assertRaisesRegex(Conflict, "unexpected: fdu-0.1.0-py3-none-any.whl"):
            pypi_verdict(PYTHON, {**PYTHON, "fdu-0.1.0-py3-none-any.whl": "0" * 64})

    def test_a_pypi_conflict_stops_the_audit_before_crates_io_is_written(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            manifest = write_manifest(Path(temporary), {"fdu-core": CORE, "fdu": CLI}, PYTHON)
            sdist = next(iter(PYTHON))
            answers: dict[str, bytes | None] = {
                f"{CRATES_IO}/fdu-core/{VERSION}": None,
                f"{INDEX}/fd/u-/fdu-core": None,
                f"{CRATES_IO}/fdu/{VERSION}": None,
                f"{INDEX}/3/f/fdu": None,
                PYPI: None,
            }
            self.assertEqual(
                audit(manifest, VERSION, answers.__getitem__),
                {
                    "crates": "true",
                    "fdu": "missing",
                    "fdu_core": "missing",
                    "pypi": "missing",
                    "upload": "true",
                },
            )
            answers[PYPI] = pypi_record({**PYTHON, sdist: "0" * 64})
            with self.assertRaises(Conflict):
                audit(manifest, VERSION, answers.__getitem__)
            answers[PYPI] = pypi_record(dict(PYTHON))
            self.assertEqual(audit(manifest, VERSION, answers.__getitem__)["upload"], "false")


class WaitTests(unittest.TestCase):
    """Waiting is bounded, retries unreadable registries, and stops on a conflict."""

    def setUp(self) -> None:
        self.now = 0.0
        self.sleeps: list[float] = []

    def clock(self) -> float:
        return self.now

    def sleep(self, seconds: float) -> None:
        self.sleeps.append(seconds)
        self.now += seconds

    def wait(self, answers: list[object], timeout: float = 60) -> int:
        def probe() -> bool:
            answer = answers.pop(0)
            if isinstance(answer, Exception):
                raise answer
            return bool(answer)

        with redirect_stderr(io.StringIO()):
            return wait_until(
                probe,
                what="fixture",
                timeout=timeout,
                interval=10,
                clock=self.clock,
                sleep=self.sleep,
            )

    def test_a_lagging_or_unreadable_registry_is_polled_again(self) -> None:
        self.assertEqual(self.wait([False, RegistryError("503"), True]), 3)
        self.assertEqual(self.sleeps, [10, 10])

    def test_a_conflict_is_final(self) -> None:
        with self.assertRaises(Conflict):
            self.wait([False, Conflict("different bytes"), True])

    def test_the_wait_is_bounded(self) -> None:
        with self.assertRaisesRegex(WaitTimeout, "last read failed: 503"):
            self.wait([False] * 6 + [RegistryError("503")], timeout=60)


class EnvironmentTests(unittest.TestCase):
    """The publish job never names an environment that is missing or unprotected."""

    BASE = "https://api.github.com/repos/jlevy/fdu/environments/release"

    def answers(self, **overrides: object) -> dict[str, object]:
        record: dict[str, object] = {
            "name": "release",
            "can_admins_bypass": False,
            "protection_rules": [
                {"type": "required_reviewers", "reviewers": [{"type": "User"}]},
                {"type": "branch_policy"},
            ],
            "deployment_branch_policy": {
                "protected_branches": False,
                "custom_branch_policies": True,
            },
        }
        record.update(overrides)
        policies = {"branch_policies": [{"name": "v*", "type": "tag"}]}
        return {self.BASE: record, f"{self.BASE}/deployment-branch-policies": policies}

    def check(self, answers: dict[str, object]) -> list[str]:
        return check_environment("jlevy/fdu", "release", answers.__getitem__)

    def test_a_protected_environment_passes(self) -> None:
        self.assertIn("deployments only from tags v*", self.check(self.answers()))

    def test_each_missing_protection_is_refused(self) -> None:
        missing = self.answers()
        missing[self.BASE] = None
        branch = self.answers()
        branch[f"{self.BASE}/deployment-branch-policies"] = {
            "branch_policies": [{"name": "v*", "type": "tag"}, {"name": "main", "type": "branch"}]
        }
        cases = {
            "does not exist": missing,
            "no required reviewer": self.answers(protection_rules=[{"type": "branch_policy"}]),
            "bypass": self.answers(can_admins_bypass=True),
            "only from selected tags": self.answers(deployment_branch_policy=None),
            "admits branch 'main'": branch,
        }
        for message, answers in cases.items():
            with self.subTest(message=message), self.assertRaisesRegex(ValueError, message):
                self.check(answers)

    def test_a_refused_token_falls_back_to_the_public_record(self) -> None:
        url = self.BASE
        refused = HTTPError(url, 403, "Forbidden", Message(), None)

        class Response(io.BytesIO):
            def __enter__(self) -> Response:
                return self

        calls: list[dict[str, str]] = []

        def opener(request: Request, timeout: float) -> Response:
            headers = dict(request.header_items())
            calls.append(headers)
            if "Authorization" in headers:
                raise refused
            return Response(b'{"name": "release"}')

        with patch.object(publish_gate, "urlopen", side_effect=opener):
            self.assertEqual(publish_gate.github_get(url, "secret-token"), {"name": "release"})
        self.assertEqual(["Authorization" in call for call in calls], [True, False])
        missing = HTTPError(url, 404, "Not Found", Message(), None)
        with patch.object(publish_gate, "urlopen", side_effect=missing):
            self.assertIsNone(publish_gate.github_get(url, None))


class MainTests(unittest.TestCase):
    """A conflict and a timeout exit with statuses a log reader can tell apart."""

    def test_exit_statuses(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            manifest = write_manifest(Path(temporary), {"fdu-core": CORE, "fdu": CLI}, PYTHON)
            conflicting = {
                f"{CRATES_IO}/fdu-core/{VERSION}": version_record("f" * 64),
                f"{INDEX}/fd/u-/fdu-core": None,
            }
            arguments = ["--manifest", str(manifest), "--version", VERSION]
            with (
                redirect_stderr(io.StringIO()) as stderr,
                self.assertRaises(SystemExit) as raised,
            ):
                publish_gate.main(
                    ["wait-crate", *arguments, "--package", "fdu-core"],
                    fetch=conflicting.__getitem__,
                )
            self.assertEqual(raised.exception.code, 2)
            self.assertIn("conflict:", stderr.getvalue())
            with (
                redirect_stderr(io.StringIO()),
                self.assertRaises(SystemExit) as raised,
            ):
                publish_gate.main(
                    ["wait-pypi", *arguments, "--timeout", "0", "--interval", "0"],
                    fetch=lambda _url: None,
                )
            self.assertEqual(raised.exception.code, 3)
            with redirect_stdout(io.StringIO()) as stdout:
                publish_gate.main(
                    ["wait-pypi", *arguments], fetch=lambda _url: pypi_record(dict(PYTHON))
                )
            self.assertIn("serves exactly the rehearsed", stdout.getvalue())


if __name__ == "__main__":
    unittest.main()
