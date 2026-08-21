"""The projection and the drawing rules that decide whether a figure tells the truth.

Every case here exists because getting it wrong produces a chart that looks finished and
is wrong, which is the failure mode a report of measurements can least afford.
"""

from __future__ import annotations

import unittest
from typing import Any, Dict, List

from benchmarks.realtree.report_html import STYLE, axis_ticks, figure_absolute, render
from benchmarks.realtree.timeline import (
    BASELINE_COMMIT,
    kept_variant,
    project,
    subject_family,
    subject_key,
)


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
) -> Dict[str, Any]:
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
                "metrics": {"wall_ns": wall or metric(200e6, 100e6, -50.0, -55.0, -45.0)},
            }
        ],
        "complexity": {"lines_changed": lines},
        "verdict": {
            "decision": decision,
            "primary_job": "cold-scan-index",
            "primary_metric": "wall_ns",
            "change_pct": -50.0,
            "reason": "because",
            "commit": None,
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
        self.assertEqual(kept_variant("accepted"), "candidate")
        for decision in ("rejected", "superseded", "in-progress", "baseline"):
            self.assertEqual(kept_variant(decision), "control", decision)


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


if __name__ == "__main__":
    unittest.main()
