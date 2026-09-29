#!/usr/bin/env python3
"""
Check that a compatible release leaves each published Rust API compatible.

fdu is pre-1.0 and follows Cargo's compatibility rule: a patch release (0.2.0 to 0.2.1)
never changes the Rust API incompatibly, while a minor release (0.2 to 0.3) may. So for
each published library crate this finds the release the current version must stay
compatible with, the highest unyanked version on crates.io at or below it with the same
left-most non-zero component, and runs the pinned `cargo-semver-checks` against it twice:
with no build features, as library consumers build, and with all of them. A version that
starts a new series, such as 0.3.0, has nothing to stay compatible with and is reported
as such rather than checked.

Before the version is bumped, the current version is itself the baseline, and
cargo-semver-checks then assumes the smallest next release, which for 0.y.z is a patch:
the check answers whether this tree could still ship as one.

The tool version is read from `supply-chain-policy.json`, whose supply-chain check
verifies it against crates.io and the workflow that installs it, so a different local
install is refused rather than trusted.

Exit status: 0 when every check passed or none applied, 1 otherwise.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tomllib
from collections.abc import Callable, Iterable, Sequence
from pathlib import Path

if __package__ in (None, ""):
    # Run as a script: make the repository root importable, as the tests have it.
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from scripts.release import registry_state
from scripts.release.publish_gate import SPARSE_INDEX, index_path
from scripts.release.registry_state import CRATE_PACKAGES

TOOL = "cargo-semver-checks"
MANIFESTS = {"fdu-core": "crates/fdu-core/Cargo.toml", "fdu": "crates/fdu/Cargo.toml"}
# No build features, as `default-features = false` consumers build, and every one.
FEATURE_SETS = (
    ("no build features", "--only-explicit-features"),
    ("all build features", "--all-features"),
)

Fetch = Callable[[str], bytes | None]
Run = Callable[[Sequence[str]], int]


def release_key(version: str) -> tuple[int, int, int] | None:
    """A plain `X.Y.Z` release as a tuple; None for a pre-release or anything else."""
    match = re.fullmatch(r"(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)", version)
    return None if match is None else (int(match[1]), int(match[2]), int(match[3]))


def compatible(left: tuple[int, int, int], right: tuple[int, int, int]) -> bool:
    """Whether Cargo treats the two releases as compatible: the same left-most non-zero part."""
    if left[0] != right[0]:
        return False
    if left[0] > 0:
        return True
    if left[1] != right[1]:
        return False
    return left[1] > 0 or left[2] == right[2]


def baseline(current: str, published: Iterable[tuple[str, bool]]) -> str | None:
    """
    The release `current` must stay compatible with, or None when it starts a new series.

    That is the highest unyanked, compatible release at or below `current`. A yanked
    version is skipped: it was withdrawn, and the promise is to what remains.
    """
    key = release_key(current)
    if key is None:
        raise ValueError(f"{current} is not a plain X.Y.Z release version")
    candidates = [
        (candidate, version)
        for version, yanked in published
        if not yanked
        and (candidate := release_key(version)) is not None
        and candidate <= key
        and compatible(candidate, key)
    ]
    return max(candidates)[1] if candidates else None


def published_versions(package: str, fetch: Fetch) -> list[tuple[str, bool]]:
    """Every version of `package` the sparse index lists, with its yanked flag."""
    url = f"{SPARSE_INDEX}/{index_path(package)}"
    body = fetch(url)
    if body is None:
        return []
    versions = []
    for line in body.decode("utf-8").splitlines():
        if not line.strip():
            continue
        entry = registry_state.parse_json(url, line.encode())
        if not isinstance(entry, dict) or not isinstance(entry.get("vers"), str):
            raise ValueError(f"{url} lists an entry without a version: {line!r}")
        versions.append((entry["vers"], entry.get("yanked") is True))
    return versions


def cargo_version(manifest: Path) -> str:
    """The package version a Cargo manifest declares."""
    value = tomllib.loads(manifest.read_text(encoding="utf-8"))["package"]["version"]
    if not isinstance(value, str) or not value:
        raise ValueError(f"{manifest} declares no package version")
    return value


def pinned_tool_version(root: Path) -> str:
    """The reviewed cargo-semver-checks version the supply-chain policy inventories."""
    policy = json.loads((root / "supply-chain-policy.json").read_text(encoding="utf-8"))
    pins = [
        item.get("version")
        for item in policy.get("bootstrap", {}).get("cargoTools", [])
        if isinstance(item, dict) and item.get("name") == TOOL
    ]
    if len(pins) != 1 or not isinstance(pins[0], str):
        raise ValueError(f"supply-chain-policy.json must inventory exactly one {TOOL} version")
    return pins[0]


def install_command(version: str) -> str:
    """The one reviewed way to install the tool."""
    return f"cargo install --locked {TOOL} --version {version}"


def installed_tool_version(root: Path) -> str | None:
    """The version `cargo semver-checks --version` reports, or None when it is absent."""
    try:
        output = subprocess.run(
            ["cargo", "semver-checks", "--version"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    except (OSError, subprocess.CalledProcessError):
        return None
    match = re.fullmatch(rf"{TOOL} (\S+)\s*", output)
    return None if match is None else match[1]


def commands(package: str, baseline_version: str) -> list[tuple[str, list[str]]]:
    """One cargo-semver-checks invocation per feature set, each with its label."""
    return [
        (
            label,
            [
                "cargo",
                "semver-checks",
                "check-release",
                "--package",
                package,
                "--baseline-version",
                baseline_version,
                flag,
            ],
        )
        for label, flag in FEATURE_SETS
    ]


def plan(root: Path, packages: Sequence[str], fetch: Fetch) -> list[tuple[str, str, str | None]]:
    """Each package with its current version and the baseline it is checked against."""
    planned = []
    for package in packages:
        current = cargo_version(root / MANIFESTS[package])
        planned.append((package, current, baseline(current, published_versions(package, fetch))))
    return planned


def check(
    root: Path,
    packages: Sequence[str],
    fetch: Fetch,
    run: Run,
    tool_version: Callable[[], str | None],
) -> list[str]:
    """Run every applicable check, and return a line for each one that failed."""
    planned = plan(root, packages, fetch)
    for package, current, base in planned:
        if base is None:
            print(f"{package} {current}: starts a new compatibility series; nothing to check")
    if all(base is None for _, _, base in planned):
        return []
    pinned = pinned_tool_version(root)
    installed = tool_version()
    if installed != pinned:
        found = "is not installed" if installed is None else f"is {installed}"
        raise ValueError(f"{TOOL} {found}, not the reviewed {pinned}: {install_command(pinned)}")
    failures = []
    for package, current, base in planned:
        if base is None:
            continue
        for label, argv in commands(package, base):
            relation = "the same version" if base == current else base
            print(f"{package} {current} against {relation}, {label}: {' '.join(argv)}", flush=True)
            # A tool or build error fails as surely as a finding: neither shows compatibility.
            status = run(argv)
            if status != 0:
                failures.append(
                    f"{package} {current} against {base}, {label}: {TOOL} exited {status}"
                )
    return failures


def parser() -> argparse.ArgumentParser:
    """Build the command-line parser."""
    result = argparse.ArgumentParser(description=__doc__.split("\n\n", 1)[0].strip())
    result.add_argument("--root", type=Path, default=Path.cwd())
    result.add_argument("--package", action="append", choices=CRATE_PACKAGES)
    return result


def main(argv: Sequence[str] | None = None) -> None:
    """Check each requested library crate, both by default."""
    args = parser().parse_args(argv)
    root = args.root.resolve()

    def run(command: Sequence[str]) -> int:
        return subprocess.run(command, cwd=root, check=False).returncode

    try:
        failures = check(
            root,
            args.package or CRATE_PACKAGES,
            registry_state.get,
            run,
            lambda: installed_tool_version(root),
        )
    except (ValueError, registry_state.RegistryError) as error:
        # An unreadable index is no answer: never read it as "nothing published".
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
    for failure in failures:
        print(f"FAIL {failure}", file=sys.stderr)
    if failures:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
