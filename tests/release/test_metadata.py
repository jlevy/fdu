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
        # The GitHub release stays a maintainer step, so nothing may write the repository.
        self.assertNotIn("contents: write", workflow)
        self.assertNotIn("pull_request_target", workflow)
        self.assertNotIn("gh-action-pypi-publish", workflow)

    def test_the_publish_job_runs_only_on_the_planned_tag_after_the_rehearsal(self) -> None:
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        publish = workflow_jobs(workflow)[PUBLISH_JOB]
        self.assertIn("    environment: release\n", publish)
        self.assertIn(
            "    permissions:\n      contents: read\n      id-token: write\n    ", publish
        )
        self.assertIn(
            "    needs: [plan, crate, sdist, wheels, evidence, release-environment]\n", publish
        )
        condition = publish.split("    if: >-\n", 1)[1].split("\n    runs-on:", 1)[0]
        for clause in (
            "inputs.publish",
            "startsWith(github.ref, 'refs/tags/v')",
            "needs.plan.outputs.publish == 'true'",
            "github.ref == format('refs/tags/{0}', needs.plan.outputs.release_tag)",
        ):
            self.assertIn(clause, condition)
        # Conjoined, and never widened: `a || b`, `always() ||`, and `!cancelled()` would
        # each pass a presence check while letting the job run on a ref it must not.
        for widening in ("||", "always()", "!cancelled()"):
            self.assertNotIn(widening, condition)
        self.assertEqual(condition.count("&&"), 3)
        # A dispatch rehearses unless publishing is asked for explicitly, and nothing but
        # a dispatch runs the workflow at all.
        trigger = workflow.split("\non:\n", 1)[1].split("\npermissions:", 1)[0]
        self.assertEqual(re.findall(r"(?m)^  ([a-z_]+):$", trigger), ["workflow_dispatch"])
        self.assertIn("      publish:\n", trigger)
        self.assertIn("        type: boolean\n        default: false\n", trigger)

    def test_each_cargo_publish_waits_for_its_own_comparison(self) -> None:
        # The audit says what is missing, but an upload must also see its comparison step
        # run and pass. An audit output that came back empty (a renamed key, a mistyped
        # reference) skips the comparison, and that has to skip the upload as well.
        workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        steps = workflow_steps(workflow_jobs(workflow)[PUBLISH_JOB])
        for crate, reproduce in (("fdu-core", "reproduce-both"), ("fdu", "reproduce-fdu")):
            with self.subTest(crate=crate):
                comparisons = [text for text in steps.values() if f"id: {reproduce}\n" in text]
                self.assertEqual(len(comparisons), 1)
                self.assertIn(f"--package {crate}", comparisons[0])
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
        referenced = set(re.findall(r"steps\.(?:audit|pypi)\.outputs\.([A-Za-z0-9_]+)", publish))
        self.assertTrue(referenced)
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
        self.assertNotIn("cargo build", publish)
        self.assertNotIn("uv build", publish)
        self.assertNotIn("maturin", publish)

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


def workflow_jobs(workflow: str) -> dict[str, str]:
    """Split a workflow's `jobs:` mapping into each job's text, keyed by job ID."""
    jobs: dict[str, str] = {}
    current = None
    for line in workflow.split("\njobs:\n", 1)[1].splitlines():
        header = re.fullmatch(r"  ([A-Za-z0-9_-]+):", line)
        if header is not None:
            current = header[1]
            jobs[current] = ""
        elif current is not None and not line.lstrip().startswith("#"):
            jobs[current] += line + "\n"
    return jobs


def workflow_steps(job: str) -> dict[str, str]:
    """Split one job's text into its steps, keyed by `name:` (or `uses:` for an unnamed one)."""
    steps: dict[str, str] = {}
    for text in job.split("\n      - ")[1:]:
        key = re.search(r"(?m)^\s*(?:name|uses): (.+)$", text)
        if key is None:
            raise ValueError(f"step without a name or uses: {text!r}")
        steps[key[1]] = text
    return steps


if __name__ == "__main__":
    unittest.main()
