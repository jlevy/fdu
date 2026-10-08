"""Repository-level release metadata invariants."""

from __future__ import annotations

import json
import re
import tempfile
import tomllib
import unittest
from pathlib import Path

from scripts.release import publish_gate

ROOT = Path(__file__).resolve().parents[2]


class MetadataTests(unittest.TestCase):
    """Version, naming, licensing, and rehearsal-authority checks."""

    def test_product_versions_and_names_have_one_identity(self) -> None:
        workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        crate = tomllib.loads((ROOT / "crates/fdu/Cargo.toml").read_text(encoding="utf-8"))
        python_crate = tomllib.loads(
            (ROOT / "crates/fdu-py/Cargo.toml").read_text(encoding="utf-8")
        )
        pyproject = tomllib.loads(
            (ROOT / "crates/fdu-py/pyproject.toml").read_text(encoding="utf-8")
        )
        version = crate["package"]["version"]
        self.assertEqual(version, "0.4.0")
        self.assertEqual(python_crate["package"]["version"], version)
        self.assertEqual(workspace["workspace"]["dependencies"]["fdu"]["version"], version)
        self.assertEqual(pyproject["project"]["name"], "fdu")
        self.assertEqual(pyproject["project"]["scripts"]["fdu"], "fdu:_main")
        self.assertEqual(pyproject["tool"]["maturin"]["module-name"], "fdu._native")

    def test_golden_generator_strings_name_the_product_version(self) -> None:
        # A version bump must move every golden's `generator`, in JSON and YAML spellings
        # alike; the parity artifact holds no YAML session, so nothing else catches a miss.
        version = tomllib.loads((ROOT / "crates/fdu/Cargo.toml").read_text(encoding="utf-8"))[
            "package"
        ]["version"]
        generator = re.compile(r'"?generator"?:\s*"fdu ([^"]+)"')
        seen = 0
        for golden in sorted((ROOT / "tests/golden").rglob("*")):
            if not golden.is_file():
                continue
            for number, line in enumerate(
                golden.read_text(encoding="utf-8", errors="replace").splitlines(), 1
            ):
                for found in generator.findall(line):
                    seen += 1
                    self.assertEqual(found, version, f"{golden.relative_to(ROOT)}:{number}")
        self.assertGreater(seen, 0)

    def test_artifact_license_copies_match_repository_license(self) -> None:
        expected = (ROOT / "LICENSE").read_bytes()
        self.assertEqual((ROOT / "crates/fdu-core/LICENSE").read_bytes(), expected)
        self.assertEqual((ROOT / "crates/fdu/LICENSE").read_bytes(), expected)
        self.assertEqual((ROOT / "crates/fdu-py/LICENSE").read_bytes(), expected)

    def test_runbook_derives_the_release_body_from_the_checked_in_script(self) -> None:
        runbook = (ROOT / "docs/project/guides/release-process.md").read_text(encoding="utf-8")
        self.assertIn("scripts/release/release_body.py", runbook)
        self.assertNotIn('re.sub(r"<!--.*?-->', runbook)

    def test_publication_authority_is_confined_to_the_gated_publish_job(self) -> None:
        # Every credential, OIDC grant, and registry write sits in the one job that names
        # the protected environment; every build job keeps the read-only default.
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        jobs = workflow_jobs(workflow)
        authority = (
            "id-token: write",
            "environment:",
            "steps.crates-io-auth.outputs.token",
            "crates-io-auth-action",
            "cargo publish",
            "uv publish",
        )
        for name, body in jobs.items():
            for marker in authority:
                with self.subTest(job=name, marker=marker):
                    if name == PUBLISH_JOB:
                        self.assertIn(marker, body)
                    else:
                        self.assertNotIn(marker, body)
        self.assertRegex(workflow, r"(?m)^permissions:\n  contents: read\n\n")
        # Only registry publication and announcement hold write authority.
        # Build jobs cannot push or move a tag.
        self.assertEqual(re.findall(r"(?i)\bwrite(?:-all)?\b", code(workflow)), ["write", "write"])
        self.assertNotIn("pull_request_target", workflow)
        self.assertNotIn("gh-action-pypi-publish", workflow)
        # No registry secret exists: the only secret read is the workflow's own read-only
        # GITHUB_TOKEN, for the supply-chain check. 0.1.0's bootstrap CARGO_REGISTRY_TOKEN
        # outranked OIDC whenever it was set, so a stale one would have published in place
        # of trusted publishing; 0.2.0 published through OIDC alone and the path is gone
        # (fdu-brkf). The minted crates.io token is read by the two uploads alone, never in
        # the job's `env`, where every action would see it.
        self.assertEqual(re.findall(r"secrets\.(\w+)", code(workflow)), ["GITHUB_TOKEN"])
        steps = workflow_steps(jobs[PUBLISH_JOB])
        token = "steps.crates-io-auth.outputs.token"
        holders = [name for name, text in steps.items() if token in text]
        self.assertEqual(holders, ["Publish fdu-core", "Publish fdu"])
        self.assertEqual(workflow.count(token), 2)
        for holder in holders:
            with self.subTest(step=holder):
                self.assertIn(f"CARGO_REGISTRY_TOKEN: ${{{{ {token} }}}}\n", steps[holder])

    def test_announcement_authority_and_dependencies(self) -> None:
        jobs = workflow_jobs((ROOT / ".github/workflows/release.yml").read_text())
        announce = jobs["announce"]
        self.assertIn("    needs: [plan, publish, notes]\n", announce)
        self.assertIn("    if: needs.plan.outputs.publish == 'true'\n", announce)
        self.assertIn("    permissions:\n      contents: write\n", announce)
        self.assertIn("scripts/release/announce.py", announce)
        self.assertNotRegex(announce, r"\b(?:pip|npm|cargo|uvx|make)\b")
        self.assertNotIn("--project", announce)
        for name, job in jobs.items():
            if name != "announce":
                self.assertNotIn("contents: write", job)
        notes = jobs["notes"]
        self.assertIn("scripts/release/maintainer.py body", notes)
        self.assertIn("PREVIOUS: ${{ inputs.previous }}", notes)
        self.assertIn('args+=(--previous "${PREVIOUS}")', notes)
        self.assertIn("name: announcement-notes-", notes)
        for artifact in (
            "announcement-notes-",
            "release-evidence-",
            "release-crate",
            "release-sdist",
            "release-wheel-*",
        ):
            self.assertIn(artifact, announce)
        self.assertNotIn("pattern: release-*", announce)

    def test_the_publish_job_runs_only_on_the_planned_tag_after_the_rehearsal(self) -> None:
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        publish = workflow_jobs(workflow)[PUBLISH_JOB]
        self.assertIn("    environment: release\n", publish)
        self.assertIn(
            "    permissions:\n      contents: read\n      id-token: write\n    env:\n", publish
        )
        self.assertIn(
            "    needs: [plan, crate, semver, sdist, wheels, evidence, notes, release-environment]\n",
            publish,
        )
        # The whole condition, exactly: a presence check on each clause would pass
        # `a || b`, `!startsWith(...)`, or `inputs.publish != true` while letting the job
        # run on a ref it must not.
        condition = publish.split("    if: >-\n", 1)[1].split("\n    runs-on:", 1)[0]
        self.assertEqual(
            " ".join(condition.split()),
            "inputs.publish"
            " && startsWith(github.ref, 'refs/tags/v')"
            " && needs.plan.outputs.publish == 'true'"
            " && github.ref == format('refs/tags/{0}', needs.plan.outputs.release_tag)",
        )
        # Nor may anything outlive a failure before it: a status function in any condition,
        # or a step or job that fails without failing what depends on it, would carry an
        # upload past a check, the environment check included. Expression functions are
        # case-insensitive, so `Always()` is refused as well.
        self.assertNotRegex(code(workflow), r"(?i)\b(?:always|cancelled|failure|success)\s*\(")
        self.assertNotRegex(code(workflow), r"(?i)continue-on-error")
        # A dispatch rehearses unless publishing is asked for explicitly, and nothing but
        # a dispatch runs the workflow at all. The trigger is compared whole, less its
        # prose, so `push: {}`, `push: # note`, or an inline mapping cannot slip in.
        self.assertEqual(len(re.findall(r"(?m)^[\"']?on[\"']?\s*:", workflow)), 1)
        trigger = re.search(r"(?ms)^on:\n(.*?)^(?=[^\s#])", workflow)
        assert trigger is not None
        shape, prose = [], False
        for line in trigger[1].splitlines():
            if line.startswith("        description:"):
                prose = True
            elif prose and line.startswith("          "):
                continue
            else:
                prose = False
                if line.strip() and not line.lstrip().startswith("#"):
                    shape.append(line)
        self.assertEqual(
            shape,
            [
                "  workflow_dispatch:",
                "    inputs:",
                "      publish:",
                "        type: boolean",
                "        default: false",
                "      previous:",
                "        type: string",
                '        default: ""',
            ],
        )

    def test_the_publish_jobs_guards_are_exactly_as_reviewed(self) -> None:
        # A presence check passes `|| true` on a comparison or a wait, `--package-dir
        # "${FILES}"` (the rehearsal crate compared with itself), `A && B || A` in an upload's
        # condition, or the OIDC exchange taken from another repository. So the steps that
        # guard or verify an upload, the plan step that validates the tag, and the job that
        # checks the environment are compared whole, whitespace and pinned revisions aside;
        # the step order is pinned so a wait or the final audit cannot be dropped. Editing
        # one in release.yml means editing it here as well.
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        jobs = workflow_jobs(workflow)
        publish = jobs[PUBLISH_JOB]
        self.assertEqual(step_keys(publish), PUBLISH_STEP_ORDER)
        for job, reviewed_steps in (
            (PUBLISH_JOB, REVIEWED_PUBLISH_STEPS),
            ("plan", REVIEWED_PLAN_STEPS),
        ):
            steps = workflow_steps(jobs[job])
            for name, reviewed in workflow_steps(reviewed_steps).items():
                with self.subTest(job=job, step=name):
                    self.assertEqual(flat(steps[name]), flat(reviewed))
        self.assertEqual(flat(jobs["release-environment"]), flat(REVIEWED_ENVIRONMENT_JOB))
        # Inside a folded `run: >-`, a line starting with `#` is not a YAML comment: it
        # folds into the command, where the shell reads it as a comment and drops every
        # argument after it, `--validate-checkout` included. The comparisons above skip
        # comment lines, so these jobs may hold none indented as deep as a command.
        raw = workflow_jobs(workflow, comments=True)
        for job in (PUBLISH_JOB, "release-environment", "plan"):
            with self.subTest(job=job):
                self.assertNotRegex(raw[job], r"(?m)^ {10,}#")

    def test_each_cargo_publish_waits_for_its_own_comparison(self) -> None:
        # The audit says what is missing, but an upload must also see its comparison step
        # run and pass. An audit output that came back empty (a renamed key, a mistyped
        # reference) skips the comparison, and that has to skip the upload as well.
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        steps = workflow_steps(workflow_jobs(workflow)[PUBLISH_JOB])
        for crate, reproduce, packaged in (
            ("fdu-core", "reproduce-both", ["fdu-core", "fdu"]),
            ("fdu", "reproduce-fdu", ["fdu"]),
        ):
            with self.subTest(crate=crate):
                comparisons = [text for text in steps.values() if f"id: {reproduce}\n" in text]
                self.assertEqual(len(comparisons), 1)
                # Exactly what is packaged and exactly what is compared: `--package fdu`
                # as a substring also matches `--package fdu-core`, which would let `fdu`
                # upload without a comparison of its own.
                self.assertEqual(
                    re.findall(r"(?m)^\s*(cargo package .*)$", comparisons[0]),
                    ["cargo package --locked --no-verify " + " ".join(f"-p {p}" for p in packaged)],
                )
                self.assertEqual(re.findall(r"--package (\S+)", comparisons[0]), packaged)
                upload = steps[f"Publish {crate}"]
                self.assertRegex(
                    upload,
                    rf"(?m)^\s*run: cargo publish --locked --no-verify -p {re.escape(crate)}$",
                )
                self.assertIn(f"&& steps.{reproduce}.outcome == 'success'", upload)

    def test_every_audit_output_the_publish_job_reads_is_one_the_gate_writes(self) -> None:
        # A step condition on an output the gate never writes is silently false, so the
        # names the workflow reads are held to the keys `audit` returns.
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        publish = workflow_jobs(workflow)[PUBLISH_JOB]
        # `-` is part of the name, so `outputs.fdu-core` is read as the key it names rather
        # than as `fdu`; the index form would dodge the pattern, so it is refused outright.
        referenced = set(re.findall(r"steps\.(?:audit|pypi)\.outputs\.([A-Za-z0-9_-]+)", publish))
        self.assertTrue(referenced)
        self.assertNotRegex(publish, r"\.outputs\s*\[")
        artifacts = [
            {"filename": "fdu-core-0.2.1.crate", "kind": "crate", "sha256": "a" * 64},
            {"filename": "fdu-0.2.1.crate", "kind": "crate", "sha256": "b" * 64},
            {"filename": "fdu-0.2.1.tar.gz", "kind": "sdist", "sha256": "c" * 64},
            {"filename": "fdu-0.2.1-cp312-abi3-win_amd64.whl", "kind": "wheel", "sha256": "d" * 64},
        ]
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / "manifest.json"
            manifest.write_text(
                json.dumps({"version": "0.2.1", "artifacts": artifacts}), encoding="utf-8"
            )
            written = publish_gate.audit(manifest, "0.2.1", lambda _url: None)
        self.assertLessEqual(referenced, written.keys())

    def test_the_publish_job_runs_no_dependency_code_and_uploads_only_rehearsed_bytes(
        self,
    ) -> None:
        # `--no-verify` keeps build scripts and proc macros away from the credentials; the
        # rehearsal's crate job already built and installed both crates.
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        publish = workflow_jobs(workflow)[PUBLISH_JOB]
        cargo = [line.strip() for line in publish.splitlines() if "cargo p" in line]
        self.assertTrue(cargo)
        for line in cargo:
            with self.subTest(line=line):
                self.assertIn("--locked --no-verify", line)
        self.assertEqual(publish.count("compare-crates"), 2)
        self.assertIn("publish_gate.py verify-files", publish)
        self.assertIn("uv publish\n          --trusted-publishing always\n", publish)
        self.assertIn("--check-url https://pypi.org/simple/", publish)
        # An allow-list rather than a deny-list: any build or test step would run dependency
        # code in a job whose OIDC token the pending PyPI publisher trusts. `uv run` is
        # allowed only as the exact no-project form that runs a release script on the
        # standard library, so a `--with`, a project, or a tool cannot ride in on it.
        self.assertEqual(set(re.findall(r"\bcargo\s+([a-z-]+)", publish)), {"package", "publish"})
        self.assertEqual(set(re.findall(r"\buvx?\s+([a-z-]+)", publish)), {"publish", "run"})
        runs = re.findall(r"\buv run\b[^\n]*", publish)
        self.assertTrue(runs)
        for run in runs:
            with self.subTest(run=run):
                self.assertRegex(run, rf"^{re.escape(UV_PYTHON)} scripts/release/[a-z_]+\.py\b")
        self.assertNotRegex(publish, r"\b(?:uvx|pip|pip3|npm|npx|make|rustc|maturin)\b")

    def test_every_release_script_runs_on_the_pinned_uv_python(self) -> None:
        # The runner image's `python3` is one image update from a different interpreter,
        # at the least recoverable moment; `make release-test` and `release-rehearse` use
        # uv's 3.12, so the workflow does too. A job that runs uv installs the pinned one
        # before its first use.
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        self.assertNotIn("python3", code(workflow))
        for line in code(workflow).splitlines():
            if "scripts/release/" in line or "tests/release" in line:
                with self.subTest(line=line.strip()):
                    self.assertIn(f"{UV_PYTHON} ", line)
        for name, job in workflow_jobs(workflow).items():
            steps = list(workflow_steps(job).values())
            first_run = next((i for i, step in enumerate(steps) if "uv run" in step), None)
            if first_run is None:
                continue
            with self.subTest(job=name):
                setup = next((i for i, s in enumerate(steps) if "astral-sh/setup-uv@" in s), None)
                self.assertIsNotNone(setup)
                assert setup is not None
                self.assertLess(setup, first_run)
        scripts = re.findall(r"\buv run --no-project --python 3\.12 python (\S+)", workflow)
        self.assertEqual(
            sorted(set(scripts)),
            [
                "-m",
                "scripts/release/announce.py",
                "scripts/release/inspect_artifacts.py",
                "scripts/release/maintainer.py",
                "scripts/release/publish_gate.py",
                "scripts/release/registry_state.py",
                "scripts/release/resolve_plan.py",
                "scripts/release/semver_check.py",
                "scripts/release/smoke_crate.py",
            ],
        )
        # The same suite, the same way, as `make release-test`.
        suite = "-m unittest discover -s tests/release -p 'test_*.py'"
        makefile = (ROOT / "Makefile").read_text(encoding="utf-8")
        self.assertIn(f"$(UV) {UV_PYTHON.removeprefix('uv ')} {suite}", makefile)
        self.assertIn(f"{UV_PYTHON} {suite}", workflow)

    def test_every_download_lands_its_files_directly_in_its_path(self) -> None:
        # The inspector and `verify-files` expect the eight files flat in one directory.
        # download-artifact extracts straight into `path` only for a download by `name`
        # or a `pattern` with `merge-multiple: true`; otherwise each artifact gets its own
        # subdirectory, and which case that is has changed between major versions. And
        # since v8 a digest mismatch fails the download, which only holds while no step
        # overrides `digest-mismatch` or skips decompression.
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        downloads = [
            step.split("\n      - ", 1)[0]
            for step in workflow.split("- uses: actions/download-artifact@")[1:]
        ]
        self.assertEqual(len(downloads), 10)
        for step in downloads:
            with self.subTest(step=step):
                self.assertRegex(step, r"(?m)^\s+path: \S.*$")
                named = re.search(r"(?m)^\s+name: \S.*$", step) is not None
                merged = re.search(r"(?m)^\s+pattern: \S.*$", step) is not None and (
                    re.search(r"(?m)^\s+merge-multiple: true$", step) is not None
                )
                self.assertTrue(named != merged, "exactly one of name or a merged pattern")
                self.assertNotRegex(
                    step, r"(?m)^\s+(?:artifact-ids|skip-decompress|digest-mismatch):"
                )

    def test_every_release_checkout_drops_its_credentials(self) -> None:
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        checkouts = workflow.split("uses: actions/checkout@")[1:]
        self.assertTrue(checkouts)
        for checkout in checkouts:
            self.assertIn("persist-credentials: false", checkout.split("\n      - ", 1)[0])

    def test_workflow_and_local_rehearsal_run_the_same_crate_smoke(self) -> None:
        # A plain `cargo install --locked` of the packaged `fdu` cannot resolve an
        # unpublished `fdu-core`; both paths must use the script that patches it (fdu-y5zc).
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        makefile = (ROOT / "Makefile").read_text(encoding="utf-8")
        rehearsal = makefile.split("\nrelease-rehearse:", 1)[1].split("\n\n", 1)[0]
        for text in (workflow, rehearsal):
            self.assertIn("scripts/release/smoke_crate.py", text)
        # The one `cargo install` is the reviewed semver tool; nothing installs `fdu` so.
        self.assertEqual(
            re.findall(r"cargo install[^\n]*", workflow),
            [f"cargo install --locked cargo-semver-checks --version {semver_tool_version()}"],
        )

    def test_a_patch_release_cannot_publish_past_the_semver_check(self) -> None:
        # The job compares both library crates with the pinned tool, and the publish job
        # needs it directly, so a patch that breaks the Rust API stops before any upload.
        # The local target runs the same script, so a maintainer sees the same verdict.
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        jobs = workflow_jobs(workflow)
        reviewed = REVIEWED_SEMVER_JOB.replace("<version>", semver_tool_version())
        self.assertEqual(flat(jobs["semver"]), flat(reviewed))
        self.assertRegex(jobs[PUBLISH_JOB], r"(?m)^    needs: \[[^\]]*\bsemver\b")
        makefile = (ROOT / "Makefile").read_text(encoding="utf-8")
        target = makefile.split("\nsemver-check:", 1)[1].split("\n\n", 1)[0]
        self.assertIn("python scripts/release/semver_check.py", target)


PUBLISH_JOB = "publish"

# How every release script runs, in the workflow as in the Makefile's release targets.
UV_PYTHON = "uv run --no-project --python 3.12 python"

# The publish job's steps in order, an action named without its pinned revision so that a
# reviewed action update does not read as a reordering.
PUBLISH_STEP_ORDER = [
    "actions/checkout",
    "astral-sh/setup-uv",
    "Confirm the checkout is the planned release tag",
    "Locate the rehearsal's files",
    "actions/download-artifact",
    "actions/download-artifact",
    "actions/download-artifact",
    "actions/download-artifact",
    "Verify the downloaded files against the manifest and SHA256SUMS",
    "Audit both registries before publishing",
    "dtolnay/rust-toolchain",
    "Reproduce both crates and compare them with the rehearsal",
    "Exchange GitHub OIDC for a short-lived crates.io token",
    "Publish fdu-core",
    "Wait until crates.io serves the rehearsed fdu-core",
    "Reproduce fdu against the published fdu-core and compare it",
    "Publish fdu",
    "Wait until crates.io serves the rehearsed fdu",
    "Audit PyPI before uploading",
    "Upload the rehearsed source distribution and wheels to PyPI",
    "Wait until PyPI serves exactly the rehearsed files",
    "Audit every registry against the manifest",
]

# The steps that guard or verify an upload, exactly as reviewed, compared with release.yml
# whitespace and pinned revisions aside (`scripts/check-supply-chain.mjs` checks those).
# Changing one there is a change to publishing safety, so it is made here too.
REVIEWED_PUBLISH_STEPS = """
      - name: Confirm the checkout is the planned release tag
        env:
          GITHUB_TOKEN: ${{ github.token }}
        run: >-
          uv run --no-project --python 3.12 python scripts/release/resolve_plan.py
          --root .
          --mode release
          --ref "${GITHUB_REF}"
          --commit "${GITHUB_SHA}"
          --repository "${GITHUB_REPOSITORY}"
          --validate-checkout
      - name: Verify the downloaded files against the manifest and SHA256SUMS
        run: >-
          uv run --no-project --python 3.12 python scripts/release/publish_gate.py verify-files "${FILES}"
          --manifest "${MANIFEST}"
          --checksums "${EVIDENCE}/SHA256SUMS"
          --version "${VERSION}"
      - name: Audit both registries before publishing
        id: audit
        run: >-
          uv run --no-project --python 3.12 python scripts/release/publish_gate.py audit
          --manifest "${MANIFEST}"
          --version "${VERSION}"
          --github-output "${GITHUB_OUTPUT}"
      - name: Reproduce both crates and compare them with the rehearsal
        id: reproduce-both
        if: steps.audit.outputs.crates == 'true'
        run: |
          cargo package --locked --no-verify -p fdu-core -p fdu
          uv run --no-project --python 3.12 python scripts/release/publish_gate.py compare-crates \\
            --package-dir target/package \\
            --manifest "${MANIFEST}" \\
            --version "${VERSION}" \\
            --package fdu-core \\
            --package fdu
      - name: Exchange GitHub OIDC for a short-lived crates.io token
        id: crates-io-auth
        if: steps.audit.outputs.crates == 'true'
        uses: rust-lang/crates-io-auth-action@<pinned>
      - name: Publish fdu-core
        if: steps.audit.outputs.fdu_core == 'missing' && steps.reproduce-both.outcome == 'success'
        env:
          CARGO_REGISTRY_TOKEN: ${{ steps.crates-io-auth.outputs.token }}
        run: cargo publish --locked --no-verify -p fdu-core
      - name: Wait until crates.io serves the rehearsed fdu-core
        run: >-
          uv run --no-project --python 3.12 python scripts/release/publish_gate.py wait-crate
          --manifest "${MANIFEST}"
          --version "${VERSION}"
          --package fdu-core
      - name: Reproduce fdu against the published fdu-core and compare it
        id: reproduce-fdu
        if: steps.audit.outputs.fdu == 'missing'
        run: |
          cargo package --locked --no-verify -p fdu
          uv run --no-project --python 3.12 python scripts/release/publish_gate.py compare-crates \\
            --package-dir target/package \\
            --manifest "${MANIFEST}" \\
            --version "${VERSION}" \\
            --package fdu
      - name: Publish fdu
        if: steps.audit.outputs.fdu == 'missing' && steps.reproduce-fdu.outcome == 'success'
        env:
          CARGO_REGISTRY_TOKEN: ${{ steps.crates-io-auth.outputs.token }}
        run: cargo publish --locked --no-verify -p fdu
      - name: Wait until crates.io serves the rehearsed fdu
        run: >-
          uv run --no-project --python 3.12 python scripts/release/publish_gate.py wait-crate
          --manifest "${MANIFEST}"
          --version "${VERSION}"
          --package fdu
      - name: Audit PyPI before uploading
        id: pypi
        run: >-
          uv run --no-project --python 3.12 python scripts/release/publish_gate.py audit
          --manifest "${MANIFEST}"
          --version "${VERSION}"
          --github-output "${GITHUB_OUTPUT}"
      - name: Upload the rehearsed source distribution and wheels to PyPI
        if: steps.pypi.outputs.upload == 'true'
        run: >-
          uv publish
          --trusted-publishing always
          --check-url https://pypi.org/simple/
          "${FILES}/fdu-${VERSION}.tar.gz"
          "${FILES}"/fdu-${VERSION}-*.whl
      - name: Wait until PyPI serves exactly the rehearsed files
        run: >-
          uv run --no-project --python 3.12 python scripts/release/publish_gate.py wait-pypi
          --manifest "${MANIFEST}"
          --version "${VERSION}"
      - name: Audit every registry against the manifest
        run: >-
          uv run --no-project --python 3.12 python scripts/release/registry_state.py
          --manifest "${MANIFEST}"
          --version "${VERSION}"
          --require-identical
          --wait 300
"""

# The plan job's step that refuses, in release mode, a run whose ref is not the version's
# annotated tag naming the checked-out commit on `main`, unsigned or GitHub-verified: the
# first of the two places the tag is validated.
REVIEWED_PLAN_STEPS = """
      - name: Resolve exact release identity
        id: plan
        env:
          GITHUB_TOKEN: ${{ github.token }}
          PUBLISH: ${{ inputs.publish }}
        run: |
          mode=rehearsal
          if [ "${PUBLISH}" = "true" ]; then
            mode=release
          fi
          uv run --no-project --python 3.12 python scripts/release/resolve_plan.py \\
            --root . \\
            --mode "${mode}" \\
            --ref "${GITHUB_REF}" \\
            --commit "${GITHUB_SHA}" \\
            --repository "${GITHUB_REPOSITORY}" \\
            --validate-checkout \\
            --github-output "${GITHUB_OUTPUT}"
"""

# The job whose failure stops publishing when the `release` environment is not protected.
REVIEWED_ENVIRONMENT_JOB = """
    name: Confirm the release environment is protected
    needs: plan
    if: inputs.publish && needs.plan.outputs.publish == 'true'
    runs-on: ubuntu-latest
    permissions:
      actions: read
      contents: read
    steps:
      - uses: actions/checkout@<pinned>
        with:
          persist-credentials: false
      - uses: astral-sh/setup-uv@<pinned>
        with:
          version: "0.12.1"
          enable-cache: false
      - name: Require a reviewer, v* tag deployments only, and no administrator bypass
        run: >-
          uv run --no-project --python 3.12 python scripts/release/publish_gate.py check-environment
          --repository "${GITHUB_REPOSITORY}"
          --environment release
        env:
          GITHUB_TOKEN: ${{ github.token }}
"""


# The job whose failure stops a patch release that breaks the Rust API; `<version>` is the
# cargo-semver-checks version supply-chain-policy.json inventories.
REVIEWED_SEMVER_JOB = """
    name: Check the Rust API against the last compatible release
    needs: plan
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@<pinned>
        with:
          persist-credentials: false
      - uses: dtolnay/rust-toolchain@<pinned>
        with:
          toolchain: 1.97.1
      - uses: astral-sh/setup-uv@<pinned>
        with:
          version: "0.12.1"
          enable-cache: false
      - name: Install the reviewed cargo-semver-checks
        run: cargo install --locked cargo-semver-checks --version <version>
      - name: Compare fdu-core and fdu with the release they must stay compatible with
        run: uv run --no-project --python 3.12 python scripts/release/semver_check.py --root .
"""


def semver_tool_version() -> str:
    """The cargo-semver-checks version the supply-chain policy inventories."""
    policy = json.loads((ROOT / "supply-chain-policy.json").read_text(encoding="utf-8"))
    (tool,) = [t for t in policy["bootstrap"]["cargoTools"] if t["name"] == "cargo-semver-checks"]
    return tool["version"]


def workflow_jobs(workflow: str, *, comments: bool = False) -> dict[str, str]:
    """Split a workflow's `jobs:` mapping into each job's text, keyed by job ID."""
    jobs: dict[str, str] = {}
    current = None
    for line in workflow.split("\njobs:\n", 1)[1].splitlines():
        header = re.fullmatch(r"  ([A-Za-z0-9_-]+):", line)
        if header is not None:
            current = header[1]
            jobs[current] = ""
        elif current is not None and (comments or not line.lstrip().startswith("#")):
            jobs[current] += line + "\n"
    return jobs


def code(text: str) -> str:
    """The text less its full-line comments, which may mention anything."""
    return "".join(line + "\n" for line in text.splitlines() if not line.lstrip().startswith("#"))


def flat(text: str) -> str:
    """
    The text with every run of whitespace made one space and every pinned revision, with
    its version comment, made `@<pinned>`: `check-supply-chain` vets the revision, so a
    reviewed bump need not edit the literals here.
    """
    unpinned = re.sub(r"@[0-9a-f]{40}(?:[ \t]+#[^\n]*)?", "@<pinned>", text)
    return " ".join(unpinned.split())


def step_keys(job: str) -> list[str]:
    """Each step's `name:`, or its `uses:` less the pinned revision, in order."""
    keys = []
    for text in job.split("\n      - ")[1:]:
        key = re.search(r"(?m)^\s*(?:name|uses): (.+)$", text)
        if key is None:
            raise ValueError(f"step without a name or uses: {text!r}")
        keys.append(key[1].split("@", 1)[0])
    return keys


def workflow_steps(job: str) -> dict[str, str]:
    """Split one job's steps, keyed by `name:`, `uses:`, or an unnamed `run:`."""
    steps: dict[str, str] = {}
    for text in job.split("\n      - ")[1:]:
        key = re.search(r"(?m)^\s*(?:name|uses|run): (.+)$", text)
        if key is None:
            raise ValueError(f"step without a name or uses: {text!r}")
        steps[key[1]] = text
    return steps


if __name__ == "__main__":
    unittest.main()
