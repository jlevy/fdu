from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from benchmarks.corpus import (
    CorpusError,
    cleanup_run_directory,
    create_corpus,
    reserve_run_directory,
    verify_corpus,
)
from benchmarks import corpus_cache
from benchmarks.corpus_cache import CorpusBasePool


class CorpusBasePoolTests(unittest.TestCase):
    def test_cleanup_failure_is_attached_to_the_primary_error(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            pool = CorpusBasePool(Path(temporary_directory))
            with mock.patch.object(
                pool,
                "close",
                side_effect=CorpusError("intentional cleanup failure"),
            ):
                with self.assertRaisesRegex(RuntimeError, "primary failure") as caught:
                    with pool:
                        raise RuntimeError("primary failure")

        self.assertEqual(
            caught.exception.__notes__,
            [
                "corpus base-pool cleanup also failed: "
                "intentional cleanup failure"
            ],
        )

    def test_bounded_copy_fails_closed_at_logical_byte_safety_limit(self) -> None:
        strategy = corpus_cache._CopyStrategy("bounded-copy-v1", None)
        with tempfile.TemporaryDirectory() as temporary_directory:
            work_root = Path(temporary_directory)
            run_root = reserve_run_directory(work_root)
            with mock.patch(
                "benchmarks.corpus_cache._choose_copy_strategy",
                return_value=strategy,
            ), mock.patch(
                "benchmarks.corpus_cache.MAX_BOUNDED_COPY_BYTES",
                0,
            ):
                with CorpusBasePool(work_root) as pool:
                    with self.assertRaisesRegex(
                        CorpusError, "logical-byte safety limit"
                    ):
                        pool.materialize(
                            run_root,
                            "contract",
                            target_entries=14,
                            seed=None,
                        )

            cleanup_run_directory(run_root)
            self.assertEqual(list(work_root.iterdir()), [])

    def test_bounded_copy_preserves_internal_hardlinks(self) -> None:
        strategy = corpus_cache._CopyStrategy("bounded-copy-v1", None)
        with tempfile.TemporaryDirectory() as temporary_directory:
            work_root = Path(temporary_directory)
            run_root = reserve_run_directory(work_root)
            with mock.patch(
                "benchmarks.corpus_cache._choose_copy_strategy",
                return_value=strategy,
            ):
                with CorpusBasePool(work_root) as pool:
                    manifest, evidence = pool.materialize(
                        run_root,
                        "contract",
                        target_entries=14,
                        seed=None,
                    )

            self.assertEqual(evidence["copy_strategy"], "bounded-copy-v1")
            self.assertEqual(evidence["cloned_files"], 0)
            self.assertGreater(evidence["copied_files"], 0)
            if manifest["capabilities"]["hardlink"]["supported"]:
                linked = next(
                    record
                    for record in manifest["records"]
                    if record.get("hardlink_to") is not None
                )
                link_path = run_root / "corpus" / linked["path"]
                source_path = run_root / "corpus" / linked["hardlink_to"]
                self.assertTrue(link_path.samefile(source_path))
                self.assertEqual(evidence["hardlinks"], 1)
            self.assertEqual(verify_corpus(run_root), manifest)

            cleanup_run_directory(run_root)
            self.assertEqual(list(work_root.iterdir()), [])

    def test_one_pristine_base_materializes_independent_verified_trials(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            work_root = Path(temporary_directory)
            first_run = reserve_run_directory(work_root)
            second_run = reserve_run_directory(work_root)
            with mock.patch(
                "benchmarks.corpus_cache.create_corpus",
                wraps=create_corpus,
            ) as create:
                with CorpusBasePool(work_root) as pool:
                    first, first_evidence = pool.materialize(
                        first_run,
                        "contract",
                        target_entries=14,
                        seed=None,
                    )
                    second, second_evidence = pool.materialize(
                        second_run,
                        "contract",
                        target_entries=14,
                        seed=None,
                    )

                    source_record = next(
                        record
                        for record in first["records"]
                        if record["kind"] == "file"
                        and record.get("hardlink_to") is None
                        and record["size"] > 0
                    )
                    first_path = first_run / "corpus" / source_record["path"]
                    second_path = second_run / "corpus" / source_record["path"]
                    second_contents = second_path.read_bytes()
                    with first_path.open("r+b") as output:
                        output.write(b"X")

                    with self.assertRaisesRegex(CorpusError, "verification failed"):
                        verify_corpus(first_run)
                    self.assertEqual(second_path.read_bytes(), second_contents)
                    self.assertEqual(verify_corpus(second_run), second)

            self.assertEqual(create.call_count, 1)
            self.assertTrue(first_evidence["base_created"])
            self.assertFalse(second_evidence["base_created"])
            self.assertEqual(
                first_evidence["base_cache_key"],
                second_evidence["base_cache_key"],
            )
            self.assertGreater(first_evidence["base_generation_ns"], 0)
            self.assertIsNone(second_evidence["base_generation_ns"])
            self.assertGreater(first_evidence["materialization_ns"], 0)
            self.assertGreater(second_evidence["materialization_ns"], 0)
            self.assertGreater(
                first_evidence["cloned_files"] + first_evidence["copied_files"],
                0,
            )
            if first["capabilities"]["hardlink"]["supported"]:
                self.assertEqual(first["counts"]["hardlink_groups"], 1)
                self.assertEqual(second["counts"]["hardlink_groups"], 1)

            cleanup_run_directory(first_run)
            cleanup_run_directory(second_run)
            self.assertEqual(list(work_root.iterdir()), [])


if __name__ == "__main__":
    unittest.main()
