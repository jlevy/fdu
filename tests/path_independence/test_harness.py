"""The harness's own contracts: judging answers, the registry, and the fixture.

None of these tests runs fdu, so they hold on a machine with no build.
"""

from __future__ import annotations

import os
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))

import registry
from fixture import build_fixture
from runner import Invocation, case_key, compare, normalize


def answer(**overrides: Any) -> dict[str, Any]:
    base: dict[str, Any] = {
        "schema": "fdu.report/6",
        "source": "cold_scan",
        "freshness": "fresh",
        "scan_started_at": "2026-01-01T00:00:00Z",
        "generated_at": "2026-01-01T00:00:01Z",
        "complete": True,
        "errors": [],
        "reports": [{"rows": [{"name": "a", "bytes": 1}, {"name": "b", "bytes": 2}]}],
    }
    base.update(overrides)
    return base


def cli(result: dict[str, Any] | None, *, exit: int = 0, stderr: str = "") -> Invocation:
    return Invocation("cli-report", "fdu", exit, stderr, result)


def py(result: dict[str, Any] | None, *, error: str = "") -> Invocation:
    return Invocation("py-open", "py-open", 0 if result else 1, error, result)


class CompareTests(unittest.TestCase):
    """What counts as the same answer, and which differences are allowed outcomes."""

    def test_provenance_is_excluded_and_nothing_else(self) -> None:
        content, provenance = normalize(answer())
        self.assertEqual(
            set(provenance), {"source", "freshness", "scan_started_at", "generated_at"}
        )
        self.assertIn("complete", content)
        self.assertIn("errors", content)
        warm = answer(source="warm_revalidate", freshness="stale", generated_at="later")
        self.assertEqual(compare(cli(answer()), cli(warm), policy="auto").kind, "same")

    def test_tree_status_is_compared(self) -> None:
        partial = answer(complete=False, errors=["x: permission denied"])
        verdict = compare(cli(answer()), cli(partial), policy="auto")
        self.assertEqual(verdict.kind, "differs")
        self.assertEqual(verdict.paths, ("complete", "errors[]"))

    def test_list_indices_are_generalized(self) -> None:
        swapped = answer(reports=[{"rows": [{"name": "a", "bytes": 1}, {"name": "b", "bytes": 3}]}])
        verdict = compare(cli(answer()), cli(swapped), policy="auto")
        self.assertEqual(verdict.paths, ("reports[].rows[].bytes",))

    def test_a_cache_only_failure_is_a_named_refusal(self) -> None:
        miss = cli(None, exit=1, stderr="fdu: snapshot is not usable")
        self.assertEqual(compare(cli(answer()), miss, policy="only").kind, "refused")
        self.assertEqual(compare(cli(answer()), miss, policy="auto").kind, "outcome_class")

    def test_an_answer_where_cold_refused_is_an_outcome_difference(self) -> None:
        refused = cli(None, exit=2, stderr="fdu: requires content analysis")
        verdict = compare(refused, cli(answer()), policy="only")
        self.assertEqual(verdict.kind, "outcome_class")

    def test_refusals_compare_wording_only_on_the_same_surface(self) -> None:
        cold = cli(None, exit=2, stderr="fdu: requires content analysis")
        self.assertEqual(
            compare(cold, cli(None, exit=2, stderr=cold.stderr), policy="auto").kind, "same"
        )
        reworded = cli(None, exit=2, stderr="fdu: needs analysis")
        self.assertEqual(compare(cold, reworded, policy="auto").paths, ("<error>",))
        self.assertEqual(compare(cold, py(None, error="ValueError: x"), policy="auto").kind, "same")

    def test_stale_needs_cache_only_the_earlier_answer_and_its_label(self) -> None:
        earlier = cli(answer())
        after_change = cli(answer(reports=[]))
        stale = cli(answer(freshness="stale"))
        unlabelled = cli(answer(freshness="fresh"))
        self.assertEqual(compare(after_change, stale, policy="only", earlier=earlier).kind, "stale")
        verdict = compare(after_change, unlabelled, policy="only", earlier=earlier)
        self.assertEqual((verdict.kind, verdict.paths), ("differs", ("freshness",)))
        self.assertEqual(
            compare(after_change, stale, policy="auto", earlier=earlier).kind, "differs"
        )

    def test_case_keys_name_every_axis(self) -> None:
        self.assertEqual(
            case_key("warm", "cli-report", "auto", "W_all", "", "a_lines"),
            "warm/cli-report/auto/W_all/-/a_lines",
        )


REGISTRY = """
[classes.content-containment]
description = "A narrower request served from a wider record"
clears_with = "Phase 1 item 2"
bead = "fdu-gija"

[[violation]]
class = "content-containment"
paths = ["analysis.analyze[]"]
keys = ["warm/a", "warm/b"]
"""


class RegistryTests(unittest.TestCase):
    """Each way a run and the registry can disagree fails the run."""

    def verify(self, judged: list[registry.Judged], text: str = REGISTRY, **kw: Any) -> list[str]:
        options: dict[str, Any] = {"full": False, "platform": "linux", "cold_answers": 1} | kw
        failures = registry.verify(registry.parse(text), judged, **options)
        return [failure.reason for failure in failures]

    def conforming(self) -> list[registry.Judged]:
        paths = ("analysis.analyze[]",)
        return [("warm/a", False, paths), ("warm/b", False, paths), ("warm/c", True, ())]

    def test_a_conforming_run_passes(self) -> None:
        self.assertEqual(self.verify(self.conforming(), full=True), [])

    def test_an_unregistered_difference_fails(self) -> None:
        judged = [*self.conforming(), ("warm/d", False, ("reports[]",))]
        self.assertEqual(self.verify(judged), ["unregistered difference"])

    def test_a_changed_difference_fails(self) -> None:
        judged = [("warm/a", False, ("reports[]",)), ("warm/b", False, ("analysis.analyze[]",))]
        self.assertEqual(self.verify(judged), ["difference changed shape"])

    def test_a_registered_case_that_now_conforms_fails(self) -> None:
        judged = [("warm/a", True, ()), ("warm/b", False, ("analysis.analyze[]",))]
        self.assertEqual(self.verify(judged), ["registered case now conforms; remove its entry"])

    def test_empty_classes_and_unexecuted_entries_fail_only_a_full_run(self) -> None:
        text = (
            REGISTRY + '\n[classes.unused]\ndescription = "x"\nclears_with = "x"\nbead = "fdu-x"\n'
        )
        judged = [("warm/a", False, ("analysis.analyze[]",))]
        self.assertEqual(self.verify(judged, text), [])
        self.assertEqual(
            sorted(self.verify(judged, text, full=True)),
            ["class has no entries; remove it", "registered case is not in the full matrix"],
        )

    def test_unclassified_and_undefined_classes_fail(self) -> None:
        text = REGISTRY.replace('class = "content-containment"', 'class = "unclassified"')
        self.assertIn("entry is unclassified", self.verify(self.conforming(), text))
        text = REGISTRY.replace('class = "content-containment"', 'class = "nowhere"')
        self.assertIn("entry names an undefined class", self.verify(self.conforming(), text))

    def test_an_empty_run_fails(self) -> None:
        self.assertEqual(
            self.verify([], cold_answers=0), ["run executed no cases", "run parsed no cold answers"]
        )

    def test_platform_entries_apply_only_on_their_platform(self) -> None:
        text = REGISTRY.replace("keys =", 'platforms = ["win32"]\nkeys =')
        self.assertEqual(self.verify(self.conforming(), text, platform="win32"), [])
        self.assertEqual(
            self.verify(self.conforming(), text, platform="linux"), ["unregistered difference"] * 2
        )

    def test_record_keeps_classes_marks_new_cases_and_round_trips(self) -> None:
        known = registry.parse(REGISTRY)
        judged = [
            ("warm/a", False, ("analysis.analyze[]",)),
            ("warm/b", True, ()),
            ("warm/new", False, ("reports[]",)),
        ]
        updated = registry.record(known, judged, platform="linux")
        self.assertEqual(updated.entries["warm/a"].klass, "content-containment")
        self.assertNotIn("warm/b", updated.entries)
        self.assertEqual(updated.entries["warm/new"].klass, registry.UNCLASSIFIED)
        reparsed = registry.parse(registry.dump(updated))
        self.assertEqual(reparsed.entries, updated.entries)
        self.assertEqual(reparsed.classes, updated.classes)

    def test_record_keeps_cases_it_did_not_execute(self) -> None:
        updated = registry.record(
            registry.parse(REGISTRY), [("warm/a", True, ())], platform="linux"
        )
        self.assertEqual(set(updated.entries), {"warm/b"})

    def test_dump_escapes_strings(self) -> None:
        known = registry.Registry(
            classes={"c": registry.ViolationClass("c", "d", 'quote " and \\ and \t', "fdu-x")},
            entries={'k"1': registry.Entry('k"1', "c", ("p",))},
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
                    seen[relative] = ("link", os.readlink(path))
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


if __name__ == "__main__":
    unittest.main()
