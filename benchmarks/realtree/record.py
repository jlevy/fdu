"""Write a measurement run out as a soft-schema experiment artifact.

    python -m benchmarks.realtree record \\
      --run benchmarks/results/realtree/run-exp002-....json \\
      --id exp-002 --title "Parallel revalidation sweep" \\
      --hypothesis H9 --decision rejected \\
      --control "exp001 parallel scan" --candidate "parallel revalidation too" \\
      --reason "-2.59% is real but under the 3% bar for 180 lines of concurrency" \\
      --lines-changed 180

The measured half of the artifact is read from the run, never retyped. The judged
half — hypothesis, complexity, decision, reasoning — comes from the operator, because
those are the parts a measurement cannot supply.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, List, Mapping, Sequence

from benchmarks.realtree import evidence, experiment as experiment_model

EXPERIMENTS_DIR = Path("docs/project/experiments")
SCHEMA_NAME = "experiment.schema.yaml"


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(prog="benchmarks.realtree record", description=__doc__)
    parser.add_argument("--run", required=True, type=Path)
    parser.add_argument("--id", required=True, help="exp-NNN")
    parser.add_argument("--title", required=True)
    parser.add_argument("--hypothesis", action="append", default=[], dest="hypotheses")
    parser.add_argument("--control", required=True)
    parser.add_argument("--candidate", required=True)
    parser.add_argument(
        "--decision",
        required=True,
        choices=(
            "accepted", "rejected", "superseded", "blocked", "in-progress", "baseline"
        ),
    )
    parser.add_argument("--primary-job", required=True)
    parser.add_argument("--primary-metric", default="wall_ns")
    parser.add_argument("--reason", required=True)
    parser.add_argument("--commit", default=None)
    parser.add_argument("--lines-changed", type=int, default=0)
    parser.add_argument("--new-dependency", action="append", default=[])
    parser.add_argument("--new-unsafe", type=int, default=0)
    parser.add_argument("--failure-mode", action="append", default=[])
    parser.add_argument("--complexity-note", default="")
    parser.add_argument("--control-variant", default=None)
    parser.add_argument("--candidate-variant", default=None)
    parser.add_argument("--max-cpu-regression-pct", type=float, default=10.0)
    parser.add_argument("--max-rss-regression-pct", type=float, default=10.0)
    parser.add_argument(
        "--resource-waiver",
        default=None,
        help="Product rationale for accepting a failed or unmeasured CPU/RSS guardrail.",
    )
    parser.add_argument("--body", type=Path, help="Markdown body file; a stub is written otherwise")
    parser.add_argument("--output-dir", type=Path, default=EXPERIMENTS_DIR)
    parser.add_argument(
        "--evidence-grade",
        choices=("claim-grade", "legacy"),
        default="claim-grade",
    )
    parser.add_argument(
        "--evidence-tree-label",
        default="reference-tree",
        help="Public, path-free label stored in the archived raw run.",
    )
    parser.add_argument("--no-validate", action="store_true")
    arguments = parser.parse_args(list(argv))

    run = json.loads(arguments.run.read_text(encoding="utf-8"))
    try:
        headline = _headline(run, arguments)
        verdict_evidence = _verdict_evidence(run, arguments, headline)
    except ValueError as error:
        parser.error(str(error))
    evidence_path = arguments.output_dir / "evidence" / f"{arguments.id}-run.json"
    try:
        run, evidence_digest = evidence.archive_run(
            run,
            destination=evidence_path,
            tree_label=arguments.evidence_tree_label,
        )
    except evidence.EvidenceError as error:
        parser.error(str(error))

    payload = experiment_model.from_run(
        run,
        experiment_id=arguments.id,
        title=arguments.title,
        hypotheses=arguments.hypotheses,
        control=arguments.control,
        candidate=arguments.candidate,
        run_artifact=_display_path(evidence_path),
        run_artifact_sha256=evidence_digest,
        evidence_grade=arguments.evidence_grade,
        control_variant=arguments.control_variant,
        candidate_variant=arguments.candidate_variant,
        complexity={
            "lines_changed": arguments.lines_changed,
            "new_dependencies": arguments.new_dependency,
            "new_unsafe_blocks": arguments.new_unsafe,
            "new_failure_modes": arguments.failure_mode,
            "notes": arguments.complexity_note,
        },
        verdict={
            "decision": arguments.decision,
            "primary_job": arguments.primary_job,
            "primary_metric": arguments.primary_metric,
            "change_pct": headline,
            "reason": arguments.reason,
            "commit": arguments.commit,
            **verdict_evidence,
        },
    )
    try:
        experiment_model.Experiment.model_validate(payload)
    except ValueError as error:
        parser.error(str(error))

    body = (
        arguments.body.read_text(encoding="utf-8")
        if arguments.body
        else _stub_body(payload)
    )
    destination = arguments.output_dir / f"{arguments.id}-{_slug(arguments.title)}.md"
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(_render(payload, body), encoding="utf-8")
    print(f"wrote {destination}", file=sys.stderr)

    if not arguments.no_validate:
        return _validate(destination)
    return 0


def _headline(run: Mapping[str, Any], arguments: argparse.Namespace) -> Any:
    """The change for *this* experiment's control and candidate.

    A sweep run holds a comparison per variant, so taking whichever one came first
    would report the wrong pair — the two-thread result for a four-thread experiment,
    say. The recorded variant names are the ones that decide.
    """
    comparison = _selected_comparison(run, arguments)
    if comparison is None:
        return None
    entry = comparison["metrics"].get(arguments.primary_metric)
    if entry:
        return entry["median_change_pct"]
    return None


def _selected_comparison(
    run: Mapping[str, Any], arguments: argparse.Namespace
) -> Any:
    """Return exactly the comparison named by the recording arguments."""
    statistics = run["statistics"].get(arguments.primary_job)
    if not statistics:
        return None
    comparisons = statistics["comparisons"]
    explicit = bool(arguments.control_variant) or bool(arguments.candidate_variant)
    if explicit and not (
        arguments.control_variant and arguments.candidate_variant
    ):
        raise ValueError(
            "--control-variant and --candidate-variant must be supplied together"
        )
    if len(comparisons) > 1 and not explicit:
        raise ValueError(
            "a run with multiple comparisons requires --control-variant and "
            "--candidate-variant"
        )
    if explicit:
        key = f"{arguments.candidate_variant}_vs_{arguments.control_variant}"
        comparison = comparisons.get(key)
        if comparison is None:
            raise ValueError(f"run has no comparison named {key!r}")
    elif len(comparisons) == 1:
        comparison = next(iter(comparisons.values()))
    else:
        return None
    return comparison


def _verdict_evidence(
    run: Mapping[str, Any], arguments: argparse.Namespace, headline: Any
) -> Dict[str, Any]:
    """Evaluate latency and resource costs as two explicit decision gates."""
    if arguments.decision == "baseline":
        return {
            "latency_gate_passed": None,
            "resource_guardrails": [],
            "resource_waiver_reason": None,
        }

    comparison = _selected_comparison(run, arguments)
    primary = (
        (comparison or {}).get("metrics", {}).get(arguments.primary_metric) or {}
    )
    primary_interval = primary.get("ci95_change_pct") or [None, None]
    significant_improvement = primary.get("significant_improvement")
    if significant_improvement is None:
        significant_improvement = bool(
            primary_interval[1] is not None and primary_interval[1] < 0
        )
    latency_gate_passed = bool(
        significant_improvement
        and headline is not None
        and headline <= -3.0
    )
    if arguments.decision == "accepted" and not latency_gate_passed:
        raise ValueError(
            "an accepted verdict requires a significant primary-metric improvement "
            "of at least 3%"
        )

    guardrails = [
        _resource_guardrail(
            comparison,
            metric="cpu_ns",
            maximum_regression_pct=arguments.max_cpu_regression_pct,
        ),
        _resource_guardrail(
            comparison,
            metric="peak_rss_bytes",
            maximum_regression_pct=arguments.max_rss_regression_pct,
        ),
    ]
    disqualifying = [
        guardrail
        for guardrail in guardrails
        if guardrail["status"] in {"failed", "not-measured"}
    ]
    waiver = arguments.resource_waiver
    if arguments.decision == "accepted" and disqualifying and not waiver:
        metrics = ", ".join(guardrail["metric"] for guardrail in disqualifying)
        raise ValueError(
            f"accepted verdict has failed or unmeasured resource guardrails ({metrics}); "
            "supply --resource-waiver with the product rationale"
        )
    if arguments.decision == "accepted" and waiver:
        for guardrail in disqualifying:
            guardrail["status"] = "waived"
            guardrail["reason"] = f"waived: {waiver}"

    return {
        "latency_gate_passed": latency_gate_passed,
        "resource_guardrails": guardrails,
        "resource_waiver_reason": waiver,
    }


def _resource_guardrail(
    comparison: Any, *, metric: str, maximum_regression_pct: float
) -> Dict[str, Any]:
    if maximum_regression_pct < 0:
        raise ValueError(f"{metric} regression threshold must be non-negative")
    entry = (comparison or {}).get("metrics", {}).get(metric)
    if not entry or entry.get("median_change_pct") is None:
        return {
            "metric": metric,
            "maximum_regression_pct": maximum_regression_pct,
            "observed_change_pct": None,
            "ci95_low_pct": None,
            "ci95_high_pct": None,
            "status": "not-measured",
            "reason": "no paired samples",
        }
    interval = entry.get("ci95_change_pct") or [None, None]
    change = entry["median_change_pct"]
    lower = interval[0]
    failed = lower is not None and lower > maximum_regression_pct
    return {
        "metric": metric,
        "maximum_regression_pct": maximum_regression_pct,
        "observed_change_pct": change,
        "ci95_low_pct": lower,
        "ci95_high_pct": interval[1],
        "status": "failed" if failed else "passed",
        "reason": (
            f"95% interval is wholly above the +{maximum_regression_pct:g}% limit"
            if failed
            else f"no statistically established regression above +{maximum_regression_pct:g}%"
        ),
    }


def _render(payload: Mapping[str, Any], body: str) -> str:
    """Emit frontmatter plus body.

    YAML is written by hand rather than through a dumper so the key order matches the
    order a reader wants: what it is, what it ran on, what happened, what we decided.
    An alphabetising dumper would bury the verdict between `method` and `results`.
    """
    lines: List[str] = ["---"]
    lines.append(f"title: {_scalar(payload['title'])}")
    lines.append("softschema:")
    lines.append(f"  contract: {experiment_model.CONTRACT}")
    lines.append(f"  schema: {SCHEMA_NAME}")
    lines.append("  envelope: experiment")
    lines.append("  status: enforced")
    lines.append("experiment:")
    lines.extend(_yaml(payload, indent=2))
    lines.append("---")
    lines.append("")
    return "\n".join(lines) + body.rstrip() + "\n"


def _yaml(value: Any, *, indent: int) -> List[str]:
    pad = " " * indent
    lines: List[str] = []
    if isinstance(value, dict):
        for key, item in value.items():
            if isinstance(item, dict):
                if not item:
                    lines.append(f"{pad}{key}: {{}}")
                    continue
                lines.append(f"{pad}{key}:")
                lines.extend(_yaml(item, indent=indent + 2))
            elif isinstance(item, list):
                if not item:
                    lines.append(f"{pad}{key}: []")
                    continue
                lines.append(f"{pad}{key}:")
                for element in item:
                    if isinstance(element, dict):
                        rendered = _yaml(element, indent=indent + 4)
                        lines.append(f"{pad}  - {rendered[0].strip()}")
                        lines.extend(rendered[1:])
                    else:
                        lines.append(f"{pad}  - {_scalar(element)}")
            else:
                lines.append(f"{pad}{key}: {_scalar(item)}")
    return lines


def _scalar(value: Any) -> str:
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return repr(value)
    text = str(value)
    if _needs_quoting(text):
        escaped = text.replace("\\", "\\\\").replace('"', '\\"')
        return f'"{escaped}"'
    return text


#: Characters that make a plain scalar ambiguous, and would change how YAML parses it.
_YAML_SPECIAL = set(":#{}[]&*!|>'\"%@`,")

#: Plain scalars YAML reads as something other than a string.
_YAML_KEYWORDS = {
    "true", "false", "yes", "no", "on", "off", "null", "none", "~", "y", "n",
}


def _needs_quoting(text: str) -> bool:
    """Whether a string has to be quoted to survive a YAML round trip as a string.

    A string such as ``"1"`` written plainly comes back as the integer 1, which then
    fails the schema for the right reason but the wrong cause. Quoting is decided by
    what YAML would do with the text, not by whether it looks tidy.
    """
    if text == "" or text.strip() != text:
        return True
    if text.lower() in _YAML_KEYWORDS:
        return True
    if (
        len(text) == 10
        and text[4] == "-"
        and text[7] == "-"
        and text[:4].isdigit()
        and text[5:7].isdigit()
        and text[8:].isdigit()
    ):
        # YAML 1.1 readers promote a plain ISO date to a timestamp. The artifact
        # contract deliberately carries a string so validators agree across versions.
        return True
    if any(character in _YAML_SPECIAL for character in text):
        return True
    if text[0] in "-?%":
        return True
    try:
        float(text)
    except ValueError:
        pass
    else:
        # Parses as a number, so it would not come back as a string.
        return True
    return False


def _slug(title: str) -> str:
    kept = [character.lower() if character.isalnum() else "-" for character in title]
    slug = "".join(kept)
    while "--" in slug:
        slug = slug.replace("--", "-")
    return slug.strip("-")[:60]


def _stub_body(payload: Mapping[str, Any]) -> str:
    verdict = payload["verdict"]
    change = verdict.get("change_pct")
    measured = "not comparative" if change is None else f"{change:+.2f}%"
    return f"""# {payload["title"]}

## Hypothesis

This run tested {", ".join(payload["hypotheses"]) or "a baseline measurement"} on the
declared primary job and metric.

## What was tried

Control: {payload["method"]["control"]}

Candidate: {payload["method"]["candidate"]}

## What the numbers said

The paired primary change was {measured}. The archived run contains the exact trial
order, raw samples, invalid-sample reasons, and bootstrap inputs.

## Verdict

**{verdict["decision"].upper()}** — {verdict["reason"]}

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
"""


def _display_path(path: Path) -> str:
    """Prefer a repository-relative evidence link and never record a personal path."""
    resolved = path.resolve()
    try:
        return str(resolved.relative_to(Path.cwd().resolve()))
    except ValueError:
        return f"evidence/{path.name}"


def _validate(path: Path) -> int:
    """Validate through the locked softschema CLI and fail closed if unavailable."""
    try:
        completed = subprocess.run(
            ["softschema", "validate", str(path)],
            capture_output=True,
            timeout=300,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired) as error:
        print(f"validation failed: {error}", file=sys.stderr)
        return 1
    stdout = completed.stdout.decode("utf-8", errors="replace")
    stderr = completed.stderr.decode("utf-8", errors="replace")
    if completed.returncode != 0:
        print(stdout or stderr, file=sys.stderr)
        return completed.returncode
    print("validated against " + experiment_model.CONTRACT, file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
