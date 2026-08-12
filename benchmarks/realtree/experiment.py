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
but a person. See `uvx softschema@latest docs guide`.

The model here is the source of truth for the contract. Compile it with:

    uvx softschema@latest compile benchmarks.realtree.experiment:Experiment \\
      --out docs/project/experiments/experiment.schema.yaml \\
      --contract fdu.performance:Experiment/v1
"""

from __future__ import annotations

from typing import Any, Dict, List, Literal, Mapping, Optional, Sequence

from pydantic import BaseModel, ConfigDict, Field

CONTRACT = "fdu.performance:Experiment/v1"

#: ``baseline`` is a ledger entry with no candidate: it establishes the numbers later
#: experiments are measured against. ``blocked`` is a hypothesis we cannot test yet and
#: the reason why, which is worth recording so nobody re-derives the obstacle.
Decision = Literal[
    "accepted", "rejected", "superseded", "blocked", "in-progress", "baseline"
]


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
    significant: bool = Field(
        default=False,
        description=(
            "Deprecated alias of passes_acceptance, kept so artifacts recorded before "
            "the fields were split still validate. Read passes_acceptance instead."
        ),
    )
    passes_acceptance: bool = Field(
        default=False,
        description=(
            "Whether the whole interval lies below zero: the project's one-sided "
            "accept rule. False does NOT mean unchanged - a regression fails it too."
        ),
    )
    ci_excludes_zero: bool = Field(
        default=False,
        description=(
            "Whether the interval is clear of zero in either direction, i.e. the "
            "measurement says something rather than nothing."
        ),
    )
    direction: Literal["improved", "regressed", "unclear", "unknown"] = Field(
        default="unknown",
        description="Which way the evidence points, separate from whether we accept it.",
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
    run_artifact: Optional[str] = Field(
        default=None, description="Path to the raw run JSON this was derived from."
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


def _flags_from_interval(low: Any, high: Any) -> Dict[str, Any]:
    """Evidence flags computed from the 95% interval alone.

    Deriving rather than copying is what lets a run recorded before the fields were
    split produce a correct artifact: the bounds are the ground truth, and the flags
    are a reading of them.
    """
    if low is None or high is None:
        return {"ci_excludes_zero": False, "direction": "unknown"}
    if high < 0:
        return {"ci_excludes_zero": True, "direction": "improved"}
    if low > 0:
        return {"ci_excludes_zero": True, "direction": "regressed"}
    return {"ci_excludes_zero": False, "direction": "unclear"}


def _binary(run: Mapping[str, Any], name: str) -> Dict[str, Any]:
    identity = (run.get("variants") or {}).get(name) or {}
    return {
        "name": name,
        "sha256": identity.get("sha256", ""),
        "size_bytes": identity.get("size_bytes", 0),
        "args": list(identity.get("args") or []),
    }


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
            metrics[metric] = {
                "control_median": float(control_summary["median"]),
                "candidate_median": float(candidate_summary["median"]),
                "change_pct": float(entry.get("median_change_pct") or 0.0),
                "ci95_low_pct": interval[0],
                "ci95_high_pct": interval[1],
                "significant": bool(entry.get("significant", False)),
                "passes_acceptance": bool(
                    entry.get("passes_acceptance", entry.get("significant", False))
                ),
                # Derived from the interval, never defaulted. A run recorded before the
                # fields were split carries neither key, and writing `false`/`unknown`
                # into the artifact would contradict the very bounds sitting beside them
                # — an artifact that validates while its own flags disagree with its own
                # interval is worse than one that lacks the fields.
                **_flags_from_interval(interval[0], interval[1]),
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
            "run_artifact": run_artifact,
        },
        "results": results,
        "reference_tools": references,
        "complexity": dict(complexity),
        "verdict": dict(verdict),
    }
