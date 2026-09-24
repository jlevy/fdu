#!/usr/bin/env python3
"""
Guard every registry write the release workflow's publish job makes.

The publish job uploads only what the same run built, smoke-tested, and inspected. The
wheels and source distribution are uploaded as they are. Cargo is the exception: `cargo
publish` takes no archive argument and always repackages from the checkout, so each
crate is reproduced from the tag first and its SHA-256 digest compared with the
manifest's before anything is sent. The registries then have to serve exactly those
digests before the next write.

Each check fails closed. A registry that already holds the version is skipped only when
every file it holds carries the manifest's digest, which is what makes a rerun after a
partial publication safe; any other difference is a conflict that stops the job.

Exit status: 0 on success, 2 on a registry conflict, 3 when a wait times out, and 1 on
any other failure, including a registry that could not be read.
"""

from __future__ import annotations

import argparse
import http.client
import json
import os
import re
import sys
import time
from collections.abc import Callable, Sequence
from dataclasses import asdict
from pathlib import Path
from typing import Any
from urllib.error import HTTPError
from urllib.request import Request, urlopen

if __package__ in (None, ""):
    # Run as a script: make the repository root importable, as the tests have it.
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from scripts.release import inspect_artifacts, registry_state
from scripts.release.registry_state import CRATE_PACKAGES, RegistryError

Fetch = Callable[[str], bytes | None]

SPARSE_INDEX = "https://index.crates.io"
CONFLICT_STATUS = 2
TIMEOUT_STATUS = 3


class Conflict(RuntimeError):
    """A registry holds files for this version that the rehearsal did not produce."""


class WaitTimeout(RuntimeError):
    """A registry did not serve the expected files within the allowed time."""


def require(condition: bool, message: str) -> None:
    """Raise a stable validation error when a publication contract is violated."""
    if not condition:
        raise ValueError(message)


def verify_files(
    directory: Path,
    manifest: Path,
    checksums: Path,
    version: str,
) -> list[inspect_artifacts.Artifact]:
    """
    Prove a downloaded directory holds exactly the artifact set the evidence job recorded.

    The files are inspected again with the release matrix required, and the result must
    equal the manifest entry for entry: same names, kinds, packages, sizes, and digests,
    with nothing extra. `SHA256SUMS` must list the same digests, so the two retained
    records cannot disagree about what was tested.
    """
    document = json.loads(manifest.read_text(encoding="utf-8"))
    require(document.get("version") == version, f"manifest is not for version {version}")
    recorded_list = document.get("artifacts")
    require(isinstance(recorded_list, list), "manifest must contain an artifacts list")
    recorded = {str(item["filename"]): item for item in recorded_list}
    require(len(recorded) == len(recorded_list), "manifest lists a filename twice")

    present = sorted(path.name for path in directory.iterdir())
    require(
        all((directory / name).is_file() for name in present),
        f"{directory} must contain only files",
    )
    extra = sorted(set(present) - recorded.keys())
    missing = sorted(recorded.keys() - set(present))
    require(not extra, f"files outside the manifest: {', '.join(extra)}")
    require(not missing, f"manifest files not downloaded: {', '.join(missing)}")

    inspected = inspect_artifacts.inspect_directory(directory, version, require_release_matrix=True)
    for artifact in inspected:
        require(
            asdict(artifact) == recorded.get(artifact.filename),
            f"{artifact.filename}: differs from the manifest entry",
        )
    require(
        {artifact.filename for artifact in inspected} == recorded.keys(),
        "inspection and manifest name different artifact sets",
    )

    listed: dict[str, str] = {}
    for line in checksums.read_text(encoding="utf-8").splitlines():
        match = re.fullmatch(r"([0-9a-f]{64})  (\S+)", line)
        if match is None:
            raise ValueError(f"malformed SHA256SUMS line: {line!r}")
        require(match[2] not in listed, f"SHA256SUMS lists {match[2]} twice")
        listed[match[2]] = match[1]
    require(
        listed == {name: str(item["sha256"]) for name, item in recorded.items()},
        "SHA256SUMS and the manifest disagree",
    )
    return inspected


def compare_crates(
    package_dir: Path,
    manifest: Path,
    version: str,
    packages: Sequence[str],
) -> dict[str, str]:
    """
    Compare freshly packaged crates with the manifest's digests.

    A mismatch means crates.io would receive bytes nothing tested, so it stops the job
    before the upload, naming both digests.
    """
    expected = registry_state.expected_artifacts(manifest, "crate")
    compared = {}
    for package in packages:
        require(package in CRATE_PACKAGES, f"{package} is not a published crate")
        filename = f"{package}-{version}.crate"
        require(filename in expected, f"manifest has no {filename}")
        path = package_dir / filename
        require(path.is_file(), f"{path} was not packaged")
        actual = inspect_artifacts.digest(path)
        require(
            actual == expected[filename],
            f"{filename}: reproduced {actual}, but the rehearsal recorded {expected[filename]}",
        )
        compared[filename] = actual
    return compared


def index_path(name: str) -> str:
    """The sparse-index path crates.io serves one crate's version list at."""
    lower = name.lower()
    if len(lower) <= 2:
        return f"{len(lower)}/{lower}"
    if len(lower) == 3:
        return f"3/{lower[0]}/{lower}"
    return f"{lower[:2]}/{lower[2:4]}/{lower}"


def index_checksum(package: str, version: str, fetch: Fetch) -> str | None:
    """
    Read one version's `.crate` SHA-256 from the sparse index Cargo resolves against.

    The index is what a later `cargo package -p fdu` reads `fdu-core` from, so waiting on
    the API record alone would not show the dependency is resolvable.
    """
    url = f"{SPARSE_INDEX}/{index_path(package)}"
    body = fetch(url)
    if body is None:
        return None
    checksums = []
    for line in body.decode("utf-8").splitlines():
        if not line.strip():
            continue
        entry = registry_state.parse_json(url, line.encode())
        if isinstance(entry, dict) and entry.get("vers") == version:
            checksums.append(entry.get("cksum"))
    if not checksums:
        return None
    require(len(checksums) == 1, f"{url} lists {package} {version} more than once")
    checksum = checksums[0]
    require(
        isinstance(checksum, str) and re.fullmatch(r"[0-9a-f]{64}", checksum) is not None,
        f"{url} has no SHA-256 checksum for {package} {version}",
    )
    return checksum


def crate_states(manifest: Path, version: str, fetch: Fetch) -> dict[str, str]:
    """
    Classify each crate as `missing` or `identical`, raising `Conflict` otherwise.

    The API record decides. When it has no record yet but the index already lists the
    version, the upload landed and the API is trailing: that is `identical` if the index
    carries the manifest's digest and a conflict if it does not.
    """
    expected = registry_state.expected_artifacts(manifest, "crate")
    states = {}
    for state in registry_state.crates_io_state(manifest, version, fetch=fetch):
        if state.state == "conflict":
            raise Conflict(f"crates.io {state.package} {version}: {state.detail}")
        verdict = state.state
        if verdict == "missing":
            listed = index_checksum(state.package, version, fetch)
            if listed is not None:
                if listed != expected[f"{state.package}-{version}.crate"]:
                    raise Conflict(
                        f"crates.io index lists {state.package} {version} with {listed}, "
                        "not the rehearsed digest"
                    )
                verdict = "identical"
        states[state.package] = verdict
    return states


def crate_served(manifest: Path, version: str, package: str, fetch: Fetch) -> bool:
    """
    Whether both the API record and the sparse index carry the rehearsed digest.

    Either one carrying another digest is a conflict, and raises at once.
    """
    require(package in CRATE_PACKAGES, f"{package} is not a published crate")
    filename = f"{package}-{version}.crate"
    expected = registry_state.expected_artifacts(manifest, "crate").get(filename)
    require(expected is not None, f"manifest has no {filename}")
    url = f"https://crates.io/api/v1/crates/{package}/{version}"
    body = fetch(url)
    recorded = (
        None
        if body is None
        else registry_state.crate_checksum(registry_state.parse_json(url, body), package, version)
    )
    listed = index_checksum(package, version, fetch)
    for source, checksum in (("API record", recorded), ("index", listed)):
        if checksum is not None and checksum != expected:
            raise Conflict(
                f"crates.io {source} for {package} {version} has {checksum}, not {expected}"
            )
    return recorded == expected and listed == expected


def pypi_verdict(expected: dict[str, str], published: dict[str, str] | None) -> str:
    """
    Classify PyPI's files for the release as `missing`, `partial`, or `identical`.

    `partial` is an interrupted upload or an API still catching up: every file PyPI holds
    is one the rehearsal produced, so uploading the rest is safe. A file with another
    digest, or one the manifest does not name, raises `Conflict`.
    """
    if published is None:
        return "missing"
    unexpected = sorted(published.keys() - expected.keys())
    changed = sorted(
        name for name in published.keys() & expected.keys() if published[name] != expected[name]
    )
    if unexpected or changed:
        parts = []
        if unexpected:
            parts.append(f"unexpected: {', '.join(unexpected)}")
        if changed:
            parts.append(f"hash mismatch: {', '.join(changed)}")
        raise Conflict(f"PyPI fdu: {'; '.join(parts)}")
    return "identical" if published == expected else "partial"


def pypi_state(manifest: Path, version: str, fetch: Fetch) -> str:
    """Classify PyPI's files for one release against the manifest."""
    expected = registry_state.pypi_expected(manifest)
    return pypi_verdict(expected, registry_state.pypi_published(version, fetch))


def wait_until(
    probe: Callable[[], bool],
    *,
    what: str,
    timeout: float,
    interval: float,
    clock: Callable[[], float] = time.monotonic,
    sleep: Callable[[float], None] = time.sleep,
) -> int:
    """
    Poll `probe` until it returns True, and return the number of attempts it took.

    A registry that cannot be read is retried until the deadline, since mid-publication
    that is usually a transient refusal; a `Conflict` is final and propagates at once.
    """
    deadline = clock() + timeout
    attempts = 0
    last_error = ""
    while True:
        attempts += 1
        try:
            if probe():
                return attempts
            last_error = ""
        except RegistryError as error:
            last_error = f"; last read failed: {error}"
            print(f"{what}: registry read failed, retrying: {error}", file=sys.stderr)
        if clock() >= deadline:
            raise WaitTimeout(f"{what} was not served after {timeout:g} seconds{last_error}")
        sleep(interval)


def github_get(url: str, token: str | None) -> Any:
    """
    Read one GitHub REST resource as JSON, with the workflow token when there is one.

    A token refused for a public resource is retried once without it, since the answer is
    the same public record. The token is only ever sent as a header, never printed.
    """
    headers = {
        "Accept": "application/vnd.github+json",
        "User-Agent": registry_state.USER_AGENT,
        "X-GitHub-Api-Version": "2022-11-28",
    }
    attempts = [{**headers, "Authorization": f"Bearer {token}"}, headers] if token else [headers]
    for index, attempt in enumerate(attempts):
        try:
            with urlopen(Request(url, headers=attempt), timeout=30) as response:
                return registry_state.parse_json(url, response.read())
        except HTTPError as error:
            if error.code == 404:
                return None
            if error.code in {401, 403} and index + 1 < len(attempts):
                continue
            raise RegistryError(f"{url}: {error}") from error
        except (OSError, http.client.HTTPException) as error:
            raise RegistryError(f"{url}: {error}") from error
    raise AssertionError("unreachable")


def check_environment(
    repository: str,
    environment: str,
    read: Callable[[str], Any],
) -> list[str]:
    """
    Refuse to publish through an environment that is missing or unprotected.

    GitHub creates an unprotected environment the first time a job names one, and the
    trusted publishers trust any job in `release`. So before the publish job names it,
    the environment must exist with a required reviewer, admit only tag deployments whose
    patterns start with `v`, and deny administrators a bypass. Returns what was verified.
    """
    base = f"https://api.github.com/repos/{repository}/environments/{environment}"
    record = read(base)
    require(
        isinstance(record, dict),
        f"environment {environment} does not exist; create and protect it before publishing",
    )
    rules = record.get("protection_rules") or []
    reviewers = [
        reviewer
        for rule in rules
        if isinstance(rule, dict) and rule.get("type") == "required_reviewers"
        for reviewer in rule.get("reviewers") or []
    ]
    require(bool(reviewers), f"environment {environment} has no required reviewer")
    require(
        record.get("can_admins_bypass") is False,
        f"environment {environment} lets administrators bypass its protection rules",
    )
    policy = record.get("deployment_branch_policy")
    require(
        isinstance(policy, dict)
        and policy.get("custom_branch_policies") is True
        and policy.get("protected_branches") is False,
        f"environment {environment} must deploy only from selected tags",
    )
    listing = read(f"{base}/deployment-branch-policies")
    policies = listing.get("branch_policies") if isinstance(listing, dict) else None
    if not isinstance(policies, list) or not policies:
        raise ValueError(f"environment {environment} has no deployment tag rule")
    for item in policies:
        name = item.get("name") if isinstance(item, dict) else None
        kind = item.get("type") if isinstance(item, dict) else None
        require(
            kind == "tag" and isinstance(name, str) and name.startswith("v"),
            f"environment {environment} admits {kind} {name!r}; only v* tag rules are allowed",
        )
    return [
        f"{len(reviewers)} required reviewer(s)",
        "administrator bypass off",
        f"deployments only from tags {', '.join(str(item['name']) for item in policies)}",
    ]


def write_outputs(path: Path | None, values: dict[str, str]) -> None:
    """Append step outputs for GitHub Actions."""
    if path is None:
        return
    with path.open("a", encoding="utf-8") as output:
        for name, value in values.items():
            output.write(f"{name}={value}\n")


def audit(manifest: Path, version: str, fetch: Fetch) -> dict[str, str]:
    """
    Audit both registries before a write, and name what still needs uploading.

    Any conflict on either registry raises before anything is written, so a PyPI
    conflict found up front stops the crates from being published first.
    """
    crates = crate_states(manifest, version, fetch)
    pypi = pypi_state(manifest, version, fetch)
    return {
        "fdu_core": crates["fdu-core"],
        "fdu": crates["fdu"],
        "crates": str(any(state == "missing" for state in crates.values())).lower(),
        "pypi": pypi,
        "upload": str(pypi != "identical").lower(),
    }


def parser() -> argparse.ArgumentParser:
    """Build the command-line parser."""
    result = argparse.ArgumentParser(description="Guard the release workflow's registry writes.")
    commands = result.add_subparsers(dest="command", required=True)

    verify = commands.add_parser("verify-files", help="prove downloads match the manifest")
    verify.add_argument("directory", type=Path)
    verify.add_argument("--checksums", type=Path, required=True)

    audit_command = commands.add_parser("audit", help="classify both registries before a write")
    audit_command.add_argument("--github-output", type=Path)

    compare = commands.add_parser("compare-crates", help="compare packaged crates with manifest")
    compare.add_argument("--package-dir", type=Path, required=True)
    compare.add_argument("--package", action="append", required=True, choices=CRATE_PACKAGES)

    wait_crate = commands.add_parser("wait-crate", help="wait until crates.io serves a crate")
    wait_crate.add_argument("--package", required=True, choices=CRATE_PACKAGES)

    wait_pypi = commands.add_parser("wait-pypi", help="wait until PyPI serves every file")

    environment = commands.add_parser(
        "check-environment", help="refuse a missing or unprotected deployment environment"
    )
    environment.add_argument("--repository", required=True)
    environment.add_argument("--environment", required=True)

    for command in (verify, audit_command, compare, wait_crate, wait_pypi):
        command.add_argument("--manifest", type=Path, required=True)
        command.add_argument("--version", required=True)
    for command in (wait_crate, wait_pypi):
        command.add_argument("--timeout", type=float, default=600.0)
        command.add_argument("--interval", type=float, default=15.0)
    return result


def run(args: argparse.Namespace, fetch: Fetch) -> None:
    """Carry out one subcommand."""
    if args.command == "check-environment":
        token = os.environ.get("GITHUB_TOKEN") or None
        verified = check_environment(
            args.repository, args.environment, lambda url: github_get(url, token)
        )
        print(f"environment {args.environment}: {'; '.join(verified)}")
    elif args.command == "verify-files":
        artifacts = verify_files(args.directory, args.manifest, args.checksums, args.version)
        for artifact in artifacts:
            print(f"verified {artifact.sha256}  {artifact.filename}")
    elif args.command == "audit":
        values = audit(args.manifest, args.version, fetch)
        print(json.dumps(values, indent=2, sort_keys=True))
        write_outputs(args.github_output, values)
    elif args.command == "compare-crates":
        compared = compare_crates(args.package_dir, args.manifest, args.version, args.package)
        for filename, digest in compared.items():
            print(f"reproduced {digest}  {filename}")
    elif args.command == "wait-crate":
        attempts = wait_until(
            lambda: crate_served(args.manifest, args.version, args.package, fetch),
            what=f"crates.io {args.package} {args.version}",
            timeout=args.timeout,
            interval=args.interval,
        )
        print(f"crates.io serves the rehearsed {args.package} {args.version} ({attempts} reads)")
    elif args.command == "wait-pypi":
        attempts = wait_until(
            lambda: pypi_state(args.manifest, args.version, fetch) == "identical",
            what=f"PyPI fdu {args.version}",
            timeout=args.timeout,
            interval=args.interval,
        )
        print(f"PyPI serves exactly the rehearsed fdu {args.version} files ({attempts} reads)")


def main(argv: Sequence[str] | None = None, fetch: Fetch | None = None) -> None:
    """Run the requested guard, mapping a conflict and a timeout to their exit statuses."""
    args = parser().parse_args(argv)
    try:
        run(args, fetch or registry_state.get)
    except Conflict as error:
        print(f"conflict: {error}", file=sys.stderr)
        raise SystemExit(CONFLICT_STATUS) from error
    except WaitTimeout as error:
        print(f"timeout: {error}", file=sys.stderr)
        raise SystemExit(TIMEOUT_STATUS) from error


if __name__ == "__main__":
    main()
