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
        def sample(
            ordinal: int,
            decisions: list[str],
            *,
            valid: bool = True,
            outcome: str = "held",
        ):
            return {
                "pair": "dust",
                "tool": "fdu",
                "ordinal": ordinal,
                "warmup": False,
                "valid": valid,
                "scan_diagnostics": {
                    "worker_policy": {
                        "outcome": outcome,
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
                sample(1, ["hold", "observe_slow"], outcome="scaled_up"),
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

    def test_render_reports_absolute_file_and_allocated_byte_rates(self) -> None:
        wall = {"median": 2_000_000_000}
        rss = {"median": 8 * 1024 * 1024}
        comparison = {
            "median_change_pct": 50.0,
            "ci95_change_pct": [40.0, 60.0],
        }
        document = {
            "anchor": "fdu",
            "baseline_drift": [],
            "competitor_order": ["du"],
            "conditions": {"storage": "internal APFS SSD"},
            "host": {"cpu_model": "Fixture CPU"},
            "invalid_samples": 0,
            "overall": {"fdu": {"metrics": {"wall_ns": wall, "peak_rss_bytes": rss}}},
            "semantic_mismatches": [],
            "statistics": {
                "du": {
                    "competitor_vs_fdu": {"wall_ns": comparison},
                    "fdu_vs_competitor": {
                        "policy_stability": {"stable": False},
                        "qualification": {
                            "classification": "inconclusive",
                            "confirmable": False,
                            "reasons": [],
                        },
                    },
                    "tools": {"du": {"metrics": {"wall_ns": wall, "peak_rss_bytes": rss}}},
                }
            },
            "summary_oracle_mismatches": [],
            "tools": {
                "fdu": {"work_class": "indexed-tree"},
                "du": {"work_class": "total-only"},
            },
            "tree": {
                "counts": {"files": 500_000, "total": 600_001},
                "hardlinks": {"duplicate_allocated_bytes": 0, "duplicate_file_entries": 0},
                "sizes": {"allocated_bytes": 3_000_000_000},
            },
            "tree_mutated_during_run": [],
        }

        rendered = compare_tools.render(document)

        self.assertIn(
            "| Median wall-clock time | Wall time vs. fdu | Files/s | GB/s |",
            rendered,
        )
        self.assertIn("| fdu | indexed-tree | 2.0 s | baseline | 250k | 1.5 |", rendered)
        self.assertIn("| du | total-only | 2.0 s | +50% | 250k | 1.5 |", rendered)
        self.assertIn("Rates divide the subject's 500,000 regular files", rendered)
        self.assertIn("3,000,000,000 allocated bytes", rendered)

    def test_compact_rates_preserve_small_measurements(self) -> None:
        wall = {"median": 36_000_000}
        tree = {"counts": {"files": 9}, "sizes": {"allocated_bytes": 4096}}

        self.assertEqual(compare_tools._seconds(wall), "0.036 s")
        self.assertEqual(compare_tools._throughput(tree, wall), ("250", "0.00011"))

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

    def test_dust_adapter_counts_directory_blocks_where_the_filesystem_has_them(self) -> None:
        # ext4 gives every directory a block; dust sums it and fdu does not.
        oracle = {
            "counts": {"symlinks": 0},
            "hardlinks": {"duplicate_allocated_bytes": 0},
            "sizes": {"allocated_bytes": 8_192, "directory_allocated_bytes": 8_192},
        }

        observed, error = compare_tools._dust_semantics(b"16384B corpus\n", "", oracle)
        self.assertEqual(observed, 16_384)
        self.assertIsNone(error)

        _observed, error = compare_tools._dust_semantics(b"8192B corpus\n", "", oracle)
        self.assertIn("disagrees with independent oracle", error or "")

    def test_a_censored_peak_rss_renders_as_a_bound(self) -> None:
        censored = [
            {"metrics": {"peak_rss_bytes": None}, "peak_rss_floor_bytes": 50 * 1024 * 1024},
            {"metrics": {"peak_rss_bytes": None}, "peak_rss_floor_bytes": 60 * 1024 * 1024},
        ]
        mixed = [*censored, {"metrics": {"peak_rss_bytes": 70 * 1024 * 1024}}]

        self.assertEqual(compare_tools._censored_peak_rss(censored), 60 * 1024 * 1024)
        self.assertIsNone(compare_tools._censored_peak_rss(mixed))
        self.assertEqual(
            compare_tools._rss_cell(None, {"peak_rss_at_most_bytes": 60 * 1024 * 1024}),
            "≤ 60.0 MiB",
        )
        self.assertEqual(compare_tools._rss_cell(None, {}), "—")
        self.assertEqual(
            compare_tools._rss_cell({"median": 3 * 1024 * 1024}, {}), "3.0 MiB"
        )

    def test_throughput_renders_a_tool_with_no_valid_sample(self) -> None:
        tree = {"counts": {"files": 9}, "sizes": {"allocated_bytes": 4096}}

        self.assertEqual(compare_tools._throughput(tree, None), ("—", "—"))

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
        self.assertIn("--no-gitignore", contract.argv)
        self.assertIn("summary", contract.argv)
        self.assertIn("no path index", contract.description)
        index_summary = compare_tools.CONTRACTS["fdu-index-summary"]
        self.assertNotIn("--no-gitignore", index_summary.argv)

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

    def test_current_report_layout_is_read_from_provenance_and_status(self) -> None:
        """`fdu.report/7` nests the run facts; a flat reader rejected every sample."""
        base = {
            "schema": "fdu.report/7",
            "root": "/private/one",
            "status": {"complete": True, "coverage": {"kind": "complete"}, "errors": []},
            "provenance": {
                "source": "cold_scan",
                "freshness": "fresh",
                "generated_at": "2026-01-01T00:00:01Z",
            },
            "reports": [{"view": "summary", "summary": {"files": 3}}],
        }
        later = {
            **base,
            "root": "/private/two",
            "provenance": {**base["provenance"], "generated_at": "2027-01-01T00:00:01Z"},
        }

        first, first_error = compare_tools._summary_semantic_digest(json.dumps(base).encode())
        second, second_error = compare_tools._summary_semantic_digest(json.dumps(later).encode())
        self.assertIsNone(first_error)
        self.assertIsNone(second_error)
        self.assertEqual(first, second)

        for key, change in (
            ("provenance", {"source": "snapshot", "freshness": "fresh"}),
            ("provenance", {"source": "cold_scan", "freshness": "stale"}),
            ("status", {"complete": False, "errors": []}),
            ("status", {"complete": True, "errors": [{"message": "unreadable"}]}),
        ):
            with self.subTest(key=key, change=change):
                _digest, error = compare_tools._summary_semantic_digest(
                    json.dumps({**base, key: change}).encode()
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


class DefaultTreeContractTests(unittest.TestCase):
    """The contract for what users actually type, and its one dangerous requirement.

    Every other fdu contract passes `--cache off`, so the harness never had to care
    where a tool keeps state. This one measures the default invocation, which may write a
    snapshot (every binary before `--cache auto` stopped writing one did), and measuring
    that against the operator's real cache directory would let an unrelated earlier run
    decide this run's starting state.
    """

    def test_the_contract_is_the_bare_default_invocation(self) -> None:
        contract = compare_tools.CONTRACTS["fdu-default-tree"]

        self.assertEqual(contract.work_class, "default-tree")
        # No --cache, no --view, no --depth: the point is that a user typed none of them.
        self.assertNotIn("--cache", contract.argv)
        self.assertNotIn("--view", contract.argv)
        self.assertNotIn("--depth", contract.argv)
        self.assertEqual(contract.argv, ("{binary}", "--color", "never", "{root}"))
        self.assertIn("writes it inside the timed run", contract.description)

    def test_only_default_cache_contracts_declare_a_cache_write(self) -> None:
        # The two contracts that leave the cache policy at its default; every other fdu
        # contract passes `--cache off`.
        writers = {
            name for name, contract in compare_tools.CONTRACTS.items() if contract.writes_cache
        }
        self.assertEqual(writers, {"fdu-default-tree", "fdu-code-cached-no-ignore"})
        for name in writers:
            self.assertFalse(
                any(item.startswith("--cache") for item in compare_tools.CONTRACTS[name].argv)
            )

    def test_a_cache_writing_run_without_an_isolated_directory_fails_closed(self) -> None:
        # The failure that matters: silently falling back to $HOME/Library/Caches would
        # measure against whatever the operator happened to have there, and leave a
        # snapshot of the subject tree behind. Refuse instead.
        tool = compare_tools.Tool(
            name="fdu",
            contract=compare_tools.CONTRACTS["fdu-default-tree"],
            binary=Path("/usr/bin/true"),
        )
        with self.assertRaises(compare_tools.ComparisonError) as raised:
            compare_tools._run_one(
                tool,
                pair="dust",
                ordinal=0,
                warmup=False,
                root=Path("/tmp"),
                summary_oracle={},
                host_regime=None,
                timeout_seconds=1.0,
                cache_home=None,
            )
        self.assertIn("no isolated cache directory", str(raised.exception))

    def test_the_default_contract_may_anchor_a_comparison(self) -> None:
        # It measures fdu, so it is a legal anchor; it is not a summary contract, so the
        # held-out release gate still refuses it. Both halves matter.
        self.assertNotIn("fdu-default-tree", compare_tools.FDU_SUMMARY_CONTRACTS)


FDU_CODE_TABLE = """\
Code lines   Share   Comments      Blank  Analyzed files  Language
26,059,137   99.0%  4,229,132  4,286,449   59,786/59,786  C           (26,059,137 unknown)
         —       —          —          —         0/1,338  Assembly
26,312,547  100.0%  4,290,517  4,343,072   61,452/65,896  TOTAL       (26,312,547 unknown)
6 analyzed languages (include population)
""".encode()

SCC_TABLE = """\
───────────────────────────────────────────────────────────────────────────────
Language                    Files       Lines     Blanks    Comments       Code
───────────────────────────────────────────────────────────────────────────────
C                          34,661  24,595,873  3,545,815   2,752,964 18,297,094
───────────────────────────────────────────────────────────────────────────────
Total                      81,820  39,097,537  4,906,599   4,465,118 29,725,820
───────────────────────────────────────────────────────────────────────────────
""".encode()

TOKEI_TABLE = b"""\
 Language              Files        Lines         Code     Comments       Blanks
 C                     34661     24595873     18301279      2751125      3543469
 Rust                     76        12370         8982         1819         1569
 |- Markdown              70         6425          772         4247         1406
 (Total)                            18795         9754         6066         2975
 Total                 81894     39095423     29615689      4577792      4901942
"""


class LineCountContractTests(unittest.TestCase):
    """Source-line counters: two arms, three tools, and no cross-tool semantic check."""

    def test_each_arm_runs_the_protocol_commands(self) -> None:
        expected = {
            "fdu-code-no-ignore": (
                "{binary}",
                "--analyze=code",
                "--view=code",
                "--no-gitignore",
                "--cache=off",
                "--color=never",
                "--quiet",
                "{root}",
            ),
            "fdu-code-gitignore": (
                "{binary}",
                "--analyze=code",
                "--view=code",
                "--ignored=exclude",
                "--exclude=.git/**",
                "--cache=off",
                "--color=never",
                "--quiet",
                "{root}",
            ),
            "scc-no-ignore": (
                "{binary}",
                "--no-gitignore",
                "--no-ignore",
                "--no-scc-ignore",
                "--no-gitmodule",
                "-c",
                "--no-cocomo",
                "--no-size",
                "{root}",
            ),
            "scc-gitignore": ("{binary}", "-c", "--no-cocomo", "--no-size", "{root}"),
            "tokei-no-ignore": ("{binary}", "--no-ignore", "--hidden", "{root}"),
            "tokei-gitignore": ("{binary}", "--hidden", "--exclude", ".git", "{root}"),
        }
        for name, argv in expected.items():
            with self.subTest(contract=name):
                contract = compare_tools.CONTRACTS[name]
                self.assertEqual(contract.argv, argv)
                self.assertEqual(contract.work_class, "code-by-language")
                self.assertIsNotNone(contract.code_table)
                # Text, never JSON: tokei's JSON carries every file's record.
                self.assertFalse({"-o", "-f", "--format=json"} & set(argv))
                arm = "no-ignore" if name.endswith("-no-ignore") else "gitignore"
                self.assertEqual(contract.measures, f"source-lines-{arm}")

    def test_the_cached_count_repeats_the_ignore_off_command_with_its_cache(self) -> None:
        cached = compare_tools.CONTRACTS["fdu-code-cached-no-ignore"]
        uncached = compare_tools.CONTRACTS["fdu-code-no-ignore"]

        self.assertEqual(
            cached.argv, tuple(item for item in uncached.argv if item != "--cache=off")
        )
        self.assertTrue(cached.writes_cache)
        self.assertEqual(cached.measures, uncached.measures)
        self.assertEqual(cached.code_table, "fdu")
        self.assertEqual(cached.work_class, "code-by-language-cached")

    def test_only_the_fdu_arms_may_anchor(self) -> None:
        self.assertEqual(
            compare_tools.FDU_CODE_CONTRACTS,
            {"fdu-code-no-ignore", "fdu-code-cached-no-ignore", "fdu-code-gitignore"},
        )
        self.assertTrue(compare_tools.FDU_CODE_CONTRACTS <= compare_tools.FDU_ANCHOR_CONTRACTS)
        self.assertFalse(compare_tools.FDU_CODE_CONTRACTS & compare_tools.FDU_SUMMARY_CONTRACTS)
        self.assertNotIn("scc-no-ignore", compare_tools.FDU_ANCHOR_CONTRACTS)

    def test_a_comparison_admits_one_measure_and_one_arm(self) -> None:
        cases = (
            ("fdu-code-no-ignore", "dust"),
            ("fdu-code-no-ignore", "scc-gitignore"),
            ("fdu-code-gitignore", "tokei-no-ignore"),
            ("fdu-default-tree", "scc-no-ignore"),
        )
        for anchor, competitor in cases:
            with (
                self.subTest(anchor=anchor, competitor=competitor),
                self.assertRaisesRegex(compare_tools.ComparisonError, "measure the same"),
            ):
                compare_tools.run(
                    root=Path("/does/not-matter"),
                    label="fixture",
                    anchor=tool(anchor),
                    competitors=[tool(competitor)],
                    trials=3,
                    warmups=1,
                    baseline_fingerprint=None,
                    baseline_output=None,
                    storage="fixture",
                )

    def test_each_layout_parses_its_total_row(self) -> None:
        cases = (
            ("fdu", FDU_CODE_TABLE, (61_452, 26_312_547, 4_290_517, 4_343_072)),
            ("scc", SCC_TABLE, (81_820, 29_725_820, 4_465_118, 4_906_599)),
            # The grand total, not a language's `(Total)` with its embedded children.
            ("tokei", TOKEI_TABLE, (81_894, 29_615_689, 4_577_792, 4_901_942)),
        )
        for layout, stdout, (files, code, comment, blank) in cases:
            with self.subTest(layout=layout):
                totals, error = compare_tools._code_table_totals(layout, stdout, "")
                self.assertIsNone(error)
                self.assertEqual(
                    totals, {"files": files, "code": code, "comment": comment, "blank": blank}
                )

    def test_the_total_row_parser_fails_closed(self) -> None:
        unmeasured = FDU_CODE_TABLE.replace(b"26,312,547  100.0%", "—  —".encode())
        cases = (
            ("scc", SCC_TABLE, "error reading file", "warnings or errors"),
            ("scc", b"Language Files\n", "", "exactly one total row"),
            ("tokei", TOKEI_TABLE + TOKEI_TABLE, "", "exactly one total row"),
            ("fdu", unmeasured, "", "exactly one total row"),
            ("fdu", b"\xff\xfe", "", "not UTF-8"),
        )
        for layout, stdout, stderr, message in cases:
            with self.subTest(layout=layout, message=message):
                totals, error = compare_tools._code_table_totals(layout, stdout, stderr)
                self.assertIsNone(totals)
                self.assertIn(message, error or "")

    def test_a_sample_records_its_totals(self) -> None:
        result = {
            "exit_code": 0,
            "resources": {field: 1 for field in compare_tools.measure._RESOURCE_FIELDS},
            "stderr": "",
            "stdout": SCC_TABLE,
            "timed_out": False,
            "wall_ns": 10,
        }
        regime = compare_tools.measure.HostRegime(name="uncontrolled", initial={})
        with (
            mock.patch.object(compare_tools.measure, "_spawn", return_value=result),
            mock.patch.object(compare_tools.measure, "_host_pressure_snapshot", return_value={}),
        ):
            sample = compare_tools._run_one(
                tool("scc-no-ignore"),
                pair="scc-no-ignore",
                ordinal=0,
                warmup=False,
                root=Path("/fixture"),
                summary_oracle={},
                host_regime=regime,
                timeout_seconds=1,
            )

        self.assertTrue(sample["valid"], sample["reasons"])
        self.assertEqual(sample["code_totals"]["code"], 29_725_820)
        # Line totals never enter the byte-report digest that pairs are checked against.
        self.assertIsNone(sample["semantic_sha256"])

    def test_a_tool_with_two_answers_on_one_tree_loses_every_sample(self) -> None:
        def sample(name: str, code: int) -> dict:
            totals = {"files": 1, "code": code, "comment": 0, "blank": 0}
            return {"tool": name, "valid": True, "reasons": [], "code_totals": totals}

        samples = [sample("fdu", 5), sample("fdu", 5), sample("tokei", 4), sample("tokei", 6)]

        mismatches = compare_tools._invalidate_unstable_code_totals(samples)

        self.assertEqual([entry["tool"] for entry in mismatches], ["tokei"])
        self.assertTrue(all(entry["valid"] for entry in samples[:2]))
        self.assertFalse(any(entry["valid"] for entry in samples[2:]))
        self.assertIn("2 different line totals", samples[2]["reasons"][0])

    def test_the_report_lists_each_tools_totals(self) -> None:
        document = {
            "anchor": "fdu",
            "competitor_order": ["scc", "tokei"],
            "code_totals": {
                "fdu": {"files": 2, "code": 1_000, "comment": 3, "blank": 4},
                "scc": {"files": 5, "code": 2_000, "comment": 6, "blank": 7},
                "tokei": None,
            },
            "code_total_mismatches": [],
        }

        section = "\n".join(compare_tools._code_totals_section(document))

        self.assertIn("| fdu | 2 | 1,000 | 3 | 4 |", section)
        self.assertIn("| tokei | — | — | — | — |", section)
        self.assertIn("differ by design", section)
        self.assertEqual(compare_tools._code_totals_section({"anchor": "fdu"}), [])


if __name__ == "__main__":
    unittest.main()
