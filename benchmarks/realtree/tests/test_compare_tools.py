from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from benchmarks.realtree import compare_tools


def tool(name: str) -> compare_tools.Tool:
    return compare_tools.Tool(name, compare_tools.CONTRACTS[name], Path("/bin/true"))


class ToolComparisonTests(unittest.TestCase):
    def test_held_out_release_comparison_rejects_an_uncontrolled_host(self) -> None:
        with self.assertRaisesRegex(compare_tools.ComparisonError, "held-out release evidence"):
            compare_tools.run(
                root=Path("/does/not-matter"),
                label="fixture",
                anchor=tool("fdu-transient-summary"),
                competitors=[tool("dust")],
                trials=3,
                warmups=1,
                baseline_fingerprint=None,
                baseline_output=None,
                storage="fixture",
                campaign_stage="held-out",
                host_regime="uncontrolled",
            )

    def test_held_out_release_comparison_requires_a_traceable_summary_plan(self) -> None:
        with self.assertRaisesRegex(
            compare_tools.ComparisonError, "complete installed-command policy trace"
        ):
            compare_tools.run(
                root=Path("/does/not-matter"),
                label="fixture",
                anchor=tool("fdu"),
                competitors=[tool("dust")],
                trials=3,
                warmups=1,
                baseline_fingerprint=None,
                baseline_output=None,
                storage="fixture",
                campaign_stage="held-out",
                host_regime="quiet",
            )

    def test_warm_cache_evidence_requires_an_explicit_tool_warmup(self) -> None:
        with self.assertRaisesRegex(compare_tools.ComparisonError, "at least one warmup"):
            compare_tools._warm_cache_evidence(0)

        evidence = compare_tools._warm_cache_evidence(3)

        self.assertEqual(evidence["pre_run_full_tree_fingerprints"], 1)
        self.assertEqual(evidence["minimum_full_tree_warmups_per_tool"], 3)
        self.assertEqual(evidence["residency_claim"], "repeated-workload-steady-state")
        note = compare_tools._cache_note({"os_cache": "warm-steady", "os_cache_evidence": evidence})
        self.assertIn("3 full-tree warmups per tool", note)
        self.assertIn("not a claim that every metadata object remained resident", note)

    def test_target_scope_requires_apple_silicon_apfs_and_stable_host_state(self) -> None:
        manifest = {
            "collectors": {"process_rusage": {"supported": True}},
            "filesystem": {"solid_state": True, "type": "apfs"},
            "host": {
                "arch": "arm64",
                "efficiency_cores": 2,
                "performance_cores": 8,
                "power": {"source": "AC"},
                "system": "Darwin",
                "thermal": {"pressure": "normal"},
            },
        }

        self.assertEqual(compare_tools._apple_silicon_apfs_scope_reasons(manifest), [])
        manifest["host"]["power"] = {"source": "battery"}
        manifest["filesystem"]["type"] = "ext4"
        reasons = compare_tools._apple_silicon_apfs_scope_reasons(manifest)
        self.assertIn("AC power is not established", reasons)
        self.assertIn("subject filesystem is not APFS", reasons)

    def test_schedule_keeps_pairs_adjacent_and_alternates_the_anchor(self) -> None:
        competitors = [tool("dust"), tool("gdu"), tool("pdu")]
        schedule = compare_tools._schedule(competitors, trials=4, warmups=1)

        self.assertEqual(len(schedule), 15)
        for ordinal in range(-1, 4):
            at_ordinal = [entry for entry in schedule if entry[1] == ordinal]
            self.assertCountEqual(
                [entry[0].name for entry in at_ordinal],
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
            candidate = compare_tools.Tool("bsd-du", compare_tools.CONTRACTS["bsd-du"], binary)

            identity = compare_tools._identity(candidate)

        self.assertNotIn(raw, str(identity))
        self.assertEqual(identity["binary_size_bytes"], 7)
        self.assertEqual(identity["contract"], "bsd-du")
        self.assertEqual(identity["command"], ["{binary}", "-sk", "{root}"])

    def test_aliases_compare_two_binaries_under_the_same_contract(self) -> None:
        binary = str(Path("/usr/bin/true").resolve())
        candidate = compare_tools._parse_tool(f"h62:fdu-transient-summary={binary}")
        control = compare_tools._parse_tool(f"h59:fdu-transient-summary={binary}")

        self.assertEqual(candidate.name, "h62")
        self.assertEqual(control.name, "h59")
        self.assertEqual(candidate.contract, control.contract)
        self.assertEqual(compare_tools._identity(candidate)["contract"], "fdu-transient-summary")

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
        self.assertEqual(
            statistics["dust"]["fdu_vs_competitor"]["wall_ns"]["noninferiority"],
            "superior",
        )
        self.assertEqual(
            statistics["dust"]["fdu_vs_competitor"]["qualification"]["classification"],
            "superior",
        )

    def test_release_qualification_needs_held_out_complete_pairs(self) -> None:
        entry = {
            "pairs": 12,
            "ci95_change_pct": [-2.0, 2.0],
            "ci95_delta": [0, 0],
            "noninferiority": "noninferior",
        }
        comparison = {
            "wall_ns": entry,
            **{metric: entry for metric in compare_tools.measure.RESOURCE_REGRESSION_LIMITS_PCT},
            "major_faults": {**entry, "ci95_delta": [-1, 0]},
        }

        held_out = compare_tools._release_qualification(
            comparison,
            campaign_stage="held-out",
            required_pairs=12,
            policy_stability={"stable": True},
        )
        discovery = compare_tools._release_qualification(
            comparison, campaign_stage="discovery", required_pairs=12
        )
        missing_pair = compare_tools._release_qualification(
            comparison,
            campaign_stage="held-out",
            required_pairs=13,
            policy_stability={"stable": True},
        )
        unstable = compare_tools._release_qualification(
            comparison,
            campaign_stage="held-out",
            required_pairs=12,
            policy_stability={"stable": False},
        )

        self.assertTrue(held_out["confirmable"])
        self.assertFalse(discovery["confirmable"])
        self.assertFalse(missing_pair["confirmable"])
        self.assertFalse(unstable["confirmable"])
        self.assertEqual(missing_pair["classification"], "inconclusive")

    def test_installed_policy_stability_fails_on_harm_or_missing_history(self) -> None:
        def sample(ordinal: int, decisions: list[str], *, valid: bool = True):
            return {
                "pair": "dust",
                "tool": "fdu",
                "ordinal": ordinal,
                "warmup": False,
                "valid": valid,
                "scan_diagnostics": {
                    "worker_policy": {
                        "outcome": "held",
                        "windows": [{"decision": decision} for decision in decisions],
                    }
                },
            }

        stable = compare_tools._installed_policy_stability(
            [sample(ordinal, ["hold", "observe_fast"]) for ordinal in range(3)],
            pair="dust",
            anchor="fdu",
            required_samples=3,
        )
        harmful = compare_tools._installed_policy_stability(
            [
                sample(0, ["hold", "observe_fast"]),
                sample(1, ["hold", "observe_slow"]),
                sample(2, ["hold", "observe_fast"], valid=False),
            ],
            pair="dust",
            anchor="fdu",
            required_samples=3,
        )

        self.assertTrue(stable["stable"])
        self.assertFalse(harmful["stable"])
        self.assertEqual(harmful["harmful_histories"], 1)
        self.assertEqual(harmful["missing_histories"], 1)

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

    def test_dust_adapter_parses_and_verifies_one_exact_total(self) -> None:
        oracle = {
            "counts": {"symlinks": 0},
            "hardlinks": {
                "duplicate_allocated_bytes": 4_096,
                "duplicate_file_entries": 1,
            },
            "sizes": {"allocated_bytes": 12_288},
        }

        observed, error = compare_tools._dust_semantics("8192B ┌── corpus\n".encode(), "", oracle)

        self.assertEqual(observed, 8_192)
        self.assertIsNone(error)

    def test_dust_adapter_fails_closed_on_every_invalid_output_class(self) -> None:
        oracle = {
            "counts": {"symlinks": 0},
            "hardlinks": {"duplicate_allocated_bytes": 0},
            "sizes": {"allocated_bytes": 8_192},
        }
        cases = (
            (b"8.0K corpus\n", "", "exact byte value"),
            (b"8192B corpus\n4096B child\n", "", "exactly one total row"),
            (b"8192B corpus\n", "permission denied", "warnings or errors"),
            (b"4096B corpus\n", "", "disagrees with independent oracle"),
        )
        for stdout, stderr, message in cases:
            with self.subTest(message=message):
                _observed, error = compare_tools._dust_semantics(stdout, stderr, oracle)
                self.assertIn(message, error or "")

        symlink_oracle = {**oracle, "counts": {"symlinks": 1}}
        _observed, error = compare_tools._dust_semantics(b"8192B corpus\n", "", symlink_oracle)
        self.assertIn("excludes symlink", error or "")

    def test_invalid_tool_sample_exposes_no_claim_metrics(self) -> None:
        result = {
            "exit_code": 0,
            "resources": {field: 1 for field in compare_tools.measure._RESOURCE_FIELDS},
            "stderr": "permission denied",
            "stdout": b"8192B corpus\n",
            "timed_out": False,
            "wall_ns": 10,
        }
        oracle = {
            "counts": {"symlinks": 0},
            "hardlinks": {"duplicate_allocated_bytes": 0},
            "sizes": {"allocated_bytes": 8192},
        }
        regime = compare_tools.measure.HostRegime(name="uncontrolled", initial={})
        with (
            mock.patch.object(compare_tools.measure, "_spawn", return_value=result),
            mock.patch.object(compare_tools.measure, "_host_pressure_snapshot", return_value={}),
        ):
            sample = compare_tools._run_one(
                tool("dust"),
                pair="dust",
                ordinal=0,
                warmup=False,
                root=Path("/fixture"),
                summary_oracle=oracle,
                host_regime=regime,
                timeout_seconds=1,
            )

        self.assertFalse(sample["valid"])
        self.assertTrue(all(value is None for value in sample["metrics"].values()))

    def test_installed_fdu_trace_transport_is_exact_and_separate_from_stderr(self) -> None:
        trace = {"schema": "fdu-scan-diagnostics-v1", "worker_policy": {}}
        transported = (
            compare_tools.FDU_SCAN_DIAGNOSTICS_PREFIX
            + json.dumps(trace, separators=(",", ":"))
            + "\nwarning: fixture\n"
        )

        document, residual, reasons = compare_tools._extract_scan_diagnostics(transported)

        self.assertEqual(document, trace)
        self.assertEqual(residual, "warning: fixture")
        self.assertEqual(reasons, [])

    def test_installed_fdu_trace_transport_fails_closed(self) -> None:
        prefix = compare_tools.FDU_SCAN_DIAGNOSTICS_PREFIX
        cases = (
            ("", "0 policy traces"),
            (prefix + "not-json\n", "was not JSON"),
            (prefix + "[]\n", "not a JSON object"),
            (prefix + "{}\n" + prefix + "{}\n", "2 policy traces"),
        )
        for stderr, message in cases:
            with self.subTest(message=message):
                document, _residual, reasons = compare_tools._extract_scan_diagnostics(stderr)
                self.assertIsNone(document)
                self.assertTrue(any(message in reason for reason in reasons), reasons)

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

    def test_partial_or_cached_summary_cannot_be_timing_evidence(self) -> None:
        base = {
            "schema": "fdu.report/1",
            "source": "cold_scan",
            "freshness": "fresh",
            "complete": True,
            "errors": [],
            "reports": [{"view": "summary", "summary": {"files": 3}}],
        }
        for change in (
            {"source": "snapshot"},
            {"freshness": "stale"},
            {"complete": False},
            {"errors": [{"message": "unreadable"}]},
        ):
            _digest, error = compare_tools._summary_semantic_digest(
                json.dumps({**base, **change}).encode()
            )
            self.assertIsNotNone(error)

    def test_summary_oracle_checks_counts_and_both_byte_totals(self) -> None:
        oracle = {
            "counts": {"directories": 5, "files": 9},
            "sizes": {"apparent_bytes": 1234, "allocated_bytes": 4096},
            "newest_file_mtime_ns": 17,
        }
        summary = {
            "files": 9,
            "dirs": 4,
            "bytes": 1234,
            "allocated": 4096,
            "newest_mtime_ns": 17,
        }

        self.assertIsNone(compare_tools._summary_oracle_error(summary, oracle))

        summary["allocated"] = 8192
        error = compare_tools._summary_oracle_error(summary, oracle)

        self.assertIsNotNone(error)
        self.assertIn("allocated=8192", error or "")
        self.assertIn("oracle 4096", error or "")

    def test_summary_oracle_excludes_the_subject_root_from_directory_count(self) -> None:
        oracle = {
            "counts": {"directories": 1, "files": 0},
            "sizes": {"apparent_bytes": 0, "allocated_bytes": 0},
            "newest_file_mtime_ns": None,
        }
        summary = {
            "files": 0,
            "dirs": 0,
            "bytes": 0,
            "allocated": 0,
            "newest_mtime_ns": None,
        }

        self.assertIsNone(compare_tools._summary_oracle_error(summary, oracle))

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
