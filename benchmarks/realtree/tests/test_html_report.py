"""Tests for the generated data behind the interactive research report."""

from __future__ import annotations

import json
import tempfile
import unittest
from html.parser import HTMLParser
from pathlib import Path

from benchmarks.realtree import html_report


class _AssetParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.scripts: list[str] = []
        self.stylesheets: list[str] = []
        self.ids: set[str] = set()

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        values = dict(attrs)
        if values.get("id"):
            self.ids.add(str(values["id"]))
        if tag == "script" and values.get("src"):
            self.scripts.append(str(values["src"]))
        if tag == "link" and values.get("rel") == "stylesheet" and values.get("href"):
            self.stylesheets.append(str(values["href"]))


class ResearchReportDataTests(unittest.TestCase):
    def test_the_report_is_a_complete_projection_of_the_experiment_ledger(self) -> None:
        data = html_report.build_report_data()

        self.assertEqual(data["summary"]["experiment_count"], 56)
        self.assertEqual(data["summary"]["decision_counts"]["accepted"], 27)
        self.assertEqual(data["summary"]["decision_counts"]["rejected"], 24)
        self.assertEqual(data["summary"]["platform_counts"], {"Linux": 3, "macOS": 53})
        self.assertEqual(
            [experiment["id"] for experiment in data["experiments"]],
            [f"exp-{number:03d}" for number in range(56)],
        )
        self.assertEqual(data["softschema_table"]["row_count"], 56)
        self.assertEqual(
            data["softschema_table"]["contract"], "fdu.performance:Experiment/v1"
        )

        early_result = data["experiments"][1]
        self.assertEqual(early_result["effect"], "improved")
        self.assertEqual(early_result["primary"]["ci95_pct"], [-51.024, -43.743])

    def test_report_data_names_derived_cross_platform_results_as_derived(self) -> None:
        data = html_report.build_report_data()
        composite = data["current_outcomes"]["composite"]

        self.assertEqual(composite["kind"], "derived")
        self.assertEqual(composite["method"], "weighted_geometric_latency_ratio")
        self.assertAlmostEqual(
            composite["default_latency_reduction_pct"], 12.7123, places=4
        )
        self.assertEqual(
            sum(cell["default_weight"] for cell in composite["cells"]), 1.0
        )

    def test_generated_javascript_is_deterministic_and_carries_no_local_paths(
        self,
    ) -> None:
        data = html_report.build_report_data()
        rendered = html_report.render_report_data_js(data)

        self.assertEqual(rendered, html_report.render_report_data_js(data))
        self.assertTrue(rendered.startswith("window.FDU_PERFORMANCE_RESEARCH = "))
        self.assertNotIn("/Users/", rendered)
        self.assertNotIn("file://", rendered)

        payload = rendered.removeprefix(
            "window.FDU_PERFORMANCE_RESEARCH = "
        ).removesuffix(";\n")
        self.assertEqual(json.loads(payload)["summary"]["experiment_count"], 56)

    def test_check_mode_detects_and_then_accepts_generated_output(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "report-data.js"

            self.assertFalse(html_report.output_is_current(output))
            html_report.write_report_data(output)
            self.assertTrue(html_report.output_is_current(output))

    def test_static_report_is_local_first_and_exposes_the_interactive_surfaces(
        self,
    ) -> None:
        report_directory = (
            html_report.REPOSITORY_ROOT / "docs/project/reports/performance-research"
        )
        source = (report_directory / "index.html").read_text(encoding="utf-8")
        parser = _AssetParser()
        parser.feed(source)

        self.assertEqual(parser.scripts, ["report-data.js", "report.js"])
        self.assertEqual(parser.stylesheets, ["report.css"])
        self.assertFalse(any(value.startswith("http") for value in parser.scripts))
        self.assertFalse(any(value.startswith("http") for value in parser.stylesheets))
        self.assertTrue(
            {
                "composite-score",
                "experiment-plot",
                "experiment-detail",
                "artifact-body",
                "record-detail",
            }.issubset(parser.ids)
        )


if __name__ == "__main__":
    unittest.main()
