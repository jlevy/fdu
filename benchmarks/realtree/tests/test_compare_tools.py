from __future__ import annotations

import json
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
                            "semantic_sha256": None,
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

    def test_statistics_accept_the_summary_anchor(self) -> None:
        samples = []
        for ordinal in range(3):
            for name, wall in (("fdu-transient-summary", 100), ("dumac", 80)):
                samples.append(
                    {
                        "pair": "dumac",
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
                        "semantic_sha256": "same",
                    }
                )
        document = {
            "anchor": "fdu-transient-summary",
            "competitor_order": ["dumac"],
            "samples": samples,
        }

        statistics = compare_tools._statistics(document)

        self.assertEqual(
            statistics["dumac"]["competitor_vs_fdu"]["wall_ns"]["median_change_pct"],
            -20.0,
        )

    def test_output_directory_cannot_be_inside_the_subject(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            with self.assertRaisesRegex(compare_tools.ComparisonError, "outside --root"):
                compare_tools._require_external_output(root, root / "results")

            with self.assertRaisesRegex(compare_tools.ComparisonError, "outside --root"):
                compare_tools._require_external_file(root, root / "tree.json")

            compare_tools._require_external_output(root, root.parent / "external-results")
            compare_tools._require_external_file(root, root.parent / "external-tree.json")

    def test_render_discloses_hardlink_semantics_without_names(self) -> None:
        note = compare_tools._hardlink_note(
            {
                "hardlinks": {
                    "duplicate_file_entries": 3,
                    "duplicate_allocated_bytes": 8192,
                }
            }
        )

        self.assertIn("3 duplicate", note)
        self.assertIn("8,192", note)
        self.assertIn("not an assertion", note)

    def test_dumac_is_explicitly_total_only(self) -> None:
        contract = compare_tools.CONTRACTS["dumac"]

        self.assertEqual(contract.work_class, "total-only")
        self.assertIn("getattrlistbulk", contract.description)

    def test_fdu_index_summary_disables_the_snapshot_but_discloses_the_index(self) -> None:
        contract = compare_tools.CONTRACTS["fdu-index-summary"]

        self.assertEqual(contract.work_class, "indexed-summary")
        self.assertIn("--cache", contract.argv)
        self.assertIn("off", contract.argv)
        self.assertIn("summary", contract.argv)
        self.assertIn("reusable exact metadata index", contract.description)

    def test_fdu_transient_summary_discloses_bounded_retention(self) -> None:
        contract = compare_tools.CONTRACTS["fdu-transient-summary"]

        self.assertEqual(contract.work_class, "transient-summary")
        self.assertIn("--cache", contract.argv)
        self.assertIn("off", contract.argv)
        self.assertIn("summary", contract.argv)
        self.assertIn("no path index", contract.description)

    def test_summary_semantic_digest_ignores_run_specific_envelope_fields(self) -> None:
        first = {
            "schema": "fdu.report/1",
            "generator": "fdu old",
            "root": "/private/one",
            "scan_started_at": "2026-01-01T00:00:00Z",
            "generated_at": "2026-01-01T00:00:01Z",
            "source": "cold_scan",
            "freshness": "fresh",
            "complete": True,
            "errors": [],
            "reports": [{"view": "summary", "summary": {"files": 3}}],
        }
        second = {
            **first,
            "generator": "fdu new",
            "root": "/private/two",
            "scan_started_at": "2027-01-01T00:00:00Z",
            "generated_at": "2027-01-01T00:00:01Z",
        }

        first_digest, first_error = compare_tools._summary_semantic_digest(
            json.dumps(first).encode()
        )
        second_digest, second_error = compare_tools._summary_semantic_digest(
            json.dumps(second).encode()
        )

        self.assertIsNone(first_error)
        self.assertIsNone(second_error)
        self.assertEqual(first_digest, second_digest)

    def test_summary_semantic_mismatch_invalidates_both_sides_of_the_pair(self) -> None:
        samples = [
            {
                "pair": "fdu-index-summary",
                "tool": "fdu-transient-summary",
                "ordinal": 2,
                "warmup": False,
                "valid": True,
                "reasons": [],
                "semantic_sha256": "compact",
            },
            {
                "pair": "fdu-index-summary",
                "tool": "fdu-index-summary",
                "ordinal": 2,
                "warmup": False,
                "valid": True,
                "reasons": [],
                "semantic_sha256": "indexed",
            },
        ]

        mismatches = compare_tools._invalidate_semantic_mismatches(
            samples, anchor="fdu-transient-summary"
        )

        self.assertEqual(len(mismatches), 1)
        self.assertFalse(samples[0]["valid"])
        self.assertFalse(samples[1]["valid"])
        self.assertIn("semantics differ", samples[0]["reasons"][0])

    def test_matching_summary_semantics_remain_valid(self) -> None:
        samples = [
            {
                "pair": "fdu-index-summary",
                "tool": name,
                "ordinal": 0,
                "warmup": False,
                "valid": True,
                "reasons": [],
                "semantic_sha256": "same",
            }
            for name in ("fdu-transient-summary", "fdu-index-summary")
        ]

        mismatches = compare_tools._invalidate_semantic_mismatches(
            samples, anchor="fdu-transient-summary"
        )

        self.assertEqual(mismatches, [])
        self.assertTrue(all(sample["valid"] for sample in samples))


if __name__ == "__main__":
    unittest.main()
