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

from benchmarks.realtree import experiment as experiment_model
from benchmarks.realtree.summary import SummaryError, _validator

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
    parser.add_argument("--body", type=Path, help="Markdown body file; a stub is written otherwise")
    parser.add_argument("--output-dir", type=Path, default=EXPERIMENTS_DIR)
    parser.add_argument("--no-validate", action="store_true")
    arguments = parser.parse_args(list(argv))

    run = json.loads(arguments.run.read_text(encoding="utf-8"))
    try:
        headline = _headline(run, arguments)
    except ValueError as error:
        parser.error(str(error))
    payload = experiment_model.from_run(
        run,
        experiment_id=arguments.id,
        title=arguments.title,
        hypotheses=arguments.hypotheses,
        control=arguments.control,
        candidate=arguments.candidate,
        run_artifact=str(arguments.run),
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
        },
    )

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
    return entry["median_change_pct"] if entry else None


def _selected_comparison(run: Mapping[str, Any], arguments: argparse.Namespace) -> Any:
    """Exactly the comparison the recording arguments name, or an error.

    Every ambiguous case is refused rather than resolved, because each one silently
    records a number that belongs to a different pair of variants — and an experiment
    artifact is the permanent record the ledger is regenerated from.
    """
    if bool(arguments.control_variant) != bool(arguments.candidate_variant):
        raise ValueError("--control-variant and --candidate-variant must be given together")

    statistics = run["statistics"].get(arguments.primary_job)
    if not statistics:
        return None
    comparisons = statistics["comparisons"]

    if arguments.control_variant and arguments.candidate_variant:
        key = f"{arguments.candidate_variant}_vs_{arguments.control_variant}"
        comparison = comparisons.get(key)
        if comparison is None:
            available = ", ".join(sorted(comparisons)) or "none"
            raise ValueError(f"run has no comparison named {key!r}; it has {available}")
        return comparison

    if len(comparisons) > 1:
        raise ValueError(
            "a run with several comparisons needs --control-variant and "
            "--candidate-variant to say which pair this experiment is about"
        )
    if not comparisons:
        return None
    return next(iter(comparisons.values()))


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


def _is_iso_date(text: str) -> bool:
    """Whether YAML would read this plain scalar as a date rather than a string."""
    return (
        len(text) == 10
        and text[4] == "-"
        and text[7] == "-"
        and text[:4].isdigit()
        and text[5:7].isdigit()
        and text[8:].isdigit()
    )


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
    if _is_iso_date(text):
        # YAML resolves a plain ISO date to a date object, not a string, so an
        # unquoted `date: 2026-08-13` fails the contract's `date: str` on the way back
        # in. Every artifact written before this was quoted had that defect, invisible
        # because `_validate` skips when the softschema CLI is unavailable.
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
    return f"""# {payload["title"]}

## Hypothesis

{", ".join(payload["hypotheses"]) or "—"}: _state what you expected to be slow, why,
and which metric would move._

## What was tried

_The smallest change that tests the hypothesis._

## What the numbers said

_Read the tables in the frontmatter. Say what surprised you._

## Verdict

**{verdict["decision"].upper()}** — {verdict["reason"]}
"""


def _validate(path: Path) -> int:
    """Validate through the softschema CLI, if it is reachable."""
    try:
        completed = subprocess.run(
            [*_validator(), "validate", str(path)],
            capture_output=True,
            timeout=300,
        )
    except (SummaryError, FileNotFoundError, subprocess.TimeoutExpired) as error:
        # SummaryError is what `_validator` raises when it cannot find softschema at
        # all. It belongs here rather than propagating: the validator lives in the dev
        # group, `record` is normally invoked without it, and recording an experiment
        # must not fail because the optional post-write check is unavailable.
        print(f"skipped validation: {error}", file=sys.stderr)
        return 0
    stdout = completed.stdout.decode("utf-8", errors="replace")
    stderr = completed.stderr.decode("utf-8", errors="replace")
    if completed.returncode != 0:
        print(stdout or stderr, file=sys.stderr)
        return completed.returncode
    print("validated against " + experiment_model.CONTRACT, file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
