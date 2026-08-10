"""Deterministic statistics, compatibility checks, and Markdown reports."""

from __future__ import annotations

import math
import os
import statistics as statistics_module
import textwrap
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence

from benchmarks.schema import load_result, validate_result_document


def scenario_statistics(result: Mapping[str, Any]) -> List[Dict[str, Any]]:
    """Calculate declared statistics from valid timed samples only."""
    validate_result_document(result)
    definitions = {
        scenario["id"]: scenario for scenario in result["scenario_definitions"]
    }
    rows: List[Dict[str, Any]] = []
    for scenario_id in sorted(definitions):
        timed = [
            trial
            for trial in result["trials"]
            if trial["scenario_id"] == scenario_id
            and trial["sample_kind"] == "timed"
        ]
        valid = [
            trial
            for trial in timed
            if trial["validation"]["valid"]
            and trial["timing"]["wall_ns"] is not None
        ]
        walls = [int(trial["timing"]["wall_ns"]) for trial in valid]
        row = _statistics_row(
            scenario_id,
            walls,
            invalid_trials=len(timed) - len(valid),
            entry_count=_entry_count(valid),
        )
        rows.append(row)
    return rows


def baseline_compatibility_reasons(
    current: Mapping[str, Any], baseline: Mapping[str, Any]
) -> List[str]:
    """Name inputs that make two result sets unsuitable for numeric comparison."""
    reasons: List[str] = []
    if current.get("schema") != baseline.get("schema"):
        reasons.append("result schema differs")
    if current.get("host_class") != baseline.get("host_class"):
        reasons.append("host class or host capabilities differ")
    if current.get("harness") != baseline.get("harness"):
        reasons.append("harness or Python runtime differs")
    current_executables = current.get("executables", {})
    baseline_executables = baseline.get("executables", {})
    if not isinstance(current_executables, Mapping) or not isinstance(
        baseline_executables, Mapping
    ):
        reasons.append("executable identity records are malformed")
    else:
        if set(current_executables) != set(baseline_executables):
            reasons.append("executable adapter sets differ")
        for adapter in sorted(set(current_executables) & set(baseline_executables)):
            current_shape = _executable_shape(current_executables[adapter])
            baseline_shape = _executable_shape(baseline_executables[adapter])
            if current_shape != baseline_shape:
                reasons.append(
                    f"executable command shape differs for adapter {adapter!r}"
                )

    current_scenarios = _scenario_map(current.get("scenario_definitions"))
    baseline_scenarios = _scenario_map(baseline.get("scenario_definitions"))
    if set(current_scenarios) != set(baseline_scenarios):
        reasons.append("scenario id sets differ")
    for scenario_id in sorted(set(current_scenarios) & set(baseline_scenarios)):
        _compare_scenario_contract(
            scenario_id,
            current_scenarios[scenario_id],
            baseline_scenarios[scenario_id],
            reasons,
        )
    if _corpus_contracts(current) != _corpus_contracts(baseline):
        reasons.append("observed corpus contracts differ")
    if _collector_shape(current) != _collector_shape(baseline):
        reasons.append("collector availability differs")
    return reasons


def executable_identity_differences(
    current: Mapping[str, Any], baseline: Mapping[str, Any]
) -> List[str]:
    """Report binary changes separately from compatibility requirements."""
    differences: List[str] = []
    current_executables = current.get("executables", {})
    baseline_executables = baseline.get("executables", {})
    if not isinstance(current_executables, Mapping) or not isinstance(
        baseline_executables, Mapping
    ):
        return ["executable identity records are malformed"]
    for adapter in sorted(set(current_executables) & set(baseline_executables)):
        current_record = current_executables[adapter]
        baseline_record = baseline_executables[adapter]
        if not isinstance(current_record, Mapping) or not isinstance(
            baseline_record, Mapping
        ):
            differences.append(f"executable identity is malformed for {adapter!r}")
            continue
        current_identity = current_record.get("identity_hash")
        baseline_identity = baseline_record.get("identity_hash")
        if current_identity != baseline_identity:
            differences.append(f"executable identity changed for adapter {adapter!r}")
    return differences


def render_report(result: Mapping[str, Any]) -> str:
    """Render a result set as deterministic reviewable Markdown."""
    validate_result_document(result)
    statistics = scenario_statistics(result)
    identity = result["identity"]
    source = result["source"]
    host = result["host_class"]
    harness = result["harness"]
    cpu_logical = host["cpu_logical"] or "unavailable"
    memory_bytes = host["memory_bytes"] or "unavailable"
    evidence_status = (
        "exploratory (dirty source tree)"
        if source["dirty"] is not False
        else "reproducible input record (no claim implied)"
    )
    lines = [
        "# fdu Performance Evidence Report",
        "",
        *_paragraph(
            "**No performance claim is established by this report.** It summarizes "
            "one validated raw result set; release claims require the complete "
            "dedicated-host matrix and review gates in the active performance plan."
        ),
        "",
        "## Run Identity",
        "",
        f"- Evidence status: {evidence_status}",
        f"- Run id: `{identity['run_id']}`",
        f"- Started: `{identity['started_at_utc']}`",
        f"- Finished: `{identity['finished_at_utc']}`",
        f"- Source revision: `{source['revision'] or 'unavailable'}`",
        f"- Result schema: `{result['schema']}`",
        f"- Result hash: `{result['result_hash']}`",
        f"- Host class: `{host['host_class']}`",
        f"- Platform: `{host['operating_system']} {host['architecture']}`",
        f"- Kernel: `{host['kernel']}`",
        f"- Logical CPUs: `{cpu_logical}`",
        f"- Memory bytes: `{memory_bytes}`",
        f"- Filesystem: `{host['filesystem'] or 'unavailable'}`",
        f"- Harness identity: `{harness['identity_hash']}`",
        f"- Python: `{harness['python_implementation']} {harness['python_version']}`",
        "",
        "## Executables",
        "",
        "| Adapter | Identity SHA-256 | Components |",
        "| --- | --- | --- |",
    ]
    for adapter in sorted(result["executables"]):
        executable = result["executables"][adapter]
        components = ", ".join(
            f"{component['name']}@{component['sha256'][:12]}"
            for component in executable["components"]
        )
        lines.append(
            f"| {_escape(adapter)} | `{executable['identity_hash']}` | "
            f"{_escape(components)} |"
        )
    lines.extend(
        [
        "",
        "## Scenario Statistics",
        "",
        *_paragraph(
            "Only valid timed samples contribute. Warmups and invalid trials remain "
            "in the raw-trial table below."
        ),
        "",
        (
            "| Scenario | Valid | Invalid | Median | MAD | P95 | Min | Max | "
            "Mean | Stddev | CV | Entries/s |"
        ),
        (
            "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | "
            "---: | ---: | ---: | ---: |"
        ),
        ]
    )
    for row in statistics:
        lines.append(
            "| {scenario} | {valid} | {invalid} | {median} | {mad} | {p95} | "
            "{minimum} | {maximum} | {mean} | {stddev} | {cv} | {throughput} |".format(
                scenario=_escape(row["scenario_id"]),
                valid=row["valid_trials"],
                invalid=row["invalid_trials"],
                median=_format_duration(row["median_wall_ns"]),
                mad=_format_duration(row["mad_wall_ns"]),
                p95=_format_p95(row),
                minimum=_format_duration(row["minimum_wall_ns"]),
                maximum=_format_duration(row["maximum_wall_ns"]),
                mean=_format_duration(row["mean_wall_ns"]),
                stddev=_format_duration(row["standard_deviation_wall_ns"]),
                cv=_format_cv(row["coefficient_of_variation"]),
                throughput=_format_throughput(row["entries_per_second"]),
            )
        )

    lines.extend(["", "## Review Triggers", ""])
    triggers = _review_triggers(statistics)
    if triggers:
        for trigger in triggers:
            lines.extend(_bullet(trigger))
    else:
        lines.append("No mechanical review trigger fired for this result set.")

    lines.extend(
        [
            "",
            "## Raw Trials",
            "",
            (
                "| Ordinal | Kind | Scenario | Snapshot | FS cache | Valid | "
                "First output | Wall | Exit | Stdout bytes | Stdout SHA-256 | Reason |"
            ),
            (
                "| ---: | --- | --- | --- | --- | --- | ---: | ---: | ---: | "
                "---: | --- | --- |"
            ),
        ]
    )
    for trial in result["trials"]:
        reasons = "<br>".join(
            _escape(reason) for reason in trial["validation"]["reasons"]
        )
        lines.append(
            "| {ordinal} | {kind} | {scenario} | {snapshot} | {cache} | {valid} | "
            "{first_output} | {wall} | {exit_code} | {stdout_bytes} | "
            "`{digest}` | {reasons} |".format(
                ordinal=trial["ordinal"],
                kind=trial["sample_kind"],
                scenario=_escape(trial["scenario_id"]),
                snapshot=trial["snapshot_state"],
                cache=trial["filesystem_cache_state"],
                valid="yes" if trial["validation"]["valid"] else "no",
                first_output=_format_duration(trial["timing"]["first_output_ns"]),
                wall=_format_duration(trial["timing"]["wall_ns"]),
                exit_code=(
                    trial["process"]["exit_code"]
                    if trial["process"]["exit_code"] is not None
                    else "n/a"
                ),
                stdout_bytes=trial["output"]["stdout_bytes"],
                digest=trial["output"]["stdout_sha256"] or "unavailable",
                reasons=reasons or "—",
            )
        )

    lines.extend(
        [
            "",
            "## Reproduction Contract",
            "",
            f"- Invocation ordering: `{result['method']['order_algorithm']}`",
            f"- Order seed: `{result['method']['order_seed']}`",
            "- Snapshot state and filesystem-cache state are recorded per raw trial.",
            *_bullet(
                "Exact commands use tokenized argument arrays; personal absolute "
                "paths are not persisted."
            ),
            "- Invalid samples are retained and are never included in the statistics.",
            "",
            "<!-- This document follows common-doc-guidelines.md.",
            "See github.com/jlevy/practical-prose and review guidelines before editing.",
            "-->",
            "",
        ]
    )
    return "\n".join(lines)


def _review_triggers(statistics: Sequence[Mapping[str, Any]]) -> List[str]:
    triggers: List[str] = []
    for row in statistics:
        scenario_id = row["scenario_id"]
        if row["invalid_trials"]:
            triggers.append(
                f"{scenario_id}: {row['invalid_trials']} timed trial(s) were invalid."
            )
        coefficient = row["coefficient_of_variation"]
        if coefficient is not None and coefficient > 0.10:
            triggers.append(
                f"{scenario_id}: coefficient of variation is "
                f"{coefficient * 100:.2f}%, above the 10% investigation threshold."
            )
        if row["valid_trials"] < 10:
            triggers.append(
                f"{scenario_id}: {row['valid_trials']} valid timed trial(s) are below "
                "the release-headline minimum of 10."
            )
    return triggers


def render_report_file(result_path: Path) -> str:
    """Load and render one result file."""
    return render_report(load_result(result_path))


def write_report(result_path: Path, output_path: Path) -> None:
    """Write a report without replacing an existing reviewed artifact."""
    rendered = render_report_file(result_path)
    with output_path.open("x", encoding="utf-8", newline="\n") as output:
        output.write(rendered)
        output.flush()
        os.fsync(output.fileno())


def _statistics_row(
    scenario_id: str,
    walls: Sequence[int],
    *,
    invalid_trials: int,
    entry_count: Optional[int],
) -> Dict[str, Any]:
    if not walls:
        return {
            "coefficient_of_variation": None,
            "entries_per_second": None,
            "invalid_trials": invalid_trials,
            "mad_wall_ns": None,
            "maximum_wall_ns": None,
            "mean_wall_ns": None,
            "median_wall_ns": None,
            "minimum_wall_ns": None,
            "p95_reason": "no valid timed trials",
            "p95_wall_ns": None,
            "scenario_id": scenario_id,
            "standard_deviation_wall_ns": None,
            "valid_trials": 0,
        }
    median = statistics_module.median(walls)
    deviations = [abs(value - median) for value in walls]
    mad = statistics_module.median(deviations)
    mean = statistics_module.fmean(walls)
    standard_deviation = statistics_module.pstdev(walls)
    coefficient = standard_deviation / mean if mean != 0 else None
    if len(walls) >= 20:
        sorted_walls = sorted(walls)
        p95 = sorted_walls[math.ceil(0.95 * len(sorted_walls)) - 1]
        p95_reason = None
    else:
        p95 = None
        p95_reason = "requires at least 20 valid trials"
    throughput = None
    if entry_count is not None and median > 0:
        throughput = entry_count * 1_000_000_000 / median
    return {
        "coefficient_of_variation": coefficient,
        "entries_per_second": throughput,
        "invalid_trials": invalid_trials,
        "mad_wall_ns": mad,
        "maximum_wall_ns": max(walls),
        "mean_wall_ns": mean,
        "median_wall_ns": median,
        "minimum_wall_ns": min(walls),
        "p95_reason": p95_reason,
        "p95_wall_ns": p95,
        "scenario_id": scenario_id,
        "standard_deviation_wall_ns": standard_deviation,
        "valid_trials": len(walls),
    }


def _entry_count(trials: Sequence[Mapping[str, Any]]) -> Optional[int]:
    for trial in trials:
        corpus = trial.get("corpus")
        if isinstance(corpus, Mapping):
            counts = corpus.get("counts")
            if isinstance(counts, Mapping) and isinstance(counts.get("total"), int):
                return int(counts["total"])
    return None


def _scenario_map(value: Any) -> Dict[str, Mapping[str, Any]]:
    if not isinstance(value, list):
        return {}
    return {
        scenario["id"]: scenario
        for scenario in value
        if isinstance(scenario, Mapping) and isinstance(scenario.get("id"), str)
    }


def _compare_scenario_contract(
    scenario_id: str,
    current: Mapping[str, Any],
    baseline: Mapping[str, Any],
    reasons: List[str],
) -> None:
    fields = (
        "adapter",
        "command",
        "corpus",
        "filesystem_cache_state",
        "job",
        "output_sink",
        "preparation",
        "process_state",
        "snapshot_state",
        "validation",
    )
    for field in fields:
        if current.get(field) != baseline.get(field):
            reasons.append(f"scenario {scenario_id!r} {field} differs")
    current_method = current.get("method", {})
    baseline_method = baseline.get("method", {})
    for field in ("order_group", "timeout_seconds"):
        if current_method.get(field) != baseline_method.get(field):
            reasons.append(f"scenario {scenario_id!r} method.{field} differs")


def _executable_shape(value: Any) -> Any:
    if not isinstance(value, Mapping):
        return None
    components = value.get("components")
    if not isinstance(components, list):
        return None
    return tuple(
        component.get("name") if isinstance(component, Mapping) else None
        for component in components
    )


def _corpus_contracts(result: Mapping[str, Any]) -> Dict[str, Any]:
    contracts: Dict[str, Any] = {}
    trials = result.get("trials")
    if not isinstance(trials, list):
        return contracts
    for trial in trials:
        if not isinstance(trial, Mapping) or trial.get("sample_kind") != "timed":
            continue
        scenario_id = trial.get("scenario_id")
        corpus = trial.get("corpus")
        if not isinstance(scenario_id, str) or not isinstance(corpus, Mapping):
            continue
        contract = {
            "counts": corpus.get("counts"),
            "recipe_hash": corpus.get("recipe_hash"),
            "recipe_id": corpus.get("recipe_id"),
            "semantic_digest": corpus.get("semantic_digest"),
            "state": corpus.get("state"),
        }
        existing = contracts.get(scenario_id)
        if existing is None:
            contracts[scenario_id] = contract
        elif existing != contract:
            contracts[scenario_id] = "inconsistent-within-result"
    return contracts


def _collector_shape(result: Mapping[str, Any]) -> Dict[str, str]:
    trials = result.get("trials")
    if not isinstance(trials, list) or not trials:
        return {}
    observations: Dict[str, List[bool]] = {}
    for trial in trials:
        if not isinstance(trial, Mapping) or trial.get("sample_kind") != "timed":
            continue
        resources = trial.get("resources")
        if not isinstance(resources, Mapping):
            continue
        for key, value in resources.items():
            if key != "reason":
                observations.setdefault(key, []).append(value is not None)
    shape: Dict[str, str] = {}
    for key, values in observations.items():
        if all(values):
            shape[key] = "all"
        elif any(values):
            shape[key] = "partial"
        else:
            shape[key] = "none"
    return shape


def _format_duration(value: Optional[float]) -> str:
    return "n/a" if value is None else f"{value / 1_000_000:.3f} ms"


def _format_p95(row: Mapping[str, Any]) -> str:
    value = row["p95_wall_ns"]
    if value is not None:
        return _format_duration(value)
    reason = row["p95_reason"]
    return f"n/a ({_escape(str(reason))})"


def _format_cv(value: Optional[float]) -> str:
    return "n/a" if value is None else f"{value * 100:.2f}%"


def _format_throughput(value: Optional[float]) -> str:
    return "n/a" if value is None else f"{value:,.1f}"


def _escape(value: str) -> str:
    return value.replace("|", "\\|").replace("\n", " ")


def _paragraph(value: str) -> List[str]:
    return textwrap.wrap(
        value, width=88, break_long_words=False, break_on_hyphens=False
    )


def _bullet(value: str) -> List[str]:
    return textwrap.wrap(
        value,
        width=88,
        initial_indent="- ",
        subsequent_indent="  ",
        break_long_words=False,
        break_on_hyphens=False,
    )
