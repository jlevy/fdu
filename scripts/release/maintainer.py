#!/usr/bin/env python3
"""
Run the maintainer's local release steps, one command each.

[The release process](../../docs/project/guides/release-process.md) is the procedure;
this program is the part of it that needs no judgment. Each step reads git, GitHub, and
the registries, or writes only what can be undone: it pushes and deletes the
`release/v{VERSION}` branch a rehearsal runs on, dispatches that rehearsal (which cannot
publish), and downloads and verifies artifacts into the release directory. Pushing the
tag, dispatching the publishing run, approving the `release` environment, and creating
the GitHub release stay the maintainer's own commands; nothing here issues any of them.

Every step takes the release identity from `VERSION`, `COMMIT`, and `RELEASE` in the
environment (or `--version`, `--commit`, and `--dir`), and records the run IDs it finds
in `$RELEASE/state.json`, so a later step never needs one pasted in.

Exit status: 0 when every check passes, 1 when any check fails or a step cannot finish,
and 3 when nothing failed but a check is still pending, such as a docs.rs build that has
not run yet: rerun the step until it passes.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import subprocess
import sys
import tempfile
import time
import tomllib
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

if __package__ in (None, ""):
    # Run as a script: make the repository root importable, as the tests have it.
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from scripts.release import (
    inspect_artifacts,
    publish_gate,
    registry_state,
    release_body,
    resolve_plan,
)
from scripts.release.registry_state import CRATE_PACKAGES, RegistryError, RegistryState

REPOSITORY = "jlevy/fdu"
WORKFLOW = "release.yml"
# The workflow's `name:`, and its `publish` job's `name:`, as `gh run view` reports them;
# tests/release/test_maintainer.py reads release.yml to keep them in step.
WORKFLOW_NAME = "Release"
ENVIRONMENT = "release"
PUBLISH_JOB = "Publish to crates.io and PyPI"
VERSION_PATTERN = re.compile(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)")
PACKAGE_MANIFESTS = (
    "crates/fdu/Cargo.toml",
    "crates/fdu-core/Cargo.toml",
    "crates/fdu-py/Cargo.toml",
)
EVIDENCE_FILES = ("SHA256SUMS", "registry-state.json", "release-manifest.json")
# Two crates, one source distribution, and five wheels.
RELEASE_FILE_COUNT = 8
INSTALL_PYTHON = "3.12"
PENDING_STATUS = 3
# In checklist order, then the recovery audit; each is also `make release-<step>`.
STEPS = (
    "preflight",
    "candidate",
    "body",
    "verify-tag",
    "published",
    "announced",
    "cleanup",
    "audit",
)


class StepError(RuntimeError):
    """A step cannot continue; the message says why and what to do."""


class CommandError(RuntimeError):
    """A command exited non-zero."""

    def __init__(self, argv: Sequence[str], status: int, stderr: str) -> None:
        self.argv = list(argv)
        self.status = status
        self.stderr = stderr
        detail = stderr.strip().splitlines()[-1] if stderr.strip() else f"exit status {status}"
        super().__init__(f"{shlex.join(self.argv)}: {detail}")


class Host:
    """Everything a step does outside this process, in one place a test can script."""

    def run(self, argv: Sequence[str], *, cwd: Path | None = None, stderr: bool = False) -> str:
        """
        Run a command and return its stdout, raising `CommandError` on failure.

        With `stderr`, the command's stderr follows its stdout in the result, for tools
        such as `git tag -v` that report on stderr.
        """
        completed = subprocess.run(list(argv), cwd=cwd, capture_output=True, text=True, check=False)
        if completed.returncode != 0:
            raise CommandError(argv, completed.returncode, completed.stderr)
        return completed.stdout + completed.stderr if stderr else completed.stdout

    def attach(self, argv: Sequence[str], *, cwd: Path | None = None) -> int:
        """Run a command on the maintainer's terminal and return its exit status."""
        return subprocess.run(list(argv), cwd=cwd, check=False).returncode

    def fetch(self, url: str) -> bytes | None:
        """Read a public resource; None is an authoritative 404, anything else raises."""
        return registry_state.get(url)

    def sleep(self, seconds: float) -> None:
        """Wait between polls."""
        time.sleep(seconds)


@dataclass(frozen=True, slots=True)
class Check:
    """
    One named verdict, printed as a line of a step's checklist.

    A check that is not ok is a failure, unless it is `pending`: something outside fdu
    has not happened yet, and running the step again later can pass it.
    """

    name: str
    ok: bool
    detail: str
    pending: bool = False


@dataclass(frozen=True, slots=True)
class Release:
    """The identity every step works on: one version, one commit, one directory."""

    version: str
    commit: str
    directory: Path
    root: Path
    repository: str = REPOSITORY

    @property
    def tag(self) -> str:
        return f"v{self.version}"

    @property
    def branch(self) -> str:
        """The branch that pins the release commit for the rehearsal's dispatch."""
        return f"release/v{self.version}"

    @property
    def notes_path(self) -> str:
        return f"docs/project/release-notes/{self.version}.md"

    def run_url(self, run_id: int) -> str:
        return f"https://github.com/{self.repository}/actions/runs/{run_id}"


def version_key(version: str) -> tuple[int, int, int] | None:
    """Parse `X.Y.Z` for ordering, or None for anything else."""
    match = VERSION_PATTERN.fullmatch(version)
    if match is None:
        return None
    return (int(match[1]), int(match[2]), int(match[3]))


def report(checks: Sequence[Check]) -> int:
    """Print each check on its own line and return the step's exit status."""
    for check in checks:
        label = "ok  " if check.ok else "wait" if check.pending else "FAIL"
        print(f"{label}  {check.name}: {check.detail}")
    if any(not check.ok and not check.pending for check in checks):
        return 1
    return PENDING_STATUS if any(not check.ok for check in checks) else 0


# --- Release identity and state -------------------------------------------------------


def resolve(
    host: Host,
    *,
    version: str | None,
    commit: str | None,
    directory: str | None,
    repository: str,
    cwd: Path,
) -> Release:
    """Validate the release identity and create the release directory."""
    if not version:
        raise StepError("set VERSION to the version being released, such as 0.2.1")
    if version_key(version) is None:
        raise StepError(f"VERSION must be X.Y.Z, got {version!r}")
    if not commit:
        raise StepError("set COMMIT to the release commit on main")
    if not directory:
        raise StepError("set RELEASE to an empty directory outside any checkout")
    root = Path(host.run(["git", "rev-parse", "--show-toplevel"], cwd=cwd).strip())
    try:
        full = host.run(
            ["git", "rev-parse", "--verify", "--quiet", f"{commit}^{{commit}}"], cwd=root
        ).strip()
    except CommandError as error:
        raise StepError(f"{commit} is not a commit here; run `git fetch origin` first") from error
    path = Path(directory).expanduser().resolve()
    if path.is_relative_to(root.resolve()):
        raise StepError(f"RELEASE must be outside the checkout, not {path}")
    path.mkdir(parents=True, exist_ok=True)
    release = Release(version, full, path, root, repository)
    # Every step binds the directory to this version and commit, or refuses it.
    save_state(release)
    return release


def load_state(release: Release) -> dict[str, Any]:
    """Read the run IDs recorded for this release, refusing another release's directory."""
    path = release.directory / "state.json"
    if not path.exists():
        return {"version": release.version, "commit": release.commit}
    state: dict[str, Any] = json.loads(path.read_text(encoding="utf-8"))
    if state.get("version") != release.version or state.get("commit") != release.commit:
        raise StepError(
            f"{path} belongs to {state.get('version')} at {state.get('commit')}; "
            "use a new RELEASE directory for each release commit"
        )
    return state


def save_state(release: Release, **values: int) -> None:
    """Record run IDs for later steps."""
    state = load_state(release)
    state.update(values)
    (release.directory / "state.json").write_text(
        json.dumps(state, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


# --- Git and GitHub reads -------------------------------------------------------------


def show(host: Host, release: Release, path: str) -> str | None:
    """Read one file as the release commit has it, or None when it has none."""
    try:
        return host.run(["git", "show", f"{release.commit}:{path}"], cwd=release.root)
    except CommandError:
        return None


def remote_refs(host: Host, release: Release, *refs: str) -> dict[str, str]:
    """Read refs from origin, including a tag's peeled `^{}` entry when asked for."""
    output = host.run(["git", "ls-remote", "origin", *refs], cwd=release.root)
    found: dict[str, str] = {}
    for line in output.splitlines():
        if "\t" in line:
            sha, name = line.split("\t", 1)
            found[name] = sha
    return found


def remote_tag(host: Host, release: Release) -> tuple[str, str] | None:
    """The release tag on origin as `(tag object, commit)`, or None when it is absent."""
    ref = f"refs/tags/{release.tag}"
    found = remote_refs(host, release, ref, f"{ref}^{{}}")
    if ref not in found:
        return None
    # A lightweight tag has no peeled entry: it names the commit directly.
    return found[ref], found.get(f"{ref}^{{}}", found[ref])


def rev_parse(host: Host, release: Release, name: str) -> str | None:
    """Resolve a local name, or None when it does not exist."""
    try:
        return host.run(["git", "rev-parse", "--verify", "--quiet", name], cwd=release.root).strip()
    except CommandError:
        return None


def gh(release: Release, *args: str) -> list[str]:
    """A `gh` command against the release's repository."""
    return ["gh", *args, "--repo", release.repository]


def gh_json(host: Host, path: str) -> Any:
    """Read one GitHub REST resource through `gh`; None for a 404."""
    try:
        return json.loads(host.run(["gh", "api", path]))
    except CommandError as error:
        if "HTTP 404" in error.stderr:
            return None
        raise


def previous_version(host: Host, release: Release) -> str | None:
    """The highest released version below this one, from origin's `v*` tags."""
    output = host.run(["git", "ls-remote", "--tags", "--refs", "origin", "v*"], cwd=release.root)
    current = version_key(release.version)
    assert current is not None
    earlier = []
    for line in output.splitlines():
        name = line.split("\t", 1)[-1]
        if not name.startswith("refs/tags/v"):
            continue
        key = version_key(name.removeprefix("refs/tags/v"))
        if key is not None and key < current:
            earlier.append((key, name.removeprefix("refs/tags/v")))
    return max(earlier)[1] if earlier else None


# --- Content checks -------------------------------------------------------------------


def cargo_versions(host: Host, release: Release) -> Check:
    """Every package manifest and workspace pin at the release commit names VERSION."""

    def field(path: str, *keys: str) -> str:
        text = show(host, release, path)
        if text is None:
            return "missing"
        try:
            value: Any = tomllib.loads(text)
            for key in keys:
                value = value[key]
        except (KeyError, TypeError, ValueError):
            return "unreadable"
        return str(value)

    found = {manifest: field(manifest, "package", "version") for manifest in PACKAGE_MANIFESTS}
    for package in CRATE_PACKAGES:
        found[f"workspace {package}"] = field(
            "Cargo.toml", "workspace", "dependencies", package, "version"
        )
    wrong = {name: value for name, value in found.items() if value != release.version}
    detail = (
        f"all {len(found)} name {release.version}"
        if not wrong
        else ", ".join(f"{name} is {value}" for name, value in wrong.items())
    )
    return Check("Cargo versions at COMMIT", not wrong, detail)


def notes_problems(notes: str, version: str, previous: str | None, repository: str) -> list[str]:
    """
    Find what would make the release body wrong for this version.

    Notes start as a copy of the previous release's, so a link still pinned to the old
    tag, or a compare link from the wrong base, is the likely leftover. The notes keep
    exactly one HTML comment, the guideline footer, as `release_body.py` requires.
    """
    tag = f"v{version}"
    slug = re.escape(repository)
    problems = []
    semver = r"v[0-9]+\.[0-9]+\.[0-9]+"
    # A link into the repository names the tag, never a branch such as `main` whose
    # content moves after the release.
    refs = set(re.findall(rf"github\.com/{slug}/(?:blob|tree)/([^/\s)#?]+)/", notes))
    refs |= set(re.findall(rf"github\.com/{slug}/releases/tag/([^/\s)#?]+)", notes))
    problems.extend(f"a link pins {other}, not {tag}" for other in sorted(refs - {tag}))
    compares = re.findall(rf"github\.com/{slug}/compare/({semver})\.\.\.({semver})", notes)
    expected = (f"v{previous}", tag) if previous is not None else None
    for pair in compares:
        if pair != expected and not (expected is None and pair[1] == tag):
            problems.append(f"compare link {pair[0]}...{pair[1]}")
    if expected is not None and expected not in compares:
        problems.append(f"no compare link {expected[0]}...{expected[1]}")
    comments = release_body.html_comment_count(notes)
    if comments != release_body.EXPECTED_HTML_COMMENTS:
        problems.append(f"{comments} HTML comments, where only the guideline footer belongs")
    return problems


def compare_base(host: Host, release: Release, previous: str | None) -> str | None:
    """
    The version the notes' compare link starts from: the highest earlier tag, or
    `--previous`, for a release after a version that was tagged but never shipped.
    """
    if previous is None:
        return previous_version(host, release)
    base = previous.removeprefix("v")
    key, current = version_key(base), version_key(release.version)
    if key is None or current is None or key >= current:
        raise StepError(f"--previous must be a version below {release.version}, got {previous}")
    ref = f"refs/tags/v{base}"
    if ref not in remote_refs(host, release, ref):
        raise StepError(f"--previous names v{base}, which origin has no tag for")
    return base


def notes_check(host: Host, release: Release, previous: str | None = None) -> Check:
    """The release notes exist at the commit and name this version's links."""
    notes = show(host, release, release.notes_path)
    if notes is None:
        return Check("release notes", False, f"{release.notes_path} is not at COMMIT")
    base = compare_base(host, release, previous)
    problems = notes_problems(notes, release.version, base, release.repository)
    return Check("release notes", not problems, "; ".join(problems) or release.notes_path)


def changelog_check(host: Host, release: Release) -> Check:
    """The CHANGELOG at the commit has a dated section for this version."""
    changelog = show(host, release, "CHANGELOG.md") or ""
    heading = re.search(
        rf"(?m)^## \[{re.escape(release.version)}\] - [0-9]{{4}}-[0-9]{{2}}-[0-9]{{2}}$", changelog
    )
    detail = heading[0] if heading else f"no `## [{release.version}] - YYYY-MM-DD` section"
    return Check("CHANGELOG", heading is not None, detail)


def unpublished_checks(host: Host, release: Release) -> list[Check]:
    """
    Each registry must answer 404 for this version.

    A name check proves nothing after the first release, since the names exist; the
    version is what a pushed tag commits to.
    """
    urls = {
        f"crates.io {package} {release.version}": (
            f"https://crates.io/api/v1/crates/{package}/{release.version}"
        )
        for package in CRATE_PACKAGES
    }
    urls[f"PyPI fdu {release.version}"] = f"https://pypi.org/pypi/fdu/{release.version}/json"
    checks = []
    for name, url in urls.items():
        try:
            body = host.fetch(url)
        except RegistryError as error:
            checks.append(Check(name, False, f"could not read: {error}"))
            continue
        checks.append(
            Check(name, body is None, "unpublished" if body is None else "already published")
        )
    return checks


def signing_key_fields(key: Path) -> tuple[str, str]:
    """The algorithm and key of an SSH public key file, refusing a private key."""
    fields = key.expanduser().read_text(encoding="utf-8").split()
    if len(fields) < 2 or not fields[0].startswith(("ssh-", "sk-", "ecdsa-")):
        raise StepError(f"SIGNING_KEY must name an SSH public key (.pub), not {key}")
    return fields[0], fields[1]


def signing_key_check(host: Host, key: Path | None) -> Check:
    """
    An optional SIGNING_KEY is a public key that GitHub lists as a signing key.

    A tag signature is optional (see `resolve_plan`), so an unset key passes and the tag
    goes out unsigned; a key that is set must be one GitHub will verify, since a tag signed
    with any other key is refused.
    """
    name = "signing key"
    if key is None:
        return Check(name, True, "SIGNING_KEY unset: the tag will be unsigned, which is allowed")
    try:
        fields = signing_key_fields(key)
    except (OSError, StepError) as error:
        return Check(name, False, str(error))
    login = host.run(["gh", "api", "user", "--jq", ".login"]).strip()
    listed = gh_json(host, f"users/{login}/ssh_signing_keys") or []
    registered = {tuple(str(item.get("key", "")).split()[:2]) for item in listed}
    ok = fields in registered
    return Check(
        name,
        ok,
        f"registered as a GitHub signing key of {login}"
        if ok
        else f"not among {login}'s GitHub signing keys; add it with `gh ssh-key add --type signing`",
    )


def token_check(host: Host, release: Release) -> Check:
    """
    No long-lived registry token waits in a secret.

    The publish job reads no registry secret: both registries publish through trusted
    publishing, and 0.1.0's bootstrap `CARGO_REGISTRY_TOKEN` path is gone. A token stored
    anyway is a standing credential that nothing needs, so it is deleted and revoked.
    """
    names: list[str] = []
    for path in (
        f"repos/{release.repository}/environments/{ENVIRONMENT}/secrets",
        f"repos/{release.repository}/actions/secrets",
    ):
        listing = gh_json(host, path)
        if not isinstance(listing, dict):
            return Check("registry secrets", False, f"could not list the secrets at {path}")
        names += [str(item.get("name")) for item in listing.get("secrets", [])]
    tokens = sorted(name for name in names if "CARGO" in name.upper() or "PYPI" in name.upper())
    if tokens:
        return Check("registry secrets", False, f"delete {', '.join(tokens)}: publishing uses OIDC")
    return Check("registry secrets", True, "none in the release environment or the repository")


def environment_check(host: Host, release: Release) -> Check:
    """The `release` environment is protected exactly as the publish job requires."""

    def read(url: str) -> Any:
        return gh_json(host, url.removeprefix("https://api.github.com/"))

    try:
        verified = publish_gate.check_environment(release.repository, ENVIRONMENT, read)
    except (ValueError, CommandError) as error:
        return Check("release environment", False, str(error))
    return Check("release environment", True, "; ".join(verified))


# --- Steps ----------------------------------------------------------------------------


def on_main_check(host: Host, release: Release) -> Check:
    """The release commit is reviewed history: an ancestor of origin's main."""
    host.run(["git", "fetch", "--quiet", "origin", "main"], cwd=release.root)
    try:
        host.run(
            ["git", "merge-base", "--is-ancestor", release.commit, "origin/main"], cwd=release.root
        )
    except CommandError:
        return Check("COMMIT on origin/main", False, f"{release.commit} is not on origin/main")
    return Check("COMMIT on origin/main", True, release.commit)


def tag_absent_check(host: Host, release: Release) -> Check:
    """No release tag exists yet on origin."""
    tag = remote_tag(host, release)
    detail = "not on origin" if tag is None else f"already on origin at {tag[1]}"
    return Check(f"tag {release.tag}", tag is None, detail)


def reporting_check(host: Host, release: Release) -> Check:
    """SECURITY.md and the notes send reporters to GitHub's private reporting form."""
    path = f"repos/{release.repository}/private-vulnerability-reporting"
    enabled = host.run(["gh", "api", path, "--jq", ".enabled"]).strip()
    return Check("private vulnerability reporting", enabled == "true", f"enabled: {enabled}")


def preflight(
    host: Host, release: Release, signing_key: Path | None, *, previous: str | None = None
) -> list[Check]:
    """Read-only readiness checks before anything is dispatched, tagged, or published."""
    probes: list[tuple[str, Callable[[], Check | list[Check]]]] = [
        ("COMMIT on origin/main", lambda: on_main_check(host, release)),
        ("Cargo versions at COMMIT", lambda: cargo_versions(host, release)),
        ("release notes", lambda: notes_check(host, release, previous)),
        ("CHANGELOG", lambda: changelog_check(host, release)),
        (f"tag {release.tag}", lambda: tag_absent_check(host, release)),
        ("registries", lambda: unpublished_checks(host, release)),
        ("private vulnerability reporting", lambda: reporting_check(host, release)),
        ("release environment", lambda: environment_check(host, release)),
        ("registry secrets", lambda: token_check(host, release)),
        ("signing key", lambda: signing_key_check(host, signing_key)),
    ]
    checks: list[Check] = []
    for name, probe in probes:
        # One unreadable source fails its own line, not the whole checklist.
        try:
            result = probe()
        except (CommandError, RegistryError, StepError) as error:
            result = Check(name, False, str(error))
        checks.extend(result if isinstance(result, list) else [result])
    return checks


def pin_branch(host: Host, release: Release) -> None:
    """Create `release/vX.Y.Z` at the commit on origin, or confirm it is already there."""
    ref = f"refs/heads/{release.branch}"
    current = remote_refs(host, release, ref).get(ref)
    if current == release.commit:
        print(f"{release.branch} already names {release.commit}")
        return
    if current is not None:
        raise StepError(
            f"origin's {release.branch} names {current}, not {release.commit}; "
            "delete it with the cleanup step's --abandon, or rehearse the commit it names"
        )
    # The empty lease makes the push create the branch or fail: it never moves one.
    host.run(
        ["git", "push", f"--force-with-lease={ref}:", "origin", f"{release.commit}:{ref}"],
        cwd=release.root,
    )
    print(f"pinned {release.branch} at {release.commit}")


def workflow_runs(host: Host, release: Release, branch: str) -> list[dict[str, Any]]:
    """The release workflow's dispatched runs on one branch or tag, newest first."""
    fields = "databaseId,displayTitle,headSha,status,conclusion,createdAt"
    filters = ["--workflow", WORKFLOW, "--branch", branch, "--event", "workflow_dispatch"]
    runs: list[dict[str, Any]] = json.loads(
        host.run(gh(release, "run", "list", *filters, "--limit", "50", "--json", fields))
    )
    return runs


def dispatch_rehearsal(
    host: Host, release: Release, *, timeout: float = 120.0, interval: float = 5.0
) -> int:
    """
    Dispatch a rehearsal on the pinned branch and return the new run's ID.

    No input is passed, so `publish` keeps its default of false: the run builds and
    inspects every artifact and its publish job is skipped.
    """
    before = {run["databaseId"] for run in workflow_runs(host, release, release.branch)}
    output = host.run(gh(release, "workflow", "run", WORKFLOW, "--ref", release.branch))
    printed = re.search(r"/actions/runs/([0-9]+)", output)
    if printed is not None:
        return int(printed[1])
    waited = 0.0
    while True:
        fresh = [
            run
            for run in workflow_runs(host, release, release.branch)
            if run["databaseId"] not in before and run["headSha"] == release.commit
        ]
        if len(fresh) == 1:
            return int(fresh[0]["databaseId"])
        if len(fresh) > 1:
            ids = ", ".join(str(run["databaseId"]) for run in fresh)
            raise StepError(
                f"several new rehearsals appeared ({ids}); pass --run with the one to use"
            )
        if waited >= timeout:
            raise StepError(
                f"no rehearsal run appeared on {release.branch} within {timeout:g} seconds"
            )
        host.sleep(interval)
        waited += interval


def verify_run(
    host: Host, release: Release, run_id: int, *, publishing: bool, succeeded: bool = True
) -> None:
    """
    Check that a finished run built the release commit and did what its kind should.

    A rehearsal's publish job must have been skipped; a publishing run must be on the
    tag and, unless `succeeded` is false for a recovery audit, its publish job must have
    succeeded.
    """
    fields = "workflowName,event,headBranch,headSha,status,conclusion,displayTitle,jobs"
    view: dict[str, Any] = json.loads(
        host.run(gh(release, "run", "view", str(run_id), "--json", fields))
    )
    problems = []
    if view.get("workflowName") != WORKFLOW_NAME:
        problems.append(f"it is a {view.get('workflowName')} run, not {WORKFLOW_NAME}")
    if view.get("event") != "workflow_dispatch":
        problems.append(f"event is {view.get('event')}")
    if view.get("headSha") != release.commit:
        problems.append(f"it built {view.get('headSha')}, not COMMIT {release.commit}")
    if view.get("status") != "completed":
        problems.append(f"it is still {view.get('status')}")
    elif succeeded and view.get("conclusion") != "success":
        problems.append(f"it concluded {view.get('conclusion')}")
    title = str(view.get("displayTitle"))
    if publishing and title != f"Publish {release.tag}":
        problems.append(f"its title is {title!r}, not a publishing run's")
    if not publishing and not title.startswith("Release rehearsal on "):
        problems.append(f"its title is {title!r}, not a rehearsal's")
    publish = next((job for job in view.get("jobs", []) if job.get("name") == PUBLISH_JOB), None)
    publish_result = None if publish is None else publish.get("conclusion")
    if publishing:
        if view.get("headBranch") != release.tag:
            problems.append(f"it ran on {view.get('headBranch')}, not {release.tag}")
        if succeeded and publish_result != "success":
            problems.append(f"its publish job is {publish_result}")
    elif publish_result not in (None, "skipped"):
        problems.append(f"its publish job is {publish_result}, where a rehearsal skips it")
    if problems:
        raise StepError(f"run {run_id} ({release.run_url(run_id)}): " + "; ".join(problems))


def flatten(download: Path, files: Path, evidence: Path, evidence_artifact: str) -> None:
    """
    Move `gh run download`'s per-artifact directories into files and evidence.

    Exactly one evidence artifact, with the expected name, holds the manifest, the
    checksums, and the rehearsal-time registry audit; every other artifact holds
    release files. Nothing is deleted except the emptied download directories.
    """
    artifacts = sorted(download.iterdir())
    strays = [path.name for path in artifacts if not path.is_dir()]
    if strays:
        raise StepError(f"unexpected download entries: {', '.join(strays)}")
    names = [path.name for path in artifacts if path.name.startswith("release-evidence-")]
    if names != [evidence_artifact]:
        raise StepError(f"expected one evidence artifact {evidence_artifact}, found {names}")
    files.mkdir()
    evidence.mkdir()
    for artifact in artifacts:
        destination = evidence if artifact.name == evidence_artifact else files
        for path in sorted(artifact.rglob("*")):
            if path.is_dir():
                continue
            target = destination / path.name
            if target.exists():
                raise StepError(f"two artifacts both hold {path.name}")
            path.rename(target)
    for directory in sorted(download.rglob("*"), reverse=True):
        directory.rmdir()
    download.rmdir()
    held = sorted(path.name for path in evidence.iterdir())
    if held != sorted(EVIDENCE_FILES):
        raise StepError(f"the evidence artifact holds {held}, not {sorted(EVIDENCE_FILES)}")


def fetch_run(host: Host, release: Release, run_id: int, name: str, evidence_artifact: str) -> Path:
    """
    Download one run's artifacts into `$RELEASE/<name>` and verify them.

    The check is the publish job's own: the files are exactly the manifest's set, each
    with its recorded size and SHA-256, inspected again, and `SHA256SUMS` agrees.
    """
    target = release.directory / name
    marker = target / "run-id"
    if target.exists():
        if not marker.exists() or marker.read_text(encoding="utf-8").strip() != str(run_id):
            raise StepError(f"{target} exists and does not hold run {run_id}; move it aside first")
        print(f"{target} already holds run {run_id}; verifying it again")
    else:
        download = target / "download"
        download.mkdir(parents=True)
        host.run(gh(release, "run", "download", str(run_id), "--dir", str(download)))
        flatten(download, target / "files", target / "evidence", evidence_artifact)
        marker.write_text(f"{run_id}\n", encoding="utf-8")
    verify_kept(release, target)
    return target


def verify_kept(release: Release, target: Path) -> None:
    """Verify kept files against the manifest and checksums kept beside them."""
    try:
        verified = publish_gate.verify_files(
            target / "files",
            target / "evidence" / "release-manifest.json",
            target / "evidence" / "SHA256SUMS",
            release.version,
        )
    except ValueError as error:
        raise StepError(f"{target}: {error}") from error
    if len(verified) != RELEASE_FILE_COUNT:
        raise StepError(f"{target} holds {len(verified)} release files, not {RELEASE_FILE_COUNT}")
    for artifact in verified:
        print(f"verified {artifact.sha256}  {artifact.filename}")


def candidate(host: Host, release: Release, *, run_id: int | None, redispatch: bool) -> Path:
    """
    Rehearse the release commit on GitHub and keep its verified artifacts.

    The rehearsal runs on `release/vX.Y.Z`, pinned at the commit, because a dispatch
    takes a branch or tag rather than a commit and `main` may have moved on.
    """
    versions = cargo_versions(host, release)
    if not versions.ok:
        raise StepError(f"{versions.name}: {versions.detail}")
    state = load_state(release)
    if run_id is None and not redispatch and state.get("rehearsal_run"):
        run_id = int(state["rehearsal_run"])
        print(f"resuming rehearsal run {run_id} from {release.directory / 'state.json'}")
    if run_id is None:
        on_main = on_main_check(host, release)
        if not on_main.ok:
            raise StepError(f"{on_main.name}: {on_main.detail}; rehearse only reviewed history")
        pin_branch(host, release)
        run_id = dispatch_rehearsal(host, release)
    save_state(release, rehearsal_run=run_id)
    print(f"rehearsal: {release.run_url(run_id)}")
    watch = gh(release, "run", "watch", str(run_id), "--exit-status", "--interval", "30")
    if host.attach(watch) != 0:
        raise StepError(
            f"rehearsal run {run_id} failed. Rerun its failed jobs with "
            f"`gh run rerun {run_id} --failed` and repeat this step, or fix the cause "
            "and rehearse a new commit in a new RELEASE directory"
        )
    verify_run(host, release, run_id, publishing=False)
    target = fetch_run(
        host, release, run_id, "rehearsal", f"release-evidence-rehearsal-{release.commit[:9]}"
    )
    audit = json.loads((target / "evidence" / "registry-state.json").read_text(encoding="utf-8"))
    for entry in audit.get("registries", []):
        print(f"registry before publishing: {entry['channel']} {entry['package']} {entry['state']}")
    return target


def body(
    host: Host,
    release: Release,
    *,
    unwrap: Callable[[str], str] | None = None,
    previous: str | None = None,
) -> tuple[Path, Path]:
    """
    Derive the GitHub release body from the commit's notes and check GitHub's render.

    The notes are read from the release commit, not the working tree, so the body is the
    text the tag will name wherever this checkout points. Unwrapping uses the pinned
    flowmark `make docs-format` uses, from this checkout.
    """
    notes = show(host, release, release.notes_path)
    if notes is None:
        raise StepError(f"{release.notes_path} is not at COMMIT")
    base = compare_base(host, release, previous)
    problems = notes_problems(notes, release.version, base, release.repository)
    if problems:
        raise StepError(f"{release.notes_path}: " + "; ".join(problems))

    def flowmark(text: str) -> str:
        return release_body.unwrap_with_flowmark(text, root=release.root)

    source, text = release_body.derive_release_body(notes, unwrap=unwrap or flowmark)
    try:
        release_body.check_release_body(notes, source, text)
    except ValueError as error:
        raise StepError(f"{release.notes_path}: {error}") from error
    (release.directory / "notes-source.md").write_text(source, encoding="utf-8")
    notes_md = release.directory / "notes.md"
    notes_md.write_text(text, encoding="utf-8")
    context = f"context={release.repository}"
    html = host.run(
        ["gh", "api", "markdown", "-f", "mode=gfm", "-f", context, "-F", f"text=@{notes_md}"]
    )
    notes_html = release.directory / "notes.html"
    notes_html.write_text(html, encoding="utf-8")
    breaks = len(re.findall(r"<br\s*/?>", html))
    if breaks:
        raise StepError(
            f"GitHub renders {breaks} line break(s) inside a paragraph of {notes_md}; "
            "fix the notes in a new commit before tagging"
        )
    print(f"release body: {notes_md}")
    print(f"read it as GitHub renders it: {notes_html}")
    return notes_md, notes_html


def signature_check(host: Host, release: Release, key: Path | None) -> Check:
    """Verify the local tag's SSH signature against SIGNING_KEY alone, when one is set."""
    name = "tag signature"
    if key is None:
        return Check(name, True, "SIGNING_KEY unset: not checked here; GitHub's verdict follows")
    try:
        algorithm, material = signing_key_fields(key)
    except (OSError, StepError) as error:
        return Check(name, False, str(error))
    tagger = ["git", "for-each-ref", f"refs/tags/{release.tag}", "--format=%(taggeremail)"]
    email = host.run(tagger, cwd=release.root).strip().strip("<>")
    # A temporary allowed-signers file, so verification needs no global git config.
    with tempfile.TemporaryDirectory() as scratch:
        allowed = Path(scratch) / "allowed_signers"
        allowed.write_text(f'{email} namespaces="git" {algorithm} {material}\n', encoding="utf-8")
        try:
            output = host.run(
                ["git", "-c", f"gpg.ssh.allowedSignersFile={allowed}", "tag", "-v", release.tag],
                cwd=release.root,
                stderr=True,
            )
        except CommandError as error:
            return Check(name, False, f"does not verify against {key.name} for {email}: {error}")
    # Both the exit status and git's own verdict line, so neither alone decides.
    if f'Good "git" signature for {email}' not in output:
        return Check(name, False, f"git exited 0 without a good signature for {email}")
    return Check(name, True, f"good SSH signature by {key.name} for {email}")


def verify_tag(host: Host, release: Release, key: Path | None) -> list[Check]:
    """
    Check the release tag here and, once pushed, on origin and GitHub.

    Before the push it must be an annotated tag on COMMIT, and its signature must verify
    when SIGNING_KEY is set; after the push, origin must hold the same tag object and
    GitHub must report it verified or unsigned, the two verdicts the workflow accepts.
    """
    ref = f"refs/tags/{release.tag}"
    remote = remote_tag(host, release)
    local = rev_parse(host, release, ref)
    if local is None and remote is not None:
        host.run(["git", "fetch", "--quiet", "origin", f"{ref}:{ref}"], cwd=release.root)
        local = rev_parse(host, release, ref)
    if local is None:
        return [Check(f"tag {release.tag}", False, "not here and not on origin")]
    kind = host.run(["git", "cat-file", "-t", local], cwd=release.root).strip()
    target = rev_parse(host, release, f"{ref}^{{commit}}")
    subject = host.run(
        ["git", "for-each-ref", ref, "--format=%(contents:subject)"], cwd=release.root
    ).strip()
    checks = [
        Check("annotated tag", kind == "tag", f"{release.tag} is a {kind} object"),
        Check("tag names COMMIT", target == release.commit, str(target)),
        cargo_versions(host, release),
        Check("tag message", subject == f"fdu {release.version}", subject),
        signature_check(host, release, key),
    ]
    if remote is None:
        checks.append(Check("on origin", True, f"not pushed yet: git push origin {release.tag}"))
        return checks
    checks.append(
        Check(
            "on origin",
            remote == (local, release.commit),
            f"origin's {release.tag} is tag object {remote[0]} on {remote[1]}",
        )
    )
    record = gh_json(host, f"repos/{release.repository}/git/tags/{remote[0]}") or {}
    verification = record.get("verification") or {}
    reason = verification.get("reason")
    checks.append(
        Check(
            "GitHub verified",
            verification.get("verified") is True
            or (verification.get("verified") is False and reason == resolve_plan.UNSIGNED),
            f"reason: {reason}",
        )
    )
    return checks


def find_publishing_run(host: Host, release: Release) -> int:
    """The one publishing run dispatched on the release tag."""
    runs = [
        run
        for run in workflow_runs(host, release, release.tag)
        if run.get("displayTitle") == f"Publish {release.tag}"
    ]
    if not runs:
        raise StepError(f"no publishing run on {release.tag}; dispatch it as the guide describes")
    if len(runs) > 1:
        ids = ", ".join(str(run["databaseId"]) for run in runs)
        raise StepError(f"several publishing runs on {release.tag} ({ids}); pass --run")
    return int(runs[0]["databaseId"])


def announce_command(release: Release) -> list[str]:
    """The maintainer's `gh release create`, naming every file the release attaches."""
    title = f"fdu {release.version}"
    notes = str(release.directory / "notes.md")
    create = [
        "release",
        "create",
        release.tag,
        "--verify-tag",
        "--title",
        title,
        "--notes-file",
        notes,
    ]
    return [*gh(release, *create), *(str(path) for path in expected_assets(release).values())]


def require_pushed_tag(host: Host, release: Release) -> None:
    """Origin holds the release tag, on the release commit."""
    tag = remote_tag(host, release)
    if tag is None or tag[1] != release.commit:
        raise StepError(f"origin has no {release.tag} on COMMIT; tag and push it first")


def publishing_run(host: Host, release: Release, run_id: int | None) -> int:
    """The publishing run on the pushed tag: given, recorded, or the only one found."""
    require_pushed_tag(host, release)
    if run_id is None:
        recorded = load_state(release).get("publish_run")
        run_id = int(recorded) if recorded else find_publishing_run(host, release)
    save_state(release, publish_run=run_id)
    print(f"publishing run: {release.run_url(run_id)}")
    return run_id


def registry_states(host: Host, release: Release, manifest: Path) -> list[RegistryState]:
    """Every registry's state for this version against one manifest, printed as JSON."""
    states = [
        *registry_state.crates_io_state(manifest, release.version, fetch=host.fetch),
        registry_state.pypi_state(manifest, release.version, fetch=host.fetch),
    ]
    print(registry_state.registry_document(release.version, states), end="")
    return states


def published(
    host: Host, release: Release, *, run_id: int | None, by_hand: bool = False
) -> list[str]:
    """
    Verify the publishing run and every registry, and prepare the announcement.

    Writes `$RELEASE/registry-state.json` only when every registry holds exactly the
    published manifest's files, and returns the `gh release create` command to run.
    `by_hand` audits the files a hand publication uploaded, already in
    `$RELEASE/published`, instead of a publishing run's.
    """
    if not (release.directory / "notes.md").exists():
        raise StepError("run the body step first: the announcement uses its notes.md")
    if by_hand:
        require_pushed_tag(host, release)
        target = release.directory / "published"
        if not (target / "files").is_dir() or not (target / "evidence").is_dir():
            raise StepError(
                f"{target} does not hold the uploaded files; Publishing by Hand puts them there"
            )
        verify_kept(release, target)
    else:
        run_id = publishing_run(host, release, run_id)
        verify_run(host, release, run_id, publishing=True)
        target = fetch_run(host, release, run_id, "published", f"release-evidence-{release.tag}")
    states = registry_states(host, release, target / "evidence" / "release-manifest.json")
    if registry_state.exit_status(states, require_identical=True) != 0:
        raise StepError(
            "not every registry holds exactly the published files; if a registry is only "
            "trailing, rerun this step in a few minutes, otherwise run the audit step and "
            "follow Recover From a Partial Publication"
        )
    document = registry_state.registry_document(release.version, states)
    (release.directory / "registry-state.json").write_text(document, encoding="utf-8")
    command = announce_command(release)
    print("every registry holds exactly the published files. Announce the release with:")
    print(shlex.join(command))
    return command


def audit(host: Host, release: Release, *, run_id: int | None) -> list[Check]:
    """
    Keep a publishing run's files and audit every registry against them, whatever the
    run's outcome: the first step of recovering from a partial publication.

    The files land in `$RELEASE/published`, which is what a rerun inspected and what
    publishing by hand uploads.
    """
    run_id = publishing_run(host, release, run_id)
    verify_run(host, release, run_id, publishing=True, succeeded=False)
    target = fetch_run(host, release, run_id, "published", f"release-evidence-{release.tag}")
    states = registry_states(host, release, target / "evidence" / "release-manifest.json")
    return [
        Check(
            f"{state.channel} {state.package}",
            state.state == "identical",
            f"{state.state}: {state.detail}",
        )
        for state in states
    ]


def expected_assets(release: Release) -> dict[str, Path]:
    """The eleven files a GitHub release attaches, by name."""
    published_dir = release.directory / "published"
    paths = [
        release.directory / "registry-state.json",
        published_dir / "evidence" / "release-manifest.json",
        published_dir / "evidence" / "SHA256SUMS",
        *sorted((published_dir / "files").iterdir()),
    ]
    return {path.name: path for path in paths}


def asset_check(release: Release, assets: Sequence[dict[str, Any]]) -> Check:
    """The release attaches exactly the local files, byte for byte where GitHub says."""
    expected = expected_assets(release)
    held = {str(asset["name"]): asset for asset in assets}
    problems = [f"missing {name}" for name in sorted(expected.keys() - held.keys())]
    problems += [f"unexpected {name}" for name in sorted(held.keys() - expected.keys())]
    for name in sorted(expected.keys() & held.keys()):
        path, asset = expected[name], held[name]
        if asset.get("size") != path.stat().st_size:
            problems.append(f"{name} size differs")
        digest = asset.get("digest")
        if digest and digest != f"sha256:{inspect_artifacts.digest(path)}":
            problems.append(f"{name} digest differs")
    return Check(
        "release assets",
        not problems,
        "; ".join(problems) or f"{len(expected)} files match {release.directory}",
    )


def installed_version(host: Host, release: Release, argv: Sequence[str]) -> Check:
    """Run an install command outside any checkout and compare its `--version`."""
    name = shlex.join(argv)
    try:
        output = host.run(argv, cwd=release.directory)
    except CommandError as error:
        return Check(name, False, str(error))
    first = output.strip().splitlines()[0] if output.strip() else ""
    return Check(name, first == f"fdu {release.version}", first or "no output")


def docs_rs_check(host: Host, release: Release, package: str) -> Check:
    """
    docs.rs has built the crate: pending until it has a status, failed if its build did.

    docs.rs builds from a queue, so a version published minutes ago has no status yet
    (a 404); that is pending, not a failure, and the step exits 3 so it can be rerun.
    """
    name = f"docs.rs {package}"
    builds = f"https://docs.rs/crate/{package}/{release.version}/builds"
    try:
        status = host.fetch(f"https://docs.rs/crate/{package}/{release.version}/status.json")
    except RegistryError as error:
        return Check(name, False, f"could not read: {error}")
    if status is None:
        return Check(name, False, "not built yet; rerun until docs.rs reports built", True)
    try:
        built = json.loads(status).get("doc_status") is True
    except (AttributeError, ValueError):
        return Check(name, False, f"unreadable status; see {builds}")
    return Check(name, built, "built" if built else f"build failed; see {builds}")


def announced(host: Host, release: Release, *, cargo: bool) -> list[Check]:
    """What users see after the announcement: release, docs.rs, and installs."""
    needed = [
        release.directory / "notes.md",
        release.directory / "registry-state.json",
        release.directory / "published" / "files",
        release.directory / "published" / "evidence",
    ]
    missing = [str(path.relative_to(release.directory)) for path in needed if not path.exists()]
    if missing:
        detail = f"run the body and published steps first; missing {', '.join(missing)}"
        return [Check("release directory", False, detail)]
    record = gh_json(host, f"repos/{release.repository}/releases/tags/{release.tag}")
    if record is None:
        return [Check("GitHub release", False, f"no release on {release.tag} yet")]
    notes = (release.directory / "notes.md").read_text(encoding="utf-8").strip()
    same_body = str(record.get("body") or "").strip() == notes
    title = str(record.get("name"))
    checks = [
        Check(
            "GitHub release",
            not record.get("draft") and not record.get("prerelease"),
            f"draft={record.get('draft')}, prerelease={record.get('prerelease')}",
        ),
        Check("release title", title == f"fdu {release.version}", title),
        Check(
            "release body", same_body, "matches notes.md" if same_body else "differs from notes.md"
        ),
        asset_check(release, record.get("assets") or []),
    ]
    checks.extend(docs_rs_check(host, release, package) for package in CRATE_PACKAGES)
    # `--no-config` sets aside a user-level `exclude-newer` cool-off, which would hide a
    # release published minutes ago; `--no-build` makes the check test a wheel, never
    # the source distribution a free-threaded interpreter would fall back to.
    uv_tool = ["uv", "tool", "run", "--no-config", "--no-build", "--isolated"]
    uv_tool += ["--refresh-package", "fdu", "--python", INSTALL_PYTHON]
    for spec in (f"fdu@{release.version}", "fdu@latest"):
        checks.append(installed_version(host, release, [*uv_tool, spec, "--version"]))
    if cargo:
        root = release.directory / "cargo-user"
        install = ["cargo", "install", "--locked", "fdu", "--version", release.version]
        install += ["--root", str(root)]
        try:
            host.run(install, cwd=release.directory)
            checks.append(
                installed_version(host, release, [str(root / "bin" / "fdu"), "--version"])
            )
        except CommandError as error:
            checks.append(Check(shlex.join(install), False, str(error)))
    return checks


def cleanup(host: Host, release: Release, *, abandon: str | None) -> None:
    """
    Delete the rehearsal's `release/vX.Y.Z` branch once the tag has replaced it.

    The tag must name the commit the branch names. For a release that will not be tagged,
    `--abandon` must name the commit the branch holds, so the deletion is of a commit the
    maintainer has seen. The lease makes the push delete exactly that commit and nothing a
    later push moved the branch to.
    """
    ref = f"refs/heads/{release.branch}"
    branch = remote_refs(host, release, ref).get(ref)
    if branch is None:
        print(f"origin has no {release.branch}; nothing to delete")
        return
    print(f"origin's {release.branch} names {branch}")
    tag = remote_tag(host, release)
    if tag is None:
        if abandon is None:
            raise StepError(
                f"origin has no {release.tag}, so {release.branch} is still the only name for "
                f"the rehearsed commit; to delete it anyway, pass --abandon {branch}"
            )
        if len(abandon) < 7 or not branch.startswith(abandon.lower()):
            raise StepError(f"--abandon names {abandon}, but {release.branch} names {branch}")
    elif tag[1] != branch:
        raise StepError(f"{release.tag} names {tag[1]} but {release.branch} names {branch}")
    host.run(
        ["git", "push", f"--force-with-lease={ref}:{branch}", "origin", "--delete", release.branch],
        cwd=release.root,
    )
    print(f"deleted {release.branch} ({branch}) from origin")


# --- Command line ---------------------------------------------------------------------


def parser() -> argparse.ArgumentParser:
    """Build the command-line parser."""
    result = argparse.ArgumentParser(description=__doc__.split("\n\n")[0] if __doc__ else None)
    result.add_argument("--version", default=os.environ.get("VERSION"))
    result.add_argument("--commit", default=os.environ.get("COMMIT"))
    result.add_argument("--dir", default=os.environ.get("RELEASE"))
    result.add_argument("--repo", default=REPOSITORY)
    key = os.environ.get("SIGNING_KEY")
    result.add_argument("--signing-key", type=Path, default=Path(key) if key else None)
    steps = result.add_subparsers(dest="step", required=True)
    previous_help = "the version the notes compare from, after a version that never shipped"
    check = steps.add_parser("preflight", help="read-only readiness checks")
    check.add_argument("--previous", help=previous_help)
    rehearse = steps.add_parser("candidate", help="rehearse COMMIT on GitHub, keep its files")
    rehearse.add_argument("--run", type=int, help="use this rehearsal run instead of dispatching")
    rehearse.add_argument(
        "--redispatch", action="store_true", help="dispatch a new rehearsal of the same commit"
    )
    derive = steps.add_parser("body", help="derive and render-check the GitHub release body")
    derive.add_argument("--previous", help=previous_help)
    steps.add_parser("verify-tag", help="check the release tag here, on origin, and on GitHub")
    publish = steps.add_parser("published", help="verify the publishing run and every registry")
    publish.add_argument("--run", type=int, help="the publishing run, when several exist")
    publish.add_argument(
        "--by-hand", action="store_true", help="audit the hand-uploaded files in $RELEASE/published"
    )
    announce = steps.add_parser("announced", help="check the GitHub release, docs.rs, installs")
    announce.add_argument("--cargo", action="store_true", help="also build it with cargo install")
    clean = steps.add_parser("cleanup", help="delete the pinned rehearsal branch")
    clean.add_argument(
        "--abandon", metavar="COMMIT", help="delete it with no tag; name the commit it holds"
    )
    recover = steps.add_parser("audit", help="keep a publishing run's files, audit registries")
    recover.add_argument("--run", type=int, help="the publishing run, when several exist")
    return result


def main(
    argv: Sequence[str] | None = None, host: Host | None = None, cwd: Path | None = None
) -> int:
    """Run one step and map its outcome to an exit status."""
    args = parser().parse_args(argv)
    host = host or Host()
    try:
        release = resolve(
            host,
            version=args.version,
            commit=args.commit,
            directory=args.dir,
            repository=args.repo,
            cwd=cwd or Path.cwd(),
        )
        print(f"fdu {release.version} at {release.commit}, in {release.directory}")
        if args.step == "preflight":
            return report(preflight(host, release, args.signing_key, previous=args.previous))
        if args.step == "candidate":
            candidate(host, release, run_id=args.run, redispatch=args.redispatch)
        elif args.step == "body":
            body(host, release, previous=args.previous)
        elif args.step == "verify-tag":
            return report(verify_tag(host, release, args.signing_key))
        elif args.step == "published":
            published(host, release, run_id=args.run, by_hand=args.by_hand)
        elif args.step == "announced":
            return report(announced(host, release, cargo=args.cargo))
        elif args.step == "cleanup":
            cleanup(host, release, abandon=args.abandon)
        elif args.step == "audit":
            return report(audit(host, release, run_id=args.run))
    except (StepError, CommandError, RegistryError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
