"""Build the data projection for the interactive performance research report.

The experiment frontmatter remains authoritative. This module validates each artifact
through the same Pydantic contract as the ledger, then emits the smaller browser-facing
projection used by the static report. The Markdown bodies are never parsed for values.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
from collections import Counter
from pathlib import Path
from typing import Any

from frontmatter_format import fmf_read_frontmatter
from strif import atomic_output_file

from benchmarks.realtree import experiment as experiment_model
from benchmarks.realtree import softschema_table

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_EXPERIMENTS_DIRECTORY = REPOSITORY_ROOT / "docs/project/experiments"
DEFAULT_OUTPUT_PATH = (
    REPOSITORY_ROOT / "docs/project/reports/performance-research/report-data.js"
)
REPORT_DATE = "2026-08-14"

PHASES = (
    {
        "id": "baseline",
        "label": "Baseline and harness",
        "first": 0,
        "last": 0,
        "summary": "Establish the real-tree baseline, correctness oracle, and paired schedule.",
    },
    {
        "id": "parallel-index",
        "label": "Parallel scan and index constants",
        "first": 1,
        "last": 11,
        "summary": "Overlap metadata latency, then remove repeated path and roll-up work.",
    },
    {
        "id": "scheduler",
        "label": "Traversal and adaptive concurrency",
        "first": 12,
        "last": 21,
        "summary": "Preserve useful breadth-first progress while tuning concurrency to service time.",
    },
    {
        "id": "macos-bulk",
        "label": "macOS bulk metadata and reconciliation",
        "first": 22,
        "last": 32,
        "summary": "Remove kernel transitions and discard warm no-ops before the serial mutation boundary.",
    },
    {
        "id": "integration",
        "label": "Integration and scale validation",
        "first": 33,
        "last": 39,
        "summary": "Rebuild controls after the CLI merge and retest the operating point near one million entries.",
    },
    {
        "id": "tiers-content",
        "label": "Retained-state tiers and content",
        "first": 40,
        "last": 50,
        "summary": "Avoid retaining state a one-shot request cannot use, then optimize content decoding.",
    },
    {
        "id": "linux-instrumentation",
        "label": "Linux structural fixes and instrumentation",
        "first": 51,
        "last": 55,
        "summary": "Attack repeated parent resolution, measure the instrument, and validate transfer back to macOS.",
    },
)

CURRENT_PLATFORM_OUTCOMES = (
    {
        "platform": "Linux",
        "job": "cold-scan-index",
        "label": "Cold indexed scan",
        "control_ms": 2107.8,
        "candidate_ms": 1909.2,
        "change_pct": -9.1,
        "ci95_pct": [-13.2, -7.6],
        "source": "../report-2026-08-14-performance-campaign-status.md",
    },
    {
        "platform": "Linux",
        "job": "warm-revalidate",
        "label": "Warm revalidation",
        "control_ms": 2317.6,
        "candidate_ms": 1726.6,
        "change_pct": -25.3,
        "ci95_pct": [-26.4, -23.8],
        "source": "../report-2026-08-14-performance-campaign-status.md",
    },
    {
        "platform": "Linux",
        "job": "warm-snapshot-load",
        "label": "Warm snapshot load",
        "control_ms": 1897.2,
        "candidate_ms": 1303.8,
        "change_pct": -31.4,
        "ci95_pct": [-32.0, -30.8],
        "source": "../report-2026-08-14-performance-campaign-status.md",
    },
    {
        "platform": "Linux",
        "job": "cold-scan-producer",
        "label": "Cold scan producer",
        "control_ms": 2381.3,
        "candidate_ms": 2200.2,
        "change_pct": -7.3,
        "ci95_pct": [-8.9, -6.1],
        "source": "../report-2026-08-14-performance-campaign-status.md",
        "caveat": "The component was unchanged; treat the wall movement as environmental rather than a producer win.",
    },
    {
        "platform": "macOS",
        "job": "cold-scan-index",
        "label": "Cold indexed scan",
        "control_ms": 298.127,
        "candidate_ms": 306.243,
        "change_pct": 1.393,
        "ci95_pct": [-0.186, 3.895],
        "source": "../../experiments/exp-054-validate-the-linux-campaign-cumulative-effect-on-macos.md",
        "caveat": "The interval crosses zero; this is neutral evidence, not a measured regression.",
    },
    {
        "platform": "macOS",
        "job": "warm-revalidate",
        "label": "Warm revalidation",
        "control_ms": 392.991,
        "candidate_ms": 335.747,
        "change_pct": -15.682,
        "ci95_pct": [-16.286, -13.993],
        "source": "../../experiments/exp-054-validate-the-linux-campaign-cumulative-effect-on-macos.md",
    },
)

COMPOSITE_CELLS = (
    {"platform": "Linux", "job": "cold-scan-index", "change_pct": -9.1},
    {"platform": "Linux", "job": "warm-revalidate", "change_pct": -25.3},
    {"platform": "macOS", "job": "cold-scan-index", "change_pct": 1.393},
    {"platform": "macOS", "job": "warm-revalidate", "change_pct": -15.682},
)

CHECKPOINT_PROFILES = {
    "index-core-v1": {
        "label": "Index core",
        "description": (
            "The kept build after each index experiment, measured across product "
            "latency, CPU, and memory dimensions."
        ),
        "dimensions": (
            {
                "id": "cold-index-wall",
                "label": "Cold indexed scan",
                "job": "cold-scan-index",
                "metric": "wall_ns",
            },
            {
                "id": "warm-revalidate-wall",
                "label": "Warm revalidation",
                "job": "warm-revalidate",
                "metric": "wall_ns",
            },
            {
                "id": "snapshot-load-wall",
                "label": "Warm snapshot load",
                "job": "warm-snapshot-load",
                "metric": "wall_ns",
            },
            {
                "id": "cold-index-cpu",
                "label": "Cold indexed CPU",
                "job": "cold-scan-index",
                "metric": "cpu_ns",
            },
            {
                "id": "cold-index-rss",
                "label": "Cold indexed peak RSS",
                "job": "cold-scan-index",
                "metric": "peak_rss_bytes",
            },
        ),
    },
}

SUPPORTING_STUDIES = (
    {
        "id": "linux-noop-save",
        "title": "Skip a byte-identical snapshot rewrite",
        "platform": "Linux",
        "tier": "index",
        "evidence": "paired study",
        "status": "shipped",
        "primary_change_pct": -20.6,
        "ci95_pct": [-21.2, -16.6],
        "headline": "Warm tree wall fell 20.6% and peak RSS fell from 411.2 to 194.7 MB.",
        "lesson": "Deleting an unconditional clone and rewrite removed both latency and the warm path's memory spike.",
        "source": "../../research/research-2026-08-13-linux-three-tier-baseline.md#first-fix-the-byte-identical-rewrite",
    },
    {
        "id": "linux-known-parent-load",
        "title": "Load snapshot children beneath the parent already held",
        "platform": "Linux",
        "tier": "index",
        "evidence": "paired study",
        "status": "shipped",
        "primary_change_pct": -51.9,
        "ci95_pct": [-53.2, -51.0],
        "headline": "Snapshot load fell 51.9%; the complete warm open fell 41.9%.",
        "lesson": "The loader was paying path construction, normalization, and root descent to rediscover an EntryId already in a local variable.",
        "source": "../../guides/performance-loop.md#hypotheses",
    },
    {
        "id": "content-cache-hash",
        "title": "Use hash lookup for content-cache candidates",
        "platform": "Linux",
        "tier": "content",
        "evidence": "paired study",
        "status": "shipped",
        "primary_change_pct": -3.48,
        "ci95_pct": [-5.41, -1.27],
        "headline": "Warm content open improved 3.48%; cache-only improved 3.03%.",
        "lesson": "The caller tree corrected a flat-profile estimate by an order of magnitude: the map was real but only 0.9% of instructions.",
        "source": "../../guides/performance-instrumentation-playbook.md#things-that-will-bite-you",
    },
    {
        "id": "mimalloc-tier-split",
        "title": "Replace the global allocator with mimalloc",
        "platform": "Linux",
        "tier": "aggregate",
        "evidence": "scouting study",
        "status": "not adopted",
        "primary_change_pct": -23.03,
        "ci95_pct": [-28.36, -16.72],
        "headline": "Aggregate wall improved 23.0%, but peak RSS rose 139%; index and snapshot-load intervals crossed zero.",
        "lesson": "Allocator choice exposed a cross-thread-free pattern, not a universal allocator win. The dependency and memory cost failed the product trade.",
        "source": "../../guides/performance-loop.md#hypotheses",
    },
    {
        "id": "linux-io-uring",
        "title": "Batch statx through io_uring",
        "platform": "Linux",
        "tier": "aggregate",
        "evidence": "scouting study",
        "status": "rejected",
        "primary_change_pct": 327.0,
        "ci95_pct": [309.0, 345.0],
        "headline": "Warm wall regressed 327%; the virtualized cold study regressed 77.6%.",
        "lesson": "Linux already converged on getdents64 plus dirfd-relative statx; rearranging the same kernel work added queue and io-wq overhead.",
        "source": "../../research/research-2026-08-13-linux-first-measurements.md#cold-results",
    },
)

MECHANISM_GROUPS = (
    {
        "title": "Overlap unavoidable latency",
        "summary": "Bounded concurrency paid while the producer waited on metadata, then adaptive limits prevented the same threads becoming contention.",
        "experiment_ids": ["exp-001", "exp-021", "exp-025", "exp-036"],
    },
    {
        "title": "Remove kernel transitions",
        "summary": "macOS bulk attributes changed the syscall shape rather than shaving a user-space constant.",
        "experiment_ids": ["exp-022", "exp-026", "exp-029", "exp-039"],
    },
    {
        "title": "Preserve identity across boundaries",
        "summary": "The strongest portable wins stopped rebuilding paths or rediscovering parents already known to the caller.",
        "experiment_ids": ["exp-004", "exp-005", "exp-007", "exp-051"],
    },
    {
        "title": "Move work before serialization",
        "summary": "Parallel reconciliation mattered only after workers proved no-ops and kept them out of the single mutation authority.",
        "experiment_ids": ["exp-002", "exp-030"],
    },
    {
        "title": "Retain only what the request can consume",
        "summary": "A one-shot summary can avoid a reusable index, but progressively narrower summary representations no longer moved the filesystem floor.",
        "experiment_ids": ["exp-040", "exp-041", "exp-042", "exp-043", "exp-044"],
    },
    {
        "title": "Optimize content at its real boundary",
        "summary": "Inline analysis and speculative reserves failed; eliminating an extra UTF-8 copy paid.",
        "experiment_ids": ["exp-047", "exp-048", "exp-049", "exp-050"],
    },
)

SOURCE_DOCUMENTS = (
    {
        "label": "Campaign status",
        "href": "../report-2026-08-14-performance-campaign-status.md",
        "role": "Current outcomes, method, limitations, and open work",
    },
    {
        "label": "Experiment ledger",
        "href": "../report-2026-08-10-fdu-performance-experiments.md",
        "role": "Generated roll-up of all 56 enforced experiment artifacts",
    },
    {
        "label": "Performance loop",
        "href": "../../guides/performance-loop.md",
        "role": "Protocol, accept rule, and live hypothesis registry",
    },
    {
        "label": "Instrumentation playbook",
        "href": "../../guides/performance-instrumentation-playbook.md",
        "role": "Reusable three-tier measurement method",
    },
    {
        "label": "Performance architecture",
        "href": "../report-2026-08-12-fdu-performance-architecture.md",
        "role": "Cost model and architectural synthesis",
    },
    {
        "label": "Platform tuning",
        "href": "../../guides/platform-tuning.md",
        "role": "Evidence provenance for shipped constants",
    },
    {
        "label": "Linux first measurements",
        "href": "../../research/research-2026-08-13-linux-first-measurements.md",
        "role": "Off-ledger syscall and product scouting",
    },
    {
        "label": "Linux three-tier baseline",
        "href": "../../research/research-2026-08-13-linux-three-tier-baseline.md",
        "role": "Aggregate, index, and content retention tiers",
    },
)

EXPERIMENT_TABLE_COLUMNS = (
    softschema_table.ColumnSpec("id", "Experiment", "identifier"),
    softschema_table.ColumnSpec("title", "Candidate", "text"),
    softschema_table.ColumnSpec("date", "Date", "date"),
    softschema_table.ColumnSpec("hypotheses", "Hypotheses", "tags"),
    softschema_table.ColumnSpec("subject.host_system", "Host", "platform"),
    softschema_table.ColumnSpec("subject.tree_entries", "Entries", "integer"),
    softschema_table.ColumnSpec("method.trials", "Pairs", "integer"),
    softschema_table.ColumnSpec("verdict.decision", "Decision", "status"),
    softschema_table.ColumnSpec("verdict.primary_job", "Primary job", "text"),
    softschema_table.ColumnSpec("verdict.change_pct", "Change", "percent"),
    softschema_table.ColumnSpec("complexity.lines_changed", "Lines", "integer"),
)


def _phase_for(experiment_number: int) -> str:
    for phase in PHASES:
        if phase["first"] <= experiment_number <= phase["last"]:
            return str(phase["id"])
    raise ValueError(
        f"experiment number {experiment_number} is outside the report phases"
    )


def _platform(host_system: str) -> str:
    if host_system.startswith("Darwin"):
        return "macOS"
    if host_system.startswith("Linux"):
        return "Linux"
    return host_system or "Unknown"


def _metric_effect(metric: experiment_model.MetricChange | None) -> str:
    if metric is None:
        return "unknown"
    if metric.direction != "unknown":
        return metric.direction
    if metric.ci95_high_pct is not None and metric.ci95_high_pct < 0:
        return "improved"
    if metric.ci95_low_pct is not None and metric.ci95_low_pct > 0:
        return "regressed"
    if metric.ci95_low_pct is not None and metric.ci95_high_pct is not None:
        return "unclear"
    return "unknown"


def _serialize_metric(metric: experiment_model.MetricChange) -> dict[str, Any]:
    values = metric.model_dump(mode="json")
    values["effect"] = _metric_effect(metric)
    return values


def _checkpoint_regime(experiment: experiment_model.Experiment) -> dict[str, Any]:
    """Return the exact subject fields that make two absolute values comparable."""
    subject = experiment.subject
    values = {
        "tree_engine_digest": subject.tree_engine_digest,
        "tree_entries": subject.tree_entries,
        "host_cpu": subject.host_cpu,
        "host_arch": subject.host_arch,
        "host_cores": subject.host_cores,
        "host_memory_bytes": subject.host_memory_bytes,
        "host_system": subject.host_system,
        "filesystem": subject.filesystem,
        "host_virtualization": subject.host_virtualization,
        "os_cache": subject.os_cache,
    }
    encoded = json.dumps(values, sort_keys=True, separators=(",", ":")).encode()
    return {"id": hashlib.sha256(encoded).hexdigest()[:12], **values}


def _serialize_checkpoint(
    experiment: experiment_model.Experiment,
) -> dict[str, Any] | None:
    checkpoint = experiment.checkpoint
    if checkpoint is None:
        return None
    profile = CHECKPOINT_PROFILES.get(checkpoint.profile)
    if profile is None:
        raise ValueError(
            f"{experiment.id}: unsupported checkpoint profile {checkpoint.profile!r}"
        )

    results = {result.job: result for result in experiment.results}
    measurements = []
    for dimension in profile["dimensions"]:
        result = results.get(str(dimension["job"]))
        metric = (
            result.metrics.get(str(dimension["metric"])) if result is not None else None
        )
        if metric is None:
            continue
        value = (
            metric.control_median
            if checkpoint.kept_variant == "control"
            else metric.candidate_median
        )
        measurements.append(
            {
                **dimension,
                "value": value,
                "paired_change_pct": metric.change_pct,
                "ci95_pct": [metric.ci95_low_pct, metric.ci95_high_pct],
                "effect": (
                    "retained"
                    if checkpoint.kept_variant == "control"
                    else _metric_effect(metric)
                ),
                "pairs": metric.pairs,
            }
        )

    expected = len(profile["dimensions"])
    return {
        "profile": checkpoint.profile,
        "profile_label": profile["label"],
        "kept_variant": checkpoint.kept_variant,
        "source_revision": checkpoint.source_revision or None,
        "coverage": {
            "measured": len(measurements),
            "expected": expected,
            "complete": len(measurements) == expected,
        },
        "regime": _checkpoint_regime(experiment),
        "measurements": measurements,
    }


def _serialize_experiment(
    path: Path, experiment: experiment_model.Experiment
) -> dict[str, Any]:
    number = int(experiment.id.removeprefix("exp-"))
    primary_result = next(
        (
            result
            for result in experiment.results
            if result.job == experiment.verdict.primary_job
        ),
        None,
    )
    primary_metric = None
    if primary_result is not None:
        primary_metric = primary_result.metrics.get(experiment.verdict.primary_metric)

    primary = {
        "job": experiment.verdict.primary_job,
        "metric": experiment.verdict.primary_metric,
        "change_pct": experiment.verdict.change_pct,
        "control_median": None,
        "candidate_median": None,
        "ci95_pct": None,
        "pairs": 0,
        "passes_acceptance": False,
    }
    if primary_metric is not None:
        primary.update(
            {
                "change_pct": primary_metric.change_pct,
                "control_median": primary_metric.control_median,
                "candidate_median": primary_metric.candidate_median,
                "ci95_pct": [primary_metric.ci95_low_pct, primary_metric.ci95_high_pct],
                "pairs": primary_metric.pairs,
                "passes_acceptance": primary_metric.passes_acceptance
                or primary_metric.significant,
            }
        )

    jobs = []
    for result in experiment.results:
        jobs.append(
            {
                "job": result.job,
                "start_state": result.start_state,
                "invalid_samples": result.invalid_samples,
                "metrics": {
                    name: _serialize_metric(metric)
                    for name, metric in sorted(result.metrics.items())
                },
            }
        )

    return {
        "id": experiment.id,
        "number": number,
        "title": experiment.title,
        "date": experiment.date,
        "phase": _phase_for(number),
        "platform": _platform(experiment.subject.host_system),
        "effect": _metric_effect(primary_metric),
        "hypotheses": experiment.hypotheses,
        "checkpoint": _serialize_checkpoint(experiment),
        "source": f"../../experiments/{path.name}",
        "subject": experiment.subject.model_dump(mode="json"),
        "method": experiment.method.model_dump(mode="json"),
        "jobs": jobs,
        "primary": primary,
        "verdict": experiment.verdict.model_dump(mode="json"),
        "complexity": experiment.complexity.model_dump(mode="json"),
        "reference_tools": [
            tool.model_dump(mode="json") for tool in experiment.reference_tools
        ],
    }


def _load_experiments(experiments_directory: Path) -> list[dict[str, Any]]:
    experiments = []
    seen_ids: set[str] = set()
    for path in sorted(experiments_directory.glob("exp-*.md")):
        frontmatter = fmf_read_frontmatter(path)
        metadata = frontmatter.get("softschema") or {}
        if metadata.get("contract") != experiment_model.CONTRACT:
            raise ValueError(
                f"{path}: expected contract {experiment_model.CONTRACT!r}, "
                f"found {metadata.get('contract')!r}"
            )
        payload = frontmatter.get("experiment")
        if payload is None:
            raise ValueError(f"{path}: missing experiment envelope")
        experiment = experiment_model.Experiment.model_validate(payload)
        if experiment.id in seen_ids:
            raise ValueError(f"duplicate experiment id {experiment.id!r}")
        seen_ids.add(experiment.id)
        experiments.append(_serialize_experiment(path, experiment))
    if not experiments:
        raise ValueError(f"no experiment artifacts found in {experiments_directory}")
    experiments.sort(key=lambda item: item["number"])
    return experiments


def _default_composite_latency_reduction() -> float:
    equal_weight = 1.0 / len(COMPOSITE_CELLS)
    weighted_log_ratio = sum(
        equal_weight * math.log1p(float(cell["change_pct"]) / 100.0)
        for cell in COMPOSITE_CELLS
    )
    return (1.0 - math.exp(weighted_log_ratio)) * 100.0


def build_report_data(
    experiments_directory: Path = DEFAULT_EXPERIMENTS_DIRECTORY,
) -> dict[str, Any]:
    """Validate the experiment artifacts and build the browser-facing projection."""
    experiments = _load_experiments(experiments_directory)
    decision_counts = Counter(item["verdict"]["decision"] for item in experiments)
    platform_counts = Counter(item["platform"] for item in experiments)
    equal_weight = 1.0 / len(COMPOSITE_CELLS)
    composite_cells = [
        dict(cell, default_weight=equal_weight) for cell in COMPOSITE_CELLS
    ]
    table_projection = softschema_table.project_directory(
        experiments_directory,
        columns=EXPERIMENT_TABLE_COLUMNS,
        pattern="exp-*.md",
        source_href_prefix="../../experiments",
    )
    checkpoints = [
        {
            "experiment_id": item["id"],
            "number": item["number"],
            "title": item["title"],
            "date": item["date"],
            "platform": item["platform"],
            "decision": item["verdict"]["decision"],
            "source": item["source"],
            **item["checkpoint"],
        }
        for item in experiments
        if item["checkpoint"] is not None
    ]
    checkpoint_platform_counts = Counter(item["platform"] for item in checkpoints)
    checkpoint_dimension_counts = Counter(
        (item["platform"], measurement["id"])
        for item in checkpoints
        for measurement in item["measurements"]
    )
    checkpoint_profiles = {
        name: {
            "label": profile["label"],
            "description": profile["description"],
            "dimensions": list(profile["dimensions"]),
        }
        for name, profile in CHECKPOINT_PROFILES.items()
    }

    return {
        "report": {
            "title": "The fdu Performance Research Loop",
            "subtitle": "How 56 experiments turned profiles into shipped gains, reusable failures, and platform-specific evidence",
            "date": REPORT_DATE,
            "contract": experiment_model.CONTRACT,
        },
        "summary": {
            "experiment_count": len(experiments),
            "decision_counts": dict(sorted(decision_counts.items())),
            "platform_counts": dict(sorted(platform_counts.items())),
        },
        "phases": list(PHASES),
        "experiments": experiments,
        "checkpoint_history": {
            "profiles": checkpoint_profiles,
            "checkpoints": checkpoints,
            "summary": {
                "checkpoint_count": len(checkpoints),
                "platform_counts": dict(sorted(checkpoint_platform_counts.items())),
                "source_revision_count": sum(
                    bool(item["source_revision"]) for item in checkpoints
                ),
                "dimension_counts": {
                    f"{platform}:{dimension}": count
                    for (platform, dimension), count in sorted(
                        checkpoint_dimension_counts.items()
                    )
                },
            },
            "caveat": (
                "A point is the measured arm retained after that experiment. Lines "
                "join only identical host, tree, filesystem, cache, workload, and "
                "metric regimes. Missing cells were not measured; source revisions "
                "are shown only when the historical artifact recorded them."
            ),
        },
        "current_outcomes": {
            "measurements": list(CURRENT_PLATFORM_OUTCOMES),
            "composite": {
                "kind": "derived",
                "method": "weighted_geometric_latency_ratio",
                "default_latency_reduction_pct": _default_composite_latency_reduction(),
                "cells": composite_cells,
                "caveat": "A transparent decision index over unlike platform runs, not a measured benchmark. Default weights are equal across platform and cold/warm work.",
            },
        },
        "supporting_studies": list(SUPPORTING_STUDIES),
        "mechanism_groups": list(MECHANISM_GROUPS),
        "softschema_table": table_projection,
        "sources": list(SOURCE_DOCUMENTS),
    }


def render_report_data_js(data: dict[str, Any]) -> str:
    """Render one deterministic JavaScript assignment usable from a local file URL."""
    payload = json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True)
    return f"window.FDU_PERFORMANCE_RESEARCH = {payload};\n"


def write_report_data(
    output_path: Path = DEFAULT_OUTPUT_PATH,
    experiments_directory: Path = DEFAULT_EXPERIMENTS_DIRECTORY,
) -> None:
    """Atomically replace the generated report data."""
    rendered = render_report_data_js(build_report_data(experiments_directory))
    with atomic_output_file(output_path, make_parents=True) as temporary_path:
        temporary_path.write_text(rendered, encoding="utf-8")


def output_is_current(
    output_path: Path = DEFAULT_OUTPUT_PATH,
    experiments_directory: Path = DEFAULT_EXPERIMENTS_DIRECTORY,
) -> bool:
    """Return whether the committed browser projection matches the artifacts."""
    if not output_path.is_file():
        return False
    expected = render_report_data_js(build_report_data(experiments_directory))
    return output_path.read_text(encoding="utf-8") == expected


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Generate the data file for the interactive performance research report."
    )
    parser.add_argument(
        "--experiments",
        type=Path,
        default=DEFAULT_EXPERIMENTS_DIRECTORY,
        help="Directory holding exp-*.md soft-schema artifacts.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=DEFAULT_OUTPUT_PATH,
        help="JavaScript data file to write or check.",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Fail instead of writing when the generated data has drifted.",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    """Generate or check the report data file."""
    arguments = _parser().parse_args(argv)
    try:
        if arguments.check:
            if output_is_current(arguments.output, arguments.experiments):
                print(f"report data current: {arguments.output}")
                return 0
            print(f"report data drift: regenerate {arguments.output}", file=sys.stderr)
            return 1
        write_report_data(arguments.output, arguments.experiments)
        print(f"wrote report data: {arguments.output}")
        return 0
    except (OSError, TypeError, ValueError) as error:
        print(f"performance research report: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
