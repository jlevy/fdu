#!/usr/bin/env python3
"""Announce the verified artifacts of a publishing run, resuming incomplete drafts."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Any

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from scripts.atomic_write import write_text_atomic
from scripts.release import maintainer, registry_state, release_body
from scripts.release.maintainer import CommandError, Host, Release, StepError


def missing_assets(
    release: Release, record: dict[str, Any], demo: maintainer.Demo | None
) -> list[Path]:
    """
    Return absent assets; refuse unexpected files or any unverifiable existing bytes.

    The announcement attaches the eleven verified files and never the demo video, which
    the workflow has no copy of. A demo the release commit declares is attached to the
    draft by `make release-demo`, and must be exactly the declared bytes; one it does not
    declare is unexpected. An asset GitHub never finished uploading is refused too. On a
    draft, which can still change, each refusal names the command that removes the asset.
    """
    expected = maintainer.expected_assets(release)
    allowed = expected.keys() | ({demo.asset} if demo is not None else set())
    held = record.get("assets") or []
    names = [asset["name"] for asset in held]
    if len(names) != len(set(names)) or set(names) - allowed:
        raise StepError("GitHub release has duplicate or unexpected assets")
    for asset in held:
        name = asset["name"]
        if demo is not None and name == demo.asset:
            against, problems = maintainer.DEMO_DECLARATION, demo.asset_problems(asset)
        else:
            path = expected[name]
            against, problems = "verified files", maintainer.upload_problems(asset)
            if asset.get("size") != path.stat().st_size:
                problems.append(f"{name} size differs")
            if asset.get("digest") != f"sha256:{maintainer.inspect_artifacts.digest(path)}":
                problems.append(f"{name} digest differs")
        if problems:
            if record.get("draft"):
                removal = maintainer.asset_removal(release, name)
                remedy = f"the draft can still change: inspect it, run `{removal}`, and rerun"
            else:
                remedy = "the release is published and immutable: inspect it, do not repair it"
            detail = "; ".join(problems)
            raise StepError(f"GitHub asset conflicts with {against}: {name} ({detail}); {remedy}")
    return [path for name, path in expected.items() if name not in names]


def announce(host: Host, release: Release) -> None:
    """
    Audit before writes; publish only after every draft asset has the expected hash.

    A release is immutable once published, so a demo video the tagged commit declares
    must be in the draft before it is published: `make release-demo` attaches it there.
    Without it the job still completes the draft with the eleven files, then stops short
    of publishing and names that step, so the job can be rerun once it has run.
    """
    maintainer.require_pushed_tag(host, release)
    maintainer.verify_kept(release, release.directory / "published")
    demo = maintainer.declared_demo(host, release)
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
    record = maintainer.release_record(host, release)
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
        record = maintainer.release_record(host, release)
    if record is None:
        raise StepError("GitHub did not return the release draft")
    identity = maintainer.identity_problems(release, record, notes)
    if identity:
        remedy = (
            "; it is a draft, so correct its title and body to this run's notes.md, or "
            "delete it, then rerun this job"
            if record.get("draft")
            else ""
        )
        raise StepError(
            f"existing GitHub release identity or notes conflict: {'; '.join(identity)}{remedy}"
        )
    missing = missing_assets(release, record, demo)
    lacks_demo = demo is not None and maintainer.held_demo(record, demo) is None
    if not record.get("draft"):
        if missing:
            raise StepError("published GitHub release is missing assets; inspect before repair")
        if demo is not None and lacks_demo:
            raise StepError(
                f"{release.tag} was published without the declared {demo.asset}, and a "
                "published release is immutable: nothing can attach it now"
            )
        print(f"{release.tag} is already announced with identical assets")
        return
    if missing:
        host.run(
            maintainer.gh(release, "release", "upload", release.tag, *(str(p) for p in missing))
        )
    record = maintainer.release_record(host, release)
    if record is None or missing_assets(release, record, demo):
        raise StepError("draft is incomplete after upload")
    if demo is not None and maintainer.held_demo(record, demo) is None:
        raise StepError(
            f"the draft {release.tag} holds the eleven files but not {demo.asset}, which the "
            f"tagged commit declares: run `make release-demo` for {release.tag}, then rerun "
            "this job. The draft stays unpublished, since a published release could never "
            "take the video"
        )
    # Check the remote tag again immediately before making the draft public.
    maintainer.require_pushed_tag(host, release)
    host.run(maintainer.gh(release, "release", "edit", release.tag, "--draft=false"))
    record = maintainer.release_record(host, release)
    if (
        record is None
        or record.get("draft")
        or missing_assets(release, record, demo)
        or (demo is not None and maintainer.held_demo(record, demo) is None)
    ):
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
