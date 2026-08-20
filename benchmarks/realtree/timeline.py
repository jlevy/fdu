"""Project the experiment artifacts into the dataset a chart can be drawn from.

    python -m benchmarks.realtree.timeline --out <path.json>

The ledger answers "what happened in experiment N". This answers the two questions a
ledger cannot: *how fast is it, in milliseconds*, and *how did that number move over the
whole campaign*. Both are read back out of the same validated artifacts the ledger uses,
so neither can drift from the record.

Three facts about the evidence shape everything here, and getting any of them wrong
produces a chart that looks fine and lies.

**A paired change is not the ratio of the two medians.** Every experiment interleaves
control and candidate trials, and `change_pct` is the median of the paired differences.
The stored `control_median` and `candidate_median` are marginal medians of each arm.
Under host drift these disagree — in this record 21% of metric entries differ by more
than two percentage points and some differ in sign, exp-005's `cold-scan-index` reading
+2.8% by ratio and -3.9% paired. The paired figure is the one that controls for drift
and the one every verdict rests on. So the two are carried separately and a reader is
never invited to derive one from the other.

**Absolute values are only comparable within a subject.** Twenty-three distinct trees
were measured, on two platforms, across three orders of magnitude of scale. Absolute
milliseconds mean something against the tree they were measured on and nothing across
trees, which is why `per_entry_ns` is derived too: normalised that way the campaign's
own subjects agree closely, and a genuinely different workload stops hiding inside an
average.

**A re-measured baseline is the error bar.** Four cumulative experiments re-measured the
same pre-work binary on the same tree at different times. Their spread — 8% on
`cold-scan-index`, 35% on `cold-scan-producer` — is the honest uncertainty on any
cross-run absolute comparison, so it is computed and published rather than left for a
reader to assume away.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence

from benchmarks.realtree.summary import EXPERIMENTS_DIR, load_experiments

#: Emitted alongside the data so a consumer can tell which projection it holds.
DATASET_VERSION = "fdu.performance.timeline/1"

#: A change has to beat this to be kept. Mirrors `ledger.ACCEPT_THRESHOLD_PCT`; imported
#: rather than redefined would couple the chart to the accept rule's module, and the
#: number is part of what the chart draws, so it is stated here and asserted equal.
ACCEPT_THRESHOLD_PCT = -3.0

#: Metrics carried through to the dataset, with the unit each is rendered in.
METRICS = {
    "wall_ns": "ns",
    "component_ns": "ns",
    "cpu_ns": "ns",
    "user_cpu_ns": "ns",
    "system_cpu_ns": "ns",
    "blocked_ns": "ns",
    "peak_rss_bytes": "bytes",
}

#: Subjects that are not a sample of ordinary work and must never be averaged with one.
#:
#: `adaptive-fast-slow-100k` interleaves fast and slow directories deliberately to make
#: an adaptive worker policy fail; it costs about 18 µs per entry where a real tree of
#: the same age costs 5. Left unmarked it would read as a catastrophic regression on the
#: normalised axis rather than as the adversarial probe it is.
SYNTHETIC_SUBJECTS = {
    "adaptive-fast-slow-100k",
    "diagnostics-overhead",
    "generated-markdown-2000",
    "threshold-boundary-2x",
}

#: The pre-work commit every cumulative experiment measures against. An experiment whose
#: control is this binary re-measures the original baseline, which is what makes its
#: candidate value placeable on one absolute axis with the others.
BASELINE_COMMIT = "b565882"


class TimelineError(RuntimeError):
    """The artifacts could not be projected into a coherent dataset."""


def platform_of(subject: Mapping[str, Any]) -> str:
    """Which platform family a subject was measured on."""
    system = str(subject.get("host_system") or "")
    if system.startswith("Darwin"):
        return "macOS"
    if system.startswith("Linux"):
        return "Linux"
    return system.split()[0] if system else "unknown"


def kept_variant(decision: str) -> str:
    """Which measured arm remained in the product after the verdict.

    Derived rather than stored. The abandoned white-paper branch added a `kept_variant`
    field to the contract for this, which would have meant editing sixty-four artifacts
    and re-validating the schema to record something the verdict already determines: an
    accepted candidate is kept and everything else leaves the control in place. Deriving
    it keeps the contract as small as the evidence requires.

    `superseded` resolves to the control deliberately. A superseded candidate did ship
    briefly, but a later experiment replaced it, so the arm that describes the product's
    lasting state is the one it started from.
    """
    return "candidate" if decision == "accepted" else "control"


def subject_family(subject: Mapping[str, Any]) -> str:
    """The tree *path*, measured repeatedly over the campaign.

    Coarser than :func:`subject_key` on purpose. The flagship subject is one working
    directory that was re-measured for six days while ordinary work went on inside it,
    so it gained 413 entries — 0.7% — between the first campaign and the third. Those
    are strictly different trees and :func:`subject_key` is right to separate them, but
    refusing to draw them on one axis would split the campaign's only complete absolute
    series in half over a rounding error.

    Drawing them together is sound because every checkpoint carries its own freshly
    measured control: each before-and-after pair is a within-run comparison, and the
    tree's growth is reported per checkpoint rather than averaged away.
    """
    return f"{platform_of(subject)}:{str(subject.get('tree_root_id') or '')[:12]}"


def subject_key(subject: Mapping[str, Any]) -> str:
    """Identity of the measured tree, for grouping absolute values.

    Deliberately not `tree_root_id` alone. That id is the hash of the tree's *path*, and
    this record contains one path reused for two different trees — `c95b1edd…` names both
    a 901,963-entry subject and a 60,993-entry one. Entry count separates them, and the
    engine digest separates two trees that merely happen to hold the same number.
    """
    root = str(subject.get("tree_root_id") or "")[:12]
    digest = str(subject.get("tree_engine_digest") or "")[:12]
    return f"{root}:{subject.get('tree_entries')}:{digest}"


def _metric(entry: Mapping[str, Any], name: str) -> Optional[Dict[str, Any]]:
    """One metric, with the paired and the marginal readings kept apart.

    `change_pct` and `ci95_*` are the paired estimate. `control_median` and
    `candidate_median` are the marginal medians. They are grouped under separate keys so
    a chart cannot casually plot one as an explanation of the other.
    """
    if not isinstance(entry, Mapping):
        return None
    control = entry.get("control_median")
    candidate = entry.get("candidate_median")
    low, high = entry.get("ci95_low_pct"), entry.get("ci95_high_pct")
    return {
        "metric": name,
        "absolute": {"control": control, "candidate": candidate},
        "paired": {
            "change_pct": entry.get("change_pct"),
            "ci95_low_pct": low,
            "ci95_high_pct": high,
            "pairs": entry.get("pairs"),
            "significant": bool(entry.get("significant")),
            # Read off the interval rather than a stored flag, so artifacts written
            # before the fields were split describe themselves the same way.
            "evidence": (
                "unmeasured"
                if low is None or high is None
                else "improved"
                if high < 0
                else "regressed"
                if low > 0
                else "unclear"
            ),
        },
    }


def project(experiments: Sequence[Mapping[str, Any]]) -> Dict[str, Any]:
    """Build the whole dataset from validated artifacts."""
    records: List[Dict[str, Any]] = []
    subjects: Dict[str, Dict[str, Any]] = {}

    for experiment in experiments:
        subject = experiment["subject"]
        key = subject_key(subject)
        entries = int(subject["tree_entries"])
        label = str(subject["tree_label"])
        subjects.setdefault(
            key,
            {
                "key": key,
                "labels": [],
                "platform": platform_of(subject),
                "entries": entries,
                "directories": subject.get("tree_directories"),
                "files": subject.get("tree_files"),
                "apparent_bytes": subject.get("tree_apparent_bytes"),
                "host_cpu": subject.get("host_cpu"),
                "filesystem": subject.get("filesystem"),
                "os_cache": subject.get("os_cache"),
                "synthetic": label in SYNTHETIC_SUBJECTS,
                "experiments": [],
            },
        )
        if label not in subjects[key]["labels"]:
            subjects[key]["labels"].append(label)
        # One subject can be reached under several labels; synthetic is a property of
        # the tree, so any label that marks it marks the subject.
        subjects[key]["synthetic"] = subjects[key]["synthetic"] or label in SYNTHETIC_SUBJECTS
        subjects[key]["experiments"].append(experiment["id"])

        verdict = experiment["verdict"]
        decision = str(verdict["decision"])
        jobs: List[Dict[str, Any]] = []
        for result in experiment["results"]:
            metrics = {
                name: _metric(result["metrics"][name], name)
                for name in METRICS
                if name in (result.get("metrics") or {})
            }
            wall = metrics.get("wall_ns", {}).get("absolute", {}) if metrics else {}
            jobs.append(
                {
                    "job": result["job"],
                    "start_state": result.get("start_state"),
                    "invalid_samples": result.get("invalid_samples"),
                    "metrics": metrics,
                    # Normalised absolute cost, which is what makes a 60k tree and a 1M
                    # tree readable on one axis. Only wall is normalised: it is the one
                    # metric that is a straightforward per-entry cost.
                    "per_entry_ns": {
                        "control": (wall.get("control") / entries) if wall.get("control") else None,
                        "candidate": (
                            (wall.get("candidate") / entries) if wall.get("candidate") else None
                        ),
                    },
                }
            )

        control_text = str(experiment["method"].get("control") or "")
        records.append(
            {
                "id": experiment["id"],
                "number": int(str(experiment["id"]).removeprefix("exp-")),
                "date": experiment["date"],
                "title": experiment["title"],
                "hypotheses": experiment.get("hypotheses") or [],
                "decision": decision,
                "kept": kept_variant(decision),
                "primary_job": verdict.get("primary_job"),
                "primary_metric": verdict.get("primary_metric"),
                "change_pct": verdict.get("change_pct"),
                "reason": verdict.get("reason"),
                "commit": verdict.get("commit"),
                "control": control_text,
                "candidate": experiment["method"].get("candidate"),
                "trials": experiment["method"].get("trials"),
                "interleaved": bool(experiment["method"].get("interleaved")),
                "subject": key,
                "family": subject_family(subject),
                "platform": platform_of(subject),
                "entries": entries,
                "complexity": experiment.get("complexity") or {},
                # An experiment measuring against the pre-work binary re-measures the
                # original baseline, which is what lets its candidate be placed on one
                # absolute axis beside the others.
                "anchored": BASELINE_COMMIT in control_text,
                "jobs": jobs,
                "reference_tools": experiment.get("reference_tools") or [],
                "path": Path(str(experiment["_path"])).name,
            }
        )

    records.sort(key=lambda record: record["number"])
    dataset: Dict[str, Any] = {
        "version": DATASET_VERSION,
        "accept_threshold_pct": ACCEPT_THRESHOLD_PCT,
        "experiments": records,
        "subjects": sorted(subjects.values(), key=lambda item: -len(item["experiments"])),
    }
    dataset["anchors"] = _anchors(records)
    dataset["calibration"] = _calibration(records)
    dataset["totals"] = _totals(records)
    return dataset


def _anchors(records: Sequence[Mapping[str, Any]]) -> Dict[str, Any]:
    """The absolute series: cumulative checkpoints that share one subject and control.

    A checkpoint is an experiment whose control is the pre-work binary, so its candidate
    value and every other checkpoint's are separated only by the code between them. The
    controls are the same binary measured repeatedly, so their spread is this series'
    error bar — published beside it rather than assumed away, because on
    `cold-scan-producer` it reaches 35% and a reader comparing two checkpoints needs to
    know that before reading a 10% move as progress.
    """
    # Membership is decided by the control binary alone, never by the verdict. exp-014
    # is labelled a baseline but establishes a *mid-campaign* reference against the
    # then-current scheduler, so admitting it on its verdict put a 321 ms control into a
    # series of 627 ms ones and inflated the measured spread from 8% to 98%.
    anchored = [record for record in records if record["anchored"]]
    grouped: Dict[str, List[Mapping[str, Any]]] = {}
    for record in anchored:
        grouped.setdefault(record["family"], []).append(record)

    series: List[Dict[str, Any]] = []
    for subject, group in grouped.items():
        if len(group) < 2:
            continue
        jobs: Dict[str, Dict[str, Any]] = {}
        for record in sorted(group, key=lambda item: item["number"]):
            for job in record["jobs"]:
                wall = job["metrics"].get("wall_ns")
                if not wall:
                    continue
                slot = jobs.setdefault(job["job"], {"job": job["job"], "points": []})
                slot["points"].append(
                    {
                        "id": record["id"],
                        "number": record["number"],
                        "date": record["date"],
                        "title": record["title"],
                        # A baseline experiment has no candidate distinct from its
                        # control; its single value is the campaign's starting point.
                        "baseline": record["decision"] == "baseline",
                        "entries": record["entries"],
                        "control_ns": wall["absolute"]["control"],
                        "candidate_ns": wall["absolute"]["candidate"],
                        "per_entry_ns": job["per_entry_ns"]["candidate"],
                    }
                )
        for slot in jobs.values():
            controls = [point["control_ns"] for point in slot["points"] if point["control_ns"]]
            slot["baseline_ns"] = min(controls) if controls else None
            slot["baseline_low_ns"] = min(controls) if controls else None
            slot["baseline_high_ns"] = max(controls) if controls else None
            slot["baseline_spread_pct"] = (
                (max(controls) / min(controls) - 1) * 100 if len(controls) > 1 else None
            )
        ordered = sorted(group, key=lambda item: item["number"])
        series.append(
            {
                "family": subject,
                "platform": group[0]["platform"],
                "entries_first": ordered[0]["entries"],
                "entries_last": ordered[-1]["entries"],
                "subjects": sorted({record["subject"] for record in group}),
                "checkpoints": [record["id"] for record in sorted(group, key=lambda r: r["number"])],
                "jobs": sorted(jobs.values(), key=lambda item: item["job"]),
            }
        )
    series.sort(key=lambda item: -len(item["checkpoints"]))
    return {"series": series}


def _calibration(records: Sequence[Mapping[str, Any]]) -> List[Dict[str, Any]]:
    """Third-party timings on the same trees, for scale only.

    These never enter an accept decision — the tools answer a different question with
    different guarantees. They are worth publishing because they say what the numbers
    mean: on the 60k subject `dust` itself ranges from 210 ms to 327 ms across runs of
    the same unchanged binary, which is the clearest available statement of why a
    single-run before-and-after comparison would have been worthless.
    """
    rows: List[Dict[str, Any]] = []
    for record in records:
        for tool in record["reference_tools"]:
            rows.append(
                {
                    "id": record["id"],
                    "subject": record["subject"],
                    "platform": record["platform"],
                    "entries": record["entries"],
                    "tool": tool.get("name"),
                    "wall_ns": tool.get("wall_ns_median"),
                    "per_entry_ns": (
                        tool["wall_ns_median"] / record["entries"]
                        if tool.get("wall_ns_median")
                        else None
                    ),
                }
            )
    return rows


def _totals(records: Sequence[Mapping[str, Any]]) -> Dict[str, Any]:
    """Counts a reader would otherwise tally by hand, and could tally differently."""
    decisions: Dict[str, int] = {}
    platforms: Dict[str, int] = {}
    lines = 0
    for record in records:
        decisions[record["decision"]] = decisions.get(record["decision"], 0) + 1
        platforms[record["platform"]] = platforms.get(record["platform"], 0) + 1
        if record["decision"] == "accepted":
            lines += int(record["complexity"].get("lines_changed") or 0)
    dates = sorted(record["date"] for record in records)
    return {
        "experiments": len(records),
        "decisions": decisions,
        "platforms": platforms,
        "accepted_lines_changed": lines,
        "first_date": dates[0] if dates else None,
        "last_date": dates[-1] if dates else None,
        "hypotheses": len(
            {
                re.match(r"(H\d+|S\d+)", str(hypothesis)).group(1)
                for record in records
                for hypothesis in record["hypotheses"]
                if re.match(r"(H\d+|S\d+)", str(hypothesis))
            }
        ),
    }


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(prog="benchmarks.realtree.timeline", description=__doc__)
    parser.add_argument("--experiments", type=Path, default=EXPERIMENTS_DIR)
    parser.add_argument("--out", type=Path, required=True)
    arguments = parser.parse_args(list(argv))

    experiments = load_experiments(arguments.experiments)
    if not experiments:
        print("no experiment artifacts found", file=sys.stderr)
        return 1
    dataset = project(experiments)
    arguments.out.parent.mkdir(parents=True, exist_ok=True)
    arguments.out.write_text(json.dumps(dataset, indent=1, sort_keys=True), encoding="utf-8")
    print(
        f"wrote {arguments.out} from {len(experiments)} experiments "
        f"across {len(dataset['subjects'])} subjects",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
