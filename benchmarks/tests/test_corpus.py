from __future__ import annotations

import hashlib
import json
import os
import tempfile
import unittest
from pathlib import Path
from typing import Any
from unittest import mock

from benchmarks.corpus import (
    CorpusError,
    _metadata_for_observation,
    _set_path_times_ns,
    apply_transition,
    cleanup_run_directory,
    create_corpus,
    load_recipe,
    reserve_run_directory,
    verify_corpus,
)


class CorpusGenerationTests(unittest.TestCase):
    def test_non_authoritative_directory_identity_uses_a_fresh_path_stat(self) -> None:
        class CachedEntry:
            def __init__(self, path: Path) -> None:
                self.path = str(path)

            def stat(self, *, follow_symlinks: bool) -> os.stat_result:
                raise AssertionError(
                    "non-authoritative directory metadata must not be consulted"
                )

        with tempfile.TemporaryDirectory() as temporary_directory:
            path = Path(temporary_directory) / "identity.txt"
            path.write_bytes(b"identity")
            expected = path.lstat()
            with mock.patch(
                "benchmarks.corpus._DIRENTRY_METADATA_IS_AUTHORITATIVE", False
            ):
                observed = _metadata_for_observation(CachedEntry(path), path)

        self.assertEqual(observed.st_ino, expected.st_ino)
        self.assertEqual(observed.st_dev, expected.st_dev)

    def test_generation_omits_an_unsupported_no_follow_utime_flag(self) -> None:
        real_utime = os.utime

        def emulate_platform_utime(*args: Any, **kwargs: Any) -> None:
            if kwargs.get("follow_symlinks") is False:
                raise NotImplementedError("follow_symlinks unavailable")
            real_utime(*args, **kwargs)

        with tempfile.TemporaryDirectory() as temporary_directory:
            with mock.patch.object(
                os, "supports_follow_symlinks", set()
            ), mock.patch(
                "benchmarks.corpus.os.utime",
                side_effect=emulate_platform_utime,
            ):
                run_root = reserve_run_directory(Path(temporary_directory))
                manifest = create_corpus(
                    run_root, "balanced", target_entries=20
                )

            verified = verify_corpus(run_root)

        self.assertEqual(verified["semantic_digest"], manifest["semantic_digest"])

    def test_contract_matches_committed_exact_expectations(self) -> None:
        expectations_path = (
            Path(__file__).resolve().parents[1] / "expected" / "contract-v1.json"
        )
        expectations = json.loads(expectations_path.read_text(encoding="utf-8"))
        with tempfile.TemporaryDirectory() as temporary_directory:
            manifest = create_corpus(
                reserve_run_directory(Path(temporary_directory)), "contract"
            )

        records = {record["path"]: record for record in manifest["records"]}
        required_directories = {
            path
            for path, record in records.items()
            if record["kind"] == "directory"
        }
        required_files = {
            path: record["size"]
            for path, record in records.items()
            if record["kind"] == "file"
            and path != expectations["optional_records"]["hardlink"]["path"]
        }
        self.assertEqual(
            required_directories, set(expectations["required_directories"])
        )
        self.assertEqual(required_files, expectations["required_files"])
        for capability_name, optional_record in expectations[
            "optional_records"
        ].items():
            capability = manifest["capabilities"][capability_name]
            if capability["supported"]:
                actual = records[optional_record["path"]]
                for field, value in optional_record.items():
                    self.assertEqual(actual[field], value)
            else:
                self.assertNotIn(optional_record["path"], records)
        self.assertEqual(
            manifest["recipe"]["target_entries"],
            expectations["target_descendants"],
        )
        self.assertEqual(
            manifest["sizes"]["unique_apparent_bytes"],
            expectations["required_sizes"]["unique_apparent_bytes"],
        )
        required_extensions = {
            extension: dict(totals)
            for extension, totals in manifest["extensions"].items()
        }
        if manifest["capabilities"]["hardlink"]["supported"]:
            required_extensions[".bin"]["count"] -= 1
            required_extensions[".bin"]["apparent_bytes"] -= 257
        self.assertEqual(
            required_extensions, expectations["required_extension_totals"]
        )
    def test_same_recipe_produces_same_semantic_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            work_directory = Path(temporary_directory)
            first_run = reserve_run_directory(work_directory)
            second_run = reserve_run_directory(work_directory)

            first = create_corpus(first_run, "balanced", target_entries=40)
            second = create_corpus(second_run, "balanced", target_entries=40)

            self.assertEqual(first["recipe_hash"], second["recipe_hash"])
            self.assertEqual(first["semantic_digest"], second["semantic_digest"])
            self.assertEqual(first["counts"], second["counts"])
            self.assertEqual(first["extensions"], second["extensions"])

    def test_different_seed_changes_names_but_not_shape(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            work_directory = Path(temporary_directory)
            first = create_corpus(
                reserve_run_directory(work_directory),
                "balanced",
                target_entries=40,
                seed="first-seed",
            )
            second = create_corpus(
                reserve_run_directory(work_directory),
                "balanced",
                target_entries=40,
                seed="second-seed",
            )

            self.assertNotEqual(first["recipe_hash"], second["recipe_hash"])
            self.assertNotEqual(first["semantic_digest"], second["semantic_digest"])
            self.assertEqual(first["counts"], second["counts"])
            self.assertEqual(
                first["topology"]["max_depth"], second["topology"]["max_depth"]
            )

    def test_every_recipe_family_generates_and_verifies_at_smoke_scale(self) -> None:
        recipe_ids = (
            "balanced",
            "churn-distributed",
            "churn-local",
            "contract",
            "deep",
            "mixed-metadata",
            "partial",
            "wide",
            "wide-snapshot",
        )
        with tempfile.TemporaryDirectory() as temporary_directory:
            work_directory = Path(temporary_directory)
            for recipe_id in recipe_ids:
                with self.subTest(recipe_id=recipe_id):
                    run_root = reserve_run_directory(work_directory)
                    target = None if recipe_id == "contract" else 40
                    manifest = create_corpus(
                        run_root, recipe_id, target_entries=target
                    )
                    optional_entries = sum(
                        capability.get("entries_added", 0)
                        for name, capability in manifest["capabilities"].items()
                        if name in {"hardlink", "symlink"}
                    )
                    required_entries = manifest["recipe"]["target_entries"]
                    self.assertEqual(
                        manifest["counts"]["total"],
                        required_entries + 1 + optional_entries,
                    )
                    self.assertEqual(verify_corpus(run_root), manifest)

    def test_balanced_1k_smoke_scale_uses_bounded_manifest_memory(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            run_root = reserve_run_directory(Path(temporary_directory))
            manifest = create_corpus(run_root, "balanced", target_entries=1_000)

            self.assertEqual(manifest["counts"]["total"], 1_001)
            self.assertNotIn("records", manifest)
            self.assertEqual(verify_corpus(run_root), manifest)

    def test_deep_recipe_keeps_relative_paths_platform_safe(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            manifest = create_corpus(
                reserve_run_directory(Path(temporary_directory)),
                "deep",
                target_entries=100,
            )

            longest_path = max(len(record["path"]) for record in manifest["records"])
            self.assertLessEqual(longest_path, 240)
            self.assertLessEqual(
                manifest["topology"]["max_depth"],
                manifest["recipe"]["max_depth"] + 1,
            )

    def test_verifier_rejects_filesystem_tampering(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            run_root = reserve_run_directory(Path(temporary_directory))
            manifest = create_corpus(run_root, "balanced", target_entries=20)
            file_record = next(
                record for record in manifest["records"] if record["kind"] == "file"
            )
            with (run_root / "corpus" / file_record["path"]).open("ab") as output:
                output.write(b"tamper")

            with self.assertRaisesRegex(CorpusError, "verification failed"):
                verify_corpus(run_root)

    def test_verifier_detects_subsecond_mtime_change(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            run_root = reserve_run_directory(Path(temporary_directory))
            manifest = create_corpus(run_root, "balanced", target_entries=20)
            file_record = next(
                record for record in manifest["records"] if record["kind"] == "file"
            )
            path = run_root / "corpus" / file_record["path"]
            before = path.stat()
            _set_path_times_ns(
                path,
                before.st_atime_ns,
                before.st_mtime_ns + 1_000,
            )
            if path.stat().st_mtime_ns == before.st_mtime_ns:
                self.skipTest("filesystem does not retain subsecond mtimes")

            with self.assertRaisesRegex(CorpusError, "verification failed"):
                verify_corpus(run_root)

    def test_churn_transitions_change_only_declared_records(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            run_root = reserve_run_directory(Path(temporary_directory))
            baseline = create_corpus(
                run_root, "churn-distributed", target_entries=80
            )
            before = {record["path"]: record for record in baseline["records"]}

            transitioned = apply_transition(run_root, "one-change")

            after = {record["path"]: record for record in transitioned["records"]}
            changed_paths = {
                path
                for path in before.keys() | after.keys()
                if before.get(path) != after.get(path)
            }
            self.assertEqual(
                changed_paths,
                set(transitioned["transition_history"][-1]["changed_paths"]),
            )
            self.assertNotEqual(
                baseline["semantic_digest"], transitioned["semantic_digest"]
            )
            self.assertEqual(baseline["counts"], transitioned["counts"])
            self.assertEqual(baseline["extensions"], transitioned["extensions"])
            self.assertEqual(
                baseline["sizes"]["entry_apparent_bytes"],
                transitioned["sizes"]["entry_apparent_bytes"],
            )
            self.assertEqual(verify_corpus(run_root), transitioned)

    def test_complete_churn_sequence_preserves_shape_and_size_contract(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            run_root = reserve_run_directory(Path(temporary_directory))
            baseline = create_corpus(run_root, "churn-local", target_entries=80)
            current = baseline
            for transition_id in ("one-change", "modify-1pct", "mixed-1pct"):
                current = apply_transition(run_root, transition_id)
                self.assertEqual(current["counts"], baseline["counts"])
                self.assertEqual(current["extensions"], baseline["extensions"])
                self.assertEqual(current["topology"], baseline["topology"])
                self.assertEqual(
                    current["sizes"]["entry_apparent_bytes"],
                    baseline["sizes"]["entry_apparent_bytes"],
                )
                self.assertEqual(verify_corpus(run_root), current)
            file_paths = {
                path
                for path in current["transition_history"][-1]["changed_paths"]
                if path != "." and (run_root / "corpus" / path).suffix
            }
            self.assertEqual(
                len({str(Path(path).parent) for path in file_paths}),
                1,
            )

    def test_churn_transitions_must_be_applied_in_recipe_order(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            run_root = reserve_run_directory(Path(temporary_directory))
            create_corpus(run_root, "churn-local", target_entries=40)

            with self.assertRaisesRegex(CorpusError, "expected 'one-change'"):
                apply_transition(run_root, "modify-1pct")

    def test_complete_transition_sequence_is_deterministic(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            base = Path(temporary_directory)
            runs = [reserve_run_directory(base), reserve_run_directory(base)]
            manifests = [
                create_corpus(run_root, "churn-distributed", target_entries=120)
                for run_root in runs
            ]
            for transition_id in ("one-change", "modify-1pct", "mixed-1pct"):
                manifests = [
                    apply_transition(run_root, transition_id) for run_root in runs
                ]
                self.assertEqual(
                    manifests[0]["semantic_digest"],
                    manifests[1]["semantic_digest"],
                )
                first_transition = manifests[0]["transition_history"][-1]
                second_transition = manifests[1]["transition_history"][-1]
                self.assertEqual(
                    first_transition["changed_paths"],
                    second_transition["changed_paths"],
                )
                self.assertEqual(
                    first_transition["operations"], second_transition["operations"]
                )
            for run_root in runs:
                with self.assertRaisesRegex(CorpusError, "no remaining transitions"):
                    apply_transition(run_root, "mixed-1pct")

    def test_mutation_refuses_a_tampered_precondition(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            run_root = reserve_run_directory(Path(temporary_directory))
            manifest = create_corpus(
                run_root, "churn-distributed", target_entries=40
            )
            file_record = next(
                record for record in manifest["records"] if record["kind"] == "file"
            )
            with (run_root / "corpus" / file_record["path"]).open("ab") as output:
                output.write(b"tamper")

            with self.assertRaisesRegex(CorpusError, "verification failed"):
                apply_transition(run_root, "one-change")

            stored = json.loads(
                (run_root / "observed-corpus.json").read_text(encoding="utf-8")
            )
            self.assertEqual(stored["state"]["id"], "baseline")
            self.assertNotIn("transition_history", stored)

    def test_mixed_transition_requires_three_independent_files(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            run_root = reserve_run_directory(Path(temporary_directory))
            create_corpus(run_root, "churn-local", target_entries=2)
            apply_transition(run_root, "one-change")
            apply_transition(run_root, "modify-1pct")
            with self.assertRaisesRegex(CorpusError, "needs 3 independent files"):
                apply_transition(run_root, "mixed-1pct")


class CorpusSafetyTests(unittest.TestCase):
    def test_cleanup_removes_only_a_valid_marked_run(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            base = Path(temporary_directory)
            run_root = reserve_run_directory(base)
            create_corpus(run_root, "balanced", target_entries=10)

            cleanup_run_directory(run_root)

            self.assertFalse(run_root.exists())
            self.assertTrue(base.exists())

    def test_cleanup_rejects_missing_or_mismatched_marker(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            base = Path(temporary_directory)
            unmarked = base / "fdu-perf-unmarked"
            unmarked.mkdir()
            with self.assertRaisesRegex(CorpusError, "marker"):
                cleanup_run_directory(unmarked)

            run_root = reserve_run_directory(base)
            marker_path = run_root / ".fdu-perf-run-v1"
            marker = json.loads(marker_path.read_text(encoding="utf-8"))
            marker["run_id"] = "another-run"
            marker_path.write_text(json.dumps(marker), encoding="utf-8")
            with self.assertRaisesRegex(CorpusError, "does not identify"):
                cleanup_run_directory(run_root)

    def test_manifest_hash_rejects_manual_edits(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            run_root = reserve_run_directory(Path(temporary_directory))
            create_corpus(run_root, "balanced", target_entries=10)
            manifest_path = run_root / "observed-corpus.json"
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["state"]["id"] = "forged"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            with self.assertRaisesRegex(CorpusError, "manifest hash is invalid"):
                verify_corpus(run_root)

    def test_manifest_shape_fails_closed_even_with_a_matching_hash(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            run_root = reserve_run_directory(Path(temporary_directory))
            create_corpus(run_root, "balanced", target_entries=10)
            manifest_path = run_root / "observed-corpus.json"
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["oracle"] = []
            manifest.pop("manifest_hash")
            canonical = json.dumps(
                manifest,
                ensure_ascii=True,
                separators=(",", ":"),
                sort_keys=True,
            ).encode("utf-8")
            manifest["manifest_hash"] = hashlib.sha256(canonical).hexdigest()
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            with self.assertRaisesRegex(CorpusError, "manifest field 'oracle'"):
                verify_corpus(run_root)

    def test_manifest_rejects_forged_rollup_components_with_a_matching_hash(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            run_root = reserve_run_directory(Path(temporary_directory))
            create_corpus(run_root, "balanced", target_entries=10)
            manifest_path = run_root / "observed-corpus.json"
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["oracle"]["rollup_digest_components"]["count"] += 1
            manifest.pop("manifest_hash")
            canonical = json.dumps(
                manifest,
                ensure_ascii=True,
                separators=(",", ":"),
                sort_keys=True,
            ).encode("utf-8")
            manifest["manifest_hash"] = hashlib.sha256(canonical).hexdigest()
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            with self.assertRaisesRegex(CorpusError, "roll-up digest does not match"):
                verify_corpus(run_root)

    def test_oversized_manifest_is_rejected_before_json_allocation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            run_root = reserve_run_directory(Path(temporary_directory))
            create_corpus(run_root, "balanced", target_entries=10)
            manifest_path = run_root / "observed-corpus.json"
            with manifest_path.open("wb") as output:
                output.truncate(8 * 1024 * 1024 + 1)

            with self.assertRaisesRegex(CorpusError, "exceeds"):
                verify_corpus(run_root)

    def test_active_operation_lock_rejects_concurrent_access(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            run_root = reserve_run_directory(Path(temporary_directory))
            create_corpus(run_root, "balanced", target_entries=10)
            lock_path = run_root / ".fdu-perf-operation-lock"
            lock_path.mkdir()

            with self.assertRaisesRegex(CorpusError, "already active"):
                verify_corpus(run_root)
            with self.assertRaisesRegex(CorpusError, "already active"):
                cleanup_run_directory(run_root)

            lock_path.rmdir()
            self.assertEqual(verify_corpus(run_root)["state"]["id"], "baseline")

    def test_failed_operation_releases_its_lock(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            run_root = reserve_run_directory(Path(temporary_directory))
            manifest = create_corpus(run_root, "balanced", target_entries=10)
            file_record = next(
                record for record in manifest["records"] if record["kind"] == "file"
            )
            with (run_root / "corpus" / file_record["path"]).open("ab") as output:
                output.write(b"tamper")

            with self.assertRaises(CorpusError):
                verify_corpus(run_root)

            self.assertFalse((run_root / ".fdu-perf-operation-lock").exists())

    def test_cleanup_rejects_workspace_and_symlink_targets(self) -> None:
        with self.assertRaisesRegex(CorpusError, "workspace"):
            cleanup_run_directory(Path(__file__).resolve().parents[2])

        with tempfile.TemporaryDirectory() as temporary_directory:
            base = Path(temporary_directory)
            run_root = reserve_run_directory(base)
            linked = base / "linked-run"
            try:
                os.symlink(run_root, linked)
            except (NotImplementedError, OSError) as error:
                self.skipTest(f"symlinks unavailable: {error}")
            with self.assertRaisesRegex(CorpusError, "symlink"):
                cleanup_run_directory(linked)

    def test_create_refuses_to_replace_an_existing_corpus(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            run_root = reserve_run_directory(Path(temporary_directory))
            create_corpus(run_root, "balanced", target_entries=10)
            with self.assertRaisesRegex(CorpusError, "already exists"):
                create_corpus(run_root, "balanced", target_entries=10)

    def test_reservation_rejects_a_file_as_the_base(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            base_file = Path(temporary_directory) / "not-a-directory"
            base_file.write_text("fixture", encoding="utf-8")

            with self.assertRaisesRegex(CorpusError, "cannot prepare"):
                reserve_run_directory(base_file)


class RecipeValidationTests(unittest.TestCase):
    def test_recipe_identifiers_and_sizes_fail_closed(self) -> None:
        with self.assertRaisesRegex(CorpusError, "unknown recipe"):
            load_recipe("missing")
        with self.assertRaisesRegex(CorpusError, "greater than or equal"):
            load_recipe("balanced", target_entries=0)
        with self.assertRaisesRegex(CorpusError, "safety limit"):
            load_recipe("balanced", target_entries=10_000_001)
        with self.assertRaisesRegex(CorpusError, "does not accept"):
            load_recipe("contract", target_entries=13)


if __name__ == "__main__":
    unittest.main()
