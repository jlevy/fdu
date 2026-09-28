#!/usr/bin/env python3
"""
Resolve and validate one fdu release or rehearsal identity.

A rehearsal needs only a version and a commit. A release also needs proof that the run is
publishing reviewed history under the maintainer's signature, so in release mode
`--validate-checkout` requires, besides a clean checkout of the planned commit:

- locally, that `refs/tags/v{version}` is an annotated tag object naming HEAD, since
  `git tag --points-at` matches a lightweight tag as well;
- on GitHub, that origin holds that same tag object, that GitHub reports its signature
  verified, and that the commit is an ancestor of origin's `main`.

The GitHub reads use `GITHUB_TOKEN` when it is set and fall back to the public record.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tomllib
from collections.abc import Callable
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

if __package__ in (None, ""):
    # Run as a script: make the repository root importable, as the tests have it.
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from scripts.release.publish_gate import github_get

REPOSITORY = "jlevy/fdu"
MAIN = "main"

Read = Callable[[str], Any]


@dataclass(frozen=True, slots=True)
class ReleasePlan:
    """Immutable release identity consumed by later build jobs."""

    mode: str
    version: str
    release_tag: str
    artifact_tag: str
    commit: str
    publish: bool


def cargo_version(manifest: Path) -> str:
    """Read the publishable fdu package version from its Cargo manifest."""
    value = tomllib.loads(manifest.read_text(encoding="utf-8"))["package"]["version"]
    if not isinstance(value, str) or not value:
        raise ValueError("Cargo package version must be a non-empty string")
    return value


def resolve(version: str, mode: str, ref: str, commit: str) -> ReleasePlan:
    """Resolve a plan, rejecting identities that could build the wrong version."""
    if mode not in {"rehearsal", "release"}:
        raise ValueError("mode must be rehearsal or release")
    if len(commit) < 7 or any(character not in "0123456789abcdef" for character in commit.lower()):
        raise ValueError("commit must be a hexadecimal Git object ID")
    release_tag = f"v{version}"
    if mode == "release" and ref != f"refs/tags/{release_tag}":
        raise ValueError(f"release ref must be refs/tags/{release_tag}, got {ref!r}")
    artifact_tag = release_tag if mode == "release" else f"rehearsal-{commit[:9]}"
    return ReleasePlan(mode, version, release_tag, artifact_tag, commit, mode == "release")


def git_output(root: Path, *args: str) -> str:
    """Run one read-only Git query."""
    return subprocess.run(
        ["git", *args],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def validate_checkout(root: Path, plan: ReleasePlan) -> str | None:
    """
    Prove the checkout is clean and identifies the planned commit and tag.

    In release mode, returns the local tag's object ID for the GitHub checks to compare
    with origin's.
    """
    head = git_output(root, "rev-parse", "HEAD")
    if head != plan.commit:
        raise ValueError(f"checkout HEAD {head} does not match planned commit {plan.commit}")
    if git_output(root, "status", "--porcelain"):
        raise ValueError("release checkout must be clean")
    if not plan.publish:
        return None
    tags = git_output(root, "tag", "--points-at", "HEAD").splitlines()
    if plan.release_tag not in tags:
        raise ValueError(f"release tag {plan.release_tag} does not identify HEAD")
    ref = f"refs/tags/{plan.release_tag}"
    tag_object = git_output(root, "rev-parse", "--verify", ref)
    kind = git_output(root, "cat-file", "-t", tag_object)
    if kind != "tag":
        raise ValueError(
            f"release tag {plan.release_tag} is a {kind} object; a release needs the "
            "annotated, signed tag"
        )
    target = git_output(root, "rev-parse", "--verify", f"{ref}^{{commit}}")
    if target != head:
        raise ValueError(f"release tag {plan.release_tag} names {target}, not HEAD {head}")
    return tag_object


def field(record: Any, name: str) -> dict[str, Any]:
    """One nested object of a GitHub record, or an empty one when it is absent."""
    value = record.get(name) if isinstance(record, dict) else None
    return value if isinstance(value, dict) else {}


def validate_published_tag(
    plan: ReleasePlan,
    repository: str,
    tag_object: str,
    read: Read,
) -> list[str]:
    """
    Prove origin's release tag is the checkout's, signed and verified, on reviewed history.

    `read` returns one GitHub REST resource as JSON, or None for a 404. Every answer is
    required to be present and to say so explicitly: a record that is missing a field is
    not evidence the field holds. Returns what was verified.
    """
    base = f"https://api.github.com/repos/{repository}"
    ref = read(f"{base}/git/ref/tags/{plan.release_tag}")
    if not isinstance(ref, dict) or ref.get("ref") != f"refs/tags/{plan.release_tag}":
        raise ValueError(f"{repository} has no tag {plan.release_tag}")
    pointer = field(ref, "object")
    if pointer.get("type") != "tag":
        raise ValueError(
            f"{repository}'s {plan.release_tag} is a {pointer.get('type')} ref, not an "
            "annotated tag"
        )
    if pointer.get("sha") != tag_object:
        raise ValueError(
            f"{repository}'s {plan.release_tag} is tag object {pointer.get('sha')}, but the "
            f"checkout holds {tag_object}"
        )

    record = read(f"{base}/git/tags/{tag_object}")
    if not isinstance(record, dict):
        raise ValueError(f"{repository} has no tag object {tag_object}")
    if record.get("tag") != plan.release_tag:
        raise ValueError(f"tag object {tag_object} is named {record.get('tag')!r}")
    target = field(record, "object")
    if target.get("type") != "commit" or target.get("sha") != plan.commit:
        raise ValueError(
            f"{plan.release_tag} names {target.get('type')} {target.get('sha')}, not the "
            f"planned commit {plan.commit}"
        )
    verification = field(record, "verification")
    if verification.get("verified") is not True:
        raise ValueError(
            f"GitHub does not report {plan.release_tag}'s signature verified "
            f"(reason: {verification.get('reason')})"
        )

    # BASE...HEAD with the release commit as base: `ahead` or `identical`, and a merge
    # base that is the commit itself, is what `git merge-base --is-ancestor` asks.
    comparison = read(f"{base}/compare/{plan.commit}...{MAIN}?per_page=1")
    if not isinstance(comparison, dict):
        raise ValueError(f"{repository} cannot compare {plan.commit} with {MAIN}")
    merge_base_sha = field(comparison, "merge_base_commit").get("sha")
    status = comparison.get("status")
    if status not in {"ahead", "identical"} or merge_base_sha != plan.commit:
        raise ValueError(
            f"{plan.commit} is not on {repository}'s {MAIN}: comparison status {status!r}, "
            f"merge base {merge_base_sha}"
        )
    return [
        f"{plan.release_tag} is annotated tag object {tag_object} on origin",
        f"GitHub reports its signature verified (reason: {verification.get('reason')})",
        f"{plan.commit} is on {MAIN} ({status})",
    ]


def parser() -> argparse.ArgumentParser:
    """Build the command-line parser."""
    result = argparse.ArgumentParser()
    result.add_argument("--root", type=Path, default=Path.cwd())
    result.add_argument("--mode", choices=("rehearsal", "release"), required=True)
    result.add_argument("--ref", default="")
    result.add_argument("--commit", required=True)
    result.add_argument("--validate-checkout", action="store_true")
    result.add_argument("--repository", default=REPOSITORY)
    result.add_argument("--github-output", type=Path)
    return result


def main() -> None:
    """Resolve the requested plan and write JSON plus optional Actions outputs."""
    args = parser().parse_args()
    root = args.root.resolve()
    version = cargo_version(root / "crates" / "fdu" / "Cargo.toml")
    plan = resolve(version, args.mode, args.ref, args.commit)
    if args.validate_checkout:
        tag_object = validate_checkout(root, plan)
        if tag_object is not None:
            token = os.environ.get("GITHUB_TOKEN") or None
            verified = validate_published_tag(
                plan, args.repository, tag_object, lambda url: github_get(url, token)
            )
            for line in verified:
                print(f"verified: {line}", file=sys.stderr)
    document = asdict(plan)
    print(json.dumps(document, sort_keys=True))
    if args.github_output is not None:
        with args.github_output.open("a", encoding="utf-8") as output:
            for name, value in document.items():
                rendered = str(value).lower() if isinstance(value, bool) else value
                output.write(f"{name}={rendered}\n")


if __name__ == "__main__":
    main()
