"""The fdu performance index: one pre-registered score over a fixed suite of components.

The suite, its weights, and the reference build live in
`explorations/benchmarks/index-suite.json`; the design is
`docs/project/specs/active/plan-2026-10-05-fdu-performance-index.md`.

**The score is a weighted sum of log speedups.** For build *b* and component *c*, the
component ratio *r* is *b*'s runtime over the reference build's, taken from the paired
figure of one interleaved session, so no ratio chains two sessions. The index is
`exp(sum(w * ln r) / sum(w))`: a relative runtime, 1.0 at the reference build. Summing
logs keeps a 70-second benchmark from outweighing a 160-millisecond one and makes a 2x
gain and a 2x loss cancel exactly, which a sum of raw times or of raw ratios does not.

**Every optimization target is in the score.** Every benchmark job the loop has recorded
maps to a component in the manifest; `unmapped_jobs` names any that do not, and the
projection refuses to build while one exists, so a new target gets a component and a
weight before its first verdict is published.

**A build is scored only on what it can do.** A build without a component's capability
(content views before they existed) has no ratio there. The line the page draws uses
the components every plotted build supports, renormalized, and says which it leaves out;
the full index is published only for builds that have every component.
"""

from __future__ import annotations

import json
import math
from pathlib import Path
from typing import Any, Dict, Iterable, List, Mapping, Optional, Sequence

SUITE_PATH = Path("explorations/benchmarks/index-suite.json")

#: The two-sided 95% normal quantile, for combining component intervals in log space.
Z95 = 1.959963984540054


def load_suite(path: Path = SUITE_PATH) -> Dict[str, Any]:
    """The manifest, with its weights checked to sum to one."""
    suite = json.loads(path.read_text(encoding="utf-8"))
    total = sum(component["weight"] for component in suite["components"])
    if abs(total - 1.0) > 1e-9:
        raise ValueError(f"{path}: component weights sum to {total}, not 1")
    known = {component["id"] for component in suite["components"]}
    for job, component in suite["job_components"].items():
        if component not in known:
            raise ValueError(f"{path}: job {job!r} maps to unknown component {component!r}")
    return suite


def recorded_jobs(experiments: Iterable[Mapping[str, Any]]) -> List[str]:
    """Every job any experiment record measured, primary or not."""
    jobs = set()
    for record in experiments:
        if record.get("primary_job"):
            jobs.add(record["primary_job"])
        for job in record.get("jobs") or []:
            jobs.add(job["job"])
    return sorted(jobs)


def unmapped_jobs(experiments: Iterable[Mapping[str, Any]], suite: Mapping[str, Any]) -> List[str]:
    """Jobs the loop has measured that no scored component covers."""
    return [job for job in recorded_jobs(experiments) if job not in suite["job_components"]]


def _component_ratio(item: Mapping[str, Any], reference: str) -> Optional[Dict[str, float]]:
    """One build's runtime ratio to the reference on one cell, with its 95% interval.

    The paired figure is the median of adjacent pairs with the reference build in one
    session; the reference build is 1.0 by definition.
    """
    if item.get("supported") is False or not item.get("wall_ms"):
        return None
    if item["label"] == reference:
        return {"ratio": 1.0, "low": 1.0, "high": 1.0}
    change = item.get("vs_latest_pct")
    if change is None:
        return None
    interval = item.get("vs_latest_ci95_pct") or [change, change]
    return {
        "ratio": 1 + change / 100,
        "low": 1 + interval[0] / 100,
        "high": 1 + interval[1] / 100,
    }


def component_ratios(
    cells: Sequence[Mapping[str, Any]], suite: Mapping[str, Any], platform: str
) -> Dict[str, Dict[str, Dict[str, float]]]:
    """`{component: {build label: ratio}}` for one platform, memory included."""
    reference = suite["reference_build"]
    ratios: Dict[str, Dict[str, Dict[str, float]]] = {}
    by_component = {
        cell["component"]: cell
        for cell in cells
        if cell.get("component") and cell.get("platform") == platform
    }
    for component in suite["components"]:
        if component["measures"] == "peak_rss":
            continue
        cell = by_component.get(component["id"])
        if not cell:
            continue
        ratios[component["id"]] = {
            item["label"]: ratio
            for item in cell["milestones"]
            if (ratio := _component_ratio(item, reference)) is not None
        }
    for component in suite["components"]:
        if component["measures"] != "peak_rss":
            continue
        sources = [by_component[name] for name in component.get("from_components", []) if name in by_component]
        memory: Dict[str, Dict[str, float]] = {}
        for label in {item["label"] for cell in sources for item in cell["milestones"]}:
            logs = []
            for cell in sources:
                peaks = {item["label"]: item.get("peak_rss_mib") for item in cell["milestones"]}
                if peaks.get(label) and peaks.get(reference):
                    logs.append(math.log(peaks[label] / peaks[reference]))
            if logs:
                value = math.exp(sum(logs) / len(logs))
                memory[label] = {"ratio": value, "low": value, "high": value}
        if memory:
            ratios[component["id"]] = memory
    return ratios


def combine(
    ratios: Mapping[str, Mapping[str, float]], weights: Mapping[str, float]
) -> Dict[str, float]:
    """The weighted log-sum index over the given components, with a 95% interval.

    Each component's interval becomes a standard error in log space; the components are
    combined as independent, which they are, being separate sessions.
    """
    total = sum(weights[name] for name in ratios)
    log_index = sum(weights[name] * math.log(ratios[name]["ratio"]) for name in ratios) / total
    variance = 0.0
    for name, ratio in ratios.items():
        spread = math.log(ratio["high"]) - math.log(ratio["low"])
        error = spread / (2 * Z95) if spread > 0 else 0.0
        variance += (weights[name] / total) ** 2 * error**2
    error = math.sqrt(variance)
    return {
        "index": math.exp(log_index),
        "low": math.exp(log_index - Z95 * error),
        "high": math.exp(log_index + Z95 * error),
    }


def project_index(cells: Sequence[Mapping[str, Any]], suite: Mapping[str, Any]) -> List[Dict[str, Any]]:
    """Per platform: the index for every build, over common components and over all."""
    weights = {component["id"]: component["weight"] for component in suite["components"]}
    titles = {component["id"]: component["title"] for component in suite["components"]}
    platforms = sorted({cell["platform"] for cell in cells if cell.get("component")})
    projected = []
    for platform in platforms:
        ratios = component_ratios(cells, suite, platform)
        if not ratios:
            continue
        # The builds, in the order the cells list them (chronological), with the
        # metadata the chart needs, taken from the first cell that names each.
        builds: Dict[str, Dict[str, Any]] = {}
        for cell in cells:
            if cell.get("platform") != platform or not cell.get("component"):
                continue
            for item in cell["milestones"]:
                builds.setdefault(
                    item["label"],
                    {
                        key: item.get(key)
                        for key in ("label", "short", "includes", "after_experiment", "commit", "date")
                    },
                )
        labels = list(builds)
        common = [name for name, per_build in ratios.items() if all(label in per_build for label in labels)]
        rows = []
        for label in labels:
            available = {name: per_build[label] for name, per_build in ratios.items() if label in per_build}
            row = dict(builds[label])
            row["components"] = {name: available[name]["ratio"] for name in available}
            row["coverage"] = round(sum(weights[name] for name in available), 6)
            if common:
                row["common"] = combine({name: available[name] for name in common}, weights)
            if set(available) == set(weights):
                row["full"] = combine(available, weights)
            rows.append(row)
        projected.append(
            {
                "platform": platform,
                "reference_build": suite["reference_build"],
                "manifest_version": suite["version"],
                "measured": sorted(ratios),
                "missing": sorted(set(weights) - set(ratios)),
                "common": common,
                "common_weight": round(sum(weights[name] for name in common), 6),
                "titles": {name: titles[name] for name in ratios},
                "builds": rows,
            }
        )
    return projected
