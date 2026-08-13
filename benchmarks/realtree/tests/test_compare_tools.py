from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from benchmarks.realtree import compare_tools


def tool(name: str) -> compare_tools.Tool:
    return compare_tools.Tool(compare_tools.CONTRACTS[name], Path("/bin/true"))


class ToolComparisonTests(unittest.TestCase):
    def test_schedule_keeps_pairs_adjacent_and_alternates_the_anchor(self) -> None:
        competitors = [tool("dust"), tool("gdu"), tool("pdu")]
        schedule = compare_tools._schedule(competitors, trials=4, warmups=1)

        self.assertEqual(len(schedule), 15)
        for ordinal in range(-1, 4):
            at_ordinal = [entry for entry in schedule if entry[1] == ordinal]
            self.assertCountEqual(
                [entry[0].contract.name for entry in at_ordinal],
                ["dust", "gdu", "pdu"],
            )
        for competitor in competitors:
            orders = [
                anchor_first
                for scheduled, _ordinal, _warmup, anchor_first in schedule
                if scheduled == competitor
            ]
            self.assertIn(True, orders)
            self.assertIn(False, orders)

    def test_identity_contains_no_binary_path(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            binary = Path(raw) / "private-name"
            binary.write_bytes(b"fixture")
            candidate = compare_tools.Tool(compare_tools.CONTRACTS["bsd-du"], binary)

            identity = compare_tools._identity(candidate)

        self.assertNotIn(raw, str(identity))
        self.assertEqual(identity["binary_size_bytes"], 7)
        self.assertEqual(identity["command"], ["{binary}", "-sk", "{root}"])

    def test_statistics_compare_each_tool_only_with_its_adjacent_anchor(self) -> None:
        samples = []
        for ordinal in range(3):
            for pair, competitor_wall in (("dust", 200), ("gdu", 50)):
                for name, wall in (("fdu", 100), (pair, competitor_wall)):
                    samples.append(
                        {
                            "pair": pair,
                            "tool": name,
                            "ordinal": ordinal,
                            "warmup": False,
                            "valid": True,
                            "metrics": {
                                "wall_ns": wall,
                                "cpu_ns": wall,
                                "user_cpu_ns": wall,
                                "system_cpu_ns": 0,
                                "peak_rss_bytes": 1,
                                "major_faults": 0,
                                "minor_faults": 0,
                                "input_blocks": 0,
                                "output_blocks": 0,
                                "voluntary_context_switches": 0,
                                "involuntary_context_switches": 0,
                            },
                        }
                    )
        document = {
            "anchor": "fdu",
            "competitor_order": ["dust", "gdu"],
            "samples": samples,
        }

        statistics = compare_tools._statistics(document)
        overall = compare_tools._overall(document)

        self.assertEqual(
            statistics["dust"]["competitor_vs_fdu"]["wall_ns"]["median_change_pct"],
            100.0,
        )
        self.assertEqual(
            statistics["gdu"]["competitor_vs_fdu"]["wall_ns"]["median_change_pct"],
            -50.0,
        )
        self.assertEqual(overall["fdu"]["samples"], 6)
        self.assertEqual(overall["dust"]["metrics"]["wall_ns"]["median"], 200)

    def test_output_directory_cannot_be_inside_the_subject(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            with self.assertRaisesRegex(compare_tools.ComparisonError, "outside --root"):
                compare_tools._require_external_output(root, root / "results")

            with self.assertRaisesRegex(compare_tools.ComparisonError, "outside --root"):
                compare_tools._require_external_file(root, root / "tree.json")

            compare_tools._require_external_output(root, root.parent / "external-results")
            compare_tools._require_external_file(root, root.parent / "external-tree.json")


if __name__ == "__main__":
    unittest.main()
