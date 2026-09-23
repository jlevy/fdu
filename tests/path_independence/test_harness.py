"""The harness's own contracts: judging answers, the registry, and the fixture.

None of these tests runs fdu, so they hold on a machine with no build.
"""

from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest
from io import StringIO
from pathlib import Path
from typing import Any
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))

import registry
from fixture import build_fixture, copy_fixture
from runner import Invocation, _read_jsonl_report, case_key, compare, normalize


def answer(**overrides: Any) -> dict[str, Any]:
    base: dict[str, Any] = {
        "schema": "fdu.report/7",
        "request": {
            "scope": {"read_controls": True},
            "analyze": [],
            "views": ["summary"],
        },
        "status": {"complete": True, "coverage": {"kind": "complete"}, "errors": []},
        "provenance": {
            "source": "cold_scan",
            "freshness": "fresh",
            "scan_started_at": "2026-01-01T00:00:00Z",
            "generated_at": "2026-01-01T00:00:01Z",
        },
        "reports": [{"rows": [{"name": "a", "bytes": 1}, {"name": "b", "bytes": 2}]}],
    }
    base.update(overrides)
    return base


def cli(result: dict[str, Any] | None, *, exit: int = 0, stderr: str = "") -> Invocation:
    return Invocation("cli-report", "fdu", exit, stderr, result)


class WatchEligibilityTests(unittest.TestCase):
    def test_watch_compares_supported_metadata_requests(self) -> None:
        import matrix
        from runner import watch_can_serve

        for policy in ("off", "auto", "read-only"):
            self.assertTrue(watch_can_serve(matrix.spec(no_gitignore=True), policy))
            self.assertTrue(watch_can_serve(matrix.spec(analyze="none"), policy))
        for request, policy in (
            (matrix.spec(), "only"),
            (matrix.spec(analyze="lines"), "auto"),
            (matrix.spec(scan_depth=1), "auto"),
            (matrix.spec(one_fs=True), "auto"),
        ):
            self.assertFalse(watch_can_serve(request, policy))


class CompareTests(unittest.TestCase):
    """What counts as the same answer, and which differences are allowed outcomes."""

    def test_provenance_is_excluded_and_nothing_else(self) -> None:
        content, provenance = normalize(answer())
        self.assertEqual(
            set(provenance), {"source", "freshness", "scan_started_at", "generated_at"}
        )
        self.assertIn("status", content)
        self.assertIn("request", content)
        warm = answer(
            provenance={
                "source": "warm_revalidate",
                "freshness": "stale",
                "scan_started_at": "earlier",
                "generated_at": "later",
                "tiers": {"entries": {"source": "revalidated"}},
            }
        )
        self.assertEqual(compare(cli(answer()), cli(warm), policy="auto").kind, "same")

    def test_tree_status_is_compared(self) -> None:
        partial = answer(
            status={
                "complete": False,
                "coverage": {"kind": "partial", "reason": "unavailable"},
                "errors": [{"message": "permission denied", "path": "x"}],
            }
        )
        verdict = compare(cli(answer()), cli(partial), policy="auto")
        self.assertEqual(verdict.kind, "differs")
        self.assertEqual(
            verdict.paths,
            (
                "status.complete",
                "status.coverage.kind",
                "status.coverage.reason",
                "status.errors[]",
            ),
        )

    def test_request_is_compared(self) -> None:
        changed = answer(
            request={
                "scope": {"read_controls": False},
                "analyze": [],
                "views": ["summary"],
            }
        )
        verdict = compare(cli(answer()), cli(changed), policy="auto")
        self.assertEqual(verdict.paths, ("request.scope.read_controls",))

    def test_missing_or_invalid_report_status_is_a_failure(self) -> None:
        for malformed in (
            answer(status={}),
            answer(status={"complete": "yes"}),
            answer(schema="not-an-fdu-report"),
        ):
            with self.subTest(malformed=malformed):
                self.assertEqual(cli(malformed).outcome, "failure")

    def test_watch_jsonl_reader_reassembles_every_requested_section(self) -> None:
        envelope = answer(reports=[])
        envelope["request"]["views"] = ["summary", "types"]
        summary = {"view": "summary", "summary": {"files": 1}}
        types = {"view": "types", "metrics": {"rows": []}}
        later_change = {"schema": "fdu.stream/2", "record": "change"}
        stream = StringIO(
            "\n".join(json.dumps(value) for value in (envelope, summary, types, later_change))
            + "\n"
        )

        parsed = _read_jsonl_report(stream)

        self.assertEqual(parsed["reports"], [summary, types])
        self.assertEqual(json.loads(stream.readline()), later_change)

    def test_watch_jsonl_reader_refuses_a_missing_or_wrong_section(self) -> None:
        envelope = answer(reports=[])
        envelope["request"]["views"] = ["summary"]
        for section in (None, {"view": "types"}):
            values = [envelope] + ([] if section is None else [section])
            with (
                self.subTest(section=section),
                self.assertRaises((EOFError, ValueError)),
            ):
                _read_jsonl_report(
                    StringIO("\n".join(json.dumps(value) for value in values) + "\n")
                )

    def test_list_indices_are_generalized(self) -> None:
        swapped = answer(reports=[{"rows": [{"name": "a", "bytes": 1}, {"name": "b", "bytes": 3}]}])
        verdict = compare(cli(answer()), cli(swapped), policy="auto")
        self.assertEqual(verdict.paths, ("reports[].rows[].bytes",))

    def test_list_elements_align_by_identity(self) -> None:
        rows = [{"name": "b", "bytes": 2}, {"name": "c", "bytes": 3}]
        inserted = answer(reports=[{"rows": [{"name": "a", "bytes": 1}, *rows]}])
        cold = answer(reports=[{"rows": rows}])
        verdict = compare(cli(cold), cli(inserted), policy="auto")
        self.assertEqual(verdict.paths, ("reports[].rows[]",))

    def test_reordering_is_a_difference(self) -> None:
        rows = [{"name": "a", "bytes": 1}, {"name": "b", "bytes": 2}]
        reordered = answer(reports=[{"rows": list(reversed(rows))}])
        verdict = compare(cli(answer(reports=[{"rows": rows}])), cli(reordered), policy="auto")
        self.assertEqual(verdict.paths, ("reports[].rows<order>",))

    def test_the_root_is_compared_by_placeholder(self) -> None:
        here = answer(
            root="/tmp/one/tree",
            status={"complete": True, "errors": ["/tmp/one/tree/src: denied"]},
        )
        there = answer(
            root="C:\\copy\\tree",
            status={"complete": True, "errors": ["C:\\copy\\tree/src: denied"]},
        )
        self.assertEqual(compare(cli(here), cli(there), policy="auto").kind, "same")
        elsewhere = answer(
            root="/tmp/one/tree",
            status={"complete": True, "errors": ["/tmp/one/tree/docs: denied"]},
        )
        self.assertEqual(
            compare(cli(here), cli(elsewhere), policy="auto").paths,
            ("status.errors[]",),
        )

    def test_a_cache_only_failure_is_a_named_refusal(self) -> None:
        miss = cli(None, exit=1, stderr="fdu: snapshot is not usable: no usable snapshot")
        self.assertEqual(compare(cli(answer()), miss, policy="only").kind, "refused")
        self.assertEqual(compare(cli(answer()), miss, policy="auto").kind, "outcome_class")
        py_miss = Invocation("py-open", "py-open", 1, "FduError: snapshot is not usable: x", None)
        self.assertEqual(compare(cli(answer()), py_miss, policy="only").kind, "refused")

    def test_a_crash_under_cache_only_is_not_a_refusal(self) -> None:
        crashes = [
            cli(None, exit=101, stderr="thread 'main' panicked at src/lib.rs"),
            cli(None, exit=1, stderr="fdu: I/O error: permission denied"),
            cli(None, exit=-11, stderr=""),
            Invocation("py-report", "py-report", 3, "TypeError: bad argument", None),
        ]
        for crash in crashes:
            with self.subTest(crash.stderr):
                verdict = compare(cli(answer()), crash, policy="only")
                self.assertEqual(verdict.kind, "outcome_class")

    def test_an_exact_seed_must_serve_instead_of_refusing_or_scanning(self) -> None:
        oracle = cli(answer())
        miss = cli(None, exit=1, stderr="fdu: snapshot is not usable: no usable snapshot")
        cached = cli(answer(provenance={"source": "cache_only", "freshness": "stale"}))
        fresh = cli(answer(provenance={"source": "cache_only", "freshness": "fresh"}))
        self.assertFalse(compare(oracle, miss, policy="only", must_serve=True).allowed)
        self.assertFalse(compare(oracle, oracle, policy="only", must_serve=True).allowed)
        self.assertFalse(compare(oracle, fresh, policy="only", must_serve=True).allowed)
        self.assertEqual(compare(oracle, cached, policy="only", must_serve=True).kind, "same")

    def test_a_serving_control_fails_when_both_runs_fail_identically(self) -> None:
        # Two identical failures are `same` for ordinary cases, but a serving control
        # that never served has not shown the cache works, whatever the oracle did.
        crash = cli(None, exit=101, stderr="thread 'main' panicked at src/lib.rs")
        miss = cli(None, exit=1, stderr="fdu: snapshot is not usable: no usable snapshot")
        cached = cli(answer(provenance={"source": "cache_only", "freshness": "stale"}))
        for failure in (crash, miss):
            with self.subTest(failure.stderr):
                self.assertEqual(compare(failure, failure, policy="only").kind, "same")
                verdict = compare(failure, failure, policy="only", must_serve=True)
                self.assertEqual(verdict.kind, "outcome_class")
                self.assertFalse(compare(failure, cached, policy="only", must_serve=True).allowed)

    def test_cache_contract_phase_catches_a_cache_that_never_serves(self) -> None:
        import matrix
        from fixture import FixtureFacts
        from runner import MatrixRun, Surfaces, Workspace

        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            run = MatrixRun(
                matrix.SUBSET,
                Surfaces(Path("unused-fdu"), Path("unused-python")),
                Workspace(base),
                FixtureFacts(base / "tree", False, False),
            )
            run.cold = {request: cli(answer()) for request in matrix.SUBSET.requests}
            cached = cli(answer(provenance={"source": "cache_only", "freshness": "stale"}))
            miss = cli(None, exit=1, stderr="fdu: snapshot is not usable: no usable snapshot")
            with patch("runner.run_cli", return_value=cli(answer())):
                with patch("runner.run_route", return_value=cached):
                    healthy = run.phase_cache_contract()
                with patch("runner.run_route", return_value=miss):
                    broken = run.phase_cache_contract()
            self.assertEqual(len(healthy), 18)
            self.assertEqual(len(broken), len(healthy))
            self.assertTrue(all(case.verdict.allowed for case in healthy))
            self.assertTrue(all(not case.verdict.allowed for case in broken))
            with patch("runner.run_cli", return_value=miss), patch("runner.run_route") as measured:
                unseeded = run.phase_cache_contract()
                measured.assert_not_called()
            self.assertEqual(len(unseeded), len(healthy))
            self.assertTrue(all(not case.verdict.allowed for case in unseeded))

    def test_an_answer_where_cold_refused_is_an_outcome_difference(self) -> None:
        refused = cli(None, exit=2, stderr="fdu: requires content analysis")
        verdict = compare(refused, cli(answer()), policy="only")
        self.assertEqual(verdict.kind, "outcome_class")

    def test_refusals_compare_wording_only_on_the_same_surface(self) -> None:
        cold = cli(None, exit=2, stderr="fdu: requires content analysis")
        self.assertEqual(
            compare(cold, cli(None, exit=2, stderr=cold.stderr), policy="auto").kind,
            "same",
        )
        reworded = cli(None, exit=2, stderr="fdu: needs analysis")
        self.assertEqual(compare(cold, reworded, policy="auto").paths, ("<error>",))
        refused_in_python = Invocation("py-open", "py-open", 2, "ValueError: x", None)
        self.assertEqual(compare(cold, refused_in_python, policy="auto").kind, "same")
        crashed_in_python = Invocation("py-open", "py-open", 3, "TypeError: x", None)
        self.assertEqual(compare(cold, crashed_in_python, policy="auto").paths, ("<error>",))

    def test_stale_needs_cache_only_the_earlier_answer_and_its_label(self) -> None:
        earlier = cli(answer())
        after_change = cli(answer(reports=[]))
        stale = cli(answer(provenance={"freshness": "stale"}))
        unlabelled = cli(answer(provenance={"freshness": "fresh"}))
        self.assertEqual(compare(after_change, stale, policy="only", earlier=earlier).kind, "stale")
        verdict = compare(after_change, unlabelled, policy="only", earlier=earlier)
        self.assertEqual((verdict.kind, verdict.paths), ("differs", ("provenance.freshness",)))
        self.assertEqual(
            compare(after_change, stale, policy="auto", earlier=earlier).kind, "differs"
        )

    def test_case_keys_name_every_axis(self) -> None:
        self.assertEqual(
            case_key("warm", "cli-report", "auto", "W_all", "", "a_lines"),
            "warm/cli-report/auto/W_all/-/a_lines",
        )


A = "warm/cli-report/auto/W_all/-/a_lines"
B = "warm/cli-report/auto/W_all/-/a_code"
C = "warm/cli-report/auto/W_all/-/a_words"
D = "warm/cli-report/auto/W_all/-/a_all"
NEW = "warm/cli-report/auto/W_code/-/a_lines"
SHAPE = ("analysis.analyze[]",)

REGISTRY = f"""
[classes.content-containment]
description = "A narrower request served from a wider record"
clears_with = "Phase 1 item 2"
bead = "fdu-gija"

[[violation]]
class = "content-containment"
paths = ["analysis.analyze[]"]
keys = ["{A}", "{B}"]
"""


def entry(known: registry.Registry, key: str, platform: str) -> tuple[Any, ...] | None:
    found = known.entry_for(key, platform)
    return None if found is None else (found.klass, found.paths, found.platforms)


class RegistryTests(unittest.TestCase):
    """Each way a run and the registry can disagree fails the run."""

    def verify(self, judged: list[registry.Judged], text: str = REGISTRY, **kw: Any) -> list[str]:
        options: dict[str, Any] = {
            "full": False,
            "platform": "linux",
            "cold_answers": 1,
        } | kw
        failures = registry.verify(registry.parse(text), judged, **options)
        return [failure.reason for failure in failures]

    def conforming(self) -> list[registry.Judged]:
        return [(A, False, SHAPE), (B, False, SHAPE), (C, True, ())]

    def test_a_conforming_run_passes(self) -> None:
        self.assertEqual(self.verify(self.conforming(), full=True), [])

    def test_the_conformance_gate_rejects_even_matching_registered_failures(self) -> None:
        from runner import CaseResult, RunResult, Verdict, judge

        result = RunResult(tier="subset", cold_answers=1)
        result.cases = [
            CaseResult(key, Verdict("same" if allowed else "differs", paths))
            for key, allowed, paths in self.conforming()
        ]
        reasons = [failure.reason for failure in judge(result, registry.parse(REGISTRY))]
        self.assertEqual(reasons, ["known violations are not allowed by the conformance gate"])

    def test_an_unregistered_difference_fails(self) -> None:
        judged = [*self.conforming(), (D, False, ("reports[]",))]
        self.assertEqual(self.verify(judged), ["unregistered difference"])

    def test_a_changed_difference_fails(self) -> None:
        judged = [(A, False, ("reports[]",)), (B, False, SHAPE)]
        self.assertEqual(self.verify(judged), ["difference changed shape"])

    def test_a_registered_case_that_now_conforms_fails(self) -> None:
        judged = [(A, True, ()), (B, False, SHAPE)]
        self.assertEqual(self.verify(judged), ["registered case now conforms; remove its entry"])

    def test_empty_classes_and_unexecuted_entries_fail_only_a_full_run(self) -> None:
        unused = '\n[classes.unused]\ndescription = "x"\nclears_with = "x"\nbead = "fdu-x"\n'
        judged = [(A, False, SHAPE)]
        self.assertEqual(self.verify(judged, REGISTRY + unused), [])
        self.assertEqual(
            sorted(self.verify(judged, REGISTRY + unused, full=True)),
            [
                "class has no entries; remove it",
                "registered case is not in the full matrix",
            ],
        )

    def test_unclassified_and_undefined_classes_fail(self) -> None:
        text = REGISTRY.replace('class = "content-containment"', 'class = "unclassified"')
        self.assertIn("entry is unclassified", self.verify(self.conforming(), text))
        text = REGISTRY.replace('class = "content-containment"', 'class = "nowhere"')
        self.assertIn("entry names an undefined class", self.verify(self.conforming(), text))

    def test_an_empty_run_fails(self) -> None:
        self.assertEqual(
            self.verify([], cold_answers=0),
            ["run executed no cases", "run parsed no cold answers"],
        )

    def test_platform_entries_apply_only_on_their_platform(self) -> None:
        text = REGISTRY.replace("keys =", 'platforms = ["win32"]\nkeys =')
        self.assertEqual(self.verify(self.conforming(), text, platform="win32"), [])
        self.assertEqual(
            self.verify(self.conforming(), text, platform="linux"),
            ["unregistered difference"] * 2,
        )

    def test_one_case_may_take_a_different_shape_per_platform(self) -> None:
        text = REGISTRY.replace("keys =", 'platforms = ["darwin", "linux"]\nkeys =') + (
            '\n[[violation]]\nclass = "content-containment"\nplatforms = ["win32"]\n'
            f'paths = ["reports[].allocated"]\nkeys = ["{A}"]\n'
        )
        self.assertEqual(self.verify(self.conforming(), text, platform="linux"), [])
        windows = [(A, False, ("reports[].allocated",)), (B, True, ())]
        self.assertEqual(self.verify(windows, text, platform="win32"), [])

    def test_overlapping_platforms_and_malformed_entries_are_refused(self) -> None:
        overlap = f'\n[[violation]]\nclass = "content-containment"\npaths = ["x"]\nkeys = ["{A}"]\n'
        malformed = {
            "overlap": REGISTRY + overlap,
            "unknown field": REGISTRY.replace("keys =", 'platform = ["win32"]\nkeys ='),
            "short key": REGISTRY.replace(A, "warm/a"),
            "no paths": REGISTRY.replace('paths = ["analysis.analyze[]"]', "paths = []"),
            "class field": REGISTRY.replace("bead =", "owner = 1\nbead ="),
        }
        for name, text in malformed.items():
            with self.subTest(name), self.assertRaises(ValueError):
                registry.parse(text)

    def test_record_keeps_classes_marks_new_cases_and_round_trips(self) -> None:
        known = registry.parse(REGISTRY)
        judged = [(A, False, SHAPE), (B, True, ()), (NEW, False, ("reports[]",))]
        updated = registry.record(known, judged, platform="linux")
        self.assertEqual(entry(updated, A, "win32"), ("content-containment", SHAPE, None))
        self.assertIsNone(entry(updated, B, "linux"))
        self.assertEqual(entry(updated, B, "win32")[2], ("darwin", "win32"))  # type: ignore[index]
        self.assertEqual(entry(updated, NEW, "linux"), ("unclassified", ("reports[]",), ("linux",)))
        self.assertIsNone(entry(updated, NEW, "darwin"))
        reparsed = registry.parse(registry.dump(updated))
        self.assertEqual(reparsed.entries, updated.entries)
        self.assertEqual(reparsed.classes, updated.classes)

    def test_merging_every_platform_regroups_shared_shapes(self) -> None:
        known = registry.parse(REGISTRY)
        runs = {
            "linux": [(A, False, SHAPE), (NEW, False, ("reports[]",))],
            "darwin": [
                (A, False, ("reports[].bytes", *SHAPE)),
                (NEW, False, ("reports[]",)),
            ],
            "win32": [(A, False, SHAPE), (NEW, False, ("reports[]",))],
        }
        merged = registry.merge(known, runs)
        self.assertEqual(entry(merged, NEW, "win32"), ("unclassified", ("reports[]",), None))
        self.assertEqual(entry(merged, A, "linux")[2], ("linux", "win32"))  # type: ignore[index]
        self.assertEqual(entry(merged, A, "darwin")[1], ("analysis.analyze[]", "reports[].bytes"))  # type: ignore[index]
        self.assertEqual(entry(merged, B, "linux"), ("content-containment", SHAPE, None))

    def test_dump_escapes_strings(self) -> None:
        key = 'a/b/c/d/e/"f'
        known = registry.Registry(
            classes={"c": registry.ViolationClass("c", "d", 'quote " and \\ and \t', "fdu-x")},
            entries={key: [registry.Entry(key, "c", ("p",))]},
        )
        self.assertEqual(registry.parse(registry.dump(known)), known)


class FixtureTests(unittest.TestCase):
    """Two builds are byte- and timestamp-identical, so answers are reproducible."""

    def snapshot(self, root: Path) -> dict[str, tuple[Any, ...]]:
        seen: dict[str, tuple[Any, ...]] = {}
        for directory, names, files in os.walk(root):
            for name in [*names, *files]:
                path = Path(directory) / name
                stat = path.lstat()
                relative = path.relative_to(root).as_posix()
                if path.is_symlink():
                    seen[relative] = ("link", os.readlink(path), stat.st_mtime_ns)
                elif path.is_dir():
                    seen[relative] = ("dir", stat.st_mtime_ns)
                else:
                    seen[relative] = ("file", path.read_bytes(), stat.st_mtime_ns)
        seen["."] = ("dir", root.stat().st_mtime_ns)
        return seen

    def test_builds_are_identical(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            first = build_fixture(Path(scratch) / "one")
            second = build_fixture(Path(scratch) / "two")
            self.assertEqual(first.symlinks, second.symlinks)
            self.assertEqual(self.snapshot(first.root), self.snapshot(second.root))
            self.assertEqual(len((first.root / "data/blob.bin").read_bytes()), 9000)

    def test_a_copy_keeps_every_mtime(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            facts = build_fixture(Path(scratch) / "fixture")
            copy_fixture(facts, Path(scratch) / "copy")
            self.assertEqual(self.snapshot(facts.root), self.snapshot(Path(scratch) / "copy"))


if __name__ == "__main__":
    unittest.main()
