#!/usr/bin/env python3
"""Announce the verified artifacts of a publishing run, resuming incomplete drafts."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from scripts.atomic_write import write_text_atomic
from scripts.release import maintainer, registry_state, release_body
from scripts.release.maintainer import CommandError, Host, Release, StepError


def release_record(host: Host, release: Release) -> dict[str, Any] | None:
    """List with write authority to include drafts, which the tag endpoint omits."""
    pages = json.loads(
        host.run(
            [
                "gh",
                "api",
                "--paginate",
                "--slurp",
                f"repos/{release.repository}/releases?per_page=100",
            ]
        )
    )
    matches = [record for page in pages for record in page if record.get("tag_name") == release.tag]
    if len(matches) > 1:
        raise StepError(f"multiple releases name {release.tag}")
    return matches[0] if matches else None


def missing_assets(release: Release, record: dict[str, Any]) -> list[Path]:
    """Return absent assets; refuse unexpected files or any unverifiable existing bytes."""
    expected = maintainer.expected_assets(release)
    held = record.get("assets") or []
    names = [asset["name"] for asset in held]
    if len(names) != len(set(names)) or set(names) - expected.keys():
        raise StepError("GitHub release has duplicate or unexpected assets")
    for asset in held:
        path = expected[asset["name"]]
        digest = maintainer.inspect_artifacts.digest(path)
        if asset.get("size") != path.stat().st_size or asset.get("digest") != f"sha256:{digest}":
            raise StepError(f"GitHub asset conflicts with verified files: {path.name}")
    return [path for name, path in expected.items() if name not in names]


def announce(host: Host, release: Release) -> None:
    """Audit before writes; publish only after every draft asset has the expected hash."""
    maintainer.require_pushed_tag(host, release)
    maintainer.verify_kept(release, release.directory / "published")
    # A demo video committed at the tag is a twelfth asset; without one there are eleven.
    maintainer.stage_demo(host, release)
    notes = (release.directory / "notes.md").read_text(encoding="utf-8")
    source = maintainer.show(host, release, release.notes_path)
    if source is None:
        raise StepError("release commit has no release notes")
    release_body.check_release_body(source, release_body.strip_html_comments(source), notes)
    states = maintainer.registry_states(
        host, release, release.directory / "published/evidence/release-manifest.json"
    )
    if registry_state.exit_status(states, require_identical=True):
        raise StepError("every registry must match before announcing")
    write_text_atomic(
        release.directory / "registry-state.json",
        registry_state.registry_document(release.version, states),
        encoding="utf-8",
    )
    record = release_record(host, release)
    if record is None:
        # Create the draft before uploading. A failed upload leaves a resumable draft,
        # never a public announcement with only some of its files.
        host.run(
            maintainer.gh(
                release,
                "release",
                "create",
                release.tag,
                "--verify-tag",
                "--draft",
                "--title",
                f"fdu {release.version}",
                "--notes-file",
                str(release.directory / "notes.md"),
            )
        )
        record = release_record(host, release)
    if record is None:
        raise StepError("GitHub did not return the release draft")
    if (
        record.get("tag_name") != release.tag
        or record.get("prerelease")
        or record.get("name") != f"fdu {release.version}"
        or str(record.get("body") or "").strip() != notes.strip()
    ):
        raise StepError("existing GitHub release identity or notes conflict")
    missing = missing_assets(release, record)
    if not record.get("draft"):
        if missing:
            raise StepError("published GitHub release is missing assets; inspect before repair")
        print(f"{release.tag} is already announced with identical assets")
        return
    if missing:
        host.run(
            maintainer.gh(release, "release", "upload", release.tag, *(str(p) for p in missing))
        )
    record = release_record(host, release)
    if record is None or missing_assets(release, record):
        raise StepError("draft is incomplete after upload")
    # Check the remote tag again immediately before making the draft public.
    maintainer.require_pushed_tag(host, release)
    host.run(maintainer.gh(release, "release", "edit", release.tag, "--draft=false"))
    record = release_record(host, release)
    if record is None or record.get("draft") or missing_assets(release, record):
        raise StepError("GitHub release did not become public with all expected assets")
    print(record["html_url"])


def main() -> int:
    """Use the same identity and directory layout as the local release verifier."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--dir", required=True)
    parser.add_argument("--repo", required=True)
    args = parser.parse_args()
    host = Host()
    try:
        release = maintainer.resolve(
            host,
            version=args.version,
            commit=args.commit,
            directory=args.dir,
            repository=args.repo,
            cwd=Path.cwd(),
        )
        announce(host, release)
    except (StepError, CommandError, registry_state.RegistryError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
