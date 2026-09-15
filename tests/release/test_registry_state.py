"""Tests for immutable registry-state classification."""

from __future__ import annotations

import hashlib
import json
import re
import tempfile
import unittest
from email.message import Message
from pathlib import Path
from unittest.mock import patch
from urllib.error import HTTPError, URLError

from scripts.release import inspect_artifacts, registry_state
from scripts.release.registry_state import (
    CRATE_PACKAGES,
    RegistryError,
    RegistryState,
    classify_files,
    crates_io_state,
    exit_status,
    expected_artifacts,
)

CRATES_IO = "https://crates.io/api/v1/crates"


class RegistryStateTests(unittest.TestCase):
    """Retry decisions stop on any same-version artifact disagreement."""

    def test_missing_identical_and_conflict_are_distinct(self) -> None:
        expected = {"fdu-0.1.0.tar.gz": "a" * 64}
        missing = classify_files("pypi", "fdu", "0.1.0", expected, None)
        identical = classify_files("pypi", "fdu", "0.1.0", expected, expected.copy())
        conflict = classify_files(
            "pypi",
            "fdu",
            "0.1.0",
            expected,
            {"fdu-0.1.0.tar.gz": "b" * 64},
        )
        self.assertEqual(missing.state, "missing")
        self.assertEqual(identical.state, "identical")
        self.assertEqual(conflict.state, "conflict")
        self.assertIn("hash mismatch", conflict.detail)

    def test_a_conflict_always_fails_and_an_unpublished_channel_fails_on_request(self) -> None:
        # Mid-publication `missing` is expected; an announcement must see every channel
        # identical, so a chained `gh release create` cannot follow a skipped upload.
        identical, missing, conflict = (
            RegistryState("crates.io", "fdu", "0.1.0", state, "")
            for state in ("identical", "missing", "conflict")
        )
        self.assertEqual(exit_status([identical, missing], require_identical=False), 0)
        self.assertEqual(exit_status([identical, missing], require_identical=True), 3)
        self.assertEqual(exit_status([identical, conflict], require_identical=False), 2)
        self.assertEqual(exit_status([missing, conflict], require_identical=True), 2)
        self.assertEqual(exit_status([identical, identical], require_identical=True), 0)

    def test_extra_or_incomplete_file_sets_conflict(self) -> None:
        expected = {"a.whl": "a" * 64, "b.whl": "b" * 64}
        state = classify_files(
            "pypi",
            "fdu",
            "0.1.0",
            expected,
            {"a.whl": "a" * 64, "unexpected.whl": "c" * 64},
        )
        self.assertEqual(state.state, "conflict")
        self.assertIn("missing: b.whl", state.detail)
        self.assertIn("unexpected: unexpected.whl", state.detail)

    def test_manifest_filters_by_artifact_kind(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / "manifest.json"
            manifest.write_text(
                json.dumps(
                    {
                        "artifacts": [
                            {"filename": "fdu.whl", "kind": "wheel", "sha256": "a" * 64},
                            {"filename": "fdu.crate", "kind": "crate", "sha256": "b" * 64},
                        ]
                    }
                ),
                encoding="utf-8",
            )
            self.assertEqual(expected_artifacts(manifest, "crate"), {"fdu.crate": "b" * 64})

    def test_every_published_crate_is_classified_by_its_version_record(self) -> None:
        # The stub answers only the version records, so reading any other endpoint fails.
        core = hashlib.sha256(b"fdu-core crate bytes").hexdigest()
        cli = hashlib.sha256(b"fdu crate bytes").hexdigest()
        with tempfile.TemporaryDirectory() as temporary:
            manifest = write_crate_manifest(Path(temporary), {"fdu-core": core, "fdu": cli})
            published = {
                f"{CRATES_IO}/fdu-core/0.1.0": version_record(checksum=core),
                f"{CRATES_IO}/fdu/0.1.0": None,
            }
            states = crates_io_state(manifest, "0.1.0", fetch=published.__getitem__)
        self.assertEqual(
            [(state.package, state.state) for state in states],
            [("fdu-core", "identical"), ("fdu", "missing")],
        )

    def test_a_different_published_checksum_is_a_conflict(self) -> None:
        core = hashlib.sha256(b"fdu-core crate bytes").hexdigest()
        cli = hashlib.sha256(b"fdu crate bytes").hexdigest()
        with tempfile.TemporaryDirectory() as temporary:
            manifest = write_crate_manifest(Path(temporary), {"fdu-core": core, "fdu": cli})
            published = {
                f"{CRATES_IO}/fdu-core/0.1.0": version_record(checksum="c" * 64),
                f"{CRATES_IO}/fdu/0.1.0": version_record(checksum=cli),
            }
            states = crates_io_state(manifest, "0.1.0", fetch=published.__getitem__)
        self.assertEqual(
            [(state.package, state.state, state.detail) for state in states],
            [
                ("fdu-core", "conflict", "hash mismatch: fdu-core-0.1.0.crate"),
                ("fdu", "identical", "all filenames and hashes match"),
            ],
        )

    def test_a_version_record_without_a_checksum_is_refused(self) -> None:
        # A 200 that is not a version record, such as the download endpoint's URL stub, must
        # stop the audit rather than be classified.
        with tempfile.TemporaryDirectory() as temporary:
            manifest = write_crate_manifest(
                Path(temporary), {"fdu-core": "a" * 64, "fdu": "b" * 64}
            )
            for body in (
                b'{"url": "https://static.crates.io/crates/fdu-core/fdu-core-0.1.0.crate"}',
                version_record(checksum=""),
            ):
                with (
                    self.subTest(body=body),
                    self.assertRaisesRegex(ValueError, "no SHA-256 checksum"),
                ):
                    crates_io_state(manifest, "0.1.0", fetch=lambda url, body=body: body)

    def test_only_a_404_reads_as_missing(self) -> None:
        # Reading a refusal, an outage, or an unreachable host as `missing` would report an
        # upload that landed as one that did not.
        url = f"{CRATES_IO}/fdu-core/0.1.0"
        failures: list[OSError] = [
            HTTPError(url, 403, "Forbidden", Message(), None),
            HTTPError(url, 503, "Service Unavailable", Message(), None),
            URLError("nodename nor servname provided"),
            TimeoutError("The read operation timed out"),
        ]
        for failure in failures:
            with (
                self.subTest(failure=failure),
                patch.object(registry_state, "urlopen", side_effect=failure),
                self.assertRaisesRegex(RegistryError, re.escape(url)),
            ):
                registry_state.get(url)
        absent = HTTPError(url, 404, "Not Found", Message(), None)
        with patch.object(registry_state, "urlopen", side_effect=absent) as opened:
            self.assertIsNone(registry_state.get(url))
        # crates.io refuses a request without a User-Agent with a 403.
        self.assertIn("fdu-release-audit", opened.call_args.args[0].get_header("User-agent"))

    def test_a_manifest_without_the_core_crate_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            manifest = write_crate_manifest(Path(temporary), {"fdu": "b" * 64})
            with self.assertRaisesRegex(ValueError, "no fdu-core-0.1.0.crate"):
                crates_io_state(manifest, "0.1.0", fetch=lambda url: None)

    def test_inspector_and_registry_audit_name_the_same_crates(self) -> None:
        self.assertEqual(CRATE_PACKAGES, inspect_artifacts.CRATE_PACKAGES)


def write_crate_manifest(directory: Path, digests: dict[str, str]) -> Path:
    """Write a manifest carrying one crate artifact per package."""
    manifest = directory / "manifest.json"
    artifacts = [
        {
            "filename": f"{package}-0.1.0.crate",
            "kind": "crate",
            "package": package,
            "sha256": digest,
        }
        for package, digest in digests.items()
    ]
    manifest.write_text(json.dumps({"artifacts": artifacts}), encoding="utf-8")
    return manifest


def version_record(*, checksum: str) -> bytes:
    """Render the fields of a crates.io version record the audit reads, as served."""
    return json.dumps({"version": {"num": "0.1.0", "checksum": checksum, "yanked": False}}).encode()


if __name__ == "__main__":
    unittest.main()
