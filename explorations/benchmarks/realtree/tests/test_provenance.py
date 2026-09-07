from __future__ import annotations

import json
import shlex
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

    def test_cross_revision_capture_and_verification_bind_each_artifact_source(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw).resolve()
            control_source = root / "old-source"
            sources = {
                root: {**self.source, "tags_at_commit": []},
                control_source: {**self.source, "commit": "b" * 40, "tags_at_commit": []},
            }
            artifacts = []
            for label in ("control", "candidate"):
                binary = root / label
                binary.write_text(label, encoding="utf-8")
                binary.chmod(0o755)
                artifacts.append(
                    provenance.ArtifactSpec(
                        label=label,
                        kind="fdu-perf-probe",
                        executable=binary,
                        build_argv=("cargo", "build", "--release"),
                    )
                )
            binaries = {artifact.label: artifact.executable for artifact in artifacts}
            versions = {
                root / "control": "fdu-perf-probe 0.1.0-dev+g" + "b" * 9,
                root / "candidate": "fdu-perf-probe 0.1.0-dev+g" + "a" * 9,
            }
            _source_patch, host_patch, filesystem_patch, collector_patch = self._patches()
            with (
                host_patch,
                filesystem_patch,
                collector_patch,
                mock.patch.object(provenance, "_source_facts", side_effect=sources.__getitem__),
                mock.patch.object(provenance, "_version", side_effect=versions.__getitem__),
            ):
                unbound = provenance.capture(
                    source_root=root, subject_root=root, artifacts=artifacts
                )
                self.assertFalse(unbound["claim_grade"], "one HEAD cannot identify both binaries")
                self.assertIn("current source revision", str(unbound["invalidation_reasons"]))

                document = provenance.capture(
                    source_root=root,
                    subject_root=root,
                    artifacts=artifacts,
                    artifact_sources={"control": control_source},
                )
                self.assertTrue(document["claim_grade"])
                self.assertNotIn(raw, json.dumps(document))
                self.assertEqual(
                    document["artifacts"]["control"]["origin"]["source_revision"], "b" * 40
                )
                verified = provenance.verify(
                    document,
                    source_root=root,
                    subject_root=root,
                    artifacts=binaries,
                    artifact_sources={"control": control_source},
                )
                self.assertEqual(verified, document)

                # Exercise the measurement entry point too: capture accepting multiple
                # revisions is insufficient if the run silently drops their checkouts.
                from benchmarks.realtree import __main__ as entry_point

                manifest = root / "provenance.json"
                manifest.write_text(json.dumps(document), encoding="utf-8")
                variants = [
                    provenance.measure.Variant(name=label, path=binary)
                    for label, binary in binaries.items()
                ]
                with mock.patch.object(provenance, "PROJECT_ROOT", root):
                    from_entry_point = entry_point._verified_measurement_provenance(
                        manifest, root, variants, {"control": control_source}
                    )
                self.assertEqual(from_entry_point, document)

                with self.assertRaisesRegex(provenance.ProvenanceError, "control.*source"):
                    provenance.verify(
                        document, source_root=root, subject_root=root, artifacts=binaries
                    )
                for change in ({"commit": "d" * 40}, {"clean": False}):
                    with self.subTest(change=change):
                        original = sources[control_source]
                        sources[control_source] = {**original, **change}
                        with self.assertRaisesRegex(provenance.ProvenanceError, "control.*source"):
                            provenance.verify(
                                document,
                                source_root=root,
                                subject_root=root,
                                artifacts=binaries,
                                artifact_sources={"control": control_source},
                            )
                        dirty_or_stale = provenance.capture(
                            source_root=root,
                            subject_root=root,
                            artifacts=artifacts,
                            artifact_sources={"control": control_source},
                        )
                        self.assertFalse(dirty_or_stale["claim_grade"])
                        sources[control_source] = original

    def test_artifact_source_labels_are_explicit_and_unambiguous(self) -> None:
        self.assertEqual(
            provenance.parse_artifact_sources(["control=old-source", "candidate=new-source"]),
            {"control": Path("old-source"), "candidate": Path("new-source")},
        )
        for values in (["missing-separator"], ["control=one", "control=two"]):
            with self.subTest(values=values), self.assertRaises(provenance.ProvenanceError):
                provenance.parse_artifact_sources(values)
        with self.assertRaisesRegex(provenance.ProvenanceError, "unknown artifacts"):
            provenance._checked_artifact_sources({"typo": Path("source")}, ["control"])

    def test_source_toolchain_is_resolved_in_its_own_checkout(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw).resolve()
            (root / ".git").mkdir()
            (root / "Cargo.lock").write_text("fixture", encoding="utf-8")
            with (
                mock.patch.object(provenance, "_git", return_value=""),
                mock.patch.object(provenance, "_command", return_value="rustc fixture") as command,
            ):
                provenance._source_facts(root)
            command.assert_called_once_with(["rustc", "-vV"], cwd=root)
            with self.assertRaises(FileNotFoundError):
                provenance._source_facts(root / "missing-source")

    @unittest.skipUnless(shutil.which("make"), "Make is required to inspect build recipes")
    def test_performance_builds_enable_shipped_control_semantics(self) -> None:
        for target in ("perf-probe-release", "perf-probe-profiling", "performance-probe"):
            with self.subTest(target=target):
                result = subprocess.run(
                    ["make", "--dry-run", "CARGO=cargo", target],
                    cwd=provenance.PROJECT_ROOT,
                    capture_output=True,
                    text=True,
                    check=True,
                )
                builds = [
                    line for line in result.stdout.splitlines() if line.startswith("cargo build ")
                ]
                self.assertTrue(builds, "the recipe must build the measured executable")
                for build in builds:
                    argv = shlex.split(build)
                    features = set()
                    for index, argument in enumerate(argv):
                        if argument in {"--features", "-F"}:
                            features.update(argv[index + 1].replace(",", " ").split())
                        elif argument.startswith("--features="):
                            features.update(argument.split("=", 1)[1].replace(",", " ").split())
                    self.assertTrue(
                        "--all-features" in argv or features & {"gitignore", "fdu-core/gitignore"},
                        f"{target} compiles out control semantics: {build}",
                    )

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
