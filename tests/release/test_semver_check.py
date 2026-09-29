"""Tests for choosing and running the Rust API compatibility check."""

from __future__ import annotations

import io
import json
import tempfile
import unittest
from collections.abc import Sequence
from contextlib import redirect_stdout
from pathlib import Path

from scripts.release import semver_check
from scripts.release.semver_check import baseline, check, commands, published_versions

ROOT = Path(__file__).resolve().parents[2]
CORE_INDEX = "https://index.crates.io/fd/u-/fdu-core"
FDU_INDEX = "https://index.crates.io/3/f/fdu"


def index(*versions: tuple[str, bool]) -> bytes:
    """A sparse-index body listing each version with its yanked flag."""
    lines = [
        json.dumps({"name": "x", "vers": v, "yanked": y, "cksum": "0" * 64}) for v, y in versions
    ]
    return ("\n".join(lines) + "\n").encode()


class BaselineTests(unittest.TestCase):
    """Which published release a version must stay compatible with."""

    def test_a_patch_is_held_to_the_previous_release_in_its_series(self) -> None:
        published = [("0.1.0", False), ("0.2.0", False)]
        self.assertEqual(baseline("0.2.1", published), "0.2.0")
        # Before the bump the version is its own baseline: could this ship as a patch?
        self.assertEqual(baseline("0.2.0", published), "0.2.0")

    def test_a_new_series_has_nothing_to_stay_compatible_with(self) -> None:
        self.assertIsNone(baseline("0.3.0", [("0.1.0", False), ("0.2.5", False)]))
        self.assertIsNone(baseline("0.2.0", [("0.1.0", False)]))
        self.assertIsNone(baseline("0.2.0", []))
        # 0.0.z releases are compatible only with themselves.
        self.assertIsNone(baseline("0.0.3", [("0.0.2", False)]))
        self.assertEqual(baseline("0.0.3", [("0.0.3", False)]), "0.0.3")

    def test_after_1_0_a_minor_release_is_compatible_too(self) -> None:
        published = [("0.9.0", False), ("1.2.3", False), ("2.0.0", False)]
        self.assertEqual(baseline("1.4.0", published), "1.2.3")

    def test_versions_compare_as_numbers_not_strings(self) -> None:
        published = [("0.2.9", False), ("0.2.10", False)]
        self.assertEqual(baseline("0.2.11", published), "0.2.10")
        self.assertEqual(baseline("0.2.9", published), "0.2.9")

    def test_yanked_later_and_pre_release_versions_are_passed_over(self) -> None:
        published = [
            ("0.2.0", False),
            ("0.2.1", True),
            ("0.2.2-rc.1", False),
            ("0.2.5", False),
        ]
        self.assertEqual(baseline("0.2.2", published), "0.2.0")

    def test_the_current_version_must_be_a_plain_release(self) -> None:
        for version in ("0.2.1-rc.1", "0.2", "v0.2.1", "00.2.1"):
            with self.subTest(version=version), self.assertRaisesRegex(ValueError, "plain"):
                baseline(version, [])


class IndexTests(unittest.TestCase):
    """The published versions come from the index Cargo resolves against."""

    def test_versions_and_yanked_flags_are_read(self) -> None:
        body = index(("0.1.0", False), ("0.2.0", True))
        self.assertEqual(
            published_versions("fdu-core", {CORE_INDEX: body}.get),
            [("0.1.0", False), ("0.2.0", True)],
        )

    def test_an_unpublished_crate_has_no_versions(self) -> None:
        self.assertEqual(published_versions("fdu", lambda _url: None), [])

    def test_an_entry_without_a_version_is_refused(self) -> None:
        with self.assertRaisesRegex(ValueError, "without a version"):
            published_versions("fdu", lambda _url: b'{"name": "fdu"}\n')


class CheckTests(unittest.TestCase):
    """Running the pinned tool once per crate and feature set, and only when it applies."""

    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        policy = {
            "bootstrap": {"cargoTools": [{"name": "cargo-semver-checks", "version": "9.9.9"}]}
        }
        (self.root / "supply-chain-policy.json").write_text(json.dumps(policy), encoding="utf-8")
        self.ran: list[list[str]] = []
        self.status = 0
        self.tool_reads = 0

    def version(self, version: str) -> None:
        for manifest in semver_check.MANIFESTS.values():
            path = self.root / manifest
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(f'[package]\nname = "x"\nversion = "{version}"\n', encoding="utf-8")

    def run_tool(self, argv: Sequence[str]) -> int:
        self.ran.append(list(argv))
        return self.status

    def check(self, installed: str | None = "9.9.9", **fetched: bytes) -> list[str]:
        answers = {CORE_INDEX: fetched.get("core"), FDU_INDEX: fetched.get("fdu")}

        def tool_version() -> str | None:
            self.tool_reads += 1
            return installed

        with redirect_stdout(io.StringIO()):
            return check(self.root, ("fdu-core", "fdu"), answers.get, self.run_tool, tool_version)

    def test_a_patch_release_runs_both_feature_sets_for_both_crates(self) -> None:
        self.version("0.2.1")
        published = index(("0.1.0", False), ("0.2.0", False))
        self.assertEqual(self.check(core=published, fdu=published), [])
        expected = [argv for _, argv in commands("fdu-core", "0.2.0") + commands("fdu", "0.2.0")]
        self.assertEqual(self.ran, expected)
        self.assertEqual(
            [argv[-1] for argv in self.ran],
            ["--only-explicit-features", "--all-features"] * 2,
        )
        self.assertEqual(
            self.ran[0][:7],
            [
                "cargo",
                "semver-checks",
                "check-release",
                "--package",
                "fdu-core",
                "--baseline-version",
                "0.2.0",
            ],
        )

    def test_an_incompatible_change_fails_naming_the_crate_and_feature_set(self) -> None:
        self.version("0.2.1")
        published = index(("0.2.0", False))
        self.status = 1
        failures = self.check(core=published, fdu=published)
        self.assertIn(
            "fdu-core 0.2.1 against 0.2.0, no build features: cargo-semver-checks exited 1",
            failures,
        )
        self.assertEqual(len(failures), 4)

    def test_a_new_series_checks_nothing_and_needs_no_tool(self) -> None:
        self.version("0.3.0")
        published = index(("0.2.0", False))
        self.assertEqual(self.check(installed=None, core=published, fdu=published), [])
        self.assertEqual((self.ran, self.tool_reads), ([], 0))

    def test_a_missing_or_different_tool_is_refused_with_the_reviewed_install(self) -> None:
        self.version("0.2.1")
        published = index(("0.2.0", False))
        for installed, found in ((None, "is not installed"), ("9.9.8", "is 9.9.8")):
            with (
                self.subTest(installed=installed),
                self.assertRaisesRegex(
                    ValueError,
                    f"cargo-semver-checks {found}, not the reviewed 9.9.9: "
                    "cargo install --locked cargo-semver-checks --version 9.9.9",
                ),
            ):
                self.check(installed=installed, core=published, fdu=published)
        self.assertEqual(self.ran, [])


class PinTests(unittest.TestCase):
    """The script reads the one version the supply-chain policy inventories."""

    def test_the_repository_policy_pins_exactly_one_version(self) -> None:
        policy = json.loads((ROOT / "supply-chain-policy.json").read_text(encoding="utf-8"))
        (tool,) = policy["bootstrap"]["cargoTools"]
        self.assertEqual(semver_check.pinned_tool_version(ROOT), tool["version"])

    def test_an_ambiguous_policy_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            tools = [{"name": "cargo-semver-checks", "version": v} for v in ("1.0.0", "1.0.1")]
            (root / "supply-chain-policy.json").write_text(
                json.dumps({"bootstrap": {"cargoTools": tools}}), encoding="utf-8"
            )
            with self.assertRaisesRegex(ValueError, "exactly one"):
                semver_check.pinned_tool_version(root)


if __name__ == "__main__":
    unittest.main()
