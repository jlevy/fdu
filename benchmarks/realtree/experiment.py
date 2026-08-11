"""The soft-schema contract for a performance experiment, and how to build one.

An optimization loop only works if its record survives the session that produced it.
Six months later, the useful questions are "did anyone try this?", "what happened?",
and "why was it dropped?" — and the answer has to be findable without re-running
anything.

So every experiment is one Markdown artifact. The frontmatter carries the values a
tool reads: the hypothesis, the measured medians, the interval, the verdict. The body
carries the part no schema can hold: what the profile suggested, what was actually
tried, and why the number meant what we said it meant.

That split is the soft-schema practice. Numbers are promoted into YAML because the
ledger renderer consumes them; the reasoning stays prose because nothing consumes it
but a person. See `uv run --project benchmarks/realtree --frozen softschema docs guide`.

The model here is the source of truth for the contract. Compile it with:

    uv run --project benchmarks/realtree --frozen softschema compile \\
      benchmarks.realtree.experiment:Experiment \\
      --out docs/project/experiments/experiment.schema.yaml \\
      --contract fdu.performance:Experiment/v1
"""

from __future__ import annotations

from typing import Any, Dict, List, Literal, Mapping, Optional, Sequence

from pydantic import BaseModel, ConfigDict, Field, model_validator

CONTRACT = "fdu.performance:Experiment/v1"

#: ``baseline`` is a ledger entry with no candidate: it establishes the numbers later
#: experiments are measured against. ``blocked`` is a hypothesis we cannot test yet and
#: the reason why, which is worth recording so nobody re-derives the obstacle.
Decision = Literal[
    "accepted", "rejected", "superseded", "blocked", "in-progress", "baseline"
]
Direction = Literal["improvement", "regression", "unchanged"]
GuardrailStatus = Literal["passed", "failed", "not-measured", "waived"]
EvidenceGrade = Literal["claim-grade", "legacy"]


class Strict(BaseModel):
    model_config = ConfigDict(extra="forbid")


class MetricChange(Strict):
    """One metric's paired comparison between the control and the candidate."""

    control_median: float = Field(description="Control median, in the metric's own unit.")
    candidate_median: float = Field(description="Candidate median, same unit.")
    change_pct: float = Field(
        description="Paired median change, negative meaning the candidate is faster."
    )
    ci95_low_pct: Optional[float] = Field(
        default=None, description="Lower bound of the 95% bootstrap interval on change_pct."
    )
    ci95_high_pct: Optional[float] = Field(
        default=None, description="Upper bound of the 95% bootstrap interval."
    )
    direction: Optional[Direction] = Field(
        default=None, description="Direction of the paired median change."
    )
    ci_excludes_zero: Optional[bool] = Field(
        default=None,
        description="Whether the 95% interval excludes zero in either direction.",
    )
    significant_improvement: Optional[bool] = Field(
        default=None,
        description="One-sided acceptance fact: a significant change below zero.",
    )
    significant: Optional[bool] = Field(
        default=None,
        description="Compatibility alias for ci_excludes_zero in newly recorded data.",
    )
    pairs: int = Field(default=0, ge=0, description="Trial pairs behind the comparison.")


class JobResult(Strict):
    """What one measured job did, across every metric."""

    job: str = Field(description="Job id, for example cold-scan-index.")
    start_state: Literal["cold", "warm"] = Field(
        description="Whether a compatible fdu snapshot existed before the timed work."
    )
    invalid_samples: int = Field(
        default=0,
        ge=0,
        description="Trials the independent oracle rejected. Nonzero invalidates the job.",
    )
    metrics: Dict[str, MetricChange] = Field(
        default_factory=dict, description="Keyed by metric name, e.g. wall_ns."
    )


class Complexity(Strict):
    """What the change costs to carry, which the accept rule cannot compute."""

    lines_changed: int = Field(ge=0, description="Net lines of production code touched.")
    new_dependencies: List[str] = Field(
        default_factory=list, description="Crates or packages added. Empty is the goal."
    )
    new_unsafe_blocks: int = Field(default=0, ge=0)
    new_failure_modes: List[str] = Field(
        default_factory=list,
        description="Ways this can now go wrong that it could not before.",
    )
    notes: str = Field(default="", description="Anything else that argues for or against.")


class ResourceGuardrail(Strict):
    """One explicit resource-cost bound evaluated beside the latency gate."""

    metric: Literal["cpu_ns", "peak_rss_bytes"]
    maximum_regression_pct: float = Field(ge=0)
    observed_change_pct: Optional[float] = None
    ci95_low_pct: Optional[float] = None
    ci95_high_pct: Optional[float] = None
    status: GuardrailStatus
    reason: str


class Verdict(Strict):
    """The decision, the number it rests on, and the reason in one sentence."""

    decision: Decision
    primary_job: str = Field(description="The job the decision was taken on.")
    primary_metric: str = Field(default="wall_ns")
    change_pct: Optional[float] = Field(
        default=None, description="The headline paired median change."
    )
    threshold_pct: float = Field(
        default=-3.0, description="The bar a change had to clear to be worth carrying."
    )
    reason: str = Field(description="One sentence. Why this decision and not the other.")
    commit: Optional[str] = Field(
        default=None, description="Commit that landed it, or reverted it."
    )
    latency_gate_passed: Optional[bool] = Field(
        default=None,
        description="Whether the primary latency metric cleared significance and threshold.",
    )
    resource_guardrails: List[ResourceGuardrail] = Field(
        default_factory=list,
        description="CPU and RSS decisions evaluated separately from latency.",
    )
    resource_waiver_reason: Optional[str] = Field(
        default=None,
        description="Explicit product rationale when an accepted change waives a guardrail.",
    )


class Subject(Strict):
    """What was measured, and on what.

    This is the reproducibility record. Someone re-running the experiment in a year
    needs to know whether they are looking at the same tree on comparable hardware,
    and a timing without that context is a number without a claim attached.

    The tree is pinned by content, not by name: ``tree_engine_digest`` is the
    `fdu-index-record-v1` digest over every entry's path, kind, size, mtime, ctime,
    inode, and device. Two trees with that digest are the same tree in the same
    state. The path is never recorded — ``tree_root_id`` is its SHA-256, enough to
    tell two trees apart without disclosing where either lives.
    """

    tree_label: str
    tree_root_id: str = Field(description="SHA-256 of the tree's path. Never the path.")
    tree_engine_digest: str = Field(
        default="",
        description="fdu-index-record-v1 digest of the whole tree; pins exact content.",
    )
    tree_entries: int = Field(ge=0)
    tree_directories: int = Field(ge=0)
    tree_files: int = Field(default=0, ge=0)
    tree_symlinks: int = Field(default=0, ge=0)
    tree_apparent_bytes: int = Field(ge=0)
    tree_allocated_bytes: int = Field(default=0, ge=0)
    tree_max_depth: int = Field(ge=0)
    tree_mutated_during_run: bool = Field(
        default=False,
        description="True invalidates the whole experiment; the tree moved mid-run.",
    )
    host_cpu: str = Field(default="")
    host_arch: str = Field(default="")
    host_cores: int = Field(default=0, ge=0)
    host_performance_cores: int = Field(
        default=0, ge=0, description="Big cores, where they are distinguishable."
    )
    host_efficiency_cores: int = Field(default=0, ge=0)
    host_memory_bytes: int = Field(default=0, ge=0)
    host_system: str = Field(default="")
    filesystem: str = Field(default="")
    os_cache: str = Field(
        default="warm-steady",
        description="Page-cache condition. Dropping it needs root, so say which it was.",
    )


class Binary(Strict):
    """A measured artifact, identified by content rather than by where it sat."""

    name: str
    sha256: str = Field(default="", description="Of the binary itself, not its source.")
    size_bytes: int = Field(default=0, ge=0)
    args: List[str] = Field(
        default_factory=list,
        description="Flags this variant carried, e.g. a thread count.",
    )
    engine_revision: Optional[str] = Field(
        default=None,
        pattern=r"^[0-9a-f]{40}$",
        description="Exact engine source commit, independent of harness source.",
    )
    harness_revision: Optional[str] = Field(
        default=None,
        pattern=r"^[0-9a-f]{40}$",
        description="Exact commit containing the probe source used for this build.",
    )
    harness_sha256: Optional[str] = Field(
        default=None,
        pattern=r"^[0-9a-f]{64}$",
        description="Content digest of the probe source compiled into this binary.",
    )
    target: Optional[str] = Field(
        default=None, description="Rust target triple used for the measured binary."
    )
    build_profile: Optional[str] = Field(
        default=None, description="Cargo profile used to build this exact binary."
    )
    features: List[str] = Field(
        default_factory=list, description="Sorted Cargo feature set; empty means none."
    )
    build_command: Optional[str] = Field(
        default=None,
        description="Path-redacted command sufficient to reproduce the build.",
    )


class Method(Strict):
    trials: int = Field(ge=1)
    warmups: int = Field(ge=0)
    interleaved: bool = Field(
        default=True, description="Variants alternated per ordinal rather than run in blocks."
    )
    control: str = Field(description="What the candidate was compared against.")
    candidate: str = Field(description="What changed.")
    control_binary: Optional[Binary] = Field(default=None)
    candidate_binary: Optional[Binary] = Field(default=None)
    toolchain: str = Field(
        default="", description="Compiler version the binaries were built with."
    )
    build_profile: str = Field(
        default="release",
        description="Cargo profile. Timing evidence only ever comes from release.",
    )
    evidence_grade: EvidenceGrade = Field(
        default="legacy",
        description="Claim-grade satisfies the enforced provenance and raw-evidence contract.",
    )
    run_schema: str = Field(
        default="", description="Schema identifier of the archived raw measurement run."
    )
    schedule: str = Field(
        default="", description="Named interleaving algorithm used by the runner."
    )
    schedule_sha256: Optional[str] = Field(
        default=None,
        pattern=r"^[0-9a-f]{64}$",
        description="Digest of the exact expanded variant/job/ordinal schedule.",
    )
    schedule_seed: Optional[int] = Field(
        default=None,
        description="Random seed, or null when the named schedule is deterministic.",
    )
    run_artifact: Optional[str] = Field(
        default=None, description="Path to the raw run JSON this was derived from."
    )
    run_artifact_sha256: Optional[str] = Field(
        default=None,
        pattern=r"^[0-9a-f]{64}$",
        description="Content digest of the archived raw run JSON.",
    )


class ReferenceTool(Strict):
    """A third-party tool measured on the same tree, for calibration only."""

    name: str
    wall_ns_median: float
    argv: List[str] = Field(default_factory=list)


class Experiment(Strict):
    """One turn of the performance loop, from hypothesis to verdict."""

    id: str = Field(pattern=r"^exp-\d{3}$", description="Stable identifier, e.g. exp-002.")
    title: str
    date: str = Field(pattern=r"^\d{4}-\d{2}-\d{2}$")
    hypotheses: List[str] = Field(
        default_factory=list,
        description="Hypothesis ids from the performance-loop guide, e.g. H1.",
    )
    subject: Subject
    method: Method
    results: List[JobResult] = Field(default_factory=list)
    reference_tools: List[ReferenceTool] = Field(default_factory=list)
    complexity: Complexity
    verdict: Verdict

    @model_validator(mode="after")
    def evidence_and_acceptance_contracts_are_enforced(self) -> "Experiment":
        """Keep claim-grade and accepted labels stronger than prose assertions."""
        missing: List[str] = []
        claim_grade = self.method.evidence_grade == "claim-grade"
        if self.verdict.decision == "accepted" and not claim_grade:
            missing.append("method.evidence_grade=claim-grade")
        if not claim_grade:
            if missing:
                raise ValueError(
                    "accepted experiment lacks claim-grade evidence: "
                    + ", ".join(missing)
                )
            return self

        if self.method.run_schema != "fdu-realtree-run-v2":
            missing.append("method.run_schema=fdu-realtree-run-v2")
        if not self.method.toolchain:
            missing.append("method.toolchain")
        if not self.method.schedule or not self.method.schedule_sha256:
            missing.append("method.schedule/schedule_sha256")
        if not self.method.run_artifact or not self.method.run_artifact_sha256:
            missing.append("method.run_artifact/run_artifact_sha256")
        for role, binary in (
            ("control", self.method.control_binary),
            ("candidate", self.method.candidate_binary),
        ):
            if binary is None:
                missing.append(f"method.{role}_binary")
                continue
            required = {
                "sha256": binary.sha256,
                "engine_revision": binary.engine_revision,
                "harness_revision": binary.harness_revision,
                "harness_sha256": binary.harness_sha256,
                "target": binary.target,
                "build_profile": binary.build_profile,
                "build_command": binary.build_command,
            }
            missing.extend(
                f"method.{role}_binary.{name}"
                for name, value in required.items()
                if not value
            )
            if "features" not in binary.model_fields_set:
                missing.append(f"method.{role}_binary.features")
            if binary.build_profile != "release":
                missing.append(f"method.{role}_binary.build_profile=release")
        if not self.subject.tree_engine_digest:
            missing.append("subject.tree_engine_digest")
        if self.subject.tree_mutated_during_run:
            missing.append("unchanged tree across the run")
        if any(result.invalid_samples for result in self.results):
            missing.append("zero invalid samples")
        if missing:
            raise ValueError(
                "claim-grade experiment lacks required evidence: " + ", ".join(missing)
            )

        if self.verdict.decision == "accepted":
            acceptance_missing: List[str] = []
            if self.verdict.latency_gate_passed is not True:
                acceptance_missing.append("verdict.latency_gate_passed=true")
            by_metric = {
                guardrail.metric: guardrail for guardrail in self.verdict.resource_guardrails
            }
            for metric in ("cpu_ns", "peak_rss_bytes"):
                guardrail = by_metric.get(metric)
                if guardrail is None:
                    acceptance_missing.append(f"{metric} resource guardrail")
                elif guardrail.status not in {"passed", "waived"}:
                    acceptance_missing.append(f"{metric} resource guardrail passed or waived")
                elif guardrail.status == "waived" and not self.verdict.resource_waiver_reason:
                    acceptance_missing.append("verdict.resource_waiver_reason")
            if acceptance_missing:
                raise ValueError(
                    "accepted experiment lacks enforced decision gates: "
                    + ", ".join(acceptance_missing)
                )
        return self


# --------------------------------------------------------------------------------
# Building an artifact from a measurement run
# --------------------------------------------------------------------------------

#: Metrics promoted into the frontmatter. The run JSON holds every metric for every
#: trial; the artifact holds the ones a reader compares experiments on. Promoting all
#: of them would make the artifact unreadable without making it more useful.
LEDGER_METRICS = (
    "wall_ns",
    "component_ns",
    "cpu_ns",
    "user_cpu_ns",
    "system_cpu_ns",
    "blocked_ns",
    "peak_rss_bytes",
)


def _binary(run: Mapping[str, Any], name: str) -> Dict[str, Any]:
    identity = (run.get("variants") or {}).get(name) or {}
    binary = {
        "name": name,
        "sha256": identity.get("sha256", ""),
        "size_bytes": identity.get("size_bytes", 0),
        "args": list(identity.get("args") or []),
    }
    for field_name in (
        "engine_revision",
        "harness_revision",
        "harness_sha256",
        "target",
        "build_profile",
        "features",
        "build_command",
    ):
        if field_name in identity:
            binary[field_name] = identity[field_name]
    return binary


def from_run(
    run: Mapping[str, Any],
    *,
    experiment_id: str,
    title: str,
    hypotheses: Sequence[str],
    control: str,
    candidate: str,
    complexity: Mapping[str, Any],
    verdict: Mapping[str, Any],
    run_artifact: Optional[str] = None,
    run_artifact_sha256: Optional[str] = None,
    evidence_grade: EvidenceGrade = "legacy",
    control_variant: Optional[str] = None,
    candidate_variant: Optional[str] = None,
) -> Dict[str, Any]:
    """Derive an experiment payload from a measurement run document.

    Everything measurable is read from the run rather than retyped, so the ledger
    cannot drift from the data. What the caller supplies is exactly what the run
    cannot know: which hypothesis this tested, what it cost in complexity, and what
    a person decided.
    """
    # Declaration order decides which variant is the control, and it is recorded
    # explicitly because the run document sorts its keys. Falling back to the mapping
    # order would silently invert the comparison on an older run.
    variants = list(run.get("variant_order") or run["variants"])
    control_name = control_variant or variants[0]
    candidate_name = candidate_variant or (variants[1] if len(variants) > 1 else variants[0])

    tree = run["tree"]
    host = run["host"]
    conditions = run["conditions"]

    results: List[Dict[str, Any]] = []
    for job_id, job in run["jobs"].items():
        statistics = run["statistics"][job_id]
        comparison = statistics["comparisons"].get(f"{candidate_name}_vs_{control_name}")
        metrics: Dict[str, Any] = {}
        for metric in LEDGER_METRICS:
            control_summary = statistics["variants"][control_name]["metrics"].get(metric)
            candidate_summary = statistics["variants"][candidate_name]["metrics"].get(metric)
            if not control_summary or not candidate_summary:
                continue
            entry = (comparison or {}).get("metrics", {}).get(metric) or {}
            interval = entry.get("ci95_change_pct") or [None, None]
            change = float(entry.get("median_change_pct") or 0.0)
            direction = entry.get("direction") or (
                "improvement" if change < 0 else "regression" if change > 0 else "unchanged"
            )
            ci_excludes_zero = entry.get("ci_excludes_zero")
            if ci_excludes_zero is None:
                ci_excludes_zero = bool(
                    interval[0] is not None
                    and interval[1] is not None
                    and (interval[1] < 0 or interval[0] > 0)
                )
            metrics[metric] = {
                "control_median": float(control_summary["median"]),
                "candidate_median": float(candidate_summary["median"]),
                "change_pct": change,
                "ci95_low_pct": interval[0],
                "ci95_high_pct": interval[1],
                "direction": direction,
                "ci_excludes_zero": ci_excludes_zero,
                "significant_improvement": bool(
                    entry.get(
                        "significant_improvement",
                        ci_excludes_zero and direction == "improvement",
                    )
                ),
                "significant": ci_excludes_zero,
                "pairs": int(entry.get("pairs", 0)),
            }
        results.append(
            {
                "job": job_id,
                "start_state": job["start_state"],
                "invalid_samples": (
                    statistics["variants"][control_name]["invalid"]
                    + statistics["variants"][candidate_name]["invalid"]
                ),
                "metrics": metrics,
            }
        )

    references = [
        {
            "name": name,
            "wall_ns_median": float(entry["wall_ns"]["median"]),
            "argv": list(entry.get("argv", [])),
        }
        for name, entry in (run.get("reference_tools") or {}).items()
        if entry.get("wall_ns")
    ]

    return {
        "id": experiment_id,
        "title": title,
        "date": run["started_utc"][:10],
        "hypotheses": list(hypotheses),
        "subject": {
            "tree_label": tree["label"],
            "tree_root_id": tree["root_id"],
            "tree_engine_digest": tree.get("engine_digest", ""),
            "tree_entries": tree["counts"]["total"],
            "tree_directories": tree["counts"]["directories"],
            "tree_files": tree["counts"].get("files", 0),
            "tree_symlinks": tree["counts"].get("symlinks", 0),
            "tree_apparent_bytes": tree["sizes"]["apparent_bytes"],
            "tree_allocated_bytes": tree["sizes"].get("allocated_bytes", 0),
            "tree_max_depth": tree["max_depth"],
            "tree_mutated_during_run": bool(run["tree_mutated_during_run"]),
            "host_cpu": host.get("cpu_model") or host.get("system", ""),
            "host_arch": host.get("arch", ""),
            "host_cores": host.get("cpu_count") or 0,
            "host_performance_cores": host.get("performance_cores") or 0,
            "host_efficiency_cores": host.get("efficiency_cores") or 0,
            "host_memory_bytes": host.get("memory_bytes") or 0,
            "host_system": f"{host.get('system', '')} {host.get('release', '')}".strip(),
            "filesystem": host.get("filesystem") or "",
            "os_cache": conditions["os_cache"],
        },
        "method": {
            "trials": conditions["trials"],
            "warmups": conditions["warmups"],
            "interleaved": bool(conditions["interleaved"]),
            "control": control,
            "candidate": candidate,
            "control_binary": _binary(run, control_name),
            "candidate_binary": _binary(run, candidate_name),
            "toolchain": host.get("toolchain", ""),
            "build_profile": "release",
            "evidence_grade": evidence_grade,
            "run_schema": run.get("schema", ""),
            "schedule": conditions.get("schedule", ""),
            "schedule_sha256": conditions.get("schedule_sha256"),
            "schedule_seed": conditions.get("schedule_seed"),
            "run_artifact": run_artifact,
            "run_artifact_sha256": run_artifact_sha256,
        },
        "results": results,
        "reference_tools": references,
        "complexity": dict(complexity),
        "verdict": dict(verdict),
    }
