"""Tests for release identity resolution."""

from __future__ import annotations

import unittest

from scripts.release.resolve_plan import resolve


class ResolvePlanTests(unittest.TestCase):
    """Release and rehearsal identity rules."""

    def test_rehearsal_never_publishes(self) -> None:
        plan = resolve("0.1.0", "rehearsal", "refs/heads/main", "a" * 40)
        self.assertFalse(plan.publish)
        self.assertEqual(plan.release_tag, "v0.1.0")
        self.assertEqual(plan.artifact_tag, "rehearsal-aaaaaaaaa")

    def test_release_requires_exact_version_tag(self) -> None:
        plan = resolve("0.1.0", "release", "refs/tags/v0.1.0", "b" * 40)
        self.assertTrue(plan.publish)
        with self.assertRaisesRegex(ValueError, "release ref"):
            resolve("0.1.0", "release", "refs/tags/v0.1.1", "b" * 40)

    def test_commit_must_be_hexadecimal(self) -> None:
        with self.assertRaisesRegex(ValueError, "hexadecimal"):
            resolve("0.1.0", "rehearsal", "", "not-a-commit")


if __name__ == "__main__":
    unittest.main()
