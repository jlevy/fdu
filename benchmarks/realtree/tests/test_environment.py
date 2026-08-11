"""Tests for portable benchmark cells and cross-environment decision matrices."""

from __future__ import annotations

import copy
import subprocess
import unittest
from pathlib import Path
from unittest import mock

from benchmarks.realtree import environment, measure

_CONTROL_REVISION = "1" * 40
_CANDIDATE_REVISION = "2" * 40
_HARNESS_REVISION = "3" * 40
_CONTROL_SOURCE = "4" * 64
_CANDIDATE_SOURCE = "5" * 64
_SCHEDULE = "6" * 64


def _manifest() -> dict:
    return {
        "schema": "fdu-observed-corpus-v2",
        "generator_source_hash": "7" * 64,
        "generator_version": 2,
        "manifest_hash": "8" * 64,
        "recipe_id": "balanced",
        "recipe_hash": "9" * 64,
        "recipe": {"seed": "cross-environment-v1", "target_entries": 60_000},
        "semantic_digest": "a" * 64,
        "counts": {
            "total": 60_001,
            "directories": 6_001,
            "files": 54_000,
            "symlinks": 0,
            "other": 0,
            "hardlink_groups": 0,
        },
        "state": {"id": "baseline", "transition_index": 0},
    }


def _variant(name: str, *, target: str) -> dict:
    control = name == "corrected"
    return {
        "args": [],
        "kind": "fdu-probe",
        "name": name,
        "notes": "",
        "sha256": ("b" if control else "c") * 64,
        "size_bytes": 1,
        "engine_revision": _CONTROL_REVISION if control else _CANDIDATE_REVISION,
        "harness_revision": _HARNESS_REVISION,
        "harness_sha256": _CONTROL_SOURCE if control else _CANDIDATE_SOURCE,
        "target": target,
        "build_profile": "release",
        "features": [],
        "build_command": (
            "cargo build --locked --release -p fdu --example "
            + ("perf_probe_review" if control else "perf_probe")
            + " --no-default-features"
        ),
    }


def _run(
    cell: str,
    *,
    system: str,
    filesystem: str,
    target: str,
    candidate_wall: int,
    candidate_cpu: int,
    constant_rss: bool = False,
) -> dict:
    host = {
        "arch": "arm64" if system == "Darwin" else "x86_64",
        "cpu_count": 8 if system == "Darwin" else 4,
        "python": "3.13.6",
        "system": system,
        "release": "test-release",
        "toolchain": "rustc 1.97.1 (test)",
        "filesystem": filesystem,
        "cpu_model": "Test CPU",
        "memory_bytes": 16 * 2**30,
    }
    runner_class = (
        "local-uncontrolled" if system == "Darwin" else "github-hosted-shared"
    )
    runner_environment = (
        {}
        if system == "Darwin"
        else {
            "GITHUB_ACTIONS": "true",
            "ImageOS": "ubuntu24",
            "ImageVersion": "20260801.1",
            "RUNNER_OS": "Linux",
            "RUNNER_ARCH": "X64",
        }
    )
    samples = []
    for ordinal in range(12):
        for variant, wall, cpu in (
            ("corrected", 1_000, 1_000),
            ("candidate", candidate_wall, candidate_cpu),
        ):
            samples.append(
                measure.Sample(
                    variant=variant,
                    job="cold-scan-index",
                    ordinal=ordinal,
                    warmup=False,
                    valid=True,
                    metrics={
                        "wall_ns": wall,
                        "component_ns": wall,
                        "cpu_ns": cpu,
                        "user_cpu_ns": cpu,
                        "system_cpu_ns": 0,
                        "blocked_ns": None,
                        "peak_rss_bytes": 1_000 if constant_rss else 1_000 + ordinal,
                        "major_faults": 0,
                        "minor_faults": 10,
                        "involuntary_context_switches": 0,
                        "voluntary_context_switches": 0,
                    },
                ).as_json()
            )
    document = {
        "schema": environment.RUN_SCHEMA,
        "started_utc": "2026-08-11T12:00:00Z",
        "run_group": "pr3-cache-balanced-60k-v1",
        "host": host,
        "environment": environment.environment_identity(
            host,
            cell_id=cell,
            runner_class=runner_class,
            environ=runner_environment,
        ),
        "workload": environment._portable_workload_from_manifest(_manifest()),
        "conditions": {
            "os_cache": "warm-steady",
            "trials": 12,
            "warmups": 3,
            "interleaved": True,
            "schedule": "round-robin-by-ordinal-v1",
            "schedule_sha256": _SCHEDULE,
            "schedule_seed": None,
            "bootstrap_seed": measure.BOOTSTRAP_SEED,
        },
        "tree": {"engine_digest": ("d" if system == "Darwin" else "e") * 64},
        "tree_mutated_during_run": [],
        "variant_order": ["corrected", "candidate"],
        "variants": {
            "corrected": _variant("corrected", target=target),
            "candidate": _variant("candidate", target=target),
        },
        "jobs": {
            "cold-scan-index": {
                "argv": [],
                "description": "test",
                "process_cpu_can_exceed_wall": True,
                "start_state": "cold",
            }
        },
        "samples": samples,
    }
    document["statistics"] = measure.summarize(document)
    return document


def _evidence(document: dict, name: str) -> environment.RunEvidence:
    return environment.RunEvidence(document=document, artifact_name=name, artifact_sha256="f" * 64)


class WorkloadIdentityTests(unittest.TestCase):
    def test_portable_identity_ignores_local_manifest_identity(self) -> None:
        first = _manifest()
        second = copy.deepcopy(first)
        second["manifest_hash"] = "0" * 64
        second["oracle"] = {"engine_digest": "1" * 64}
        self.assertEqual(
            environment._portable_workload_from_manifest(first)[
                "portable_identity_sha256"
            ],
            environment._portable_workload_from_manifest(second)[
                "portable_identity_sha256"
            ],
        )

    def test_recipe_or_semantics_change_the_portable_identity(self) -> None:
        changed = _manifest()
        changed["semantic_digest"] = "0" * 64
        self.assertNotEqual(
            environment._portable_workload_from_manifest(_manifest())[
                "portable_identity_sha256"
            ],
            environment._portable_workload_from_manifest(changed)[
                "portable_identity_sha256"
            ],
        )


class EnvironmentIdentityTests(unittest.TestCase):
    def test_fingerprint_contains_no_hostname_or_path(self) -> None:
        host = {"system": "Linux", "filesystem": "ext4", "arch": "x86_64"}
        identity = environment.environment_identity(
            host,
            cell_id="github-ubuntu-24.04-x64",
            runner_class="github-hosted-shared",
            environ={
                "GITHUB_ACTIONS": "true",
                "ImageOS": "ubuntu24",
                "ImageVersion": "20260801.1",
                "RUNNER_NAME": "/secret/hostname",
            },
        )
        self.assertNotIn("RUNNER_NAME", str(identity))
        self.assertNotIn("secret", str(identity))
        self.assertEqual(len(identity["fingerprint_sha256"]), 64)

    def test_runner_metadata_rejects_paths(self) -> None:
        with self.assertRaises(environment.EnvironmentError):
            environment.environment_identity(
                {"system": "Linux"},
                runner_class="github-hosted-shared",
                environ={"GITHUB_ACTIONS": "true", "ImageOS": "/private/image"},
            )

    def test_linux_filesystem_is_resolved_for_the_measured_root(self) -> None:
        completed = subprocess.CompletedProcess([], 0, stdout=b"ext4\n", stderr=b"")
        measured_root: Path = Path("/measured")
        with (
            mock.patch.object(environment.sys, "platform", "linux"),
            mock.patch.object(environment.shutil, "which", return_value="/usr/bin/findmnt"),
            mock.patch.object(environment.subprocess, "run", return_value=completed) as run,
        ):
            self.assertEqual(environment._filesystem_for_path(measured_root), "ext4")
        self.assertEqual(run.call_args.args[0][-1], str(measured_root))


class DecisionMatrixTests(unittest.TestCase):
    def test_decisions_are_recomputed_per_cell_and_divergence_is_explicit(self) -> None:
        mac = _run(
            "local-macos-apfs-arm64",
            system="Darwin",
            filesystem="apfs",
            target="aarch64-apple-darwin",
            candidate_wall=500,
            candidate_cpu=1_800,
        )
        linux = _run(
            "github-ubuntu-24.04-x64",
            system="Linux",
            filesystem="ext4",
            target="x86_64-unknown-linux-gnu",
            candidate_wall=500,
            candidate_cpu=900,
        )
        matrix = environment.build_matrix(
            [_evidence(mac, "mac.json"), _evidence(linux, "linux.json")],
            matrix_id="env-001",
            control_variant="corrected",
            candidate_variant="candidate",
        )
        job = matrix.jobs[0]
        self.assertEqual(job.divergent_gates, ["cpu", "overall"])
        self.assertEqual(
            job.cells["local-macos-apfs-arm64"].overall_decision, "rejected"
        )
        self.assertEqual(
            job.cells["github-ubuntu-24.04-x64"].overall_decision, "accepted"
        )
        self.assertFalse(matrix.decision_consistent)
        self.assertTrue(matrix.all_cells_valid)

    def test_constant_runwide_rss_fails_the_resource_gate_closed(self) -> None:
        document = _run(
            "linux-cell",
            system="Linux",
            filesystem="ext4",
            target="x86_64-unknown-linux-gnu",
            candidate_wall=500,
            candidate_cpu=900,
            constant_rss=True,
        )
        matrix = environment.build_matrix(
            [_evidence(document, "linux.json")],
            matrix_id="env-001",
            control_variant="corrected",
            candidate_variant="candidate",
        )

        cell = matrix.jobs[0].cells["linux-cell"]
        rss = next(
            gate
            for gate in cell.resource_guardrails
            if gate.metric == "peak_rss_bytes"
        )
        self.assertEqual(rss.status, "not-measured")
        self.assertIn("process-launcher floor", rss.reason)
        self.assertEqual(cell.overall_decision, "rejected")
        self.assertIn("process-launcher floor", environment.render_matrix(matrix))

    def test_absolute_binary_hashes_and_targets_may_differ_by_cell(self) -> None:
        mac = _run(
            "mac-cell",
            system="Darwin",
            filesystem="apfs",
            target="aarch64-apple-darwin",
            candidate_wall=500,
            candidate_cpu=900,
        )
        linux = _run(
            "linux-cell",
            system="Linux",
            filesystem="ext4",
            target="x86_64-unknown-linux-gnu",
            candidate_wall=500,
            candidate_cpu=900,
        )
        linux["variants"]["candidate"]["sha256"] = "0" * 64
        matrix = environment.build_matrix(
            [_evidence(mac, "mac.json"), _evidence(linux, "linux.json")],
            matrix_id="env-001",
            control_variant="corrected",
            candidate_variant="candidate",
        )
        self.assertTrue(matrix.decision_consistent)

    def test_mismatched_portable_workloads_fail_closed(self) -> None:
        mac = _run(
            "mac-cell",
            system="Darwin",
            filesystem="apfs",
            target="aarch64-apple-darwin",
            candidate_wall=500,
            candidate_cpu=900,
        )
        linux = _run(
            "linux-cell",
            system="Linux",
            filesystem="ext4",
            target="x86_64-unknown-linux-gnu",
            candidate_wall=500,
            candidate_cpu=900,
        )
        linux["workload"]["portable_identity_sha256"] = "0" * 64
        with self.assertRaisesRegex(environment.EnvironmentError, "portable workload"):
            environment.build_matrix(
                [_evidence(mac, "mac.json"), _evidence(linux, "linux.json")],
                matrix_id="env-001",
                control_variant="corrected",
                candidate_variant="candidate",
            )

    def test_tampered_portable_workload_fields_fail_closed(self) -> None:
        document = _run(
            "mac-cell",
            system="Darwin",
            filesystem="apfs",
            target="aarch64-apple-darwin",
            candidate_wall=500,
            candidate_cpu=900,
        )
        document["workload"]["semantic_digest"] = "0" * 64
        with self.assertRaisesRegex(environment.EnvironmentError, "portable workload identity"):
            environment.build_matrix(
                [_evidence(document, "mac.json")],
                matrix_id="env-001",
                control_variant="corrected",
                candidate_variant="candidate",
            )

    def test_variant_arguments_must_match_across_cells(self) -> None:
        mac = _run(
            "mac-cell",
            system="Darwin",
            filesystem="apfs",
            target="aarch64-apple-darwin",
            candidate_wall=500,
            candidate_cpu=900,
        )
        linux = _run(
            "linux-cell",
            system="Linux",
            filesystem="ext4",
            target="x86_64-unknown-linux-gnu",
            candidate_wall=500,
            candidate_cpu=900,
        )
        linux["variants"]["candidate"]["args"] = ["--threads", "2"]
        with self.assertRaisesRegex(environment.EnvironmentError, "comparison source"):
            environment.build_matrix(
                [_evidence(mac, "mac.json"), _evidence(linux, "linux.json")],
                matrix_id="env-001",
                control_variant="corrected",
                candidate_variant="candidate",
            )

    def test_full_job_contract_must_match_across_cells(self) -> None:
        mac = _run(
            "mac-cell",
            system="Darwin",
            filesystem="apfs",
            target="aarch64-apple-darwin",
            candidate_wall=500,
            candidate_cpu=900,
        )
        linux = _run(
            "linux-cell",
            system="Linux",
            filesystem="ext4",
            target="x86_64-unknown-linux-gnu",
            candidate_wall=500,
            candidate_cpu=900,
        )
        linux["jobs"]["cold-scan-index"]["argv"] = ["different"]
        with self.assertRaisesRegex(environment.EnvironmentError, "equivalent jobs"):
            environment.build_matrix(
                [_evidence(mac, "mac.json"), _evidence(linux, "linux.json")],
                matrix_id="env-001",
                control_variant="corrected",
                candidate_variant="candidate",
            )

    def test_runner_class_and_evidence_grade_cannot_disagree(self) -> None:
        document = _run(
            "mac-cell",
            system="Darwin",
            filesystem="apfs",
            target="aarch64-apple-darwin",
            candidate_wall=500,
            candidate_cpu=900,
        )
        document["environment"]["evidence_grade"] = "controlled"
        with self.assertRaisesRegex(environment.EnvironmentError, "evidence grade"):
            environment.build_matrix(
                [_evidence(document, "mac.json")],
                matrix_id="env-001",
                control_variant="corrected",
                candidate_variant="candidate",
            )

    def test_tampered_environment_fingerprint_fails_closed(self) -> None:
        document = _run(
            "mac-cell",
            system="Darwin",
            filesystem="apfs",
            target="aarch64-apple-darwin",
            candidate_wall=500,
            candidate_cpu=900,
        )
        document["host"]["cpu_count"] = 99
        with self.assertRaisesRegex(environment.EnvironmentError, "fingerprint"):
            environment.build_matrix(
                [_evidence(document, "mac.json")],
                matrix_id="env-001",
                control_variant="corrected",
                candidate_variant="candidate",
            )


if __name__ == "__main__":
    unittest.main()
