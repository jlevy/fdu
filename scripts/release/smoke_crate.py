#!/usr/bin/env python3
"""
Install the packaged `fdu` crate the way a user does, against the packaged `fdu-core`.

The packaged `fdu` pins `fdu-core` from crates.io, where it does not exist until the
first publish, so `cargo install --locked` on the extracted package cannot resolve it
before then (fdu-y5zc). A `[patch.crates-io]` entry resolves it to the packaged sibling
instead. That patch changes where `fdu-core` comes from, which `--locked` refuses, so the
lock is refreshed once under the patch and must differ from the shipped lock in that
source alone: every other pin is still the one a user of the published crate resolves.

The install runs inside a throwaway git checkout, which is what makes the version this
smoke asserts evidence of anything: see `init_checkout`.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tarfile
from collections.abc import Callable
from pathlib import Path
from typing import Any

CORE = "fdu-core"
CLI = "fdu"
CRATES_IO = "registry+https://github.com/rust-lang/crates.io-index"
GIT = "git"

# `tarfile`'s "data" extraction filter, which refuses the escaping paths and links an
# untrusted archive can carry, is standard from 3.12. The release workflow runs this on
# the runner's python3, so an older image fails here rather than inside extraction.
MINIMUM_PYTHON = (3, 12)

Runner = Callable[..., subprocess.CompletedProcess[str]]


def require(condition: bool, message: str) -> None:
    """Raise a stable validation error when the smoke contract is violated."""
    if not condition:
        raise ValueError(message)


def check_python(version: tuple[int, ...]) -> None:
    """Refuse an interpreter older than `MINIMUM_PYTHON`, naming the one running."""
    minimum = ".".join(map(str, MINIMUM_PYTHON))
    require(
        version[:2] >= MINIMUM_PYTHON,
        f"smoke_crate.py needs Python {minimum} or newer; this is "
        f"{'.'.join(map(str, version[:3]))} (run it with `uv run --python {minimum}`)",
    )


def run(runner: Runner, command: list[str], **options: Any) -> subprocess.CompletedProcess[str]:
    """Run one step, and stop the smoke on any nonzero status."""
    result = runner(command, text=True, **options)
    require(result.returncode == 0, f"{' '.join(command)} exited with status {result.returncode}")
    return result


def install_env() -> dict[str, str]:
    """
    The environment for the install and for every git call around it.

    `FDU_RELEASE_TAG` stamps the version outright, which would mask the derivation this
    smoke exercises. A `GIT_*` variable points git at some other repository than the
    throwaway checkout, in the build script as well as here; git exports `GIT_DIR` into
    hook environments, so a rehearsal run from a pre-push hook inherits one.
    """
    return {
        key: value
        for key, value in os.environ.items()
        if key != "FDU_RELEASE_TAG" and not key.startswith("GIT_")
    }


def git(arguments: list[str], environment: dict[str, str]) -> str:
    """
    Run one git command in the scrubbed environment and return its trimmed output.

    Not through the injected runner: a stand-in git would leave the build script with no
    revision to find, which is the whole point of the checkout below.
    """
    return run(
        subprocess.run, [GIT, *arguments], env=environment, stdout=subprocess.PIPE
    ).stdout.strip()


def head_revision(directory: Path, environment: dict[str, str]) -> str:
    """The revision the build script's git fallback derives from `directory`."""
    return git(["-C", str(directory), "rev-parse", "--short=9", "HEAD"], environment)


def init_checkout(root: Path, environment: dict[str, str]) -> str:
    """
    Make `root` a throwaway git repository holding one commit, and return its revision.

    `crates/fdu/build.rs` stamps the revision of the checkout it builds in unless
    `.cargo_vcs_info.json` beside the manifest tells it a registry unpacked the crate.
    Installed outside any repository that derivation reports bare semver by its own
    fallback, so the assertion below held with the skip deleted and pinned nothing
    (fdu-tleo). Installed inside a repository, bare semver can only come from the skip,
    which is also the shape of the case the skip protects: a consumer whose extracted
    crate sits under a git repository of their own.
    """
    root.mkdir(parents=True)
    git(["-c", "init.defaultBranch=main", "init", "--quiet", str(root)], environment)
    # A runner leaves `user.email` unset, and an ambient config may carry a signing key
    # or hooks, so the single commit supplies an identity and declines both.
    git(
        [
            "-C",
            str(root),
            "-c",
            "user.name=fdu release smoke",
            "-c",
            "user.email=smoke@fdu.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--allow-empty",
            "--no-verify",
            "--quiet",
            "--message",
            "packaged crate smoke",
        ],
        environment,
    )
    return head_revision(root, environment)


def extract_crate(crate: Path, destination: Path) -> Path:
    """Unpack one `.crate` archive and return its package root."""
    with tarfile.open(crate, "r:gz") as archive:
        archive.extractall(destination, filter="data")
    root = destination / crate.name.removesuffix(".crate")
    require((root / "Cargo.toml").is_file(), f"{crate.name}: no {root.name}/Cargo.toml")
    return root


def lock_packages(text: str) -> dict[tuple[str, str], dict[str, Any]]:
    """Index a Cargo.lock's packages by name and version."""
    # Imported here rather than at the top so that on a Python without `tomllib` (3.10
    # and older) `main` reaches `check_python` and says so, instead of an import error.
    import tomllib

    packages = tomllib.loads(text).get("package")
    require(isinstance(packages, list), "Cargo.lock has no package list")
    indexed = {(str(item["name"]), str(item["version"])): item for item in packages}
    require(len(indexed) == len(packages), "Cargo.lock names one package version twice")
    return indexed


def verify_relock(shipped: str, relocked: str, version: str) -> None:
    """
    Require that patching `fdu-core` to its sibling dropped that entry's registry source
    and checksum and changed nothing else.
    """
    before = lock_packages(shipped)
    after = lock_packages(relocked)
    core = (CORE, version)
    require(
        before.get(core, {}).get("source") == CRATES_IO,
        f"shipped lock must pin {CORE} to crates.io",
    )
    require(core in after, f"the relocked lock has no {CORE} {version}")
    require(
        "source" not in after[core],
        f"{CORE} is still resolved from crates.io: the patch did not apply",
    )
    unpinned = {
        key: value for key, value in before[core].items() if key not in {"source", "checksum"}
    }
    changed = sorted(
        key
        for key in unpinned.keys() | after[core].keys()
        if unpinned.get(key) != after[core].get(key)
    )
    require(
        not changed,
        f"{CORE} {version} differs from the shipped lock beyond its source: {', '.join(changed)}",
    )
    others = (before.keys() | after.keys()) - {core}
    moved = sorted(
        f"{name} {release}"
        for name, release in others
        if before.get((name, release)) != after.get((name, release))
    )
    require(not moved, f"relocking moved pins other than {CORE}: {', '.join(moved)}")


def smoke_crate(
    crates: Path,
    version: str,
    work_dir: Path,
    *,
    cargo: str = "cargo",
    runner: Runner = subprocess.run,
) -> None:
    """Extract both crates into a checkout, install `fdu` locked against `fdu-core`, run it."""
    work_dir.mkdir(parents=True, exist_ok=True)
    require(
        not any(work_dir.iterdir()), f"{work_dir} must be empty, so no earlier install stands in"
    )
    environment = install_env()
    checkout = work_dir / "checkout"
    revision = init_checkout(checkout, environment)
    core = extract_crate(crates / f"{CORE}-{version}.crate", checkout).resolve()
    cli = extract_crate(crates / f"{CLI}-{version}.crate", checkout)
    # The build script runs with the crate root as its working directory, so this is the
    # revision it would stamp. Resolving another one means the crate landed under some
    # enclosing repository instead, and the version below would be about that repository.
    require(
        head_revision(cli, environment) == revision,
        f"{cli} is not inside the throwaway checkout at {checkout}",
    )
    lock = cli / "Cargo.lock"
    shipped = lock.read_text(encoding="utf-8")
    patch = f"patch.crates-io.{CORE}.path={json.dumps(str(core))}"

    run(
        runner,
        [
            cargo,
            "metadata",
            "--format-version",
            "1",
            "--manifest-path",
            str(cli / "Cargo.toml"),
            "--config",
            patch,
        ],
        stdout=subprocess.DEVNULL,
    )
    verify_relock(shipped, lock.read_text(encoding="utf-8"), version)

    install_root = work_dir / "install"
    run(
        runner,
        [
            cargo,
            "install",
            "--locked",
            "--path",
            str(cli),
            "--root",
            str(install_root),
            "--config",
            patch,
        ],
        # No release tag and no ambient git redirection, so this install exercises the
        # `.cargo_vcs_info.json` skip (fdu-nwud) rather than the release-tag stamp.
        env=environment,
    )
    binary = str(install_root / "bin" / CLI)
    reported = run(runner, [binary, "--version"], stdout=subprocess.PIPE).stdout.strip()
    require(
        "-dev+g" not in reported,
        f"installed {CLI} stamped a git revision: {reported!r}; the packaged crate took "
        f"the revision of the checkout it built in ({revision}) rather than its own "
        f".cargo_vcs_info.json",
    )
    require(
        reported == f"{CLI} {version}",
        f"installed {CLI} reports {reported!r}, not '{CLI} {version}'",
    )
    listing = run(
        runner,
        [binary, "--cache", "off", "--color", "never", "--depth", "0", str(core)],
        stdout=subprocess.PIPE,
    ).stdout
    require(bool(listing.strip()), f"installed {CLI} printed nothing for {core}")


def parser() -> argparse.ArgumentParser:
    """Build the command-line parser."""
    result = argparse.ArgumentParser()
    result.add_argument("crates", type=Path, help="directory holding both .crate files")
    result.add_argument("--version", required=True)
    result.add_argument("--work-dir", type=Path, required=True)
    result.add_argument("--cargo", default="cargo", help="the cargo that packaged the crates")
    return result


def main() -> None:
    """Smoke-test the packaged crates without contacting a publishing API."""
    check_python(sys.version_info[:3])
    args = parser().parse_args()
    smoke_crate(args.crates, args.version, args.work_dir, cargo=args.cargo)


if __name__ == "__main__":
    main()
