"""
Tests for the maintainer's local release steps.

Every step runs against a scripted host that fails on any command it was not told to
expect, and refuses outright the commands only a maintainer may issue: pushing a tag,
dispatching a publishing run, creating or editing a GitHub release, and any registry
upload. So each test also proves the step it exercises never reaches them.
"""

from __future__ import annotations

import io
import json
import shutil
import subprocess
import tempfile
import unittest
from collections.abc import Callable, Sequence
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from typing import Any

from scripts.release import inspect_artifacts, maintainer
from scripts.release.maintainer import (
    Check,
    CommandError,
    Host,
    Release,
    StepError,
)
from scripts.release.registry_state import RegistryError
from tests.release.test_metadata import workflow_jobs
from tests.release.test_publish_gate import VERSION, record_evidence, write_crate, write_release_set

ROOT = Path(__file__).resolve().parents[2]
PREVIOUS = "0.1.0"
COMMIT = "6ec77163a8a1b5a33276bfb6dcaed7dff833d618"
OTHER = "0dec1d851a51cbd0e9b4f07cc53a4b6c6a38f6f8"
TAG_OBJECT = "1" * 40
REPO = "jlevy/fdu"
TAG = f"v{VERSION}"
BRANCH = f"release/v{VERSION}"
NOTES = (
    f"# fdu {VERSION}\n\n"
    "Report vulnerabilities as\n"
    f"[SECURITY.md](https://github.com/{REPO}/blob/{TAG}/SECURITY.md) describes.\n\n"
    f"**Full commit history**: [changes](https://github.com/{REPO}/compare/v{PREVIOUS}...{TAG})\n\n"
    "<!-- This document follows common-doc-guidelines.md.\n-->\n"
)

Response = str | Exception | Callable[[list[str]], str]


def forbidden(argv: Sequence[str]) -> str | None:
    """Name the maintainer-only write an argv would perform, if any."""
    command = list(argv)
    if command[:2] == ["git", "push"] and "refs/tags" in " ".join(command):
        return "pushing a tag"
    if command[:1] == ["git"] and "tag" in command and "-v" not in command:
        return "creating or moving a tag"
    if command[:3] == ["gh", "workflow", "run"] and {"-f", "-F", "--json"} & set(command):
        return "dispatching with inputs"
    if command[:2] == ["gh", "release"]:
        return "writing a GitHub release"
    writes = {"-X", "--method", "-f", "-F", "--field", "--raw-field", "--input"}
    if command[:2] == ["gh", "api"] and command[2:3] != ["markdown"] and writes & set(command):
        return "a GitHub API write"
    if command[:2] in (["cargo", "publish"], ["uv", "publish"], ["cargo", "yank"]):
        return "a registry write"
    if command[:2] in (["gh", "secret"], ["gh", "run"]) and command[2:3] in (["set"], ["rerun"]):
        return "a secret or a rerun"
    return None


class FakeHost(Host):
    """A host that answers only the commands and URLs a test scripts."""

    def __init__(self) -> None:
        self.calls: list[list[str]] = []
        self.attached: list[list[str]] = []
        self.handlers: list[tuple[tuple[str, ...], Response]] = []
        self.urls: dict[str, bytes | Exception | None] = {}
        self.attach_status = 0
        self.slept = 0.0

    def on(self, prefix: Sequence[str], response: Response) -> None:
        """Answer every command starting with `prefix`; the latest registration wins."""
        self.handlers.append((tuple(prefix), response))

    def run(self, argv: Sequence[str], *, cwd: Path | None = None, stderr: bool = False) -> str:
        command = list(argv)
        reason = forbidden(command)
        if reason is not None:
            raise AssertionError(f"{reason} is the maintainer's step: {command}")
        self.calls.append(command)
        for prefix, response in reversed(self.handlers):
            if tuple(command[: len(prefix)]) == prefix:
                if isinstance(response, Exception):
                    raise response
                return response(command) if callable(response) else response
        raise AssertionError(f"unexpected command: {command}")

    def attach(self, argv: Sequence[str], *, cwd: Path | None = None) -> int:
        reason = forbidden(list(argv))
        if reason is not None:
            raise AssertionError(f"{reason} is the maintainer's step: {list(argv)}")
        self.attached.append(list(argv))
        return self.attach_status

    def fetch(self, url: str) -> bytes | None:
        if url not in self.urls:
            raise AssertionError(f"unexpected URL: {url}")
        value = self.urls[url]
        if isinstance(value, Exception):
            raise value
        return value

    def sleep(self, seconds: float) -> None:
        self.slept += seconds

    def commands(self, *prefix: str) -> list[list[str]]:
        """Every recorded command starting with `prefix`."""
        return [call for call in self.calls if call[: len(prefix)] == list(prefix)]


def failure(stderr: str = "fatal: no such ref") -> CommandError:
    return CommandError(["fake"], 1, stderr)


def workspace_files(version: str = VERSION) -> dict[str, str]:
    """The files `git show COMMIT:<path>` answers for a ready release commit."""
    package = f'[package]\nname = "x"\nversion = "{version}"\n'
    return {
        "crates/fdu/Cargo.toml": package,
        "crates/fdu-core/Cargo.toml": package,
        "crates/fdu-py/Cargo.toml": package,
        "Cargo.toml": (
            "[workspace.dependencies]\n"
            f'fdu-core = {{ version = "{version}", path = "crates/fdu-core" }}\n'
            f'fdu = {{ version = "{version}", path = "crates/fdu" }}\n'
        ),
        f"docs/project/release-notes/{VERSION}.md": NOTES,
        "CHANGELOG.md": f"# Changelog\n\n## [Unreleased]\n\n## [{VERSION}] - 2026-09-28\n",
    }


class ReleaseCase(unittest.TestCase):
    """A scripted host and release directory for a ready, unpublished release."""

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.base = Path(self.temporary.name)
        self.release = Release(VERSION, COMMIT, self.base / "release", Path("/checkout"))
        self.release.directory.mkdir()
        self.host = FakeHost()
        self.files = workspace_files()
        self.remote: dict[str, str] = {}
        self.host.on(["git", "show"], self.git_show)
        self.host.on(["git", "ls-remote", "origin"], self.ls_remote)
        self.host.on(
            ["git", "ls-remote", "--tags", "--refs", "origin", "v*"],
            f"{OTHER}\trefs/tags/v{PREVIOUS}\n{OTHER}\trefs/tags/perf/v9.9.9\n",
        )
        self.output = io.StringIO()

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def git_show(self, argv: list[str]) -> str:
        commit, _, path = argv[2].partition(":")
        self.assertEqual(commit, COMMIT, "every file is read from the release commit")
        if path not in self.files:
            raise failure(f"fatal: path '{path}' does not exist in '{commit}'")
        return self.files[path]

    def ls_remote(self, argv: list[str]) -> str:
        return "".join(f"{self.remote[ref]}\t{ref}\n" for ref in argv[3:] if ref in self.remote)

    def quietly(self, step: Callable[[], Any]) -> Any:
        with redirect_stdout(self.output):
            return step()

    def push_tag(self, obj: str = TAG_OBJECT, commit: str = COMMIT) -> None:
        self.remote[f"refs/tags/{TAG}"] = obj
        self.remote[f"refs/tags/{TAG}^{{}}"] = commit


class NotesTests(unittest.TestCase):
    """The notes must name this version's links, as a copy of the last notes will not."""

    def test_current_notes_pass(self) -> None:
        self.assertEqual(maintainer.notes_problems(NOTES, VERSION, PREVIOUS, REPO), [])

    def test_links_left_on_the_previous_version_are_named(self) -> None:
        stale = NOTES.replace(TAG, f"v{PREVIOUS}").replace(f"v{PREVIOUS}...", "v0.0.9...")
        problems = maintainer.notes_problems(stale, VERSION, PREVIOUS, REPO)
        self.assertIn(f"a link pins v{PREVIOUS}, not {TAG}", problems)
        self.assertIn(f"compare link v0.0.9...v{PREVIOUS}", problems)
        self.assertIn(f"no compare link v{PREVIOUS}...{TAG}", problems)

    def test_a_first_release_needs_no_compare_link(self) -> None:
        first = NOTES.replace(f"https://github.com/{REPO}/compare/v{PREVIOUS}...{TAG}", "x")
        self.assertEqual(maintainer.notes_problems(first, VERSION, None, REPO), [])

    def test_a_leftover_draft_comment_is_named(self) -> None:
        draft = NOTES.replace("Report", "<!-- TODO highlights -->\n\nReport")
        self.assertEqual(
            maintainer.notes_problems(draft, VERSION, PREVIOUS, REPO),
            ["2 HTML comments, where only the guideline footer belongs"],
        )

    def test_a_link_to_a_moving_branch_is_named(self) -> None:
        moving = NOTES.replace(f"blob/{TAG}/SECURITY.md", "blob/main/SECURITY.md")
        self.assertEqual(
            maintainer.notes_problems(moving, VERSION, PREVIOUS, REPO),
            [f"a link pins main, not {TAG}"],
        )

    def test_the_previous_version_is_the_highest_tag_below_this_one(self) -> None:
        host = FakeHost()
        host.on(
            ["git", "ls-remote"],
            f"{OTHER}\trefs/tags/v0.1.0\n{OTHER}\trefs/tags/v0.10.0\n{OTHER}\trefs/tags/v0.2.0\n"
            f"{OTHER}\trefs/tags/v0.3.0\n{OTHER}\trefs/tags/vnext\n",
        )
        release = Release("0.2.1", COMMIT, Path("/release"), Path("/checkout"))
        self.assertEqual(maintainer.previous_version(host, release), "0.2.0")
        first = Release("0.0.1", COMMIT, Path("/release"), Path("/checkout"))
        self.assertIsNone(maintainer.previous_version(host, first))


class PreflightTests(ReleaseCase):
    """Preflight reads everything a tag would commit to, and writes nothing."""

    def setUp(self) -> None:
        super().setUp()
        self.key = self.base / "release-signing.pub"
        self.key.write_text("ssh-ed25519 AAAAkey maintainer@host\n", encoding="utf-8")
        self.host.on(["git", "fetch"], "")
        self.host.on(["git", "merge-base", "--is-ancestor"], "")
        for package in ("fdu-core", "fdu"):
            self.host.urls[f"https://crates.io/api/v1/crates/{package}/{VERSION}"] = None
        self.host.urls[f"https://pypi.org/pypi/fdu/{VERSION}/json"] = None
        self.host.on(["gh", "api", f"repos/{REPO}/private-vulnerability-reporting"], "true\n")
        self.host.on(["gh", "api", "user"], "maintainer\n")
        self.host.on(
            ["gh", "api", "users/maintainer/ssh_signing_keys"],
            json.dumps([{"key": "ssh-ed25519 AAAAkey", "title": "fdu release signing"}]),
        )
        environment = f"repos/{REPO}/environments/release"
        self.secrets: dict[str, list[dict[str, str]]] = {"environment": [], "repository": []}
        self.host.on(
            ["gh", "api", f"{environment}/secrets"],
            lambda _: json.dumps({"secrets": self.secrets["environment"]}),
        )
        self.host.on(
            ["gh", "api", f"repos/{REPO}/actions/secrets"],
            lambda _: json.dumps({"secrets": self.secrets["repository"]}),
        )
        self.host.on(
            ["gh", "api", environment],
            json.dumps(
                {
                    "protection_rules": [
                        {"type": "required_reviewers", "reviewers": [{"type": "User"}]}
                    ],
                    "can_admins_bypass": False,
                    "deployment_branch_policy": {
                        "protected_branches": False,
                        "custom_branch_policies": True,
                    },
                }
            ),
        )
        self.host.on(
            ["gh", "api", f"{environment}/deployment-branch-policies?per_page=100"],
            json.dumps({"total_count": 1, "branch_policies": [{"name": "v*", "type": "tag"}]}),
        )

    def preflight(self, key: Path | None = None) -> dict[str, Check]:
        checks = maintainer.preflight(self.host, self.release, key or self.key)
        return {check.name: check for check in checks}

    def failed(self, checks: dict[str, Check]) -> list[str]:
        return [name for name, check in checks.items() if not check.ok]

    def test_a_ready_release_passes_every_check_and_writes_nothing(self) -> None:
        checks = self.preflight()
        self.assertEqual(self.failed(checks), [])
        self.assertEqual(len(checks), 12)
        self.assertEqual(self.host.commands("git", "push"), [])
        self.assertEqual(self.host.commands("gh", "workflow"), [])

    def test_a_published_version_fails_its_own_line_only(self) -> None:
        self.host.urls[f"https://crates.io/api/v1/crates/fdu/{VERSION}"] = b"{}"
        checks = self.preflight()
        self.assertEqual(self.failed(checks), [f"crates.io fdu {VERSION}"])
        self.assertEqual(checks[f"crates.io fdu {VERSION}"].detail, "already published")

    def test_an_unreadable_registry_is_a_failure_not_a_verdict(self) -> None:
        self.host.urls[f"https://pypi.org/pypi/fdu/{VERSION}/json"] = RegistryError("HTTP 503")
        checks = self.preflight()
        self.assertEqual(self.failed(checks), [f"PyPI fdu {VERSION}"])
        self.assertIn("could not read", checks[f"PyPI fdu {VERSION}"].detail)

    def test_a_manifest_left_at_the_previous_version_fails(self) -> None:
        self.files["crates/fdu-py/Cargo.toml"] = workspace_files(PREVIOUS)[
            "crates/fdu-py/Cargo.toml"
        ]
        checks = self.preflight()
        self.assertEqual(self.failed(checks), ["Cargo versions at COMMIT"])
        self.assertIn(
            f"crates/fdu-py/Cargo.toml is {PREVIOUS}", checks["Cargo versions at COMMIT"].detail
        )

    def test_missing_notes_and_changelog_section_fail(self) -> None:
        del self.files[f"docs/project/release-notes/{VERSION}.md"]
        self.files["CHANGELOG.md"] = "## [Unreleased]\n"
        self.assertEqual(self.failed(self.preflight()), ["release notes", "CHANGELOG"])

    def test_a_commit_off_main_and_an_existing_tag_fail(self) -> None:
        self.host.on(["git", "merge-base", "--is-ancestor"], failure(""))
        self.push_tag()
        self.assertEqual(self.failed(self.preflight()), ["COMMIT on origin/main", f"tag {TAG}"])

    def test_an_unregistered_signing_key_fails(self) -> None:
        self.key.write_text("ssh-ed25519 AAAAother maintainer@host\n", encoding="utf-8")
        checks = self.preflight()
        self.assertEqual(self.failed(checks), ["signing key"])
        self.assertIn("gh ssh-key add --type signing", checks["signing key"].detail)

    def test_a_private_key_is_refused_without_being_echoed(self) -> None:
        self.key.write_text("-----BEGIN OPENSSH PRIVATE KEY-----\nsecret\n", encoding="utf-8")
        detail = self.preflight()["signing key"].detail
        self.assertIn("SSH public key", detail)
        self.assertNotIn("secret", detail)

    def test_an_unreadable_secret_listing_fails_rather_than_passing(self) -> None:
        self.host.on(
            ["gh", "api", f"repos/{REPO}/actions/secrets"], failure("gh: Not Found (HTTP 404)")
        )
        checks = self.preflight()
        self.assertEqual(self.failed(checks), ["registry secrets"])
        self.assertIn("could not list", checks["registry secrets"].detail)

    def test_the_compare_base_can_skip_a_version_that_never_shipped(self) -> None:
        burned = f"{OTHER}\trefs/tags/v{PREVIOUS}\n{OTHER}\trefs/tags/v0.1.5\n"
        self.host.on(["git", "ls-remote", "--tags", "--refs", "origin", "v*"], burned)
        self.remote[f"refs/tags/v{PREVIOUS}"] = OTHER
        checks = self.preflight()
        self.assertEqual(self.failed(checks), ["release notes"])
        self.assertIn(f"no compare link v0.1.5...{TAG}", checks["release notes"].detail)
        with_base = maintainer.preflight(self.host, self.release, self.key, previous=PREVIOUS)
        self.assertEqual([check.name for check in with_base if not check.ok], [])
        for previous, message in (("0.0.9", "origin has no tag"), ("0.3.0", "below")):
            checks = {
                check.name: check
                for check in maintainer.preflight(
                    self.host, self.release, self.key, previous=previous
                )
            }
            self.assertIn(message, checks["release notes"].detail)

    def test_a_leftover_registry_token_fails(self) -> None:
        self.secrets["environment"] = [{"name": "CARGO_REGISTRY_TOKEN"}]
        self.secrets["repository"] = [{"name": "PYPI_API_TOKEN"}, {"name": "UNRELATED"}]
        checks = self.preflight()
        self.assertEqual(self.failed(checks), ["registry secrets"])
        self.assertIn("CARGO_REGISTRY_TOKEN, PYPI_API_TOKEN", checks["registry secrets"].detail)

    def test_an_unprotected_environment_fails(self) -> None:
        self.host.on(
            ["gh", "api", f"repos/{REPO}/environments/release"],
            failure("gh: Not Found (HTTP 404)"),
        )
        checks = self.preflight()
        self.assertEqual(self.failed(checks), ["release environment"])
        self.assertIn("does not exist", checks["release environment"].detail)


def write_download(directory: Path, evidence_artifact: str) -> None:
    """Lay out a run's artifacts as `gh run download` does: one directory per artifact."""
    staging = directory.parent / "staging"
    staging.mkdir()
    write_release_set(staging)
    evidence = directory / evidence_artifact
    evidence.mkdir(parents=True)
    record_evidence(staging, evidence)
    (evidence / "registry-state.json").write_text(
        json.dumps(
            {
                "version": VERSION,
                "registries": [
                    {
                        "channel": "pypi",
                        "package": "fdu",
                        "state": "missing",
                        "detail": "",
                        "version": VERSION,
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    for path in sorted(staging.iterdir()):
        artifact = {".crate": "release-crate", ".gz": "release-sdist"}.get(
            path.suffix, f"release-wheel-{path.name.rsplit('-', 1)[-1].removesuffix('.whl')}"
        )
        (directory / artifact).mkdir(exist_ok=True)
        shutil.move(path, directory / artifact / path.name)
    staging.rmdir()


class CandidateTests(ReleaseCase):
    """The rehearsal runs on a pinned branch, never publishes, and its files verify."""

    def setUp(self) -> None:
        super().setUp()
        self.runs: list[list[dict[str, Any]]] = [[], [{"databaseId": 42, "headSha": COMMIT}]]
        self.jobs = [{"name": maintainer.PUBLISH_JOB, "conclusion": "skipped"}]
        self.view: dict[str, Any] = {
            "event": "workflow_dispatch",
            "headBranch": BRANCH,
            "headSha": COMMIT,
            "status": "completed",
            "conclusion": "success",
            "displayTitle": f"Release rehearsal on {BRANCH}",
            "workflowName": "Release",
        }
        self.evidence = f"release-evidence-rehearsal-{COMMIT[:9]}"
        self.host.on(["git", "push"], "")
        self.host.on(["git", "fetch"], "")
        self.host.on(["git", "merge-base", "--is-ancestor"], "")
        self.host.on(["gh", "run", "list"], lambda _argv: json.dumps(self.runs.pop(0)))
        self.host.on(["gh", "workflow", "run"], "Created workflow_dispatch event for release.yml\n")
        self.host.on(
            ["gh", "run", "view"], lambda _argv: json.dumps({**self.view, "jobs": self.jobs})
        )
        self.host.on(["gh", "run", "download"], self.download)

    def download(self, argv: list[str]) -> str:
        write_download(Path(argv[argv.index("--dir") + 1]), self.evidence)
        return ""

    def candidate(self, *, run_id: int | None = None, redispatch: bool = False) -> Path:
        return self.quietly(
            lambda: maintainer.candidate(
                self.host, self.release, run_id=run_id, redispatch=redispatch
            )
        )

    def test_it_pins_dispatches_watches_verifies_and_keeps_the_files(self) -> None:
        target = self.candidate()
        self.assertEqual(
            self.host.commands("git", "push"),
            [
                [
                    "git",
                    "push",
                    f"--force-with-lease=refs/heads/{BRANCH}:",
                    "origin",
                    f"{COMMIT}:refs/heads/{BRANCH}",
                ]
            ],
        )
        self.assertEqual(
            self.host.commands("gh", "workflow", "run"),
            [["gh", "workflow", "run", "release.yml", "--ref", BRANCH, "--repo", REPO]],
        )
        self.assertEqual(self.host.attached[0][:4], ["gh", "run", "watch", "42"])
        self.assertIn("--exit-status", self.host.attached[0])
        self.assertEqual(len(list((target / "files").iterdir())), 8)
        self.assertEqual(
            sorted(path.name for path in (target / "evidence").iterdir()),
            sorted(maintainer.EVIDENCE_FILES),
        )
        self.assertFalse((target / "download").exists())
        state = json.loads((self.release.directory / "state.json").read_text(encoding="utf-8"))
        self.assertEqual(state, {"commit": COMMIT, "rehearsal_run": 42, "version": VERSION})

    def test_the_run_gh_prints_is_used_without_polling(self) -> None:
        self.host.on(["gh", "workflow", "run"], f"https://github.com/{REPO}/actions/runs/77\n")
        self.candidate()
        self.assertEqual(self.host.attached[0][3], "77")
        self.assertEqual(len(self.host.commands("gh", "run", "list")), 1)

    def test_a_run_that_is_slow_to_appear_is_polled_for(self) -> None:
        self.runs = [[], [], [{"databaseId": 42, "headSha": COMMIT}]]
        self.candidate()
        self.assertEqual(self.host.slept, 5.0)

    def test_an_ambiguous_or_absent_new_run_is_never_guessed(self) -> None:
        # An earlier run on the branch, or a new one of another commit, is not this one.
        earlier = {"databaseId": 7, "headSha": COMMIT}
        self.runs = [[earlier], [earlier, {"databaseId": 8, "headSha": OTHER}]] + [[earlier]] * 30
        with self.assertRaisesRegex(StepError, "no rehearsal run appeared"):
            self.candidate()
        self.assertEqual(self.host.attached, [])
        self.runs = [
            [],
            [{"databaseId": 42, "headSha": COMMIT}, {"databaseId": 43, "headSha": COMMIT}],
        ]
        with self.assertRaisesRegex(StepError, r"several new rehearsals appeared \(42, 43\)"):
            self.candidate()

    def test_a_commit_off_main_is_not_pinned_or_dispatched(self) -> None:
        self.host.on(["git", "merge-base", "--is-ancestor"], failure(""))
        with self.assertRaisesRegex(StepError, "not on origin/main"):
            self.candidate()
        self.assertEqual(self.host.commands("git", "push"), [])
        self.assertEqual(self.host.commands("gh", "workflow"), [])

    def test_a_run_of_another_workflow_is_refused(self) -> None:
        self.view["workflowName"] = "CI"
        with self.assertRaisesRegex(StepError, "it is a CI run, not Release"):
            self.candidate()

    def test_a_branch_pinned_at_another_commit_stops_before_dispatch(self) -> None:
        self.remote[f"refs/heads/{BRANCH}"] = OTHER
        with self.assertRaisesRegex(StepError, f"names {OTHER}"):
            self.candidate()
        self.assertEqual(self.host.commands("gh", "workflow"), [])
        self.assertEqual(self.host.commands("git", "push"), [])

    def test_a_branch_already_at_the_commit_is_reused(self) -> None:
        self.remote[f"refs/heads/{BRANCH}"] = COMMIT
        self.candidate()
        self.assertEqual(self.host.commands("git", "push"), [])

    def test_a_rehearsal_whose_publish_job_ran_is_refused(self) -> None:
        self.jobs = [{"name": maintainer.PUBLISH_JOB, "conclusion": "success"}]
        with self.assertRaisesRegex(StepError, "where a rehearsal skips it"):
            self.candidate()

    def test_a_run_of_another_commit_is_refused(self) -> None:
        self.view["headSha"] = OTHER
        with self.assertRaisesRegex(StepError, f"it built {OTHER}"):
            self.candidate()

    def test_a_failed_rehearsal_names_the_rerun(self) -> None:
        self.host.attach_status = 1
        with self.assertRaisesRegex(StepError, "gh run rerun 42 --failed"):
            self.candidate()
        self.assertEqual(self.host.commands("gh", "run", "download"), [])

    def test_a_second_pass_resumes_the_recorded_run(self) -> None:
        self.candidate()
        pushes = len(self.host.commands("git", "push"))
        self.candidate()
        self.assertEqual(len(self.host.commands("git", "push")), pushes)
        self.assertEqual(len(self.host.commands("gh", "workflow", "run")), 1)
        self.assertEqual(len(self.host.commands("gh", "run", "download")), 1)
        self.assertIn("verifying it again", self.output.getvalue())

    def test_bytes_changed_after_the_run_are_refused(self) -> None:
        target = self.candidate()
        write_crate(target / "files", "fdu-core", b"tampered")
        with self.assertRaisesRegex(StepError, "differs from the manifest entry"):
            self.candidate()

    def test_evidence_from_another_run_is_refused(self) -> None:
        self.evidence = f"release-evidence-rehearsal-{OTHER[:9]}"
        with self.assertRaisesRegex(StepError, "expected one evidence artifact"):
            self.candidate()

    def test_a_directory_holding_another_run_is_refused(self) -> None:
        self.candidate()
        with self.assertRaisesRegex(StepError, "does not hold run 9"):
            self.candidate(run_id=9)

    def test_a_directory_of_another_commit_is_refused(self) -> None:
        self.candidate()
        moved = Release(VERSION, OTHER, self.release.directory, self.release.root)
        with self.assertRaisesRegex(StepError, "new RELEASE directory"):
            maintainer.load_state(moved)


class BodyTests(ReleaseCase):
    """The body is the commit's notes, unwrapped, and GitHub must render no line break."""

    def setUp(self) -> None:
        super().setUp()
        self.html = "<h1>fdu</h1>\n<p>Report vulnerabilities as SECURITY.md describes.</p>\n"
        self.host.on(["gh", "api", "markdown"], lambda _argv: self.html)

    def body(self) -> tuple[Path, Path]:
        return self.quietly(
            lambda: maintainer.body(self.host, self.release, unwrap=lambda text: text)
        )

    def test_it_writes_the_body_and_github_s_render_of_it(self) -> None:
        notes_md, notes_html = self.body()
        self.assertNotIn("<!--", notes_md.read_text(encoding="utf-8"))
        self.assertIn(f"compare/v{PREVIOUS}...{TAG}", notes_md.read_text(encoding="utf-8"))
        self.assertEqual(notes_html.read_text(encoding="utf-8"), self.html)
        render = self.host.commands("gh", "api", "markdown")[0]
        self.assertIn(f"text=@{notes_md}", render)
        self.assertIn("mode=gfm", render)

    def test_a_line_break_github_would_render_stops_it(self) -> None:
        self.html = "<p>Report vulnerabilities<br>\nas SECURITY.md describes.</p>\n"
        with self.assertRaisesRegex(StepError, "1 line break"):
            self.body()

    def test_stale_notes_stop_it_before_rendering(self) -> None:
        self.files[f"docs/project/release-notes/{VERSION}.md"] = NOTES.replace(TAG, f"v{PREVIOUS}")
        with self.assertRaisesRegex(StepError, f"a link pins v{PREVIOUS}"):
            self.body()
        self.assertEqual(self.host.commands("gh", "api", "markdown"), [])


class VerifyTagTests(ReleaseCase):
    """The tag is annotated, names the commit, verifies, and origin holds the same one."""

    def setUp(self) -> None:
        super().setUp()
        self.key = self.base / "release-signing.pub"
        self.key.write_text("ssh-ed25519 AAAAkey maintainer@host\n", encoding="utf-8")
        self.allowed: list[str] = []
        self.kind = "tag"
        self.target = COMMIT
        self.verified: dict[str, Any] = {"verified": True, "reason": "valid"}
        ref = f"refs/tags/{TAG}"
        self.host.on(["git", "rev-parse", "--verify", "--quiet", ref], f"{TAG_OBJECT}\n")
        self.host.on(
            ["git", "rev-parse", "--verify", "--quiet", f"{ref}^{{commit}}"],
            lambda _: f"{self.target}\n",
        )
        self.host.on(["git", "cat-file", "-t"], lambda _: f"{self.kind}\n")
        self.host.on(
            ["git", "for-each-ref", ref, "--format=%(contents:subject)"], f"fdu {VERSION}\n"
        )
        self.host.on(
            ["git", "for-each-ref", ref, "--format=%(taggeremail)"], "<maintainer@example.com>\n"
        )
        self.host.on(["git", "-c"], self.verify_signature)
        self.host.on(
            ["gh", "api", f"repos/{REPO}/git/tags/{TAG_OBJECT}"],
            lambda _: json.dumps({"verification": self.verified}),
        )

    def verify_signature(self, argv: list[str]) -> str:
        self.assertEqual(argv[3:], ["tag", "-v", TAG])
        self.allowed.append(Path(argv[2].split("=", 1)[1]).read_text(encoding="utf-8"))
        return 'Good "git" signature for maintainer@example.com with ED25519 key SHA256:x\n'

    def verify(self) -> dict[str, Check]:
        return {
            check.name: check for check in maintainer.verify_tag(self.host, self.release, self.key)
        }

    def test_an_unpushed_signed_tag_passes_and_names_the_push(self) -> None:
        checks = self.verify()
        self.assertTrue(all(check.ok for check in checks.values()), checks)
        self.assertEqual(checks["on origin"].detail, f"not pushed yet: git push origin {TAG}")
        self.assertEqual(
            self.allowed, ['maintainer@example.com namespaces="git" ssh-ed25519 AAAAkey\n']
        )

    def test_a_pushed_tag_must_be_the_same_object_and_verified_by_github(self) -> None:
        self.push_tag()
        checks = self.verify()
        self.assertTrue(all(check.ok for check in checks.values()), checks)
        self.push_tag(obj="2" * 40)
        self.verified = {"verified": False, "reason": "unknown_key"}
        self.host.on(
            ["gh", "api", f"repos/{REPO}/git/tags/{'2' * 40}"],
            lambda _: json.dumps({"verification": self.verified}),
        )
        failed = [name for name, check in self.verify().items() if not check.ok]
        self.assertEqual(failed, ["on origin", "GitHub verified"])

    def test_a_tag_on_a_commit_of_another_version_fails_before_the_push(self) -> None:
        self.files.update(
            {
                path: text
                for path, text in workspace_files(PREVIOUS).items()
                if path.endswith("Cargo.toml")
            }
        )
        failed = [name for name, check in self.verify().items() if not check.ok]
        self.assertEqual(failed, ["Cargo versions at COMMIT"])

    def test_a_lightweight_or_misplaced_tag_fails(self) -> None:
        self.kind = "commit"
        self.target = OTHER
        failed = [name for name, check in self.verify().items() if not check.ok]
        self.assertEqual(failed, ["annotated tag", "tag names COMMIT"])

    def test_a_bad_signature_fails(self) -> None:
        self.host.on(["git", "-c"], failure("No principal matched."))
        checks = self.verify()
        self.assertFalse(checks["tag signature"].ok)
        self.assertIn("No principal matched", checks["tag signature"].detail)

    def test_a_zero_exit_without_git_s_good_line_fails(self) -> None:
        self.host.on(["git", "-c"], "object 1111\ntype commit\n")
        checks = self.verify()
        self.assertFalse(checks["tag signature"].ok)
        self.assertIn("without a good signature", checks["tag signature"].detail)

    def test_no_signing_key_fails_rather_than_skipping(self) -> None:
        checks = {
            check.name: check for check in maintainer.verify_tag(self.host, self.release, None)
        }
        self.assertFalse(checks["tag signature"].ok)


@unittest.skipUnless(shutil.which("git") and shutil.which("ssh-keygen"), "needs git and ssh-keygen")
class SigningRecipeTests(unittest.TestCase):
    """The guide's per-command signing and this program's verification, with real git."""

    def test_a_tag_signed_per_command_verifies_only_against_its_key(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            base = Path(scratch)
            for name in ("signing", "other"):
                subprocess.run(
                    [
                        "ssh-keygen",
                        "-q",
                        "-t",
                        "ed25519",
                        "-N",
                        "",
                        "-C",
                        name,
                        "-f",
                        str(base / name),
                    ],
                    check=True,
                )
            origin, checkout = base / "origin.git", base / "checkout"
            git = ["git", "-c", "user.name=Maintainer", "-c", "user.email=maintainer@example.com"]
            subprocess.run(["git", "init", "-q", "--bare", str(origin)], check=True)
            subprocess.run(["git", "init", "-q", str(checkout)], check=True)
            for path, text in workspace_files().items():
                (checkout / path).parent.mkdir(parents=True, exist_ok=True)
                (checkout / path).write_text(text, encoding="utf-8")
            subprocess.run(["git", "-C", str(checkout), "add", "-A"], check=True)
            subprocess.run(
                [
                    *git,
                    "-C",
                    str(checkout),
                    "-c",
                    "commit.gpgsign=false",
                    "commit",
                    "-q",
                    "-m",
                    "c",
                ],
                check=True,
            )
            subprocess.run(
                ["git", "-C", str(checkout), "remote", "add", "origin", str(origin)], check=True
            )
            commit = subprocess.run(
                ["git", "-C", str(checkout), "rev-parse", "HEAD"],
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            # The release guide's tag command, with no global git configuration.
            subprocess.run(
                [
                    *git,
                    "-C",
                    str(checkout),
                    "-c",
                    "gpg.format=ssh",
                    "-c",
                    f"user.signingkey={base / 'signing.pub'}",
                    "tag",
                    "-s",
                    TAG,
                    "-m",
                    f"fdu {VERSION}",
                    commit,
                ],
                check=True,
            )
            release = Release(VERSION, commit, base / "release", checkout)
            good = {c.name: c for c in maintainer.verify_tag(Host(), release, base / "signing.pub")}
            self.assertTrue(all(check.ok for check in good.values()), good)
            wrong = {c.name: c for c in maintainer.verify_tag(Host(), release, base / "other.pub")}
            self.assertFalse(wrong["tag signature"].ok)


class PublishedTests(ReleaseCase):
    """The publishing run and every registry are verified before the announcement."""

    def setUp(self) -> None:
        super().setUp()
        self.push_tag()
        (self.release.directory / "notes.md").write_text("notes\n", encoding="utf-8")
        self.runs = [{"databaseId": 88, "displayTitle": f"Publish {TAG}", "headSha": COMMIT}]
        self.host.on(["gh", "run", "list"], lambda _: json.dumps(self.runs))
        self.view: dict[str, Any] = {
            "event": "workflow_dispatch",
            "headBranch": TAG,
            "headSha": COMMIT,
            "status": "completed",
            "conclusion": "success",
            "displayTitle": f"Publish {TAG}",
            "workflowName": "Release",
            "jobs": [{"name": maintainer.PUBLISH_JOB, "conclusion": "success"}],
        }
        self.host.on(["gh", "run", "view"], lambda _: json.dumps(self.view))
        self.host.on(["gh", "run", "download"], self.download)

    def download(self, argv: list[str]) -> str:
        directory = Path(argv[argv.index("--dir") + 1])
        write_download(directory, f"release-evidence-{TAG}")
        self.serve(directory / f"release-evidence-{TAG}" / "release-manifest.json")
        return ""

    def serve(self, path: Path) -> None:
        """Have both registries hold exactly the files `path` names."""
        manifest = json.loads(path.read_text(encoding="utf-8"))
        digests = {item["filename"]: item["sha256"] for item in manifest["artifacts"]}
        for package in ("fdu-core", "fdu"):
            record = {"version": {"checksum": digests[f"{package}-{VERSION}.crate"]}}
            self.host.urls[f"https://crates.io/api/v1/crates/{package}/{VERSION}"] = json.dumps(
                record
            ).encode()
        python = [
            {"filename": name, "digests": {"sha256": digest}}
            for name, digest in digests.items()
            if not name.endswith(".crate")
        ]
        self.host.urls[f"https://pypi.org/pypi/fdu/{VERSION}/json"] = json.dumps(
            {"urls": python}
        ).encode()

    def published(self, run_id: int | None = None) -> list[str]:
        return self.quietly(lambda: maintainer.published(self.host, self.release, run_id=run_id))

    def test_identical_registries_write_the_audit_and_the_announce_command(self) -> None:
        command = self.published()
        audit = json.loads(
            (self.release.directory / "registry-state.json").read_text(encoding="utf-8")
        )
        self.assertEqual({entry["state"] for entry in audit["registries"]}, {"identical"})
        self.assertEqual(command[:4], ["gh", "release", "create", TAG])
        attached = [arg for arg in command if arg.startswith(str(self.release.directory))]
        self.assertEqual(len(attached), 12)  # the notes file and eleven assets
        self.assertIn(str(self.release.directory / "registry-state.json"), attached)
        self.assertNotIn(
            str(self.release.directory / "published" / "evidence" / "registry-state.json"), attached
        )
        state = json.loads((self.release.directory / "state.json").read_text(encoding="utf-8"))
        self.assertEqual(state["publish_run"], 88)

    def test_a_registry_that_lacks_the_files_writes_no_audit(self) -> None:
        def missing_pypi(argv: list[str]) -> str:
            self.download(argv)
            self.host.urls[f"https://pypi.org/pypi/fdu/{VERSION}/json"] = None
            return ""

        self.host.on(["gh", "run", "download"], missing_pypi)
        with self.assertRaisesRegex(StepError, "Recover From a Partial Publication"):
            self.published()
        self.assertFalse((self.release.directory / "registry-state.json").exists())

    def test_a_hand_publication_is_audited_from_its_kept_files(self) -> None:
        with self.assertRaisesRegex(StepError, "Publishing by Hand puts them there"):
            self.quietly(
                lambda: maintainer.published(self.host, self.release, run_id=None, by_hand=True)
            )
        published = self.release.directory / "published"
        (published / "files").mkdir(parents=True)
        (published / "evidence").mkdir()
        write_release_set(published / "files")
        record_evidence(published / "files", published / "evidence")
        self.serve(published / "evidence" / "release-manifest.json")
        command = self.quietly(
            lambda: maintainer.published(self.host, self.release, run_id=None, by_hand=True)
        )
        self.assertEqual(command[:4], ["gh", "release", "create", TAG])
        self.assertTrue((self.release.directory / "registry-state.json").exists())
        self.assertEqual(self.host.commands("gh", "run"), [])

    def test_several_publishing_runs_need_an_explicit_choice(self) -> None:
        self.runs.append({"databaseId": 89, "displayTitle": f"Publish {TAG}", "headSha": COMMIT})
        with self.assertRaisesRegex(StepError, "pass --run"):
            self.published()
        self.published(run_id=88)

    def test_it_waits_for_the_tag_and_the_body(self) -> None:
        self.remote.clear()
        with self.assertRaisesRegex(StepError, "tag and push it first"):
            self.published()
        (self.release.directory / "notes.md").unlink()
        with self.assertRaisesRegex(StepError, "body step"):
            self.published()

    def test_a_failed_publishing_run_is_not_announced_but_can_be_audited(self) -> None:
        self.view["conclusion"] = "failure"
        self.view["jobs"] = [{"name": maintainer.PUBLISH_JOB, "conclusion": "failure"}]
        with self.assertRaisesRegex(StepError, "concluded failure; its publish job is failure"):
            self.published()
        self.assertEqual(self.host.commands("gh", "run", "download"), [])

        def partial_pypi(argv: list[str]) -> str:
            self.download(argv)
            url = f"https://pypi.org/pypi/fdu/{VERSION}/json"
            record = json.loads(self.host.urls[url] or b"{}")
            self.host.urls[url] = json.dumps({"urls": record["urls"][:2]}).encode()
            return ""

        self.host.on(["gh", "run", "download"], partial_pypi)
        checks = self.quietly(lambda: maintainer.audit(self.host, self.release, run_id=None))
        verdicts = {check.name: (check.ok, check.detail.split(":")[0]) for check in checks}
        self.assertEqual(
            verdicts,
            {
                "crates.io fdu-core": (True, "identical"),
                "crates.io fdu": (True, "identical"),
                "pypi fdu": (False, "conflict"),
            },
        )
        self.assertIn("missing: ", checks[-1].detail)
        self.assertTrue((self.release.directory / "published" / "files").is_dir())
        self.assertFalse((self.release.directory / "registry-state.json").exists())

    def test_a_run_still_in_progress_is_not_audited(self) -> None:
        self.view["status"] = "in_progress"
        with self.assertRaisesRegex(StepError, "still in_progress"):
            self.quietly(lambda: maintainer.audit(self.host, self.release, run_id=None))


class AnnouncedTests(ReleaseCase):
    """What users see: the release, docs.rs, and a fresh install."""

    def setUp(self) -> None:
        super().setUp()
        published = self.release.directory / "published"
        (published / "files").mkdir(parents=True)
        (published / "evidence").mkdir()
        write_release_set(published / "files")
        record_evidence(published / "files", published / "evidence")
        (self.release.directory / "registry-state.json").write_text("{}\n", encoding="utf-8")
        (self.release.directory / "notes.md").write_text("The notes.\n", encoding="utf-8")
        self.assets = [
            {
                "name": path.name,
                "size": path.stat().st_size,
                "digest": f"sha256:{inspect_artifacts.digest(path)}",
            }
            for path in maintainer.expected_assets(self.release).values()
        ]
        self.record: dict[str, Any] = {
            "draft": False,
            "prerelease": False,
            "name": f"fdu {VERSION}",
            "body": "The notes.",
        }
        self.host.on(
            ["gh", "api", f"repos/{REPO}/releases/tags/{TAG}"],
            lambda _: json.dumps({**self.record, "assets": self.assets}),
        )
        for package in ("fdu-core", "fdu"):
            self.host.urls[f"https://docs.rs/crate/{package}/{VERSION}/status.json"] = json.dumps(
                {"doc_status": True, "version": VERSION}
            ).encode()
        self.host.on(["uv", "tool", "run"], f"fdu {VERSION}\n")

    def announced(self) -> dict[str, Check]:
        return {c.name: c for c in maintainer.announced(self.host, self.release, cargo=False)}

    def failed(self) -> list[str]:
        return [name for name, check in self.announced().items() if not check.ok]

    def test_a_complete_announcement_passes(self) -> None:
        checks = self.announced()
        self.assertEqual([name for name, check in checks.items() if not check.ok], [])
        with redirect_stdout(io.StringIO()):
            self.assertEqual(maintainer.report(list(checks.values())), 0)
        self.assertEqual(len(self.assets), 11)
        installs = self.host.commands("uv", "tool", "run")
        self.assertEqual([call[-2] for call in installs], [f"fdu@{VERSION}", "fdu@latest"])
        self.assertTrue(all("--no-config" in call and "--no-build" in call for call in installs))

    def test_a_missing_or_altered_asset_fails(self) -> None:
        self.assets[0]["digest"] = "sha256:" + "0" * 64
        dropped = self.assets.pop()
        detail = self.announced()["release assets"].detail
        self.assertIn(f"missing {dropped['name']}", detail)
        self.assertIn(f"{self.assets[0]['name']} digest differs", detail)

    def test_a_docs_build_not_yet_run_is_pending_and_a_failed_one_fails(self) -> None:
        self.host.urls[f"https://docs.rs/crate/fdu/{VERSION}/status.json"] = None
        checks = self.announced()
        self.assertTrue(checks["docs.rs fdu"].pending)
        self.assertIn("rerun until docs.rs reports built", checks["docs.rs fdu"].detail)
        with redirect_stdout(io.StringIO()):
            self.assertEqual(maintainer.report(list(checks.values())), maintainer.PENDING_STATUS)
        self.host.urls[f"https://docs.rs/crate/fdu/{VERSION}/status.json"] = json.dumps(
            {"doc_status": False, "version": VERSION}
        ).encode()
        checks = self.announced()
        self.assertFalse(checks["docs.rs fdu"].ok or checks["docs.rs fdu"].pending)
        self.assertIn("build failed", checks["docs.rs fdu"].detail)
        with redirect_stdout(io.StringIO()):
            self.assertEqual(maintainer.report(list(checks.values())), 1)

    def test_an_old_install_fails_even_while_docs_are_pending(self) -> None:
        self.host.urls[f"https://docs.rs/crate/fdu/{VERSION}/status.json"] = None
        self.host.on(["uv", "tool", "run"], f"fdu {PREVIOUS}\n")
        failed = self.failed()
        self.assertEqual(len([name for name in failed if name.startswith("uv tool run")]), 2)
        with redirect_stdout(io.StringIO()):
            self.assertEqual(maintainer.report(list(self.announced().values())), 1)

    def test_a_draft_or_edited_release_fails(self) -> None:
        self.record.update(draft=True, body="Edited on GitHub.")
        self.assertEqual(self.failed(), ["GitHub release", "release body"])

    def test_announcing_before_publishing_is_a_failed_line(self) -> None:
        (self.release.directory / "registry-state.json").unlink()
        checks = maintainer.announced(self.host, self.release, cargo=False)
        self.assertEqual([check.name for check in checks], ["release directory"])
        self.assertIn("registry-state.json", checks[0].detail)
        self.assertEqual(self.host.calls, [])

    def test_no_release_yet_fails_once(self) -> None:
        self.host.on(["gh", "api", f"repos/{REPO}/releases/tags/{TAG}"], failure("HTTP 404"))
        self.assertEqual(self.failed(), ["GitHub release"])


class CleanupTests(ReleaseCase):
    """The pinned branch goes once the tag names its commit, and only then."""

    def setUp(self) -> None:
        super().setUp()
        self.host.on(["git", "push"], "")

    def cleanup(self, *, abandon: str | None = None) -> None:
        self.quietly(lambda: maintainer.cleanup(self.host, self.release, abandon=abandon))

    def test_the_branch_is_deleted_under_a_lease_once_the_tag_replaces_it(self) -> None:
        self.remote[f"refs/heads/{BRANCH}"] = COMMIT
        self.push_tag()
        self.cleanup()
        self.assertEqual(
            self.host.commands("git", "push"),
            [
                [
                    "git",
                    "push",
                    f"--force-with-lease=refs/heads/{BRANCH}:{COMMIT}",
                    "origin",
                    "--delete",
                    BRANCH,
                ]
            ],
        )

    def test_without_a_tag_the_branch_stays_unless_abandoned(self) -> None:
        self.remote[f"refs/heads/{BRANCH}"] = COMMIT
        with self.assertRaisesRegex(StepError, f"pass --abandon {COMMIT}"):
            self.cleanup()
        with self.assertRaisesRegex(StepError, f"--abandon names {OTHER[:9]}"):
            self.cleanup(abandon=OTHER[:9])
        with self.assertRaisesRegex(StepError, "--abandon names 6ec"):
            self.cleanup(abandon="6ec")
        self.assertEqual(self.host.commands("git", "push"), [])
        self.cleanup(abandon=COMMIT[:9])
        self.assertEqual(len(self.host.commands("git", "push")), 1)
        self.assertIn(f"deleted {BRANCH} ({COMMIT})", self.output.getvalue())

    def test_a_branch_the_tag_does_not_name_stays(self) -> None:
        self.remote[f"refs/heads/{BRANCH}"] = OTHER
        self.push_tag()
        with self.assertRaisesRegex(StepError, "names"):
            self.cleanup(abandon=OTHER)
        self.assertEqual(self.host.commands("git", "push"), [])

    def test_no_branch_is_nothing_to_do(self) -> None:
        self.cleanup()
        self.assertEqual(self.host.commands("git", "push"), [])


class ResolveTests(unittest.TestCase):
    """The identity is set once and checked before any step runs."""

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.base = Path(self.temporary.name)
        self.host = FakeHost()
        self.host.on(["git", "rev-parse", "--show-toplevel"], f"{self.base / 'checkout'}\n")
        self.host.on(["git", "rev-parse", "--verify"], f"{COMMIT}\n")

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def resolve(self, **overrides: str | None) -> Release:
        values: dict[str, str | None] = {
            "version": VERSION,
            "commit": COMMIT[:9],
            "directory": str(self.base / "release"),
        }
        values.update(overrides)
        return maintainer.resolve(
            self.host,
            version=values["version"],
            commit=values["commit"],
            directory=values["directory"],
            repository=REPO,
            cwd=self.base,
        )

    def test_a_short_commit_resolves_and_the_directory_is_created(self) -> None:
        release = self.resolve()
        self.assertEqual(release.commit, COMMIT)
        self.assertTrue(release.directory.is_dir())

    def test_unset_or_malformed_identity_is_refused(self) -> None:
        for overrides, message in (
            ({"version": None}, "set VERSION"),
            ({"version": "v0.2.1"}, "X.Y.Z"),
            ({"version": "0.2"}, "X.Y.Z"),
            ({"commit": None}, "set COMMIT"),
            ({"directory": None}, "set RELEASE"),
            ({"directory": str(self.base / "checkout" / "release")}, "outside the checkout"),
        ):
            with self.subTest(overrides=overrides), self.assertRaisesRegex(StepError, message):
                self.resolve(**overrides)

    def test_every_step_binds_the_directory_to_its_first_commit(self) -> None:
        release = self.resolve()
        state = json.loads((release.directory / "state.json").read_text(encoding="utf-8"))
        self.assertEqual(state, {"commit": COMMIT, "version": VERSION})
        self.host.on(["git", "rev-parse", "--verify"], f"{OTHER}\n")
        with self.assertRaisesRegex(StepError, "new RELEASE directory"):
            self.resolve(commit=OTHER[:9])
        stderr = io.StringIO()
        with redirect_stdout(io.StringIO()), redirect_stderr(stderr):
            status = maintainer.main(
                ["--version", VERSION, "--commit", OTHER, "--dir", str(release.directory), "body"],
                host=self.host,
                cwd=self.base,
            )
        self.assertEqual(status, 1)
        self.assertIn("new RELEASE directory", stderr.getvalue())

    def test_an_unknown_commit_says_to_fetch(self) -> None:
        self.host.on(["git", "rev-parse", "--verify"], failure(""))
        with self.assertRaisesRegex(StepError, "git fetch origin"):
            self.resolve()

    def test_the_command_line_reports_a_missing_identity_as_a_failure(self) -> None:
        stderr = io.StringIO()
        with redirect_stdout(io.StringIO()), redirect_stderr(stderr):
            status = maintainer.main(
                ["--version", VERSION, "--commit", "", "--dir", str(self.base), "preflight"],
                host=self.host,
                cwd=self.base,
            )
        self.assertEqual(status, 1)
        self.assertIn("set COMMIT", stderr.getvalue())


class WorkflowContractTests(unittest.TestCase):
    """The names the steps match on are the ones release.yml gives its runs and job."""

    def test_the_workflow_run_and_job_names_match_the_helper(self) -> None:
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        self.assertEqual(workflow.splitlines()[0], f"name: {maintainer.WORKFLOW_NAME}")
        run_name = " ".join(workflow.split("run-name: >-\n", 1)[1].split("\n\n", 1)[0].split())
        self.assertEqual(
            run_name,
            "${{ inputs.publish && format('Publish {0}', github.ref_name)"
            " || format('Release rehearsal on {0}', github.ref_name) }}",
        )
        publish = workflow_jobs(workflow)["publish"]
        self.assertEqual(publish.splitlines()[0], f"    name: {maintainer.PUBLISH_JOB}")

    def test_the_test_host_refuses_maintainer_writes_on_every_path(self) -> None:
        host = FakeHost()
        for argv in (
            ["gh", "release", "create", TAG],
            ["gh", "workflow", "run", "release.yml", "-f", "publish=true"],
            ["git", "push", "origin", f"refs/tags/{TAG}"],
        ):
            with self.subTest(argv=argv):
                with self.assertRaisesRegex(AssertionError, "maintainer's step"):
                    host.attach(argv)
                with self.assertRaisesRegex(AssertionError, "maintainer's step"):
                    host.run(argv)


class MakefileTests(unittest.TestCase):
    """Each step is a command-line choice and a Make target behind the uv version check."""

    def test_every_step_is_a_choice_and_a_target(self) -> None:
        for step in maintainer.STEPS:
            with self.subTest(step=step):
                self.assertEqual(maintainer.parser().parse_args([step]).step, step)
        makefile = (Path(__file__).resolve().parents[2] / "Makefile").read_text(encoding="utf-8")
        rule = next(line for line in makefile.splitlines() if line.startswith("release-preflight "))
        targets, prerequisites = rule.split(":", 1)
        self.assertEqual(targets.split(), [f"release-{step}" for step in maintainer.STEPS])
        self.assertEqual(prerequisites.split(), ["uv-version"])
        self.assertIn("scripts/release/maintainer.py $(patsubst release-%,%,$@)", makefile)


if __name__ == "__main__":
    unittest.main()
