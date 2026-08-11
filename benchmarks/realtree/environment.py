"""Capture benchmark environments and compare decisions across equivalent cells.

Absolute timings from unlike machines are not comparable. This module instead gives
each run a path-free environment identity, binds generated runs to a portable corpus
identity, and recomputes the same latency and resource gates independently per cell.
"""

from __future__ import annotations

import hashlib
import json
import os
import platform
import plistlib
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Literal, Mapping, Optional, Sequence

from pydantic import BaseModel, ConfigDict, Field

from benchmarks import corpus
from benchmarks.realtree import decision

RUN_SCHEMA = "fdu-realtree-run-v3"
ENVIRONMENT_SCHEMA = "fdu-benchmark-environment-v1"
WORKLOAD_SCHEMA = "fdu-benchmark-workload-v1"
MATRIX_SCHEMA = "fdu-environment-decision-matrix-v1"
MATRIX_CONTRACT = "fdu.performance:EnvironmentMatrix/v1"

RUNNER_CLASSES = (
    "local-uncontrolled",
    "github-hosted-shared",
    "self-hosted-controlled",
)
_EVIDENCE_GRADES = {
    "local-uncontrolled": "local-uncontrolled",
    "github-hosted-shared": "shared-cloud-exploratory",
    "self-hosted-controlled": "controlled",
}
_IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9._-]{0,79}$")
_SOURCE_FIELDS = (
    "args",
    "engine_revision",
    "harness_revision",
    "harness_sha256",
    "build_profile",
    "features",
    "build_command",
)
_MATRIX_METRICS = ("wall_ns", "component_ns", "cpu_ns", "peak_rss_bytes")


class EnvironmentError(RuntimeError):
    """A run cannot support a cross-environment decision."""


def validate_identifier(value: str, *, field: str) -> str:
    """Validate a path-free stable identifier used in archived evidence."""
    if not _IDENTIFIER.fullmatch(value):
        raise EnvironmentError(
            f"{field} must match {_IDENTIFIER.pattern!r} and contain at most 80 characters"
        )
    return value


def host_facts(root: Optional[Path] = None) -> Dict[str, Any]:
    """Return timing-relevant machine facts without hostname, user, or paths."""
    measured_root = (root or Path.cwd()).resolve()
    facts: Dict[str, Any] = {
        "arch": platform.machine(),
        "cpu_count": os.cpu_count(),
        "python": platform.python_version(),
        "system": platform.system(),
        "release": platform.release(),
        "toolchain": _toolchain(),
        "filesystem": _filesystem_for_path(measured_root),
    }
    if sys.platform == "darwin":
        facts["cpu_model"] = _sysctl("machdep.cpu.brand_string")
        facts["memory_bytes"] = _sysctl_int("hw.memsize")
        facts["performance_cores"] = _sysctl_int("hw.perflevel0.logicalcpu")
        facts["efficiency_cores"] = _sysctl_int("hw.perflevel1.logicalcpu")
    elif sys.platform.startswith("linux"):
        facts["cpu_model"] = _linux_cpu_model()
        facts["memory_bytes"] = _linux_memory_bytes()
    return facts


def environment_identity(
    host: Mapping[str, Any],
    *,
    cell_id: Optional[str] = None,
    runner_class: Optional[str] = None,
    environ: Optional[Mapping[str, str]] = None,
) -> Dict[str, Any]:
    """Create a logical cell plus an exact fingerprint of this runner invocation."""
    variables = environ if environ is not None else os.environ
    selected_runner = runner_class or (
        "github-hosted-shared"
        if variables.get("GITHUB_ACTIONS") == "true"
        else "local-uncontrolled"
    )
    if selected_runner not in RUNNER_CLASSES:
        raise EnvironmentError(
            "runner_class must be one of " + ", ".join(RUNNER_CLASSES)
        )
    runner = {
        "provider": "github-actions" if variables.get("GITHUB_ACTIONS") == "true" else "local",
        "image_os": _safe_runner_value(variables.get("ImageOS")),
        "image_version": _safe_runner_value(variables.get("ImageVersion")),
        "runner_os": _safe_runner_value(variables.get("RUNNER_OS")),
        "runner_arch": _safe_runner_value(variables.get("RUNNER_ARCH")),
    }
    logical_id = cell_id or _slug(
        "-".join(
            str(value)
            for value in (
                runner["provider"],
                host.get("system"),
                host.get("filesystem") or "unknown-fs",
                host.get("arch"),
            )
            if value
        )
    )
    validate_identifier(logical_id, field="environment cell")
    fingerprint = _sha256_json({"host": dict(host), "runner": runner})
    return {
        "schema": ENVIRONMENT_SCHEMA,
        "cell_id": logical_id,
        "runner_class": selected_runner,
        "evidence_grade": _EVIDENCE_GRADES[selected_runner],
        "runner": runner,
        "fingerprint_sha256": fingerprint,
    }


def workload_identity(
    root: Path, manifest_path: Optional[Path]
) -> Dict[str, Any]:
    """Bind a run either to a verified portable corpus or to a local real tree."""
    if manifest_path is None:
        return {
            "schema": WORKLOAD_SCHEMA,
            "kind": "real-tree",
            "portable": False,
            "portable_identity_sha256": None,
        }
    resolved_manifest = manifest_path.resolve(strict=True)
    if resolved_manifest.name != corpus.MANIFEST_NAME:
        raise EnvironmentError(
            f"corpus manifest must be named {corpus.MANIFEST_NAME!r}"
        )
    try:
        manifest = corpus.verify_corpus(resolved_manifest.parent)
    except (corpus.CorpusError, OSError) as error:
        raise EnvironmentError(f"corpus manifest did not verify: {error}") from error
    expected_root = (resolved_manifest.parent / corpus.CORPUS_NAME).resolve(strict=True)
    if root.resolve(strict=True) != expected_root:
        raise EnvironmentError("measured root is not the verified manifest's corpus")
    return _portable_workload_from_manifest(manifest)


def _portable_workload_from_manifest(manifest: Mapping[str, Any]) -> Dict[str, Any]:
    """Select only cross-filesystem corpus fields for the portable identity."""
    recipe = manifest.get("recipe") or {}
    state = manifest.get("state") or {}
    portable = {
        "manifest_schema": manifest.get("schema"),
        "generator_source_hash": manifest.get("generator_source_hash"),
        "generator_version": manifest.get("generator_version"),
        "recipe_id": manifest.get("recipe_id"),
        "recipe_hash": manifest.get("recipe_hash"),
        "seed": recipe.get("seed"),
        "target_entries": recipe.get("target_entries"),
        "semantic_digest": manifest.get("semantic_digest"),
        "counts": dict(manifest.get("counts") or {}),
        "state": dict(state),
    }
    required = (
        "manifest_schema",
        "generator_source_hash",
        "generator_version",
        "recipe_id",
        "recipe_hash",
        "seed",
        "target_entries",
        "semantic_digest",
    )
    missing = [field for field in required if portable.get(field) in {None, ""}]
    if missing:
        raise EnvironmentError(
            "generated workload lacks portable identity fields: " + ", ".join(missing)
        )
    return {
        "schema": WORKLOAD_SCHEMA,
        "kind": "generated-corpus",
        "portable": True,
        "portable_identity_sha256": _sha256_json(portable),
        "local_manifest_hash": manifest.get("manifest_hash"),
        **portable,
    }


class Strict(BaseModel):
    model_config = ConfigDict(extra="forbid")


class WorkloadCounts(Strict):
    total: int = Field(ge=0)
    directories: int = Field(ge=0)
    files: int = Field(ge=0)
    symlinks: int = Field(default=0, ge=0)
    other: int = Field(default=0, ge=0)
    hardlink_groups: int = Field(default=0, ge=0)


class MatrixWorkload(Strict):
    portable_identity_sha256: str = Field(pattern=r"^[0-9a-f]{64}$")
    generator_source_hash: str = Field(pattern=r"^[0-9a-f]{64}$")
    generator_version: int = Field(ge=1)
    recipe_id: str
    recipe_hash: str = Field(pattern=r"^[0-9a-f]{64}$")
    seed: str
    target_entries: int = Field(ge=1)
    semantic_digest: str = Field(pattern=r"^[0-9a-f]{64}$")
    counts: WorkloadCounts
    state: Dict[str, Any]


class VariantSource(Strict):
    name: str
    args: List[str]
    engine_revision: str = Field(pattern=r"^[0-9a-f]{40}$")
    harness_revision: str = Field(pattern=r"^[0-9a-f]{40}$")
    harness_sha256: str = Field(pattern=r"^[0-9a-f]{64}$")
    build_profile: str
    features: List[str]
    build_command: str


class MatrixComparison(Strict):
    control: VariantSource
    candidate: VariantSource
    jobs: List[str]
    trials: int = Field(ge=1)
    warmups: int = Field(ge=0)
    os_cache: str
    schedule: str
    schedule_sha256: str = Field(pattern=r"^[0-9a-f]{64}$")


class MatrixCell(Strict):
    id: str
    runner_class: Literal[
        "local-uncontrolled", "github-hosted-shared", "self-hosted-controlled"
    ]
    evidence_grade: Literal[
        "local-uncontrolled", "shared-cloud-exploratory", "controlled"
    ]
    environment_fingerprint: str = Field(pattern=r"^[0-9a-f]{64}$")
    system: str
    release: str
    arch: str
    cpu_model: str
    logical_cores: int = Field(ge=0)
    memory_bytes: int = Field(ge=0)
    filesystem: str
    toolchain: str
    control_target: str
    candidate_target: str
    tree_engine_digest: str = Field(pattern=r"^[0-9a-f]{64}$")
    run_artifact: str
    run_artifact_sha256: str = Field(pattern=r"^[0-9a-f]{64}$")


class MetricObservation(Strict):
    control_median: float
    candidate_median: float
    change_pct: float
    ci95_low_pct: Optional[float] = None
    ci95_high_pct: Optional[float] = None
    pairs: int = Field(ge=0)
    direction: Literal["improvement", "regression", "unchanged"]


class ResourceGate(Strict):
    metric: Literal["cpu_ns", "peak_rss_bytes"]
    maximum_regression_pct: float = Field(ge=0)
    observed_change_pct: Optional[float] = None
    ci95_low_pct: Optional[float] = None
    ci95_high_pct: Optional[float] = None
    status: Literal["passed", "failed", "not-measured"]
    reason: str


class CellDecision(Strict):
    valid: bool
    invalid_samples: int = Field(ge=0)
    metrics: Dict[str, MetricObservation]
    latency_gate_passed: bool
    latency_reason: str
    resource_guardrails: List[ResourceGate]
    overall_decision: Literal["accepted", "rejected", "invalid"]
    reason: str


class JobDecision(Strict):
    job: str
    cells: Dict[str, CellDecision]
    divergent_gates: List[str]
    decision_consistent: bool


class EnvironmentMatrix(Strict):
    """A portable workload with decisions evaluated independently in every cell."""

    format: Literal["fdu-environment-decision-matrix-v1"] = Field(
        default=MATRIX_SCHEMA, alias="schema"
    )
    id: str = Field(pattern=r"^env-\d{3}$")
    date: str = Field(pattern=r"^\d{4}-\d{2}-\d{2}$")
    run_group: str
    workload: MatrixWorkload
    comparison: MatrixComparison
    cells: List[MatrixCell]
    jobs: List[JobDecision]
    all_cells_valid: bool
    divergent_jobs: List[str]
    decision_consistent: bool


@dataclass(frozen=True)
class RunEvidence:
    """One raw run plus the path-free identity of its archived file."""

    document: Mapping[str, Any]
    artifact_name: str
    artifact_sha256: str


def build_matrix(
    runs: Sequence[RunEvidence],
    *,
    matrix_id: str,
    control_variant: str,
    candidate_variant: str,
    maximum_cpu_regression_pct: float = 10.0,
    maximum_rss_regression_pct: float = 10.0,
) -> EnvironmentMatrix:
    """Validate equivalent inputs and compute every gate independently per cell."""
    if not runs:
        raise EnvironmentError("an environment matrix needs at least one run")
    if not re.fullmatch(r"env-\d{3}", matrix_id):
        raise EnvironmentError("matrix id must look like env-001")
    normalized = [
        _validate_run(run, control_variant=control_variant, candidate_variant=candidate_variant)
        for run in runs
    ]
    first = normalized[0]
    for current in normalized[1:]:
        _require_equal("run group", current["run_group"], first["run_group"])
        _require_equal(
            "portable workload",
            current["workload"]["portable_identity_sha256"],
            first["workload"]["portable_identity_sha256"],
        )
        _require_equal("comparison source", current["source"], first["source"])
        _require_equal("jobs", current["job_contracts"], first["job_contracts"])
        _require_equal("measurement conditions", current["conditions"], first["conditions"])

    cell_ids = [entry["environment"]["cell_id"] for entry in normalized]
    if len(cell_ids) != len(set(cell_ids)):
        raise EnvironmentError("every run in a matrix must have a unique environment cell")

    cells = [
        _matrix_cell(run, entry, control_variant, candidate_variant)
        for run, entry in zip(runs, normalized)
    ]
    job_decisions: List[JobDecision] = []
    for job_id in first["jobs"]:
        by_cell: Dict[str, CellDecision] = {}
        for entry in normalized:
            cell_id = entry["environment"]["cell_id"]
            by_cell[cell_id] = _cell_decision(
                entry["document"],
                job_id=job_id,
                control_variant=control_variant,
                candidate_variant=candidate_variant,
                maximum_cpu_regression_pct=maximum_cpu_regression_pct,
                maximum_rss_regression_pct=maximum_rss_regression_pct,
            )
        divergent = _divergent_gates(by_cell)
        job_decisions.append(
            JobDecision(
                job=job_id,
                cells=by_cell,
                divergent_gates=divergent,
                decision_consistent=not divergent,
            )
        )

    workload = first["workload"]
    source = first["source"]
    conditions = first["conditions"]
    divergent_jobs = [job.job for job in job_decisions if job.divergent_gates]
    return EnvironmentMatrix(
        id=matrix_id,
        date=max(entry["document"]["started_utc"][:10] for entry in normalized),
        run_group=first["run_group"],
        workload=MatrixWorkload(
            portable_identity_sha256=workload["portable_identity_sha256"],
            generator_source_hash=workload["generator_source_hash"],
            generator_version=workload["generator_version"],
            recipe_id=workload["recipe_id"],
            recipe_hash=workload["recipe_hash"],
            seed=str(workload["seed"]),
            target_entries=workload["target_entries"],
            semantic_digest=workload["semantic_digest"],
            counts=WorkloadCounts.model_validate(workload["counts"]),
            state=dict(workload["state"]),
        ),
        comparison=MatrixComparison(
            control=VariantSource(name=control_variant, **source["control"]),
            candidate=VariantSource(name=candidate_variant, **source["candidate"]),
            jobs=list(first["jobs"]),
            trials=conditions["trials"],
            warmups=conditions["warmups"],
            os_cache=conditions["os_cache"],
            schedule=conditions["schedule"],
            schedule_sha256=conditions["schedule_sha256"],
        ),
        cells=cells,
        jobs=job_decisions,
        all_cells_valid=all(
            cell.valid for job in job_decisions for cell in job.cells.values()
        ),
        divergent_jobs=divergent_jobs,
        decision_consistent=not divergent_jobs,
    )


def render_matrix(matrix: EnvironmentMatrix) -> str:
    """Render a compact human report without comparing absolute cross-host timing."""
    lines = [f"# Environment decision matrix — {matrix.id}", ""]
    lines.extend(
        [
            f"Run group `{matrix.run_group}` uses portable workload "
            f"`{matrix.workload.portable_identity_sha256[:16]}…`.",
            "Absolute timings stay inside their cells; only gate outcomes are compared.",
            "",
            "## Cells",
            "",
            "| cell | grade | platform | filesystem | CPU | raw run |",
            "| --- | --- | --- | --- | --- | --- |",
        ]
    )
    for cell in matrix.cells:
        lines.append(
            f"| `{cell.id}` | {cell.evidence_grade} | {cell.system} {cell.arch} | "
            f"{cell.filesystem or 'unknown'} | {cell.cpu_model or 'unknown'} "
            f"({cell.logical_cores}) | `{cell.run_artifact}` |"
        )
    lines.extend(["", "## Per-cell decisions", ""])
    for job in matrix.jobs:
        lines.extend(
            [
                f"### `{job.job}`",
                "",
                "| cell | wall | CPU | RSS | latency gate | overall |",
                "| --- | ---: | ---: | ---: | --- | --- |",
            ]
        )
        for cell_id, result in job.cells.items():
            lines.append(
                f"| `{cell_id}` | {_metric_cell(result, 'wall_ns')} | "
                f"{_metric_cell(result, 'cpu_ns')} | "
                f"{_metric_cell(result, 'peak_rss_bytes')} | "
                f"{'pass' if result.latency_gate_passed else 'fail'} | "
                f"{result.overall_decision} |"
            )
        divergence = ", ".join(job.divergent_gates) or "none"
        lines.extend(["", f"Divergent gates: {divergence}.", ""])
    lines.extend(
        [
            "Shared cloud runners are exploratory evidence because neighboring load and "
            "the underlying VM are not controlled. A decision is promoted to a platform "
            "policy only after it repeats on a controlled runner for that cell.",
            "",
            "<!-- This document follows common-doc-guidelines.md.",
            "See github.com/jlevy/practical-prose and review guidelines before editing.",
            "-->",
            "",
        ]
    )
    return "\n".join(lines)


def _validate_run(
    run: RunEvidence, *, control_variant: str, candidate_variant: str
) -> Dict[str, Any]:
    document = run.document
    if document.get("schema") != RUN_SCHEMA:
        raise EnvironmentError(
            f"environment matrices require {RUN_SCHEMA}, got {document.get('schema')!r}"
        )
    if Path(run.artifact_name).name != run.artifact_name:
        raise EnvironmentError("run artifact names must not contain a path")
    if not re.fullmatch(r"[0-9a-f]{64}", run.artifact_sha256):
        raise EnvironmentError("run artifact digest must be a SHA-256")
    environment = document.get("environment") or {}
    if environment.get("schema") != ENVIRONMENT_SCHEMA:
        raise EnvironmentError("run lacks a supported environment identity")
    validate_identifier(str(environment.get("cell_id", "")), field="environment cell")
    selected_runner = environment.get("runner_class")
    if selected_runner not in RUNNER_CLASSES:
        raise EnvironmentError("run has an unsupported runner class")
    if environment.get("evidence_grade") != _EVIDENCE_GRADES[selected_runner]:
        raise EnvironmentError("runner class and evidence grade disagree")
    runner = environment.get("runner")
    if not isinstance(runner, dict):
        raise EnvironmentError("run lacks structured runner facts")
    if (
        selected_runner == "github-hosted-shared"
        and runner.get("provider") != "github-actions"
    ):
        raise EnvironmentError("GitHub-hosted evidence must identify GitHub Actions")
    expected_fingerprint = _sha256_json(
        {"host": dict(document.get("host") or {}), "runner": runner}
    )
    if environment.get("fingerprint_sha256") != expected_fingerprint:
        raise EnvironmentError("environment fingerprint does not match host and runner facts")
    workload = document.get("workload") or {}
    if (
        workload.get("schema") != WORKLOAD_SCHEMA
        or workload.get("kind") != "generated-corpus"
        or workload.get("portable") is not True
    ):
        raise EnvironmentError("environment matrices require a generated portable workload")
    portable_payload = {
        field: workload.get(field)
        for field in (
            "manifest_schema",
            "generator_source_hash",
            "generator_version",
            "recipe_id",
            "recipe_hash",
            "seed",
            "target_entries",
            "semantic_digest",
            "counts",
            "state",
        )
    }
    if workload.get("portable_identity_sha256") != _sha256_json(portable_payload):
        raise EnvironmentError("portable workload identity does not match its fields")
    run_group = validate_identifier(str(document.get("run_group", "")), field="run group")
    variants = document.get("variants") or {}
    if control_variant not in variants or candidate_variant not in variants:
        raise EnvironmentError("selected control or candidate is absent from a run")
    source = {
        "control": _variant_source(variants[control_variant]),
        "candidate": _variant_source(variants[candidate_variant]),
    }
    job_contracts = document.get("jobs") or {}
    jobs = list(job_contracts)
    if not jobs:
        raise EnvironmentError("run has no measured jobs")
    conditions_document = document.get("conditions") or {}
    conditions = {
        field: conditions_document.get(field)
        for field in (
            "trials",
            "warmups",
            "os_cache",
            "interleaved",
            "schedule",
            "schedule_sha256",
            "schedule_seed",
            "bootstrap_seed",
        )
    }
    if not isinstance(conditions["trials"], int) or conditions["trials"] < 1:
        raise EnvironmentError("run has an invalid trial count")
    if not isinstance(conditions["warmups"], int) or conditions["warmups"] < 0:
        raise EnvironmentError("run has an invalid warmup count")
    if conditions["interleaved"] is not True:
        raise EnvironmentError("environment comparisons require interleaved variants")
    if not isinstance(conditions["os_cache"], str) or not conditions["os_cache"]:
        raise EnvironmentError("run lacks an OS page-cache condition")
    if not isinstance(conditions["schedule"], str) or not conditions["schedule"]:
        raise EnvironmentError("run lacks a named schedule")
    if not re.fullmatch(r"[0-9a-f]{64}", str(conditions["schedule_sha256"])):
        raise EnvironmentError("run lacks a valid schedule digest")
    return {
        "document": document,
        "environment": environment,
        "workload": workload,
        "run_group": run_group,
        "source": source,
        "jobs": jobs,
        "job_contracts": job_contracts,
        "conditions": conditions,
    }


def _variant_source(identity: Mapping[str, Any]) -> Dict[str, Any]:
    selected = {field: identity.get(field) for field in _SOURCE_FIELDS}
    missing = [field for field, value in selected.items() if value is None]
    if missing:
        raise EnvironmentError(
            "variant lacks cross-environment source identity: " + ", ".join(missing)
        )
    selected["features"] = list(selected["features"])
    selected["args"] = list(selected["args"])
    return selected


def _matrix_cell(
    run: RunEvidence,
    entry: Mapping[str, Any],
    control_variant: str,
    candidate_variant: str,
) -> MatrixCell:
    document = entry["document"]
    host = document["host"]
    environment = entry["environment"]
    variants = document["variants"]
    return MatrixCell(
        id=environment["cell_id"],
        runner_class=environment["runner_class"],
        evidence_grade=environment["evidence_grade"],
        environment_fingerprint=environment["fingerprint_sha256"],
        system=str(host.get("system") or ""),
        release=str(host.get("release") or ""),
        arch=str(host.get("arch") or ""),
        cpu_model=str(host.get("cpu_model") or ""),
        logical_cores=int(host.get("cpu_count") or 0),
        memory_bytes=int(host.get("memory_bytes") or 0),
        filesystem=str(host.get("filesystem") or ""),
        toolchain=str(host.get("toolchain") or ""),
        control_target=str(variants[control_variant].get("target") or ""),
        candidate_target=str(variants[candidate_variant].get("target") or ""),
        tree_engine_digest=document["tree"]["engine_digest"],
        run_artifact=run.artifact_name,
        run_artifact_sha256=run.artifact_sha256,
    )


def _cell_decision(
    document: Mapping[str, Any],
    *,
    job_id: str,
    control_variant: str,
    candidate_variant: str,
    maximum_cpu_regression_pct: float,
    maximum_rss_regression_pct: float,
) -> CellDecision:
    statistics = document["statistics"][job_id]
    comparison_name = f"{candidate_variant}_vs_{control_variant}"
    comparison = statistics["comparisons"].get(comparison_name)
    if comparison is None:
        raise EnvironmentError(f"job {job_id!r} lacks comparison {comparison_name!r}")
    invalid_samples = sum(
        int(statistics["variants"][variant]["invalid"])
        for variant in (control_variant, candidate_variant)
    )
    tree_mutated = bool(document.get("tree_mutated_during_run"))
    evaluated = decision.evaluate(
        comparison,
        invalid_samples=invalid_samples,
        tree_mutated=tree_mutated,
        maximum_cpu_regression_pct=maximum_cpu_regression_pct,
        maximum_rss_regression_pct=maximum_rss_regression_pct,
    )
    metrics: Dict[str, MetricObservation] = {}
    for metric in _MATRIX_METRICS:
        observed = (comparison.get("metrics") or {}).get(metric)
        control_summary = statistics["variants"][control_variant]["metrics"].get(metric)
        candidate_summary = statistics["variants"][candidate_variant]["metrics"].get(metric)
        if not observed or not control_summary or not candidate_summary:
            continue
        interval = observed.get("ci95_change_pct") or [None, None]
        change = float(observed["median_change_pct"])
        metrics[metric] = MetricObservation(
            control_median=float(control_summary["median"]),
            candidate_median=float(candidate_summary["median"]),
            change_pct=change,
            ci95_low_pct=interval[0],
            ci95_high_pct=interval[1],
            pairs=int(observed.get("pairs") or 0),
            direction=observed.get("direction")
            or ("improvement" if change < 0 else "regression" if change > 0 else "unchanged"),
        )
    return CellDecision(
        valid=not tree_mutated and invalid_samples == 0,
        invalid_samples=invalid_samples,
        metrics=metrics,
        latency_gate_passed=evaluated["latency_gate_passed"],
        latency_reason=evaluated["latency_reason"],
        resource_guardrails=[
            ResourceGate.model_validate(gate) for gate in evaluated["resource_guardrails"]
        ],
        overall_decision=evaluated["overall_decision"],
        reason=evaluated["reason"],
    )


def _divergent_gates(cells: Mapping[str, CellDecision]) -> List[str]:
    values: Dict[str, set[Any]] = {
        "latency": {cell.latency_gate_passed for cell in cells.values()},
        "cpu": {_gate_status(cell, "cpu_ns") for cell in cells.values()},
        "rss": {_gate_status(cell, "peak_rss_bytes") for cell in cells.values()},
        "overall": {cell.overall_decision for cell in cells.values()},
    }
    return [gate for gate, outcomes in values.items() if len(outcomes) > 1]


def _gate_status(cell: CellDecision, metric: str) -> str:
    return next(gate.status for gate in cell.resource_guardrails if gate.metric == metric)


def _metric_cell(cell: CellDecision, metric: str) -> str:
    observed = cell.metrics.get(metric)
    return "—" if observed is None else f"{observed.change_pct:+.2f}%"


def _require_equal(label: str, actual: Any, expected: Any) -> None:
    if actual != expected:
        raise EnvironmentError(f"runs do not have equivalent {label}")


def _sha256_json(value: Any) -> str:
    encoded = json.dumps(value, separators=(",", ":"), sort_keys=True).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def _slug(value: str) -> str:
    collapsed = re.sub(r"[^a-z0-9._-]+", "-", value.lower()).strip("-._")
    return collapsed[:80] or "unknown-environment"


def _safe_runner_value(value: Optional[str]) -> Optional[str]:
    if value is None:
        return None
    if len(value) > 120 or any(character in value for character in ("/", "\\", "\n", "\r")):
        raise EnvironmentError("runner image metadata contains a path or control character")
    return value


def _toolchain() -> str:
    binary = shutil.which("rustc")
    if binary is None:
        return ""
    completed = subprocess.run([binary, "--version"], capture_output=True, check=False)
    if completed.returncode != 0:
        return ""
    return completed.stdout.decode("utf-8", errors="replace").strip()


def _sysctl(name: str) -> Optional[str]:
    binary = shutil.which("sysctl")
    if binary is None:
        return None
    completed = subprocess.run([binary, "-n", name], capture_output=True, check=False)
    if completed.returncode != 0:
        return None
    return completed.stdout.decode("utf-8", errors="replace").strip() or None


def _sysctl_int(name: str) -> Optional[int]:
    value = _sysctl(name)
    try:
        return int(value) if value is not None else None
    except ValueError:
        return None


def _filesystem_for_path(root: Path) -> Optional[str]:
    if sys.platform == "darwin":
        binary = shutil.which("diskutil")
        if binary is not None:
            targets = [str(root)]
            df = shutil.which("df")
            if df is not None:
                mounted = subprocess.run(
                    [df, "-P", str(root)], capture_output=True, check=False
                )
                if mounted.returncode == 0:
                    lines = mounted.stdout.decode(
                        "utf-8", errors="replace"
                    ).splitlines()
                    if len(lines) >= 2 and lines[-1].split():
                        targets.append(lines[-1].split()[0])
            for target in targets:
                completed = subprocess.run(
                    [binary, "info", "-plist", target],
                    capture_output=True,
                    check=False,
                )
                if completed.returncode != 0:
                    continue
                try:
                    document = plistlib.loads(completed.stdout)
                except plistlib.InvalidFileException:
                    document = {}
                value = document.get("FilesystemType") or document.get(
                    "FilesystemName"
                )
                if value:
                    return str(value).lower()
    elif sys.platform.startswith("linux"):
        findmnt = shutil.which("findmnt")
        if findmnt is not None:
            completed = subprocess.run(
                [findmnt, "--noheadings", "--output", "FSTYPE", "--target", str(root)],
                capture_output=True,
                check=False,
            )
            if completed.returncode == 0:
                value = completed.stdout.decode("utf-8", errors="replace").strip()
                if value:
                    return value.splitlines()[0]
        stat = shutil.which("stat")
        if stat is not None:
            completed = subprocess.run(
                [stat, "--file-system", "--format=%T", str(root)],
                capture_output=True,
                check=False,
            )
            if completed.returncode == 0:
                return completed.stdout.decode("utf-8", errors="replace").strip() or None
    return None


def _linux_cpu_model() -> Optional[str]:
    try:
        text = Path("/proc/cpuinfo").read_text(encoding="utf-8", errors="replace")
    except OSError:
        return None
    for key in ("model name", "hardware", "processor"):
        for line in text.splitlines():
            name, separator, value = line.partition(":")
            if separator and name.strip().lower() == key and value.strip():
                return value.strip()
    return None


def _linux_memory_bytes() -> Optional[int]:
    try:
        text = Path("/proc/meminfo").read_text(encoding="utf-8", errors="replace")
    except OSError:
        return None
    for line in text.splitlines():
        if line.startswith("MemTotal:"):
            fields = line.split()
            if len(fields) >= 2 and fields[1].isdigit():
                return int(fields[1]) * 1024
    return None


__all__ = [
    "ENVIRONMENT_SCHEMA",
    "EnvironmentError",
    "EnvironmentMatrix",
    "MATRIX_CONTRACT",
    "MATRIX_SCHEMA",
    "RUNNER_CLASSES",
    "RUN_SCHEMA",
    "RunEvidence",
    "WORKLOAD_SCHEMA",
    "build_matrix",
    "environment_identity",
    "host_facts",
    "render_matrix",
    "validate_identifier",
    "workload_identity",
]
