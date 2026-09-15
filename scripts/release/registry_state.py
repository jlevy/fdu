#!/usr/bin/env python3
"""Classify crates.io and PyPI state against one validated artifact manifest."""

from __future__ import annotations

import argparse
import json
import re
from collections.abc import Callable
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any
from urllib.error import HTTPError
from urllib.request import Request, urlopen

USER_AGENT = "fdu-release-audit/0.1 (+https://github.com/jlevy/fdu)"

# Every published crate, in publication order. Each is its own crates.io record, so each
# is classified on its own; inspect_artifacts.py names the same crates.
CRATE_PACKAGES = ("fdu-core", "fdu")


@dataclass(frozen=True, slots=True)
class RegistryState:
    """Read-only comparison of one registry version with expected artifacts."""

    channel: str
    package: str
    version: str
    state: str
    detail: str


def expected_artifacts(manifest: Path, kind: str) -> dict[str, str]:
    """Load filename-to-SHA-256 evidence for one artifact kind."""
    document = json.loads(manifest.read_text(encoding="utf-8"))
    artifacts = document.get("artifacts")
    if not isinstance(artifacts, list):
        raise ValueError("artifact manifest must contain an artifacts list")
    expected = {
        str(item["filename"]): str(item["sha256"])
        for item in artifacts
        if isinstance(item, dict) and item.get("kind") == kind
    }
    if not expected:
        raise ValueError(f"artifact manifest contains no {kind} artifacts")
    if any(len(digest) != 64 for digest in expected.values()):
        raise ValueError(f"artifact manifest contains an invalid {kind} SHA-256 digest")
    return expected


def classify_files(
    channel: str,
    package: str,
    version: str,
    expected: dict[str, str],
    published: dict[str, str] | None,
) -> RegistryState:
    """Classify an immutable file set as missing, identical, or conflicting."""
    if published is None:
        return RegistryState(channel, package, version, "missing", "version is not present")
    if expected == published:
        return RegistryState(
            channel, package, version, "identical", "all filenames and hashes match"
        )

    missing = sorted(expected.keys() - published.keys())
    unexpected = sorted(published.keys() - expected.keys())
    changed = sorted(
        filename
        for filename in expected.keys() & published.keys()
        if expected[filename] != published[filename]
    )
    parts = []
    if missing:
        parts.append(f"missing: {', '.join(missing)}")
    if unexpected:
        parts.append(f"unexpected: {', '.join(unexpected)}")
    if changed:
        parts.append(f"hash mismatch: {', '.join(changed)}")
    return RegistryState(channel, package, version, "conflict", "; ".join(parts))


class RegistryError(RuntimeError):
    """A registry could not be read, so the audit has no verdict for it."""


def get(url: str) -> bytes | None:
    """
    Fetch a public registry resource, mapping an authoritative 404 to absence.

    Every other failure raises, naming the URL: a refusal, an outage, or an unreachable
    host read as `missing` would report an upload that landed as one that did not.
    """
    request = Request(url, headers={"Accept": "application/json", "User-Agent": USER_AGENT})
    try:
        with urlopen(request, timeout=30) as response:
            return response.read()
    except HTTPError as error:
        if error.code == 404:
            return None
        raise RegistryError(f"{url}: {error}") from error
    except OSError as error:
        raise RegistryError(f"{url}: {error}") from error


def pypi_state(manifest: Path, version: str) -> RegistryState:
    """Compare the complete expected wheel/sdist set with PyPI's release metadata."""
    expected = {
        **expected_artifacts(manifest, "wheel"),
        **expected_artifacts(manifest, "sdist"),
    }
    body = get(f"https://pypi.org/pypi/fdu/{version}/json")
    if body is None:
        return classify_files("pypi", "fdu", version, expected, None)
    document: dict[str, Any] = json.loads(body)
    urls = document.get("urls")
    if not isinstance(urls, list):
        raise ValueError("PyPI response has no release file list")
    published = {
        str(item["filename"]): str(item["digests"]["sha256"])
        for item in urls
        if isinstance(item, dict)
        and isinstance(item.get("digests"), dict)
        and item["digests"].get("sha256")
    }
    return classify_files("pypi", "fdu", version, expected, published)


def crates_io_state(
    manifest: Path,
    version: str,
    fetch: Callable[[str], bytes | None] = get,
) -> list[RegistryState]:
    """
    Compare every expected Cargo package with its crates.io version record.

    A version that does not exist is an authoritative 404, and the record's `checksum` is
    the SHA-256 of the published `.crate`. The download endpoint cannot stand in for it:
    asked for JSON it answers 200 with a URL for any version, published or not, and the
    file host it redirects to answers 403, not 404, for a missing file.
    """
    expected = expected_artifacts(manifest, "crate")
    filenames = {package: f"{package}-{version}.crate" for package in CRATE_PACKAGES}
    unexpected = sorted(expected.keys() - set(filenames.values()))
    if unexpected:
        raise ValueError(f"artifact manifest has unexpected crates: {', '.join(unexpected)}")
    states = []
    for package, filename in filenames.items():
        if filename not in expected:
            raise ValueError(f"artifact manifest has no {filename}")
        body = fetch(f"https://crates.io/api/v1/crates/{package}/{version}")
        published = None if body is None else {filename: crate_checksum(body, package, version)}
        states.append(
            classify_files("crates.io", package, version, {filename: expected[filename]}, published)
        )
    return states


def crate_checksum(body: bytes, package: str, version: str) -> str:
    """Read the published `.crate` SHA-256 from a crates.io version record."""
    document: Any = json.loads(body)
    record = document.get("version") if isinstance(document, dict) else None
    checksum = record.get("checksum") if isinstance(record, dict) else None
    if not isinstance(checksum, str) or re.fullmatch(r"[0-9a-f]{64}", checksum) is None:
        raise ValueError(f"crates.io record for {package} {version} has no SHA-256 checksum")
    return checksum


def exit_status(states: list[RegistryState], *, require_identical: bool) -> int:
    """
    Return 2 when any registry conflicts, and 3 when `require_identical` is set and any
    registry does not yet hold the expected files; otherwise 0.
    """
    if any(state.state == "conflict" for state in states):
        return 2
    if require_identical and any(state.state != "identical" for state in states):
        return 3
    return 0


def parser() -> argparse.ArgumentParser:
    """Build the command-line parser."""
    result = argparse.ArgumentParser()
    result.add_argument("--manifest", type=Path, required=True)
    result.add_argument("--version", required=True)
    result.add_argument("--channel", choices=("all", "crates.io", "pypi"), default="all")
    result.add_argument("--output", type=Path)
    result.add_argument(
        "--require-identical",
        action="store_true",
        help="exit 3 unless every audited registry already holds exactly the expected files",
    )
    return result


def main() -> None:
    """Audit registry state without credentials or mutations."""
    args = parser().parse_args()
    states = []
    if args.channel in {"all", "crates.io"}:
        states.extend(crates_io_state(args.manifest, args.version))
    if args.channel in {"all", "pypi"}:
        states.append(pypi_state(args.manifest, args.version))
    document = {"version": args.version, "registries": [asdict(state) for state in states]}
    rendered = json.dumps(document, indent=2, sort_keys=True) + "\n"
    print(rendered, end="")
    if args.output is not None:
        args.output.write_text(rendered, encoding="utf-8")
    raise SystemExit(exit_status(states, require_identical=args.require_identical))


if __name__ == "__main__":
    main()
