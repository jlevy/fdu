"""The fdu performance index: one pre-registered score over a fixed suite of components.

The suite, its weights, and the reference build live in
`explorations/benchmarks/index-suite.json`; the design is
`docs/project/specs/active/plan-2026-10-05-fdu-performance-index.md`.

**The index is a weighted sum of log runtime ratios.** For build *b* and component *c*,
the component ratio *r* is *b*'s runtime over the reference build's, taken from the
paired figure of one interleaved session, so no ratio chains two sessions. The index is
`exp(sum(w * ln r) / sum(w))`: a relative runtime, 1.0 at the reference build. Summing
logs keeps a 70-second benchmark from outweighing a 160-millisecond one, and on two
components of equal weight a 2x gain and a 2x loss cancel exactly, which a sum of raw
times or of raw ratios does not.

**Every optimization target is exercised.** Every benchmark job the loop has recorded
maps, in `index-jobs.json`, to a component whose measurement runs that job's code path;
`unmapped_jobs` names any job that does not map, and the projection refuses to build
while one exists, so a new target gets a component and a weight before its first verdict
is published.

**A build is scored only on what it can do, and says so.** A build without a component's
capability (the pre-work binary has no content views) has no ratio there, and every
score carries its coverage, the share of the suite's weight it includes. The full index
exists only for builds with every component; the page draws earlier builds as a partial
score.
"""

from __future__ import annotations

import hashlib
import json
import math
from pathlib import Path
from typing import Any, Dict, Iterable, List, Mapping, Optional, Sequence

SUITE_PATH = Path("explorations/benchmarks/index-suite.json")

#: The two-sided 95% normal quantile, for combining component intervals in log space.
Z95 = 1.959963984540054


def load_suite(path: Path = SUITE_PATH) -> Dict[str, Any]:
    """The manifest with its job table, its weights checked to sum to one.

    The job table is a separate file so that mapping a new job to an existing component
    is not a new index version; adding or reweighting a component is.
    """
    suite = json.loads(path.read_text(encoding="utf-8"))
    jobs_path = path.parent / Path(suite.get("jobs") or "index-jobs.json").name
    suite["job_components"] = json.loads(jobs_path.read_text(encoding="utf-8"))["job_components"]
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


def component_digest(component: Mapping[str, Any]) -> str:
    """A short digest of what a component measures: its tree, command, cache state, jobs.

    A cell records the digest of the definition it was measured under, so a component
    redefined in place cannot silently reuse cells measured under the old definition.
    Weight and title are not part of it: reweighting does not change what a cell measured.
    """
    fields = {key: component.get(key) for key in ("id", "tree", "command", "cache_state", "measures", "jobs")}
    return hashlib.sha256(json.dumps(fields, sort_keys=True).encode("utf-8")).hexdigest()[:16]


def component_jobs(component: Mapping[str, Any]) -> List[str]:
    """The jobs a component times: its `jobs` list, or the component itself."""
    return list(component.get("jobs") or [component["id"]])


def _geometric(ratios: Sequence[Mapping[str, float]]) -> Dict[str, float]:
    """Equally weighted geometric mean of job ratios, with their combined interval."""
    count = len(ratios)
    log_ratio = sum(math.log(ratio["ratio"]) for ratio in ratios) / count
    variance = 0.0
    for ratio in ratios:
        spread = math.log(ratio["high"]) - math.log(ratio["low"])
        variance += (spread / (2 * Z95) / count) ** 2
    error = math.sqrt(variance)
    return {
        "ratio": math.exp(log_ratio),
        "low": math.exp(log_ratio - Z95 * error),
        "high": math.exp(log_ratio + Z95 * error),
    }


def component_ratios(
    cells: Sequence[Mapping[str, Any]], suite: Mapping[str, Any], platform: str
) -> Dict[str, Dict[str, Dict[str, float]]]:
    """`{component: {build label: ratio}}` for one platform, memory included.

    Each cell is one job of one component. A component with several jobs scores a build
    only when every job has a ratio for it, as the geometric mean of those ratios. A cell
    whose recorded definition digest does not match the manifest's is refused.
    """
    reference = suite["reference_build"]
    by_job = cells_by_job(cells, suite, platform)
    ratios: Dict[str, Dict[str, Dict[str, float]]] = {}
    for component in suite["components"]:
        if component["measures"] == "peak_rss":
            continue
        jobs = component_jobs(component)
        job_cells = [by_job.get((component["id"], job)) for job in jobs]
        if not all(job_cells):
            continue
        per_job = [
            {
                item["label"]: ratio
                for item in cell["milestones"]
                if (ratio := _component_ratio(item, reference)) is not None
            }
            for cell in job_cells
        ]
        labels = set.intersection(*(set(job) for job in per_job))
        ratios[component["id"]] = {label: _geometric([job[label] for job in per_job]) for label in labels}
    for component in suite["components"]:
        if component["measures"] != "peak_rss":
            continue
        # The full score's memory reads every measured component; the partial score
        # recomputes it over its own components (`memory_ratios`), so each score line
        # averages one consistent mix.
        memory = memory_ratios(by_job, reference, [name for name in ratios])
        if memory:
            ratios[component["id"]] = memory
    return ratios


def memory_ratios(
    by_job: Mapping[tuple, Mapping[str, Any]], reference: str, components: Sequence[str]
) -> Dict[str, Dict[str, float]]:
    """Peak-RSS ratio to the reference per build, over the given components.

    Each component counts once: its jobs' peak ratios are averaged first, then the
    components' averages. A ratio of medians, not a paired figure, since the harness
    records peak RSS per arm; it carries no interval.
    """
    per_component: Dict[str, Dict[str, float]] = {}
    for name in components:
        cells = [cell for (component, _), cell in by_job.items() if component == name]
        logs_by_label: Dict[str, List[float]] = {}
        for cell in cells:
            peaks = {item["label"]: item.get("peak_rss_mib") for item in cell["milestones"]}
            if not peaks.get(reference):
                continue
            for label, peak in peaks.items():
                if peak:
                    logs_by_label.setdefault(label, []).append(math.log(peak / peaks[reference]))
        per_component[name] = {label: sum(logs) / len(logs) for label, logs in logs_by_label.items()}
    memory: Dict[str, Dict[str, float]] = {}
    labels = {label for values in per_component.values() for label in values}
    for label in labels:
        logs = [values[label] for values in per_component.values() if label in values]
        if logs:
            value = math.exp(sum(logs) / len(logs))
            memory[label] = {"ratio": value, "low": value, "high": value}
    return memory


def cells_by_job(
    cells: Sequence[Mapping[str, Any]], suite: Mapping[str, Any], platform: str
) -> Dict[tuple, Mapping[str, Any]]:
    """One platform's cells keyed by (component, job), each checked against its digest."""
    definitions = {component["id"]: component for component in suite["components"]}
    by_job: Dict[tuple, Mapping[str, Any]] = {}
    for cell in cells:
        if not cell.get("component") or cell.get("platform") != platform:
            continue
        definition = definitions.get(cell["component"])
        if definition is None:
            raise ValueError(f"history cell {cell.get('id')} names unknown component {cell['component']!r}")
        expected = component_digest(definition)
        if cell.get("component_digest") != expected:
            raise ValueError(
                f"history cell {cell.get('id')} was measured under a different definition of "
                f"{cell['component']!r} (digest {cell.get('component_digest')}, manifest {expected}); "
                "re-measure it or version the manifest"
            )
        by_job[(cell["component"], cell.get("job") or cell["component"])] = cell
    return by_job


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


def score_ratio(first: Mapping[str, float], last: Mapping[str, float]) -> tuple:
    """How much better `last` scores than `first`, with a combined 95% interval.

    Each index's interval becomes a log-space standard error, and the two combine as
    independent errors. Dividing the extremes of the two intervals instead would give a
    worst-case bound, wider than a 95% interval.
    """
    errors = []
    for value in (first, last):
        spread = math.log(value["high"]) - math.log(value["low"])
        errors.append(spread / (2 * Z95) if spread > 0 else 0.0)
    log_ratio = math.log(first["index"]) - math.log(last["index"])
    error = math.sqrt(sum(item**2 for item in errors))
    return math.exp(log_ratio), math.exp(log_ratio - Z95 * error), math.exp(log_ratio + Z95 * error)


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
        # The partial score's memory reads only the partial score's own components, so
        # every build on that line averages the same mix.
        common_memory = memory_ratios(
            cells_by_job(cells, suite, platform),
            suite["reference_build"],
            [name for name in common if name != "memory"],
        )
        rows = []
        for label in labels:
            available = {name: per_build[label] for name, per_build in ratios.items() if label in per_build}
            row = dict(builds[label])
            row["components"] = {name: available[name]["ratio"] for name in available}
            row["coverage"] = round(sum(weights[name] for name in available), 6)
            if common:
                partial = {name: available[name] for name in common}
                if "memory" in partial and label in common_memory:
                    partial["memory"] = common_memory[label]
                row["common"] = combine(partial, weights)
            # Every component measured so far; the same as the full index once the whole
            # suite has been measured on this platform.
            if set(available) == set(ratios):
                row["measured_full"] = combine(available, weights)
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
