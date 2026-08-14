"""Tests for immutable registry-state classification."""

from __future__ import annotations

import json
from pathlib import Path
import tempfile
import unittest

from scripts.release.registry_state import classify_files, expected_artifacts


class RegistryStateTests(unittest.TestCase):
    """Retry decisions stop on any same-version artifact disagreement."""

    def test_missing_identical_and_conflict_are_distinct(self) -> None:
        expected = {"fdu-0.1.0.tar.gz": "a" * 64}
        missing = classify_files("pypi", "0.1.0", expected, None)
        identical = classify_files("pypi", "0.1.0", expected, expected.copy())
        conflict = classify_files(
            "pypi",
            "0.1.0",
            expected,
            {"fdu-0.1.0.tar.gz": "b" * 64},
        )
        self.assertEqual(missing.state, "missing")
        self.assertEqual(identical.state, "identical")
        self.assertEqual(conflict.state, "conflict")
        self.assertIn("hash mismatch", conflict.detail)

    def test_extra_or_incomplete_file_sets_conflict(self) -> None:
        expected = {"a.whl": "a" * 64, "b.whl": "b" * 64}
        state = classify_files(
            "pypi",
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


if __name__ == "__main__":
    unittest.main()
