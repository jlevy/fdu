#!/usr/bin/env python3
"""Resolve and validate one fdu release or rehearsal identity."""

from __future__ import annotations

import argparse
import json
import subprocess
import tomllib
from dataclasses import asdict, dataclass
from pathlib import Path


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


def validate_checkout(root: Path, plan: ReleasePlan) -> None:
    """Prove the checkout is clean and identifies the planned commit and tag."""
    head = git_output(root, "rev-parse", "HEAD")
    if head != plan.commit:
        raise ValueError(f"checkout HEAD {head} does not match planned commit {plan.commit}")
    if git_output(root, "status", "--porcelain"):
        raise ValueError("release checkout must be clean")
    if plan.publish:
        tags = git_output(root, "tag", "--points-at", "HEAD").splitlines()
        if plan.release_tag not in tags:
            raise ValueError(f"release tag {plan.release_tag} does not identify HEAD")


def parser() -> argparse.ArgumentParser:
    """Build the command-line parser."""
    result = argparse.ArgumentParser()
    result.add_argument("--root", type=Path, default=Path.cwd())
    result.add_argument("--mode", choices=("rehearsal", "release"), required=True)
    result.add_argument("--ref", default="")
    result.add_argument("--commit", required=True)
    result.add_argument("--validate-checkout", action="store_true")
    result.add_argument("--github-output", type=Path)
    return result


def main() -> None:
    """Resolve the requested plan and write JSON plus optional Actions outputs."""
    args = parser().parse_args()
    root = args.root.resolve()
    version = cargo_version(root / "crates" / "fdu" / "Cargo.toml")
    plan = resolve(version, args.mode, args.ref, args.commit)
    if args.validate_checkout:
        validate_checkout(root, plan)
    document = asdict(plan)
    print(json.dumps(document, sort_keys=True))
    if args.github_output is not None:
        with args.github_output.open("a", encoding="utf-8") as output:
            for name, value in document.items():
                rendered = str(value).lower() if isinstance(value, bool) else value
                output.write(f"{name}={rendered}\n")


if __name__ == "__main__":
    main()
