"""Repository-level release metadata invariants."""

from __future__ import annotations

import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class MetadataTests(unittest.TestCase):
    """Version, naming, licensing, and rehearsal-authority checks."""

    def test_product_versions_and_names_have_one_identity(self) -> None:
        workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        crate = tomllib.loads((ROOT / "crates/fdu/Cargo.toml").read_text(encoding="utf-8"))
        python_crate = tomllib.loads(
            (ROOT / "crates/fdu-py/Cargo.toml").read_text(encoding="utf-8")
        )
        pyproject = tomllib.loads(
            (ROOT / "crates/fdu-py/pyproject.toml").read_text(encoding="utf-8")
        )
        version = crate["package"]["version"]
        self.assertEqual(version, "0.1.0")
        self.assertEqual(python_crate["package"]["version"], version)
        self.assertEqual(workspace["workspace"]["dependencies"]["fdu"]["version"], version)
        self.assertEqual(pyproject["project"]["name"], "fdu")
        self.assertEqual(pyproject["project"]["scripts"]["fdu"], "fdu:_main")
        self.assertEqual(pyproject["tool"]["maturin"]["module-name"], "fdu._native")

    def test_artifact_license_copies_match_repository_license(self) -> None:
        expected = (ROOT / "LICENSE").read_bytes()
        self.assertEqual((ROOT / "crates/fdu-core/LICENSE").read_bytes(), expected)
        self.assertEqual((ROOT / "crates/fdu/LICENSE").read_bytes(), expected)
        self.assertEqual((ROOT / "crates/fdu-py/LICENSE").read_bytes(), expected)

    def test_runbook_derives_the_release_body_from_the_checked_in_script(self) -> None:
        runbook = (ROOT / "docs/project/guides/release-process.md").read_text(encoding="utf-8")
        self.assertIn("scripts/release/release_body.py", runbook)
        self.assertNotIn('re.sub(r"<!--.*?-->', runbook)

    def test_rehearsal_workflow_has_no_publication_authority(self) -> None:
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        self.assertNotIn("id-token: write", workflow)
        self.assertNotIn("contents: write", workflow)
        self.assertNotIn("cargo publish", workflow)
        self.assertNotIn("gh-action-pypi-publish", workflow)

    def test_workflow_and_local_rehearsal_run_the_same_crate_smoke(self) -> None:
        # A plain `cargo install --locked` of the packaged `fdu` cannot resolve an
        # unpublished `fdu-core`; both paths must use the script that patches it (fdu-y5zc).
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        makefile = (ROOT / "Makefile").read_text(encoding="utf-8")
        rehearsal = makefile.split("\nrelease-rehearse:", 1)[1].split("\n\n", 1)[0]
        for text in (workflow, rehearsal):
            self.assertIn("scripts/release/smoke_crate.py", text)
        self.assertNotIn("cargo install", workflow)


if __name__ == "__main__":
    unittest.main()
