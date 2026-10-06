"""The projection and the drawing rules that decide whether a figure tells the truth.

Every case here exists because getting it wrong produces a chart that looks finished and
is wrong, which is the failure mode a report of measurements can least afford.
"""

from __future__ import annotations

import unittest
from typing import Any, Dict, List

from benchmarks.realtree.report_html import (
    STYLE,
    axis_ticks,
    decision_label,
    STANDING_EXPERIMENT,
    end_to_end_cells,
    figure_absolute,
    figure_effects,
    figure_end_to_end,
    figure_timeline,
    figure_per_entry,
    fmt_primary,
    iteration_kind,
    kept_improvements,
    render,
)
from benchmarks.realtree.perf_index import (
    combine,
    component_digest,
    component_ratios,
    memory_ratios,
    load_suite,
    project_index,
    score_ratio,
    unmapped_jobs,
)
from benchmarks.realtree.timeline import (
    BASELINE_COMMIT,
    SYNTHETIC_SUBJECTS,
    is_synthetic,
    kept_variant,
    project,
    subject_family,
    subject_key,
)


def _index_cell(
    component: str,
    first_ms: float,
    last_ms: float,
    extra_after: str = "",
    job: str = "",
    digest: str = "",
) -> Dict[str, Any]:
    """A two-build history cell for one job of one index component, anchored on v0.3.0.

    It carries the manifest's digest for the component unless a test passes another.
    """
    definition = next(item for item in load_suite()["components"] if item["id"] == component)
    milestones = [
        {"label": "prework", "short": "start", "commit": "aaaa", "date": "2026-08-10",
         "includes": "start", "after_experiment": "exp-000", "wall_ms": first_ms,
         "peak_rss_mib": 400.0, "vs_latest_pct": (first_ms / last_ms - 1) * 100,
         "vs_latest_ci95_pct": [(first_ms / last_ms - 1) * 90, (first_ms / last_ms - 1) * 110],
         "supported": True},
        {"label": "v0.3.0", "short": "0.3.0", "commit": "bbbb", "date": "2026-09-30",
         "includes": "end", "after_experiment": "exp-032", "wall_ms": last_ms,
         "peak_rss_mib": 60.0, "vs_latest_pct": None, "vs_latest_ci95_pct": None,
         "supported": True},
    ]
    if extra_after:
        milestones.append(
            {"label": "lost", "short": "unplaced-build", "commit": "cccc", "date": "2026-10-01",
             "includes": "none", "after_experiment": extra_after, "wall_ms": 1000.0,
             "vs_latest_pct": -96.7, "supported": True}
        )
    return {
        "id": f"{component}-{job or component}",
        "component": component,
        "job": job or component,
        "component_digest": digest or component_digest(definition),
        "platform": "macOS",
        "subject": "linux-v6.12",
        "title": "the Linux v6.12 source tree",
        "entries": 92460,
        "cpu": "M1",
        "storage": "internal SSD",
        "trials": 12,
        "regime": "uncontrolled",
        "milestones": milestones,
    }


def metric(
    control: float,
    candidate: float,
    change: float,
    low: float | None = None,
    high: float | None = None,
) -> Dict[str, Any]:
    return {
        "control_median": control,
        "candidate_median": candidate,
        "change_pct": change,
        "ci95_low_pct": low,
        "ci95_high_pct": high,
        "pairs": 12,
        "significant": low is not None and high is not None and high < 0,
    }


def experiment(
    identifier: str,
    *,
    decision: str = "accepted",
    control: str = "the previous head",
    entries: int = 1000,
    root: str = "a" * 64,
    digest: str = "b" * 64,
    system: str = "Darwin 25.5.0",
    label: str = "subject",
    wall: Dict[str, Any] | None = None,
    lines: int = 10,
    hypotheses: List[str] | None = None,
    primary_metric: str = "wall_ns",
    extra_metrics: Dict[str, Any] | None = None,
    kept: str | None = None,
) -> Dict[str, Any]:
    metrics: Dict[str, Any] = {"wall_ns": wall or metric(200e6, 100e6, -50.0, -55.0, -45.0)}
    if extra_metrics:
        metrics.update(extra_metrics)
    return {
        "id": identifier,
        "date": "2026-08-10",
        "title": f"Experiment {identifier}",
        "hypotheses": hypotheses if hypotheses is not None else ["H1"],
        "subject": {
            "tree_label": label,
            "tree_root_id": root,
            "tree_engine_digest": digest,
            "tree_entries": entries,
            "tree_directories": 10,
            "tree_files": entries - 10,
            "tree_apparent_bytes": 1000,
            "host_system": system,
            "host_cpu": "test",
            "filesystem": "apfs",
            "os_cache": "warm-steady",
        },
        "method": {"control": control, "candidate": "the change", "trials": 12, "interleaved": True},
        "results": [
            {
                "job": "cold-scan-index",
                "start_state": "cold",
                "invalid_samples": 0,
                "metrics": metrics,
            }
        ],
        "complexity": {"lines_changed": lines},
        "verdict": {
            "decision": decision,
            "primary_job": "cold-scan-index",
            "primary_metric": primary_metric,
            "change_pct": -50.0,
            "reason": "because",
            "commit": None,
            "kept": kept,
        },
        "reference_tools": [],
        "_path": f"docs/project/experiments/{identifier}.md",
    }


class AxisTests(unittest.TestCase):
    def test_the_ladder_covers_the_maximum_rather_than_stopping_below_it(self) -> None:
        # Regression. The ladder used to stop at the last tick that fitted under the
        # data, and the chart then used that tick as its scale top: a 1,219 ms bar was
        # drawn against a 1,000 ms axis and ran three hundred pixels past the plot.
        for maximum in (1218.7, 1000.0, 999.0, 12.0, 0.4, 66.3, 1.0, 7.5):
            ticks = axis_ticks(maximum)
            self.assertGreaterEqual(
                ticks[-1], maximum, f"axis top {ticks[-1]} does not cover {maximum}"
            )
            self.assertEqual(ticks[0], 0.0)
            self.assertGreater(len(ticks), 1)

    def test_a_degenerate_maximum_still_yields_an_axis(self) -> None:
        self.assertEqual(axis_ticks(0), [0.0])


class KeptVariantTests(unittest.TestCase):
    def test_only_an_accepted_candidate_stays_in_the_product(self) -> None:
        self.assertEqual(kept_variant({"decision": "accepted"}), "candidate")
        for decision in ("rejected", "superseded", "in-progress", "baseline"):
            self.assertEqual(kept_variant({"decision": decision}), "control", decision)

    def test_a_record_that_keeps_neither_arm_names_none(self) -> None:
        # exp-103 rejected H86's Linux floor claim about a candidate that stays in the
        # stack, so reading `control` off its decision named the pre-H86 binary as the
        # product. Neither arm is the shipped binary, and the record says so.
        self.assertIsNone(kept_variant({"decision": "rejected", "kept": "neither"}))
        self.assertEqual(kept_variant({"decision": "rejected"}), "control")

    def test_the_record_decides_which_arm_shipped_not_the_decision(self) -> None:
        # An accepted build-profile screen the release profile never adopted projected
        # the candidate as the product's cost; a rejected change that shipped anyway
        # projected the control. Both are wrong the same way, and both are the
        # record's to state.
        self.assertEqual(kept_variant({"decision": "accepted", "kept": "control"}), "control")
        self.assertEqual(
            kept_variant({"decision": "rejected", "kept": "candidate"}), "candidate"
        )


class SubjectIdentityTests(unittest.TestCase):
    def test_two_trees_sharing_a_path_are_not_one_subject(self) -> None:
        # The record contains one path reused for a 901,963-entry tree and a 60,993-entry
        # one. Keying on the path alone would average two unrelated workloads together.
        small = {"tree_root_id": "c" * 64, "tree_engine_digest": "d" * 64, "tree_entries": 60993}
        large = {"tree_root_id": "c" * 64, "tree_engine_digest": "e" * 64, "tree_entries": 901963}
        self.assertNotEqual(subject_key(small), subject_key(large))

    def test_one_tree_measured_twice_while_it_grew_is_one_family(self) -> None:
        # And the coarser grouping deliberately does put them together, because the
        # flagship subject gained 0.7% of its entries mid-campaign and splitting the
        # series over that would halve the only complete absolute record.
        before = {"host_system": "Darwin 25.5.0", "tree_root_id": "f" * 64}
        after = {"host_system": "Darwin 25.5.0", "tree_root_id": "f" * 64}
        self.assertEqual(subject_family(before), subject_family(after))

    def test_the_same_path_on_another_platform_is_another_family(self) -> None:
        mac = {"host_system": "Darwin 25.5.0", "tree_root_id": "f" * 64}
        linux = {"host_system": "Linux 6.18.5", "tree_root_id": "f" * 64}
        self.assertNotEqual(subject_family(mac), subject_family(linux))


class SyntheticSubjectTests(unittest.TestCase):
    """A generated tree averaged in with real ones is the mistake the record measured.

    The floor report put a uniform corpus at about 15 points of fdu's distance from the
    floor, and this page is where that distance is drawn.
    """

    def test_a_generator_recipe_marks_a_subject_generated(self) -> None:
        subject = {
            "tree_label": "never-seen-before",
            "tree_provenance": "python3 explorations/benchmarks/spikes/gen_tree.py <root> 17000",
        }
        self.assertTrue(is_synthetic(subject))

    def test_an_observed_tree_is_not_marked_generated(self) -> None:
        subject = {
            "tree_label": "cargo-registry-src",
            "tree_provenance": "The cargo registry source cache for this lockfile.",
        }
        self.assertFalse(is_synthetic(subject))

    def test_the_label_list_still_covers_artifacts_predating_provenance(self) -> None:
        self.assertTrue(is_synthetic({"tree_label": "adaptive-fast-slow-100k"}))
        self.assertFalse(is_synthetic({"tree_label": "metabrowser"}))

    def test_every_linux_generated_subject_is_covered(self) -> None:
        """The three that were missing, named so a silent removal fails here.

        `meta450k` and `vm450k` are `gen_tree.py` at 450,463 entries -- the whole Linux
        index-tier record -- and `spike-15977` is exp-064's subject, 22.6x sparse.
        """
        for label in ("meta450k", "vm450k", "spike-15977"):
            with self.subTest(label=label):
                self.assertIn(label, SYNTHETIC_SUBJECTS)

    def test_the_corpus_generated_linux_subject_is_covered(self) -> None:
        """exp-103's tree came from the corpus generator, whose recipe names no script.

        Its provenance reads "Generated balanced recipe, 450,001 entries ...", which the
        `gen_tree.py` check cannot see, so the page drew a generated tree as a real Linux
        subject.
        """
        subject = {
            "tree_label": "linux-450k",
            "tree_provenance": "Generated balanced recipe, 450,001 entries, manifest 65aa72b5",
        }
        self.assertTrue(is_synthetic(subject))

    def test_the_corpus_generator_marks_the_balanced_million_entry_tree(self) -> None:
        """The tool comparisons' tree, recorded by exp-160-165 on both platforms.

        Its provenance names the corpus generator module, not `gen_tree.py`, and its
        labels (`linux-balanced-1m`, `macos-balanced-1m`) were never listed, so the page
        drew a generated tree as a real subject on macOS and on Linux.
        """
        subject = {
            "tree_label": "linux-balanced-1m",
            "tree_provenance": (
                "Generated by explorations/benchmarks: python -m benchmarks.generate create "
                "--recipe balanced --entries 1000000 (seed fdu-balanced-v1)"
            ),
        }
        self.assertTrue(is_synthetic(subject))

    def test_a_real_tree_that_mentions_generated_files_is_not_marked(self) -> None:
        # A checkout's provenance can say it carries generated files; only a generator's
        # name marks the tree itself as generated.
        subject = {
            "tree_label": "metabrowser-clone",
            "tree_provenance": (
                "Live checkout of the repository with local generated, installed, and "
                "untracked state"
            ),
        }
        self.assertFalse(is_synthetic(subject))

    def test_one_label_marking_a_subject_marks_it_everywhere(self) -> None:
        dataset = project(
            [
                experiment("exp-001", label="meta450k"),
                experiment("exp-002", label="meta450k"),
            ]
        )
        self.assertTrue(all(subject["synthetic"] for subject in dataset["subjects"]))


class ProjectionTests(unittest.TestCase):
    def test_paired_and_marginal_readings_are_kept_apart(self) -> None:
        # They answer different questions and disagree in this record often enough that
        # letting a chart reach for either interchangeably would misreport verdicts.
        dataset = project([experiment("exp-001")])
        metrics = dataset["experiments"][0]["jobs"][0]["metrics"]["wall_ns"]
        self.assertEqual(metrics["absolute"], {"control": 200e6, "candidate": 100e6})
        self.assertEqual(metrics["paired"]["change_pct"], -50.0)
        self.assertEqual(metrics["paired"]["evidence"], "improved")

    def test_evidence_is_read_off_the_interval_not_the_point(self) -> None:
        cases = {
            "improved": metric(200e6, 100e6, -50.0, -55.0, -45.0),
            "regressed": metric(100e6, 200e6, 50.0, 45.0, 55.0),
            "unclear": metric(100e6, 99e6, -1.0, -5.0, 3.0),
            "unmeasured": metric(100e6, 99e6, -1.0, None, None),
        }
        for expected, wall in cases.items():
            dataset = project([experiment("exp-001", wall=wall)])
            paired = dataset["experiments"][0]["jobs"][0]["metrics"]["wall_ns"]["paired"]
            self.assertEqual(paired["evidence"], expected)

    def test_an_anchor_is_decided_by_the_control_binary_not_the_verdict(self) -> None:
        # exp-014 is labelled a baseline but establishes a mid-campaign reference. Reading
        # membership off the verdict admitted it to the absolute series and inflated the
        # measured baseline spread from 8% to 98%.
        anchored = project([experiment("exp-006", control=f"main @ {BASELINE_COMMIT}")])
        self.assertTrue(anchored["experiments"][0]["anchored"])
        mid = project([experiment("exp-014", decision="baseline", control="exp-013 build")])
        self.assertFalse(mid["experiments"][0]["anchored"])

    def test_the_absolute_series_reports_the_spread_of_its_repeated_baseline(self) -> None:
        # The same unchanged binary measured at several checkpoints disagrees with
        # itself, and that disagreement is the error bar on the whole figure. It has to
        # reach the dataset, because a reader comparing two checkpoints cannot otherwise
        # tell a real step from the host having a bad afternoon.
        control = f"main @ {BASELINE_COMMIT}"
        dataset = project(
            [
                experiment("exp-000", control=control, wall=metric(100e6, 100e6, 0.0)),
                experiment("exp-006", control=control, wall=metric(120e6, 60e6, -50.0)),
            ]
        )
        job = dataset["anchors"]["series"][0]["jobs"][0]
        self.assertAlmostEqual(job["baseline_spread_pct"], 20.0)
        self.assertEqual(job["baseline_low_ns"], 100e6)
        self.assertEqual(job["baseline_high_ns"], 120e6)

    def test_a_lone_anchor_makes_no_series(self) -> None:
        # One checkpoint cannot show a trajectory and has no spread to report, so
        # publishing it as a series would dress a single measurement as a history.
        dataset = project([experiment("exp-006", control=f"main @ {BASELINE_COMMIT}")])
        self.assertEqual(dataset["anchors"]["series"], [])

    def test_synthetic_subjects_are_flagged_however_they_are_labelled(self) -> None:
        dataset = project([experiment("exp-057", label="adaptive-fast-slow-100k")])
        self.assertTrue(dataset["subjects"][0]["synthetic"])
        ordinary = project([experiment("exp-001", label="metabrowser")])
        self.assertFalse(ordinary["subjects"][0]["synthetic"])

    def test_only_accepted_changes_count_toward_the_cost_carried(self) -> None:
        dataset = project(
            [
                experiment("exp-001", decision="accepted", lines=100),
                experiment("exp-002", decision="rejected", lines=900),
            ]
        )
        self.assertEqual(dataset["totals"]["accepted_lines_changed"], 100)


class RenderTests(unittest.TestCase):
    def _page(self) -> str:
        control = f"main @ {BASELINE_COMMIT}"
        dataset = project(
            [
                experiment("exp-000", control=control, wall=metric(100e6, 100e6, 0.0)),
                experiment("exp-006", control=control, wall=metric(120e6, 60e6, -50.0)),
            ]
        )
        return render(dataset)

    def test_the_page_is_a_complete_standalone_document(self) -> None:
        # It is opened from a file:// URL, not embedded in a host that supplies a head.
        page = self._page()
        self.assertTrue(page.startswith("<!doctype html>"), page[:40])
        for required in ('<meta charset="utf-8">', "<title>", "<style>", "<body>", "</html>"):
            self.assertIn(required, page)

    def test_the_page_fetches_nothing(self) -> None:
        # A committed document in a repository that pins every other dependency should not
        # acquire an unpinned one, and the page has to render offline on a machine that
        # has never opened it before.
        #
        # What is forbidden is a *fetch*, not a URL. An earlier version of this test
        # rejected every "https://" and so rejected the hyperlink to the project itself,
        # which loads nothing and is the one thing a reader most wants. So the check names
        # the constructs that actually retrieve something.
        page = self._page()
        fetching = (
            "<link ",
            "<script src",
            "<img",
            "<iframe",
            "@import",
            "url(http",
            "src=",
        )
        for construct in fetching:
            self.assertNotIn(construct, page, f"page retrieves something via {construct}")

    def test_accepted_evidence_is_not_labelled_as_retained_code(self) -> None:
        self.assertEqual(
            decision_label({"decision": "accepted", "kept": None}),
            "accepted evidence",
        )
        self.assertEqual(
            decision_label({"decision": "accepted", "kept": "candidate"}),
            "candidate kept",
        )

    def test_links_are_allowed_and_the_project_is_one_of_them(self) -> None:
        # The counterpart to the check above: a reader arriving at this page cold should
        # be one click from what the thing being measured actually is.
        self.assertIn('href="https://github.com/jlevy/fdu"', self._page())

    def test_the_stylesheet_is_ascii_so_the_entity_pass_cannot_touch_it(self) -> None:
        # Character references are not interpreted inside a `<style>` element, so any
        # non-ASCII in the stylesheet gets rewritten by the output pass into something
        # CSS then renders literally. That shipped once: a CSS escape for the minus sign
        # was written in a non-raw Python string, Python read its digits as octal, and the
        # disclosure marker became "&#145;2" on the page.
        offenders = [repr(ch) for ch in STYLE if ord(ch) > 127]
        self.assertEqual(offenders, [], "non-ASCII in the stylesheet")

    def test_the_page_is_also_emitted_as_pure_ascii(self) -> None:
        # The charset above is the primary defence; this is the second one, so the page
        # survives a host that guesses. Without it every literal "µ" arrived as "Âµ".
        self._page().encode("ascii")

    def test_a_baseline_checkpoint_is_not_drawn_as_a_comparison(self) -> None:
        # Its two arms are the same measurement. Drawing it as a before-and-after would
        # invent a comparison the run never made.
        control = f"main @ {BASELINE_COMMIT}"
        dataset = project(
            [
                experiment(
                    "exp-000", decision="baseline", control=control, wall=metric(100e6, 100e6, 0.0)
                ),
                experiment("exp-006", control=control, wall=metric(120e6, 60e6, -50.0)),
            ]
        )
        points = dataset["anchors"]["series"][0]["jobs"][0]["points"]
        self.assertTrue(points[0]["baseline"])
        self.assertFalse(points[1]["baseline"])
        figure = figure_absolute(dataset)
        self.assertIn("exp-006", figure)

    def test_a_baseline_comparing_two_builds_is_drawn_with_its_change(self) -> None:
        # exp-202 measured the 0.3.0 engine against 0.2.1's and decided nothing, so it is
        # a baseline; the page left its -48% blank and kept it off the effects figure, as
        # if it were an A/A cell like exp-175. The binaries the record names tell them
        # apart.
        release = experiment(
            "exp-202",
            decision="baseline",
            kept="neither",
            wall=metric(208.6e6, 109.9e6, -48.0, -50.5, -44.8),
        )
        release["method"]["control_binary"] = {"name": "v021", "sha256": "c" * 64, "args": []}
        release["method"]["candidate_binary"] = {"name": "release", "sha256": "d" * 64, "args": []}
        same = experiment(
            "exp-175", decision="baseline", wall=metric(181.9e6, 180.0e6, -1.1, -3.2, +1.4)
        )
        same["method"]["control_binary"] = {"name": "a", "sha256": "e" * 64, "args": []}
        same["method"]["candidate_binary"] = {"name": "b", "sha256": "e" * 64, "args": []}
        dataset = project([experiment("exp-101"), release, same])
        records = {record["id"]: record for record in dataset["experiments"]}
        self.assertTrue(records["exp-101"]["compares"])
        self.assertTrue(records["exp-202"]["compares"])
        self.assertFalse(records["exp-175"]["compares"])
        figure = figure_effects(dataset)
        self.assertIn("exp-202", figure)
        self.assertNotIn("exp-175", figure)
        self.assertIn("1 of them baselines that compare two builds", figure)
        page = render(dataset)
        self.assertIn("-48.0%", page)
        self.assertNotIn("-1.1%", page)

    def _end_to_end(self, identifier: str, *, system: str = "Linux 6.18.44-fc-v50") -> Dict[str, Any]:
        record = experiment(
            identifier,
            decision="baseline",
            kept="neither",
            system=system,
            wall=metric(208.6e6, 109.9e6, -48.0, -50.5, -44.8),
        )
        record["results"][0]["job"] = "default-tree"
        record["verdict"]["primary_job"] = "default-tree"
        record["method"]["control_binary"] = {"name": "old", "sha256": "c" * 64, "args": []}
        record["method"]["candidate_binary"] = {"name": "new", "sha256": "d" * 64, "args": []}
        return record

    def test_the_end_to_end_figure_draws_only_two_build_default_tree_cells(self) -> None:
        # An A/A cell and an accepted step are not end-to-end comparisons; drawing them
        # beside the release cell would chain sessions the caption says not to chain.
        same = self._end_to_end("exp-175")
        same["method"]["candidate_binary"] = same["method"]["control_binary"]
        dataset = project(
            [
                self._end_to_end(STANDING_EXPERIMENT),
                self._end_to_end("exp-074", system="Darwin 25.5.0"),
                same,
                experiment("exp-178", system="Linux 6.18.44-fc-v49"),
            ]
        )
        self.assertEqual(
            [record["id"] for record in end_to_end_cells(dataset, "Linux")], [STANDING_EXPERIMENT]
        )
        figure = figure_end_to_end(dataset, "Linux")
        self.assertIn("209 ms", figure)
        self.assertIn("110 ms", figure)
        self.assertNotIn("exp-175", figure)
        self.assertEqual(figure_end_to_end(project([experiment("exp-001")]), "Linux"), "")

    def test_the_header_states_the_release_standing_when_it_is_recorded(self) -> None:
        with_release = render(project([self._end_to_end(STANDING_EXPERIMENT)]))
        self.assertIn("0.2.1 to 0.3.0", with_release)
        # A projection without the release record still renders, without the figure.
        self.assertNotIn("0.2.1 to 0.3.0", render(project([experiment("exp-001")])))

    def test_every_remeasurement_names_a_record_that_would_otherwise_count(self) -> None:
        # The list is hand-maintained; an id with no record, or a record that would not
        # count as kept anyway, means the list and the record have drifted apart.
        import json
        from pathlib import Path

        from benchmarks.realtree.report_html import REMEASUREMENTS

        committed = Path("docs/project/reports/performance-evidence/timeline.json")
        records = {
            record["id"]: record
            for record in json.loads(committed.read_text(encoding="utf-8"))["experiments"]
        }
        for identifier in REMEASUREMENTS:
            self.assertIn(identifier, records)
            record = records[identifier]
            self.assertEqual(record["decision"], "accepted", identifier)
            self.assertLessEqual(record["change_pct"], -3, identifier)

    def test_a_remeasurement_is_not_counted_as_a_kept_change(self) -> None:
        # Cumulative checkpoints re-measure campaign 1; counting them as kept changes drew
        # the same work four times as tall green bars.
        dataset = project(
            [
                experiment("exp-032"),
                experiment("exp-154", kept="control"),
                experiment("exp-190"),
            ]
        )
        kinds = {record["id"]: iteration_kind(record) for record in dataset["experiments"]}
        self.assertEqual(kinds["exp-032"], "measured")
        self.assertEqual(kinds["exp-154"], "measured")
        self.assertEqual(kinds["exp-190"], "kept")
        self.assertIn("Not a new change: a cumulative checkpoint", figure_timeline(dataset))

    def test_history_cells_load_in_display_order(self) -> None:
        import json
        import tempfile
        from pathlib import Path

        from benchmarks.realtree.timeline import load_history

        with tempfile.TemporaryDirectory() as directory:
            for name, order in (("a-cell", 2), ("b-cell", 1)):
                Path(directory, f"{name}.json").write_text(
                    json.dumps(
                        {
                            "display_order": order,
                            "subject": {"label": name, "counts": {"total": 10}},
                            "milestones": [{"label": "x", "wall_ms": {"median": 5.0}}],
                        }
                    ),
                    encoding="utf-8",
                )
            cells = load_history(Path(directory))
        self.assertEqual([cell["id"] for cell in cells], ["b-cell", "a-cell"])
        self.assertEqual(cells[0]["milestones"][0]["wall_ms"], 5.0)

    def test_the_page_carries_the_theme_chooser(self) -> None:
        page = render(project([experiment("exp-001")]))
        for choice in ("system", "light", "dark"):
            self.assertIn(f'data-theme-choice="{choice}"', page)
        self.assertIn("fdu.report.themeMode", page)

    def test_the_iterations_figure_draws_every_experiment_once(self) -> None:
        # The figure is the page's record of every iteration; one silently dropped is a
        # rejected idea the reader never sees.
        dataset = project(
            [
                experiment("exp-001"),
                experiment("exp-002", decision="rejected"),
                experiment("exp-067", decision="baseline"),
                experiment("exp-250"),
            ]
        )
        figure = figure_timeline(dataset)
        for identifier in ("exp-001", "exp-002", "exp-067", "exp-250"):
            self.assertEqual(figure.count(f"{identifier}: Experiment {identifier}"), 1)
        kinds = {record["id"]: iteration_kind(record) for record in dataset["experiments"]}
        self.assertEqual(kinds["exp-001"], "kept")
        self.assertEqual(kinds["exp-002"], "rejected")
        self.assertEqual(kinds["exp-067"], "measured")

    def test_the_runtime_panel_draws_only_milestones_it_can_place(self) -> None:
        # The top panel steps through the history cell's builds at the experiment each
        # follows. A milestone naming an experiment the record lacks is left out rather
        # than drawn at an invented position.
        dataset = project([experiment("exp-000"), experiment("exp-032")])
        dataset["history"] = [_index_cell("cold-cache", 150000.0, 30000.0, extra_after="exp-999")]
        dataset["index"] = project_index(dataset["history"], load_suite())
        figure = figure_timeline(dataset)
        self.assertIn("5.0x better", figure)
        self.assertIn("5.0x faster", figure)
        # The unplaced build's name would appear in its tooltip and the caption if drawn.
        self.assertNotIn("unplaced-build", figure)
        self.assertIn('data-metric="score"', figure)
        self.assertIn('<option value="cold-cache"', figure)
        # The header uses the same placement, so it cannot claim 150x for a build the
        # chart leaves out.
        page = render(dataset)
        self.assertNotIn("150.0&times;", page)
        # Milestone times are milliseconds; passing them to the nanosecond formatter
        # printed every runtime as "0 ms".
        self.assertNotIn(" 0 ms", page)

    def test_the_score_is_a_weighted_sum_of_log_ratios(self) -> None:
        # A 2x gain on one component and a 2x loss on another of equal weight cancel
        # exactly; a sum of raw times or raw ratios would not.
        result = combine(
            {
                "a": {"ratio": 2.0, "low": 2.0, "high": 2.0},
                "b": {"ratio": 0.5, "low": 0.5, "high": 0.5},
            },
            {"a": 0.25, "b": 0.25},
        )
        self.assertAlmostEqual(result["index"], 1.0)
        # Weights renormalize over the components a build has.
        alone = combine({"a": {"ratio": 4.0, "low": 3.0, "high": 5.0}}, {"a": 0.1})
        self.assertAlmostEqual(alone["index"], 4.0)
        self.assertLess(alone["low"], 4.0)
        self.assertGreater(alone["high"], 4.0)

    def test_every_recorded_job_maps_to_a_scored_component(self) -> None:
        # The completeness rule: a job the loop measures but the score cannot see would
        # let a kept change improve something no line shows.
        suite = load_suite()
        self.assertEqual(unmapped_jobs(project([experiment("exp-001")])["experiments"], suite), [])
        stray = experiment("exp-002")
        stray["results"][0]["job"] = "brand-new-job"
        stray["verdict"]["primary_job"] = "brand-new-job"
        self.assertEqual(unmapped_jobs(project([stray])["experiments"], suite), ["brand-new-job"])

    def test_the_suite_weights_sum_to_one(self) -> None:
        suite = load_suite()
        self.assertAlmostEqual(sum(item["weight"] for item in suite["components"]), 1.0)

    def test_a_cell_measured_under_another_definition_is_refused(self) -> None:
        # Manifest v1 was once edited in place while cells kept claiming v1; the digest
        # makes that a refusal instead of a silently wrong score.
        stale = _index_cell("cold-cache", 300.0, 100.0, digest="0000000000000000")
        with self.assertRaises(ValueError):
            project_index([stale], load_suite())

    def test_memory_follows_each_score_lines_components_and_counts_each_once(self) -> None:
        # Restricting memory to the components every build has hid the content and
        # summary memory wins from the full score; each line now averages its own mix,
        # and a component with two jobs counts once, not twice.
        def cell(peaks: Dict[str, float]) -> Dict[str, Any]:
            return {"milestones": [{"label": label, "peak_rss_mib": peak} for label, peak in peaks.items()]}

        by_job = {
            ("cold-cache", "cold-cache"): cell({"prework": 400.0, "mid": 200.0, "v0.3.0": 100.0}),
            ("code", "code"): cell({"mid": 50.0, "v0.3.0": 100.0}),
            ("opened-root", "a"): cell({"mid": 400.0, "v0.3.0": 100.0}),
            ("opened-root", "b"): cell({"mid": 400.0, "v0.3.0": 100.0}),
        }
        full = memory_ratios(by_job, "v0.3.0", ["cold-cache", "code", "opened-root"])
        # mid: cold-cache 2x, code 0.5x, opened-root 4x, its two jobs counted once, gives
        # (2 * 0.5 * 4) ** (1/3); counting opened-root twice would give exactly 2x.
        self.assertAlmostEqual(full["mid"]["ratio"], 4 ** (1 / 3), places=6)
        partial = memory_ratios(by_job, "v0.3.0", ["cold-cache"])
        self.assertAlmostEqual(partial["mid"]["ratio"], 2.0, places=6)
        self.assertAlmostEqual(partial["prework"]["ratio"], 4.0, places=6)
        # A build with only one of a component's two jobs has no memory on it.
        lopsided = dict(by_job)
        lopsided[("opened-root", "b")] = cell({"v0.3.0": 100.0})
        self.assertNotIn("mid", memory_ratios(lopsided, "v0.3.0", ["opened-root"]))

    def test_the_partial_score_uses_its_own_memory_mix(self) -> None:
        # project_index swaps the common-component memory into the partial score; without
        # it, a build would be averaged over components the first build never had.
        def milestone(label: str, wall: float, peak: float, change: float | None) -> Dict[str, Any]:
            return {
                "label": label, "short": label, "commit": label[:4], "date": "2026-09-01",
                "includes": "", "after_experiment": "exp-001", "wall_ms": wall,
                "peak_rss_mib": peak, "vs_latest_pct": change,
                "vs_latest_ci95_pct": None if change is None else [change, change],
                "supported": True,
            }

        suite = load_suite()
        definitions = {item["id"]: item for item in suite["components"]}
        cold = {
            "id": "cold", "component": "cold-cache", "job": "cold-cache", "platform": "macOS",
            "component_digest": component_digest(definitions["cold-cache"]),
            "milestones": [
                milestone("prework", 400.0, 400.0, 300.0),
                milestone("mid", 200.0, 200.0, 100.0),
                milestone("v0.3.0", 100.0, 100.0, None),
            ],
        }
        code = {
            "id": "code", "component": "code", "job": "code", "platform": "macOS",
            "component_digest": component_digest(definitions["code"]),
            "milestones": [milestone("mid", 100.0, 50.0, 0.0), milestone("v0.3.0", 100.0, 100.0, None)],
        }
        rows = {row["label"]: row for row in project_index([cold, code], suite)[0]["builds"]}
        # mid's full memory mixes cold-cache 2x and code 0.5x to 1x; its partial memory,
        # over the cold-cache component every build has, is 2x.
        self.assertAlmostEqual(rows["mid"]["components"]["memory"], 1.0, places=6)
        self.assertAlmostEqual(rows["mid"]["memory_common"], 2.0, places=6)
        weights = {item["id"]: item["weight"] for item in suite["components"]}
        expected = combine(
            {
                "cold-cache": {"ratio": 2.0, "low": 2.0, "high": 2.0},
                "memory": {"ratio": 2.0, "low": 2.0, "high": 2.0},
            },
            weights,
        )
        self.assertAlmostEqual(rows["mid"]["common"]["index"], expected["index"], places=6)

    def test_a_component_with_two_jobs_scores_their_geometric_mean(self) -> None:
        # Warm metadata times cold-open-save and warm-revalidate; a 4x and a 1x job score
        # 2x, and a build missing one job is not scored on the component at all.
        cells = [
            _index_cell("warm-metadata", 400.0, 100.0, job="cold-open-save"),
            _index_cell("warm-metadata", 100.0, 100.0, job="warm-revalidate"),
        ]
        ratios = component_ratios(cells, load_suite(), "macOS")
        self.assertAlmostEqual(ratios["warm-metadata"]["prework"]["ratio"], 2.0, places=6)
        self.assertNotIn("warm-metadata", component_ratios(cells[:1], load_suite(), "macOS"))

    def test_the_headline_interval_combines_errors_rather_than_extremes(self) -> None:
        first = {"index": 4.0, "low": 3.0, "high": 5.0}
        last = {"index": 1.0, "low": 0.8, "high": 1.25}
        speedup, low, high = score_ratio(first, last)
        self.assertAlmostEqual(speedup, 4.0)
        # The worst-case bound would be [2.4, 6.25]; a combined interval is narrower.
        self.assertGreater(low, 3.0 / 1.25)
        self.assertLess(high, 5.0 / 0.8)

    def test_a_projection_without_the_field_reads_baselines_as_one_build(self) -> None:
        # A committed projection written before `compares` existed still renders.
        dataset = project([experiment("exp-101"), experiment("exp-000", decision="baseline")])
        for record in dataset["experiments"]:
            del record["compares"]
        self.assertNotIn("exp-000", figure_effects(dataset))
        self.assertIn("exp-101", figure_effects(dataset))

    def test_a_record_keeping_neither_arm_is_not_drawn_as_the_trees_current_cost(self) -> None:
        # The per-entry figure plots the arm that stayed in the product. exp-103 is
        # recorded `rejected` because H86's Linux floor claim failed, but the candidate it
        # measured stays in the stack, so drawing its control put the pre-H86 binary on
        # the page as Linux's current cost: 4.23 us per entry where the candidate
        # measured 3.42. The record carries `kept: neither`; nothing else marks it.
        dataset = project(
            [
                experiment("exp-101"),
                experiment(
                    "exp-103",
                    decision="rejected",
                    kept="neither",
                    system="Linux 6.18.44-fc-v22",
                    root="c" * 64,
                    entries=450001,
                    wall=metric(1905.6e6, 1537.0e6, -18.2, -24.3, -13.7),
                ),
            ]
        )
        record = next(item for item in dataset["experiments"] if item["id"] == "exp-103")
        self.assertIsNone(record["kept"])
        figure = figure_per_entry(dataset)
        self.assertIn("exp-101", figure)
        self.assertNotIn("exp-103", figure)

    def test_an_accepted_arm_that_never_shipped_is_drawn_as_its_control(self) -> None:
        # A PGO screen was accepted with `[profile.release]` unchanged, and the page
        # reported the candidate's 4.98 us/entry as the product's latest cost where the
        # shipped probe measured 5.46. The record states `kept: control`; the figure
        # draws that arm.
        dataset = project(
            [
                experiment("exp-101", wall=metric(100e6, 90e6, -10.0, -12.0, -8.0)),
                experiment(
                    "exp-154",
                    kept="control",
                    wall=metric(5460e3, 4980e3, -8.3, -9.0, -7.5),
                ),
            ]
        )
        record = next(item for item in dataset["experiments"] if item["id"] == "exp-154")
        self.assertEqual(record["kept"], "control")
        last = record["jobs"][0]["per_entry_ns"]["control"]
        self.assertEqual(last, 5460e3 / 1000)
        self.assertIn("5.46", figure_per_entry(dataset))
        self.assertNotIn("4.98", figure_per_entry(dataset))

    def test_a_bytes_primary_metric_is_not_printed_as_milliseconds(self) -> None:
        # exp-117 is the first accept whose primary metric is peak RSS. The table used
        # to format every primary as milliseconds, so 395,886,592 → 355,868,672 bytes
        # read as 396 ms → 356 ms.
        dataset = project(
            [
                experiment(
                    "exp-117",
                    primary_metric="peak_rss_bytes",
                    extra_metrics={
                        "peak_rss_bytes": metric(
                            395_886_592, 355_868_672, -10.127, -10.49, -10.03
                        )
                    },
                )
            ]
        )
        page = render(dataset)
        self.assertEqual(fmt_primary(395_886_592, "peak_rss_bytes"), "377.5 MiB")
        self.assertEqual(fmt_primary(355_868_672, "peak_rss_bytes"), "339.4 MiB")
        self.assertNotIn("396 ms", page)
        self.assertNotIn("356 ms", page)
        self.assertIn("377.5 MiB", page)
        self.assertIn("339.4 MiB", page)

    def test_both_platform_comparisons_are_linked(self) -> None:
        # The header linked only the macOS comparison after the Linux one was published,
        # so a reader was sent to one platform's peer table as if it were the answer.
        page = self._page()
        self.assertIn("report-2026-09-26-fdu-live-tool-comparison.md", page)
        self.assertIn("report-2026-09-27-fdu-linux-tool-comparison.md", page)

    def test_no_figure_overflows_its_plot(self) -> None:
        # A bar drawn past the plot lands in the value column and reads as a number
        # belonging to a different row. This is what the axis-ladder bug produced.
        import re

        control = f"main @ {BASELINE_COMMIT}"
        dataset = project(
            [
                experiment("exp-000", control=control, wall=metric(1218.7e6, 1218.7e6, 0.0)),
                experiment("exp-006", control=control, wall=metric(900e6, 300e6, -60.0)),
            ]
        )
        figure = figure_absolute(dataset)
        viewbox = re.search(r'viewBox="0 0 (\d+) (\d+)"', figure)
        assert viewbox
        width = int(viewbox.group(1))
        for x, extent in re.findall(r'<rect class="drift-band" x="([\d.]+)"[^>]*width="([\d.]+)"', figure):
            self.assertLessEqual(float(x) + float(extent), width, "band runs off the canvas")
        for cx in re.findall(r'<circle class="dot-[a-z]+" cx="([\d.]+)"', figure):
            self.assertLessEqual(float(cx), width)


class PlatformSectionTests(unittest.TestCase):
    """The per-platform cut: what each platform's own runs decided and kept."""

    def _dataset(self) -> Dict[str, Any]:
        linux = "Linux 6.18.44-fc-v37"
        return project(
            [
                experiment("exp-101", wall=metric(100e6, 90e6, -10.0, -12.0, -8.0)),
                # Accepted on noninferiority: an instrument, not a speed-up.
                experiment("exp-102", wall=metric(100e6, 99e6, -1.0, -4.0, 2.0)),
                # Accepted, but the release never adopted it.
                experiment("exp-103", kept="control", wall=metric(100e6, 90e6, -10.0, -12.0, -8.0)),
                experiment(
                    "exp-160",
                    system=linux,
                    root="c" * 64,
                    label="linux-balanced-1m",
                    wall=metric(1611e6, 1542e6, -3.19, -4.88, -1.79),
                ),
                experiment(
                    "exp-173",
                    system=linux,
                    root="d" * 64,
                    label="linux-v6.12",
                    wall=metric(505e6, 260e6, -47.0, -52.4, -44.3),
                ),
                experiment(
                    "exp-188",
                    decision="rejected",
                    system=linux,
                    root="d" * 64,
                    label="linux-v6.12",
                    wall=metric(576e6, 570e6, -2.19, -4.5, 1.08),
                ),
            ]
        )

    def test_each_platform_lists_only_improvements_it_kept(self) -> None:
        dataset = self._dataset()
        self.assertEqual(
            [record["id"] for record in kept_improvements(dataset, "macOS")], ["exp-101"]
        )
        self.assertEqual(
            [record["id"] for record in kept_improvements(dataset, "Linux")],
            ["exp-160", "exp-173"],
        )

    def test_the_section_counts_every_verdict_and_names_generated_deciders(self) -> None:
        dataset = self._dataset()
        # Mark the balanced tree generated the way the projection would from provenance.
        for subject in dataset["subjects"]:
            if subject["labels"][0] == "linux-balanced-1m":
                subject["synthetic"] = True
        page = render(dataset)
        self.assertIn('id="platforms"', page)
        self.assertIn("Linux: 2 accepted runs that improved", page)
        self.assertIn("macOS: 1 accepted run that improved", page)
        self.assertIn("Decided on a generated tree: 1 of 2.", page)
        self.assertIn("Decided on a generated tree: 0 of 1.", page)
        # The rejected Linux run is counted in the summary but never listed as kept.
        # The mechanism table renders nothing for this dataset, so the section ends where
        # the reading notes begin.
        section = page.split('id="platforms"', 1)[1].split('id="reading"', 1)[0]
        self.assertNotIn("Experiment exp-188", section)
        self.assertNotIn("Experiment exp-103", section)
        self.assertIn("Experiment exp-173", section)


if __name__ == "__main__":
    unittest.main()
