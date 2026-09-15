"""Tests for immutable registry-state classification."""

from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from scripts.release import inspect_artifacts
from scripts.release.registry_state import (
    CRATE_PACKAGES,
    RegistryState,
    classify_files,
    crates_io_state,
    exit_status,
    expected_artifacts,
)


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

    def test_every_published_crate_is_classified_separately(self) -> None:
        core = b"fdu-core crate bytes"
        cli = b"fdu crate bytes"
        published = {
            "https://crates.io/api/v1/crates/fdu-core/0.1.0/download": core,
            "https://crates.io/api/v1/crates/fdu/0.1.0/download": None,
        }
        with tempfile.TemporaryDirectory() as temporary:
            manifest = write_crate_manifest(
                Path(temporary),
                {
                    "fdu-core": hashlib.sha256(core).hexdigest(),
                    "fdu": hashlib.sha256(cli).hexdigest(),
                },
            )
            states = crates_io_state(manifest, "0.1.0", fetch=published.__getitem__)
        self.assertEqual(
            [(state.package, state.state) for state in states],
            [("fdu-core", "identical"), ("fdu", "missing")],
        )

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


if __name__ == "__main__":
    unittest.main()
