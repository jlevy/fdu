from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path

from benchmarks.generate import main


class GeneratorCommandTests(unittest.TestCase):
    def test_create_verify_and_cleanup_emit_structured_results(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            base = Path(temporary_directory)
            create_output = io.StringIO()
            with contextlib.redirect_stdout(create_output):
                create_status = main(
                    [
                        "create",
                        "--recipe",
                        "balanced",
                        "--entries",
                        "20",
                        "--work-dir",
                        str(base),
                    ]
                )
            created = json.loads(create_output.getvalue())
            run_root = Path(created["run_root"])

            verify_output = io.StringIO()
            with contextlib.redirect_stdout(verify_output):
                verify_status = main(["verify", "--run-root", str(run_root)])
            verified = json.loads(verify_output.getvalue())

            cleanup_output = io.StringIO()
            with contextlib.redirect_stdout(cleanup_output):
                cleanup_status = main(["cleanup", "--run-root", str(run_root)])
            cleaned = json.loads(cleanup_output.getvalue())

            self.assertEqual(create_status, 0)
            self.assertEqual(verify_status, 0)
            self.assertEqual(cleanup_status, 0)
            self.assertEqual(created["manifest_hash"], verified["manifest_hash"])
            self.assertTrue(cleaned["ok"])
            self.assertFalse(run_root.exists())

    def test_domain_error_is_one_json_object_on_stderr(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            base = Path(temporary_directory)
            error_output = io.StringIO()
            with contextlib.redirect_stderr(error_output):
                status = main(
                    [
                        "create",
                        "--recipe",
                        "missing",
                        "--work-dir",
                        str(base),
                    ]
                )
            self.assertEqual(list(base.iterdir()), [])

        error = json.loads(error_output.getvalue())
        self.assertEqual(status, 2)
        self.assertFalse(error["ok"])
        self.assertEqual(error["error_type"], "corpus_error")


if __name__ == "__main__":
    unittest.main()
