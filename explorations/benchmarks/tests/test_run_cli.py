from __future__ import annotations

import contextlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

from benchmarks.run import main


FIXTURES = Path(__file__).resolve().parent / "fixtures"


class PerformanceCommandTests(unittest.TestCase):
    def test_execute_validate_render_and_compare_form_one_workflow(self) -> None:
        scenario = json.loads(
            (FIXTURES / "schema" / "scenario-valid.json").read_text(encoding="utf-8")
        )
        scenario["scenarios"][0]["method"].update({"trials": 1, "warmups": 0})
        scenario["scenarios"][0]["validation"]["stdout_json"]["equals"][
            "sentinel"
        ] = None
        with tempfile.TemporaryDirectory() as temporary_directory:
            base = Path(temporary_directory)
            scenario_path = base / "scenarios.json"
            scenario_path.write_text(json.dumps(scenario), encoding="utf-8")
            execute_output = io.StringIO()
            with contextlib.redirect_stdout(execute_output):
                execute_status = main(
                    [
                        "execute",
                        "--scenarios",
                        str(scenario_path),
                        "--executable",
                        f"fixture={sys.executable}",
                        "--executable",
                        f"fixture={FIXTURES / 'fake_benchmark.py'}",
                        "--work-dir",
                        str(base / "work"),
                        "--output-dir",
                        str(base / "results"),
                        "--order-seed",
                        "cli-workflow",
                    ]
                )
            executed = json.loads(execute_output.getvalue())
            result_path = Path(executed["result_path"])

            validate_output = io.StringIO()
            with contextlib.redirect_stdout(validate_output):
                validate_status = main(
                    ["validate", "--kind", "result", "--path", str(result_path)]
                )

            report_path = base / "report.md"
            render_output = io.StringIO()
            with contextlib.redirect_stdout(render_output):
                render_status = main(
                    [
                        "render",
                        "--result",
                        str(result_path),
                        "--output",
                        str(report_path),
                    ]
                )

            compare_output = io.StringIO()
            with contextlib.redirect_stdout(compare_output):
                compare_status = main(
                    [
                        "compare",
                        "--current",
                        str(result_path),
                        "--baseline",
                        str(result_path),
                    ]
                )

            self.assertEqual(execute_status, 0)
            self.assertEqual(validate_status, 0)
            self.assertEqual(render_status, 0)
            self.assertEqual(compare_status, 0)
            self.assertEqual(executed["valid_timed_trials"], 1)
            self.assertTrue(report_path.read_text(encoding="utf-8").startswith("# fdu"))
            self.assertTrue(json.loads(compare_output.getvalue())["compatible"])

    def test_errors_are_one_structured_object_on_stderr(self) -> None:
        error_output = io.StringIO()
        with contextlib.redirect_stderr(error_output):
            status = main(
                ["validate", "--kind", "result", "--path", "/does/not/exist"]
            )

        error = json.loads(error_output.getvalue())
        self.assertEqual(status, 2)
        self.assertFalse(error["ok"])
        self.assertEqual(error["error_type"], "performance_evidence_error")


if __name__ == "__main__":
    unittest.main()
