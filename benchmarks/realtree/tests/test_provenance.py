from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from benchmarks.realtree import provenance


class ProvenanceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.source = {
            "cargo_lock_sha256": "c" * 64,
            "clean": True,
            "commit": "a" * 40,
            "remote": "https://github.com/jlevy/fdu",
            "rust_target": "test-target",
            "rust_toolchain": "rustc test",
        }
        self.host = {
            "arch": "arm64",
            "class_id": "h" * 64,
            "cpu_count": 10,
            "cpu_model": "Apple M1 Pro",
            "efficiency_cores": 2,
            "memory_bytes": 16 * 1024**3,
            "performance_cores": 8,
            "power": {"available": True, "reason": None, "source": "AC"},
            "release": "test-release",
            "system": "Darwin",
            "thermal": {"available": True, "pressure": "normal", "reason": None},
        }
        self.filesystem = {
            "block_size": 4096,
            "flags": 1,
            "fragment_size": 4096,
            "solid_state": True,
            "type": "apfs",
        }
        self.collectors = {
            "macos_sample": {"available": True, "name": "sample"},
            "process_rusage": {"reason": None, "supported": True},
            "time": {"available": True, "name": "time"},
        }

    def _patches(self):
        return (
            mock.patch.object(provenance, "_source_facts", return_value=self.source),
            mock.patch.object(provenance, "_host_facts", return_value=self.host),
            mock.patch.object(provenance, "_filesystem_facts", return_value=self.filesystem),
            mock.patch.object(provenance, "_collector_facts", return_value=self.collectors),
        )

    def test_capture_is_path_redacted_and_verifies_every_identity(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            artifact = provenance.ArtifactSpec(
                label="native",
                kind="native-fdu",
                executable=Path(sys.executable),
                build_argv=("cargo", "install", "$SOURCE/crates/fdu"),
            )
            source_patch, host_patch, filesystem_patch, collector_patch = self._patches()
            with source_patch, host_patch, filesystem_patch, collector_patch:
                document = provenance.capture(
                    source_root=root, subject_root=root, artifacts=[artifact]
                )
                verified = provenance.verify(
                    document,
                    source_root=root,
                    subject_root=root,
                    artifacts={"native": Path(sys.executable)},
                )

        self.assertTrue(document["claim_grade"])
        self.assertEqual(verified, document)
        self.assertNotIn(raw, json.dumps(document))
        self.assertEqual(document["manifest_id"], provenance._manifest_id(document))

    def test_dirty_source_and_incomplete_python_payload_are_exploratory(self) -> None:
        dirty = {**self.source, "clean": False}
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            (root / "crates" / "fdu-py").mkdir(parents=True)
            (root / "crates" / "fdu-py" / "uv.lock").write_text("fixture", encoding="utf-8")
            artifact = provenance.ArtifactSpec(
                label="python",
                kind="python-fdu",
                executable=Path(sys.executable),
                build_argv=("uv", "build", "$SOURCE/crates/fdu-py"),
            )
            with (
                mock.patch.object(provenance, "_source_facts", return_value=dirty),
                mock.patch.object(provenance, "_host_facts", return_value=self.host),
                mock.patch.object(provenance, "_filesystem_facts", return_value=self.filesystem),
                mock.patch.object(provenance, "_collector_facts", return_value=self.collectors),
            ):
                document = provenance.capture(
                    source_root=root, subject_root=root, artifacts=[artifact]
                )
                with self.assertRaisesRegex(provenance.ProvenanceError, "exploratory"):
                    provenance.verify(
                        document,
                        source_root=root,
                        subject_root=root,
                        artifacts={"python": Path(sys.executable)},
                    )
                verified = provenance.verify(
                    document,
                    source_root=root,
                    subject_root=root,
                    artifacts={"python": Path(sys.executable)},
                    require_claim_grade=False,
                )

        self.assertFalse(document["claim_grade"])
        self.assertEqual(verified, document)
        reasons = "\n".join(document["invalidation_reasons"])
        self.assertIn("dirty", reasons)
        self.assertIn("native extension payload", reasons)

    def test_exploratory_verify_rejects_a_source_cleanliness_change(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            artifact = provenance.ArtifactSpec(
                label="native",
                kind="native-fdu",
                executable=Path(sys.executable),
                build_argv=("cargo", "install", "$SOURCE/crates/fdu"),
            )
            dirty = {**self.source, "clean": False}
            with (
                mock.patch.object(provenance, "_source_facts", return_value=dirty),
                mock.patch.object(provenance, "_host_facts", return_value=self.host),
                mock.patch.object(provenance, "_filesystem_facts", return_value=self.filesystem),
                mock.patch.object(provenance, "_collector_facts", return_value=self.collectors),
            ):
                document = provenance.capture(
                    source_root=root, subject_root=root, artifacts=[artifact]
                )

            with (
                mock.patch.object(provenance, "_source_facts", return_value=self.source),
                mock.patch.object(provenance, "_host_facts", return_value=self.host),
                mock.patch.object(provenance, "_filesystem_facts", return_value=self.filesystem),
                mock.patch.object(provenance, "_collector_facts", return_value=self.collectors),
                self.assertRaisesRegex(provenance.ProvenanceError, "cleanliness"),
            ):
                provenance.verify(
                    document,
                    source_root=root,
                    subject_root=root,
                    artifacts={"native": Path(sys.executable)},
                    require_claim_grade=False,
                )

    def test_manifest_tampering_and_binary_substitution_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            artifact = provenance.ArtifactSpec(
                label="native",
                kind="native-fdu",
                executable=Path(sys.executable),
                build_argv=("cargo", "install", "$SOURCE/crates/fdu"),
            )
            source_patch, host_patch, filesystem_patch, collector_patch = self._patches()
            with source_patch, host_patch, filesystem_patch, collector_patch:
                document = provenance.capture(
                    source_root=root, subject_root=root, artifacts=[artifact]
                )
                tampered = json.loads(json.dumps(document))
                tampered["artifacts"]["native"]["profile"] = "debug"
                with self.assertRaisesRegex(provenance.ProvenanceError, "identity"):
                    provenance.verify(
                        tampered,
                        source_root=root,
                        subject_root=root,
                        artifacts={"native": Path(sys.executable)},
                    )
                with self.assertRaisesRegex(provenance.ProvenanceError, "hashes"):
                    substitute = root / "substitute-python"
                    shutil.copy2(sys.executable, substitute)
                    substitute.write_bytes(substitute.read_bytes() + b"tampered")
                    provenance.verify(
                        document,
                        source_root=root,
                        subject_root=root,
                        artifacts={"native": substitute},
                    )

    def test_absolute_build_paths_are_rejected_from_claim_grade_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            artifact = provenance.ArtifactSpec(
                label="native",
                kind="native-fdu",
                executable=Path(sys.executable),
                build_argv=("cargo", "install", str(root / "source")),
            )
            source_patch, host_patch, filesystem_patch, collector_patch = self._patches()
            with source_patch, host_patch, filesystem_patch, collector_patch:
                document = provenance.capture(
                    source_root=root, subject_root=root, artifacts=[artifact]
                )

        self.assertFalse(document["claim_grade"])
        self.assertIn("absolute path", "\n".join(document["invalidation_reasons"]))

    def test_stale_development_binary_is_rejected_from_claim_grade(self) -> None:
        source = {**self.source, "tags_at_commit": []}
        artifact = provenance.ArtifactSpec(
            label="native",
            kind="native-fdu",
            executable=Path(sys.executable),
            build_argv=("cargo", "install", "$SOURCE/crates/fdu"),
        )

        with mock.patch.object(provenance, "_version", return_value="fdu 0.1.0-dev+gdeadbeef0"):
            _facts, reasons = provenance._artifact_facts(artifact, Path.cwd(), source)

        self.assertIn("does not identify the current source revision", "\n".join(reasons))

    def test_exact_release_tag_can_identify_a_release_binary(self) -> None:
        source = {**self.source, "tags_at_commit": ["v0.1.0"]}

        self.assertEqual(provenance._fdu_revision_reasons("fdu 0.1.0", source), [])

    def test_remote_normalization_removes_credentials_and_git_suffix(self) -> None:
        self.assertEqual(
            provenance._normalize_remote("https://secret-token@github.com/jlevy/fdu.git"),
            "https://github.com/jlevy/fdu",
        )

    def test_macos_filesystem_lookup_resolves_the_subject_mount_device(self) -> None:
        completed = subprocess.CompletedProcess(
            args=["df"],
            returncode=0,
            stdout=(
                b"Filesystem 512-blocks Used Available Capacity Mounted on\n"
                b"/dev/disk3s5 100 50 50 50% /System/Volumes/Data\n"
            ),
            stderr=b"",
        )
        with (
            mock.patch.object(provenance.shutil, "which", return_value="/bin/df"),
            mock.patch.object(provenance.subprocess, "run", return_value=completed),
        ):
            device = provenance._darwin_mount_device(Path("/private/fixture"))

        self.assertEqual(device, "/dev/disk3s5")


if __name__ == "__main__":
    unittest.main()
