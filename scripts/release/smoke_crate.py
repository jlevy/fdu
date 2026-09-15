#!/usr/bin/env python3
"""
Install the packaged `fdu` crate the way a user does, against the packaged `fdu-core`.

The packaged `fdu` pins `fdu-core` from crates.io, where it does not exist until the
first publish, so `cargo install --locked` on the extracted package cannot resolve it
before then (fdu-y5zc). A `[patch.crates-io]` entry resolves it to the packaged sibling
instead. That patch changes where `fdu-core` comes from, which `--locked` refuses, so the
lock is refreshed once under the patch and must differ from the shipped lock in that
source alone: every other pin is still the one a user of the published crate resolves.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import tarfile
import tomllib
from collections.abc import Callable
from pathlib import Path
from typing import Any

CORE = "fdu-core"
CLI = "fdu"
CRATES_IO = "registry+https://github.com/rust-lang/crates.io-index"

Runner = Callable[..., subprocess.CompletedProcess[str]]


def require(condition: bool, message: str) -> None:
    """Raise a stable validation error when the smoke contract is violated."""
    if not condition:
        raise ValueError(message)


def run(runner: Runner, command: list[str], **options: Any) -> subprocess.CompletedProcess[str]:
    """Run one step, and stop the smoke on any nonzero status."""
    result = runner(command, text=True, **options)
    require(result.returncode == 0, f"{' '.join(command)} exited with status {result.returncode}")
    return result


def extract_crate(crate: Path, destination: Path) -> Path:
    """Unpack one `.crate` archive and return its package root."""
    with tarfile.open(crate, "r:gz") as archive:
        archive.extractall(destination, filter="data")
    root = destination / crate.name.removesuffix(".crate")
    require((root / "Cargo.toml").is_file(), f"{crate.name}: no {root.name}/Cargo.toml")
    return root


def lock_packages(text: str) -> dict[tuple[str, str], dict[str, Any]]:
    """Index a Cargo.lock's packages by name and version."""
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
    unpinned = {
        key: value for key, value in before[core].items() if key not in {"source", "checksum"}
    }
    require(
        after.get(core) == unpinned,
        f"{CORE} is still resolved from crates.io: the patch did not apply",
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
    """Extract both crates, install `fdu` locked against `fdu-core`, and run it."""
    work_dir.mkdir(parents=True, exist_ok=True)
    require(
        not any(work_dir.iterdir()), f"{work_dir} must be empty, so no earlier install stands in"
    )
    core = extract_crate(crates / f"{CORE}-{version}.crate", work_dir).resolve()
    cli = extract_crate(crates / f"{CLI}-{version}.crate", work_dir)
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
        env={**os.environ, "FDU_RELEASE_TAG": f"v{version}"},
    )
    binary = str(install_root / "bin" / CLI)
    reported = run(runner, [binary, "--version"], stdout=subprocess.PIPE).stdout.strip()
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
    return result


def main() -> None:
    """Smoke-test the packaged crates without contacting a publishing API."""
    args = parser().parse_args()
    smoke_crate(args.crates, args.version, args.work_dir)


if __name__ == "__main__":
    main()
