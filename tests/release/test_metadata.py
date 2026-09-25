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
        self.assertEqual(version, "0.1.0")
        self.assertEqual(python_crate["package"]["version"], version)
        self.assertEqual(workspace["workspace"]["dependencies"]["fdu"]["version"], version)
        self.assertEqual(pyproject["project"]["name"], "fdu")
        self.assertEqual(pyproject["project"]["scripts"]["fdu"], "fdu:_main")
        self.assertEqual(pyproject["tool"]["maturin"]["module-name"], "fdu._native")

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
            "secrets.CARGO_REGISTRY_TOKEN",
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
        # The one write grant is the publish job's `id-token: write`: the GitHub release stays
        # a maintainer step, so nothing may write the repository, and a build job that ran
        # dependency build scripts under `write-all` could push or move a tag. Matched as a
        # word, so a doubled space or a `write-all` cannot slip past a substring.
        self.assertEqual(re.findall(r"(?i)\bwrite(?:-all)?\b", code(workflow)), ["write"])
        self.assertNotIn("pull_request_target", workflow)
        self.assertNotIn("gh-action-pypi-publish", workflow)
        # The bootstrap token is read by the step that reports which credential applies and
        # by the two uploads, never in the job's `env`, where every action would see it.
        steps = workflow_steps(jobs[PUBLISH_JOB])
        holders = [name for name, text in steps.items() if "secrets.CARGO_REGISTRY_TOKEN" in text]
        self.assertEqual(
            holders, ["Choose the crates.io credential", "Publish fdu-core", "Publish fdu"]
        )
        self.assertEqual(workflow.count("secrets.CARGO_REGISTRY_TOKEN"), 3)

    def test_the_publish_job_runs_only_on_the_planned_tag_after_the_rehearsal(self) -> None:
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        publish = workflow_jobs(workflow)[PUBLISH_JOB]
        self.assertIn("    environment: release\n", publish)
        self.assertIn(
            "    permissions:\n      contents: read\n      id-token: write\n    env:\n", publish
        )
        self.assertIn(
            "    needs: [plan, crate, sdist, wheels, evidence, release-environment]\n", publish
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
            {"filename": "fdu-core-0.1.0.crate", "kind": "crate", "sha256": "a" * 64},
            {"filename": "fdu-0.1.0.crate", "kind": "crate", "sha256": "b" * 64},
            {"filename": "fdu-0.1.0.tar.gz", "kind": "sdist", "sha256": "c" * 64},
            {"filename": "fdu-0.1.0-cp312-abi3-win_amd64.whl", "kind": "wheel", "sha256": "d" * 64},
        ]
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / "manifest.json"
            manifest.write_text(
                json.dumps({"version": "0.1.0", "artifacts": artifacts}), encoding="utf-8"
            )
            written = publish_gate.audit(manifest, "0.1.0", lambda _url: None)
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
        # code in a job whose OIDC token the pending PyPI publisher trusts.
        self.assertEqual(set(re.findall(r"\bcargo\s+([a-z-]+)", publish)), {"package", "publish"})
        self.assertEqual(set(re.findall(r"\buvx?\s+([a-z-]+)", publish)), {"publish"})
        self.assertNotRegex(publish, r"\b(?:uvx|pip|pip3|npm|npx|make|rustc|maturin)\b")

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
        self.assertNotIn("cargo install", workflow)


PUBLISH_JOB = "publish"

# The publish job's steps in order, an action named without its pinned revision so that a
# reviewed action update does not read as a reordering.
PUBLISH_STEP_ORDER = [
    "actions/checkout",
    "Confirm the checkout is the planned release tag",
    "Locate the rehearsal's files",
    "actions/download-artifact",
    "actions/download-artifact",
    "actions/download-artifact",
    "actions/download-artifact",
    "Verify the downloaded files against the manifest and SHA256SUMS",
    "Audit both registries before publishing",
    "dtolnay/rust-toolchain",
    "astral-sh/setup-uv",
    "Reproduce both crates and compare them with the rehearsal",
    "Choose the crates.io credential",
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
        run: >-
          python3 scripts/release/resolve_plan.py
          --root .
          --mode release
          --ref "${GITHUB_REF}"
          --commit "${GITHUB_SHA}"
          --validate-checkout
      - name: Verify the downloaded files against the manifest and SHA256SUMS
        run: >-
          python3 scripts/release/publish_gate.py verify-files "${FILES}"
          --manifest "${MANIFEST}"
          --checksums "${EVIDENCE}/SHA256SUMS"
          --version "${VERSION}"
      - name: Audit both registries before publishing
        id: audit
        run: >-
          python3 scripts/release/publish_gate.py audit
          --manifest "${MANIFEST}"
          --version "${VERSION}"
          --github-output "${GITHUB_OUTPUT}"
      - name: Reproduce both crates and compare them with the rehearsal
        id: reproduce-both
        if: steps.audit.outputs.crates == 'true'
        run: |
          cargo package --locked --no-verify -p fdu-core -p fdu
          python3 scripts/release/publish_gate.py compare-crates \\
            --package-dir target/package \\
            --manifest "${MANIFEST}" \\
            --version "${VERSION}" \\
            --package fdu-core \\
            --package fdu
      - name: Choose the crates.io credential
        id: credential
        if: steps.audit.outputs.crates == 'true'
        env:
          BOOTSTRAP_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: |
          if [ -n "${BOOTSTRAP_TOKEN}" ]; then
            echo "source=bootstrap" >> "${GITHUB_OUTPUT}"
            echo "::warning title=crates.io bootstrap token::Publishing with the \
            CARGO_REGISTRY_TOKEN environment secret. Delete the secret and revoke the \
            token once this release is published."
          else
            echo "source=oidc" >> "${GITHUB_OUTPUT}"
          fi
      - name: Exchange GitHub OIDC for a short-lived crates.io token
        id: crates-io-auth
        if: steps.credential.outputs.source == 'oidc'
        uses: rust-lang/crates-io-auth-action@<pinned>
      - name: Publish fdu-core
        if: steps.audit.outputs.fdu_core == 'missing' && steps.reproduce-both.outcome == 'success'
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN
            || steps.crates-io-auth.outputs.token }}
        run: cargo publish --locked --no-verify -p fdu-core
      - name: Wait until crates.io serves the rehearsed fdu-core
        run: >-
          python3 scripts/release/publish_gate.py wait-crate
          --manifest "${MANIFEST}"
          --version "${VERSION}"
          --package fdu-core
      - name: Reproduce fdu against the published fdu-core and compare it
        id: reproduce-fdu
        if: steps.audit.outputs.fdu == 'missing'
        run: |
          cargo package --locked --no-verify -p fdu
          python3 scripts/release/publish_gate.py compare-crates \\
            --package-dir target/package \\
            --manifest "${MANIFEST}" \\
            --version "${VERSION}" \\
            --package fdu
      - name: Publish fdu
        if: steps.audit.outputs.fdu == 'missing' && steps.reproduce-fdu.outcome == 'success'
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN
            || steps.crates-io-auth.outputs.token }}
        run: cargo publish --locked --no-verify -p fdu
      - name: Wait until crates.io serves the rehearsed fdu
        run: >-
          python3 scripts/release/publish_gate.py wait-crate
          --manifest "${MANIFEST}"
          --version "${VERSION}"
          --package fdu
      - name: Audit PyPI before uploading
        id: pypi
        run: >-
          python3 scripts/release/publish_gate.py audit
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
          python3 scripts/release/publish_gate.py wait-pypi
          --manifest "${MANIFEST}"
          --version "${VERSION}"
      - name: Audit every registry against the manifest
        run: >-
          python3 scripts/release/registry_state.py
          --manifest "${MANIFEST}"
          --version "${VERSION}"
          --require-identical
"""

# The plan job's step that refuses, in release mode, a run whose ref is not the version's
# tag naming the checked-out commit: the first of the two places the tag is validated.
REVIEWED_PLAN_STEPS = """
      - name: Resolve exact release identity
        id: plan
        env:
          PUBLISH: ${{ inputs.publish }}
        run: |
          mode=rehearsal
          if [ "${PUBLISH}" = "true" ]; then
            mode=release
          fi
          python3 scripts/release/resolve_plan.py \\
            --root . \\
            --mode "${mode}" \\
            --ref "${GITHUB_REF}" \\
            --commit "${GITHUB_SHA}" \\
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
      - name: Require a reviewer, v* tag deployments only, and no administrator bypass
        run: >-
          python3 scripts/release/publish_gate.py check-environment
          --repository "${GITHUB_REPOSITORY}"
          --environment release
        env:
          GITHUB_TOKEN: ${{ github.token }}
"""


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
