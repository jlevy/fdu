"""Repository-level release metadata invariants."""

from __future__ import annotations

import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

SHARED_STAMP_BEGIN = "// BEGIN shared version stamp"
SHARED_STAMP_END = "// END shared version stamp."


def shared_version_stamp(build_script: Path) -> str:
    """The duplicated version-stamping region of one crate's build script."""
    text = build_script.read_text(encoding="utf-8")
    begin = text.index(SHARED_STAMP_BEGIN)
    end = text.index(SHARED_STAMP_END, begin)
    return text[begin:end]


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
        # The engine moves with the command line. `fdu` cannot be published before
        # `fdu-core` exists at the version it asks for, so a divergence here is a release
        # that cannot be published rather than one that publishes something wrong.
        core = tomllib.loads((ROOT / "crates/fdu-core/Cargo.toml").read_text(encoding="utf-8"))
        self.assertEqual(core["package"]["version"], version)
        self.assertEqual(workspace["workspace"]["dependencies"]["fdu-core"]["version"], version)
        self.assertEqual(pyproject["project"]["name"], "fdu")
        self.assertEqual(pyproject["project"]["scripts"]["fdu"], "fdu:_main")
        self.assertEqual(pyproject["tool"]["maturin"]["module-name"], "fdu._native")

    def test_artifact_license_copies_match_repository_license(self) -> None:
        expected = (ROOT / "LICENSE").read_bytes()
        self.assertEqual((ROOT / "crates/fdu-core/LICENSE").read_bytes(), expected)
        self.assertEqual((ROOT / "crates/fdu/LICENSE").read_bytes(), expected)
        self.assertEqual((ROOT / "crates/fdu-py/LICENSE").read_bytes(), expected)

    def test_both_build_scripts_stamp_the_version_the_same_way(self) -> None:
        """The two crates carry one version-stamping rule in two copies.

        A cross-crate `include!` cannot be used: the file would sit outside the
        including crate's package and would not reach the published `.crate`. So the
        copies are checked instead. If they drift, `fdu --version` and the performance
        probe's `--version` can describe the same checkout differently, and the perf
        harness matches the probe's git revision against the source it measured.
        """
        self.assertEqual(
            shared_version_stamp(ROOT / "crates/fdu/build.rs"),
            shared_version_stamp(ROOT / "crates/fdu-core/build.rs"),
        )

    def test_rehearsal_workflow_has_no_publication_authority(self) -> None:
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        self.assertNotIn("id-token: write", workflow)
        self.assertNotIn("contents: write", workflow)
        self.assertNotIn("cargo publish", workflow)
        self.assertNotIn("gh-action-pypi-publish", workflow)


if __name__ == "__main__":
    unittest.main()
