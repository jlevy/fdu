"""Tests for release identity resolution."""

from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from typing import Any
from unittest import mock

from scripts.release.resolve_plan import (
    ReleasePlan,
    resolve,
    validate_checkout,
    validate_published_tag,
)


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


COMMIT = "c" * 40
TAG_OBJECT = "d" * 40
BASE = "https://api.github.com/repos/jlevy/fdu"
REF_URL = f"{BASE}/git/ref/tags/v0.2.1"
TAG_URL = f"{BASE}/git/tags/{TAG_OBJECT}"
COMPARE_URL = f"{BASE}/compare/{COMMIT}...main?per_page=1"


class PublishedTagTests(unittest.TestCase):
    """What GitHub must say about the tag and the commit before a release may build."""

    PLAN = resolve("0.2.1", "release", "refs/tags/v0.2.1", COMMIT)

    def answers(self) -> dict[str, Any]:
        return {
            REF_URL: {"ref": "refs/tags/v0.2.1", "object": {"type": "tag", "sha": TAG_OBJECT}},
            TAG_URL: {
                "tag": "v0.2.1",
                "object": {"type": "commit", "sha": COMMIT},
                "verification": {"verified": True, "reason": "valid"},
            },
            COMPARE_URL: {"status": "ahead", "merge_base_commit": {"sha": COMMIT}},
        }

    def check(self, answers: dict[str, Any], plan: ReleasePlan | None = None) -> list[str]:
        return validate_published_tag(
            plan or self.PLAN, "jlevy/fdu", TAG_OBJECT, lambda url: answers.get(url)
        )

    def test_a_signed_annotated_tag_on_main_passes(self) -> None:
        verified = self.check(self.answers())
        self.assertIn("GitHub reports its signature verified (reason: valid)", verified)
        identical = self.answers()
        identical[COMPARE_URL]["status"] = "identical"
        self.assertIn(f"{COMMIT} is on main (identical)", self.check(identical))

    def test_each_missing_proof_is_refused(self) -> None:
        def edited(url: str, value: Any = None, **fields: Any) -> dict[str, Any]:
            answers = self.answers()
            if fields:
                answers[url] = {**answers[url], **fields}
            else:
                answers[url] = value
            return answers

        unverified = self.answers()
        unverified[TAG_URL]["verification"] = {"verified": False, "reason": "unsigned"}
        silent = self.answers()
        del silent[TAG_URL]["verification"]
        truthy = self.answers()
        truthy[TAG_URL]["verification"] = {"verified": "true", "reason": "valid"}
        cases = {
            "has no tag v0.2.1": edited(REF_URL),
            # A lightweight tag is a ref straight to the commit, with no signature to check.
            "is a commit ref, not an annotated tag": edited(
                REF_URL, object={"type": "commit", "sha": COMMIT}
            ),
            # Origin's tag must be the object the checkout holds, not one pushed since.
            f"is tag object {'e' * 40}, but the checkout holds": edited(
                REF_URL, object={"type": "tag", "sha": "e" * 40}
            ),
            "has no tag object": edited(TAG_URL),
            "is named 'v0.2.0'": edited(TAG_URL, tag="v0.2.0"),
            f"names commit {'f' * 40}, not the planned commit": edited(
                TAG_URL, object={"type": "commit", "sha": "f" * 40}
            ),
            "names tree": edited(TAG_URL, object={"type": "tree", "sha": COMMIT}),
            r"signature verified \(reason: unsigned\)": unverified,
            r"signature verified \(reason: None\)": silent,
            # Only the JSON boolean counts: a string that reads as true is not a verdict.
            r"signature verified \(reason: valid\)": truthy,
            "cannot compare": edited(COMPARE_URL),
            "status 'behind'": edited(COMPARE_URL, status="behind"),
            "status 'diverged'": edited(
                COMPARE_URL, status="diverged", merge_base_commit={"sha": "a" * 40}
            ),
            # A status alone is not the ancestry: the merge base has to be the commit itself.
            f"merge base {'a' * 40}": edited(COMPARE_URL, merge_base_commit={"sha": "a" * 40}),
            "merge base None": edited(COMPARE_URL, merge_base_commit=None),
        }
        for message, answers in cases.items():
            with self.subTest(message=message), self.assertRaisesRegex(ValueError, message):
                self.check(answers)


def git(root: Path, *args: str) -> str:
    """Run git in a throwaway repository with a fixed identity and no signing."""
    identity = [
        "-c",
        "user.name=Release Test",
        "-c",
        "user.email=release@example.invalid",
        "-c",
        "commit.gpgsign=false",
        "-c",
        "tag.gpgsign=false",
    ]
    return subprocess.run(
        ["git", *identity, *args], cwd=root, check=True, capture_output=True, text=True
    ).stdout.strip()


class CheckoutTests(unittest.TestCase):
    """The local half: a clean checkout of the commit an annotated release tag names."""

    def setUp(self) -> None:
        # A suite run from a git hook inherits GIT_DIR, which would point every query at
        # the hook's repository instead of the throwaway one.
        scrubbed = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        patcher = mock.patch.dict(os.environ, scrubbed, clear=True)
        patcher.start()
        self.addCleanup(patcher.stop)
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        git(self.root, "init", "--quiet", "--initial-branch=main")
        (self.root / "README").write_text("fdu\n", encoding="utf-8")
        git(self.root, "add", "README")
        git(self.root, "commit", "--quiet", "-m", "first")
        self.first = git(self.root, "rev-parse", "HEAD")
        (self.root / "README").write_text("fdu 0.2.1\n", encoding="utf-8")
        git(self.root, "commit", "--quiet", "-am", "second")
        self.head = git(self.root, "rev-parse", "HEAD")

    def plan(self, mode: str = "release") -> ReleasePlan:
        return resolve("0.2.1", mode, "refs/tags/v0.2.1", self.head)

    def test_an_annotated_tag_on_head_passes_and_names_its_object(self) -> None:
        git(self.root, "tag", "-a", "v0.2.1", "-m", "fdu 0.2.1")
        tag_object = validate_checkout(self.root, self.plan())
        self.assertEqual(tag_object, git(self.root, "rev-parse", "refs/tags/v0.2.1"))
        self.assertNotEqual(tag_object, self.head)

    def test_a_rehearsal_needs_no_tag(self) -> None:
        self.assertIsNone(validate_checkout(self.root, self.plan("rehearsal")))

    def test_a_lightweight_tag_is_refused(self) -> None:
        # `git tag --points-at HEAD` lists it, which is all the check once asked.
        git(self.root, "tag", "v0.2.1")
        self.assertIn("v0.2.1", git(self.root, "tag", "--points-at", "HEAD").splitlines())
        with self.assertRaisesRegex(ValueError, "is a commit object; a release needs"):
            validate_checkout(self.root, self.plan())

    def test_a_tag_on_another_commit_is_refused(self) -> None:
        git(self.root, "tag", "-a", "v0.2.1", "-m", "fdu 0.2.1", self.first)
        with self.assertRaisesRegex(ValueError, "does not identify HEAD"):
            validate_checkout(self.root, self.plan())

    def test_a_dirty_or_different_checkout_is_refused(self) -> None:
        git(self.root, "tag", "-a", "v0.2.1", "-m", "fdu 0.2.1")
        with self.assertRaisesRegex(ValueError, "does not match planned commit"):
            validate_checkout(self.root, resolve("0.2.1", "release", "refs/tags/v0.2.1", "a" * 40))
        (self.root / "README").write_text("edited\n", encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "must be clean"):
            validate_checkout(self.root, self.plan())


if __name__ == "__main__":
    unittest.main()
