"""Aggregate the experiment artifacts into one ledger report.

    python -m benchmarks.realtree.summary --out docs/project/reports/....md

Every number here is read back out of the committed experiment artifacts, never
retyped, so the report cannot drift from the record. The artifacts are the source of
truth; this is a view over them.

Reading them goes through the softschema CLI rather than a YAML parser. That keeps
the harness free of a YAML dependency, and more usefully it means the report is built
from *validated* payloads: an artifact that no longer matches the contract fails here
instead of quietly contributing a wrong row.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, List, Mapping, Sequence

EXPERIMENTS_DIR = Path("docs/project/experiments")
DEFAULT_OUT = Path("docs/project/reports/report-2026-08-10-fdu-performance-experiments.md")

#: Metrics shown per experiment, and how to render them.
METRIC_COLUMNS = (
    ("wall_ns", "wall", "ms"),
    ("component_ns", "component", "ms"),
    ("cpu_ns", "cpu", "ms"),
    ("user_cpu_ns", "user", "ms"),
    ("system_cpu_ns", "system", "ms"),
    ("blocked_ns", "blocked", "ms"),
    ("peak_rss_bytes", "peak rss", "MiB"),
)

_DECISION_MARK = {
    "accepted": "✅ accepted",
    "rejected": "❌ rejected",
    "blocked": "⛔ blocked",
    "superseded": "↩︎ superseded",
    "in-progress": "⏳ in progress",
    "baseline": "📏 baseline",
}

_PARALLEL_CPU_JOBS = {
    "cold-scan-index",
    "cold-scan-producer",
    "cold-scan-producer-raw",
    "cold-snapshot-save",
    "warm-revalidate",
}


class SummaryError(RuntimeError):
    """An experiment artifact could not be read or did not validate."""


def load_experiments(directory: Path) -> List[Dict[str, Any]]:
    experiments: List[Dict[str, Any]] = []
    for path in sorted(directory.glob("exp-*.md")):
        payload = _read(path)
        payload["_path"] = str(path)
        experiments.append(payload)
    return experiments


def _read(path: Path) -> Dict[str, Any]:
    completed = subprocess.run(
        ["softschema", "validate", str(path)],
        capture_output=True,
        timeout=600,
    )
    stdout = completed.stdout.decode("utf-8", errors="replace")
    try:
        document = json.loads(stdout)
    except json.JSONDecodeError as error:
        raise SummaryError(f"{path}: softschema produced no JSON: {error}") from error
    if document.get("outcome") != "valid":
        errors = document.get("structural", {}).get("errors") or []
        detail = "; ".join(str(item.get("message")) for item in errors[:3])
        raise SummaryError(f"{path}: artifact does not satisfy its contract: {detail}")
    # `values` is the payload already unwrapped from its envelope, so it is the
    # experiment itself rather than a mapping containing one.
    payload = document.get("values")
    if not isinstance(payload, dict) or "verdict" not in payload:
        raise SummaryError(f"{path}: no experiment payload")
    _verify_archived_evidence(payload, path)
    return payload


def _verify_archived_evidence(payload: Mapping[str, Any], artifact: Path) -> None:
    """Claim-grade records must resolve to the exact committed raw sample bundle."""
    method = payload.get("method") or {}
    if method.get("evidence_grade") != "claim-grade":
        return
    display_path = method.get("run_artifact")
    expected_digest = method.get("run_artifact_sha256")
    if not display_path or not expected_digest:
        raise SummaryError(f"{artifact}: claim-grade record has no archived run digest")
    path = Path(display_path)
    if path.is_absolute() or ".." in path.parts:
        raise SummaryError(f"{artifact}: archived run path must be repository-relative")
    if path.parts[:3] != ("docs", "project", "experiments") or "evidence" not in path.parts:
        raise SummaryError(f"{artifact}: archived run must live under experiment evidence")
    if not path.is_file():
        raise SummaryError(f"{artifact}: archived run does not resolve: {path}")
    actual_digest = hashlib.sha256(path.read_bytes()).hexdigest()
    if actual_digest != expected_digest:
        raise SummaryError(f"{artifact}: archived run digest does not match {path}")
    run = json.loads(path.read_text(encoding="utf-8"))
    if run.get("schema") != method.get("run_schema"):
        raise SummaryError(f"{artifact}: archived run schema does not match frontmatter")
    conditions = run.get("conditions") or {}
    if conditions.get("schedule_sha256") != method.get("schedule_sha256"):
        raise SummaryError(f"{artifact}: archived schedule digest does not match frontmatter")
    if _schedule_sha256(run.get("samples") or []) != conditions.get("schedule_sha256"):
        raise SummaryError(f"{artifact}: archived sample order does not match its schedule digest")
    if conditions.get("schedule") != method.get("schedule"):
        raise SummaryError(f"{artifact}: archived schedule algorithm does not match frontmatter")
    for field_name in ("trials", "warmups"):
        if conditions.get(field_name) != method.get(field_name):
            raise SummaryError(f"{artifact}: archived {field_name} does not match frontmatter")
    if (run.get("host") or {}).get("toolchain") != method.get("toolchain"):
        raise SummaryError(f"{artifact}: archived toolchain does not match frontmatter")

    subject = payload.get("subject") or {}
    tree = run.get("tree") or {}
    if tree.get("engine_digest") != subject.get("tree_engine_digest"):
        raise SummaryError(f"{artifact}: archived tree digest does not match frontmatter")
    if bool(run.get("tree_mutated_during_run")) != bool(
        subject.get("tree_mutated_during_run")
    ):
        raise SummaryError(f"{artifact}: archived tree-mutation state does not match frontmatter")
    if run.get("baseline_drift"):
        raise SummaryError(f"{artifact}: claim-grade run drifted from its declared baseline")

    variants = run.get("variants") or {}
    for role in ("control", "candidate"):
        binary = method.get(f"{role}_binary") or {}
        identity = variants.get(binary.get("name")) or {}
        for field_name in (
            "sha256",
            "size_bytes",
            "args",
            "engine_revision",
            "harness_revision",
            "harness_sha256",
            "target",
            "build_profile",
            "features",
            "build_command",
        ):
            if identity.get(field_name) != binary.get(field_name):
                raise SummaryError(
                    f"{artifact}: archived {role} {field_name} does not match frontmatter"
                )


def _schedule_sha256(samples: Sequence[Mapping[str, Any]]) -> str:
    rows = [
        {
            "job": sample.get("job"),
            "ordinal": sample.get("ordinal"),
            "variant": sample.get("variant"),
            "warmup": sample.get("warmup"),
        }
        for sample in samples
    ]
    encoded = json.dumps(rows, separators=(",", ":"), sort_keys=True).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def render(experiments: Sequence[Mapping[str, Any]]) -> str:
    lines: List[str] = []
    lines.append("# fdu Performance Experiment Ledger")
    lines.append("")
    lines.append(
        "Generated by `make perf-ledger` from the experiment artifacts in "
        "[docs/project/experiments/](../experiments/). Do not edit by hand."
    )
    lines.append("")
    lines.append(
        "Each experiment is one soft-schema artifact: validated YAML frontmatter "
        "carries the measured numbers, and the Markdown body carries the reasoning a "
        "schema cannot hold. This page rolls them up. For an experiment's full "
        "per-job, per-metric detail, follow its link."
    )
    lines.append("")
    lines.append(
        "How the numbers were produced, what each metric means, and the rule that "
        "decides whether a change is kept: "
        "[the performance loop guide](../guides/performance-loop.md)."
    )
    lines.append("")

    lines.extend(_headline(experiments))
    lines.extend(_conditions(experiments))
    lines.extend(_index(experiments))

    lines.append("## The experiments")
    lines.append("")
    for experiment in experiments:
        lines.extend(_section(experiment))

    lines.append("")
    lines.append("<!-- This document follows common-doc-guidelines.md.")
    lines.append("See github.com/jlevy/practical-prose and review guidelines before editing.")
    lines.append("-->")
    return "\n".join(lines) + "\n"


def _headline(experiments: Sequence[Mapping[str, Any]]) -> List[str]:
    """Where things ended up, taken from the most recent cumulative experiment.

    A cumulative experiment measures today's build against the pre-work baseline in
    one interleaved run, so it is the only comparison that can honestly be called a
    total. Adding up the individual experiments would not be: each was measured
    against a different control on a differently loaded machine.
    """
    cumulative = [
        item
        for item in experiments
        if "cumulative" in item["title"].lower()
        and item.get("method", {}).get("evidence_grade") == "claim-grade"
    ]
    if not cumulative:
        return [
            "## Where it stands",
            "",
            "No claim-grade cumulative comparison is currently published. Legacy aggregate "
            "records remain below for audit history, not as current performance claims.",
            "",
        ]
    latest = cumulative[-1]
    lines = ["## Where it stands", ""]
    lines.append(
        f"The frozen candidate was measured against its declared true base in one "
        f"interleaved run of {latest['method']['trials']} paired trials "
        f"({latest['id']}, {latest['verdict']['decision']})."
    )
    lines.append("")
    lines.append("| job | before | after | change | 95% interval |")
    lines.append("| --- | ---: | ---: | ---: | --- |")
    for result in latest.get("results", []):
        entry = (result.get("metrics") or {}).get("wall_ns")
        if not entry:
            continue
        low, high = entry.get("ci95_low_pct"), entry.get("ci95_high_pct")
        lines.append(
            f"| `{result['job']}` "
            f"| {entry['control_median'] / 1e6:.0f} ms "
            f"| {entry['candidate_median'] / 1e6:.0f} ms "
            f"| **{entry['change_pct']:+.1f}%** "
            + (f"| [{low:+.1f}%, {high:+.1f}%] |" if low is not None else "| — |")
        )
    references = latest.get("reference_tools") or []
    if references:
        lines.append("")
        lines.append(
            "Third-party tools on the same tree, for calibration only — they answer a "
            "slightly different question with different guarantees, and never enter "
            "the accept rule: "
            + ", ".join(
                f"`{item['name']}` {item['wall_ns_median'] / 1e6:.0f} ms"
                for item in references
            )
            + "."
        )
    lines.append("")
    return lines


def _conditions(experiments: Sequence[Mapping[str, Any]]) -> List[str]:
    if not experiments:
        return []
    subject = experiments[-1]["subject"]
    method = experiments[-1]["method"]
    lines = ["## Reproducing this", ""]
    lines.append("**The tree.** Pinned by content, not by name.")
    lines.append("")
    lines.append(
        f"- Label `{subject['tree_label']}`, "
        f"{subject['tree_entries']:,} entries "
        f"({subject['tree_directories']:,} directories, "
        f"{subject['tree_files']:,} files, {subject['tree_symlinks']} symlinks), "
        f"max depth {subject['tree_max_depth']}."
    )
    lines.append(
        f"- {subject['tree_apparent_bytes'] / 2**30:.2f} GiB apparent, "
        f"{subject['tree_allocated_bytes'] / 2**30:.2f} GiB allocated."
    )
    if subject.get("tree_engine_digest"):
        lines.append(
            f"- Content digest `{subject['tree_engine_digest']}` "
            "(`fdu-index-record-v1`). Two trees with this digest are the same tree "
            "in the same state."
        )
    lines.append(
        f"- Archived identity `{subject['tree_root_id'][:16]}…` is derived from the "
        "content digest; the operator's path hash is deliberately removed from the "
        "committed evidence."
    )
    lines.append("")
    lines.append("**The machine.**")
    lines.append("")
    core_detail = ""
    if subject.get("host_performance_cores"):
        core_detail = (
            f" ({subject['host_performance_cores']} performance, "
            f"{subject['host_efficiency_cores']} efficiency)"
        )
    machine = [subject["host_cpu"]]
    if subject.get("host_arch"):
        machine.append(subject["host_arch"])
    machine.append(f"{subject['host_cores']} logical cores{core_detail}")
    if subject.get("host_memory_bytes"):
        machine.append(f"{subject['host_memory_bytes'] / 2**30:.0f} GiB RAM")
    lines.append("- " + ", ".join(machine) + ".")
    lines.append(f"- {subject['host_system']}, {subject['filesystem']} filesystem.")
    if method.get("toolchain"):
        lines.append(f"- Built with {method['toolchain']}, `{method['build_profile']}` profile.")
    lines.append(
        f"- OS page cache: {subject['os_cache']}. Dropping it needs root, so runs "
        "that did not ask for that say so rather than implying a cold disk."
    )
    lines.append("")
    mutated = [item["id"] for item in experiments if item["subject"]["tree_mutated_during_run"]]
    if mutated:
        lines.append(
            f"**The tree changed mid-run for {', '.join(mutated)}; those numbers are "
            "not comparable.**"
        )
    elif any(
        item.get("method", {}).get("evidence_grade") == "claim-grade"
        for item in experiments
    ):
        lines.append(
            "Claim-grade runs fingerprinted the tree before and after, preserved the "
            "raw paired samples, and checked every trial's entry and per-directory "
            "roll-up digests against an independent oracle. Legacy v1 runs below did "
            "not satisfy that full contract and are labeled accordingly."
        )
    else:
        lines.append(
            "All records are legacy evidence; they predate the full roll-up oracle and "
            "committed provenance contract and are not current performance claims."
        )
    lines.append("")
    grade = method.get("evidence_grade", "legacy")
    lines.append(f"Evidence grade: **{grade}**.")
    lines.append("")
    return lines


def _index(experiments: Sequence[Mapping[str, Any]]) -> List[str]:
    lines = [
        "## Every experiment, including the failures",
        "",
        "The rejected ones are the reusable part: they stop the next person spending a "
        "day on a dead end.",
        "",
        "| # | experiment | tests | primary job | change | verdict |",
        "| --- | --- | --- | --- | ---: | --- |",
    ]
    for experiment in experiments:
        verdict = experiment["verdict"]
        change = verdict.get("change_pct")
        lines.append(
            f"| {experiment['id'].removeprefix('exp-')} "
            f"| [{experiment['title']}](#{_anchor(experiment)}) "
            f"| {', '.join(experiment['hypotheses']) or '—'} "
            f"| `{verdict['primary_job']}` "
            f"| {f'{change:+.1f}%' if change else '—'} "
            f"| {_DECISION_MARK.get(verdict['decision'], verdict['decision'])} |"
        )
    lines.append("")
    return lines


def _section(experiment: Mapping[str, Any]) -> List[str]:
    verdict = experiment["verdict"]
    method = experiment["method"]
    is_baseline = verdict["decision"] == "baseline"
    lines = [f"### {experiment['id']} — {experiment['title']}", ""]
    lines.append(
        f"{_DECISION_MARK.get(verdict['decision'], verdict['decision'])} "
        f"· {experiment['date']} "
        f"· {', '.join(experiment['hypotheses']) or 'no hypothesis id'}"
        + (f" · commit `{verdict['commit']}`" if verdict.get("commit") else "")
    )
    lines.append("")
    if not is_baseline:
        lines.append(f"Control: {method['control']}")
        lines.append("")
        lines.append(f"Candidate: {method['candidate']}")
        lines.append("")

    # The primary job in full, since that is what the decision rests on. Every other
    # job gets one line, and the artifact has the complete detail for all of them.
    primary = next(
        (r for r in experiment.get("results", []) if r["job"] == verdict["primary_job"]),
        None,
    )
    if primary and primary.get("metrics"):
        lines.append(
            f"**`{primary['job']}`** ({primary['start_state']} start) — "
            + ("measured" if is_baseline else "the comparison the verdict rests on")
        )
        lines.append("")
        if is_baseline:
            lines.append("| metric | value |")
            lines.append("| --- | ---: |")
        else:
            lines.append("| metric | control | candidate | change | 95% interval |")
            lines.append("| --- | ---: | ---: | ---: | --- |")
        for key, label, unit in METRIC_COLUMNS:
            if key == "blocked_ns" and primary["job"] in _PARALLEL_CPU_JOBS:
                continue
            entry = primary["metrics"].get(key)
            if not entry:
                continue
            if is_baseline:
                lines.append(f"| {label} ({unit}) | {_value(entry['control_median'], unit)} |")
                continue
            low, high = entry.get("ci95_low_pct"), entry.get("ci95_high_pct")
            lines.append(
                f"| {label} ({unit}) "
                f"| {_value(entry['control_median'], unit)} "
                f"| {_value(entry['candidate_median'], unit)} "
                f"| {entry['change_pct']:+.2f}%{_statistical_suffix(entry)} "
                + (f"| [{low:+.2f}%, {high:+.2f}%] |" if low is not None else "| — |")
            )
        lines.append("")
        if primary["job"] in _PARALLEL_CPU_JOBS:
            lines.append(
                "Blocked time is omitted: aggregate process CPU across worker threads "
                "cannot be subtracted from wall time as an off-CPU measurement."
            )
            lines.append("")

    others = [
        r
        for r in experiment.get("results", [])
        if r["job"] != verdict["primary_job"] and (r.get("metrics") or {}).get("wall_ns")
    ]
    if others:
        summaries = []
        for result in others:
            entry = result["metrics"]["wall_ns"]
            if is_baseline:
                summaries.append(
                    f"`{result['job']}` {entry['control_median'] / 1e6:.0f} ms"
                )
            else:
                mark = _statistical_suffix(entry)
                summaries.append(
                    f"`{result['job']}` {entry['change_pct']:+.1f}%{mark}"
                )
        lines.append("Other jobs, wall time: " + ", ".join(summaries) + ".")
        lines.append("")

    rejected = sum(r.get("invalid_samples", 0) for r in experiment.get("results", []))
    if rejected:
        lines.append(
            f"**{rejected} samples rejected by the independent oracle. Those jobs "
            "prove nothing.**"
        )
        lines.append("")

    if not is_baseline:
        latency_gate = verdict.get("latency_gate_passed")
        if latency_gate is not None:
            lines.append(
                "Latency gate: " + ("**passed**." if latency_gate else "**failed**.")
            )
            lines.append("")
        guardrails = verdict.get("resource_guardrails") or []
        if guardrails:
            lines.append("Resource guardrails:")
            lines.append("")
            for guardrail in guardrails:
                change = guardrail.get("observed_change_pct")
                observed = "unmeasured" if change is None else f"{change:+.2f}%"
                lines.append(
                    f"- `{guardrail['metric']}`: **{guardrail['status']}** "
                    f"(observed {observed}; limit "
                    f"+{guardrail['maximum_regression_pct']:g}%) — "
                    f"{guardrail['reason']}."
                )
            lines.append("")
        complexity = experiment["complexity"]
        cost = [f"{complexity['lines_changed']} lines"]
        cost.append(
            "dependencies: " + ", ".join(complexity["new_dependencies"])
            if complexity.get("new_dependencies")
            else "no new dependencies"
        )
        if complexity.get("new_unsafe_blocks"):
            cost.append(f"{complexity['new_unsafe_blocks']} unsafe blocks")
        for mode in complexity.get("new_failure_modes") or []:
            cost.append(f"new failure mode: {mode}")
        lines.append(f"Cost to carry: {'; '.join(cost)}.")
        lines.append("")
        if complexity.get("notes"):
            lines.append(complexity["notes"])
            lines.append("")

    lines.append(f"**{verdict['decision'].capitalize()}:** {verdict['reason']}.")
    lines.append("")
    name = Path(experiment["_path"]).name
    lines.append(f"Full record: [`{name}`](../experiments/{name})")
    if method.get("run_artifact"):
        raw_name = Path(method["run_artifact"]).name
        lines.append(
            f"Raw paired samples: [`{raw_name}`](../experiments/evidence/{raw_name})"
        )
    lines.append("")
    return lines


def _statistical_suffix(entry: Mapping[str, Any]) -> str:
    """Render significance without treating a clear regression as noise."""
    change = entry.get("change_pct")
    direction = entry.get("direction") or (
        "improvement" if change is not None and change < 0
        else "regression" if change is not None and change > 0
        else "unchanged"
    )
    ci_excludes_zero = entry.get("ci_excludes_zero")
    if ci_excludes_zero is None:
        low = entry.get("ci95_low_pct")
        high = entry.get("ci95_high_pct")
        ci_excludes_zero = (
            low is not None and high is not None and (high < 0 or low > 0)
        )
    if not ci_excludes_zero:
        return " (n.s.)"
    if direction == "regression":
        return " (significant regression)"
    return ""


def _value(value: float, unit: str) -> str:
    if unit == "ms":
        return f"{value / 1e6:.1f}"
    if unit == "MiB":
        return f"{value / 2**20:.1f}"
    return f"{value:,.0f}"


def _anchor(experiment: Mapping[str, Any]) -> str:
    text = f"{experiment['id']} — {experiment['title']}".lower()
    kept = [character if character.isalnum() or character == " " else "" for character in text]
    return "".join(kept).replace(" ", "-")


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(prog="benchmarks.realtree.summary", description=__doc__)
    parser.add_argument("--experiments", type=Path, default=EXPERIMENTS_DIR)
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT)
    arguments = parser.parse_args(list(argv))

    experiments = load_experiments(arguments.experiments)
    if not experiments:
        print("no experiment artifacts found", file=sys.stderr)
        return 1
    arguments.out.parent.mkdir(parents=True, exist_ok=True)
    arguments.out.write_text(render(experiments), encoding="utf-8")
    print(f"wrote {arguments.out} from {len(experiments)} experiments", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
