"""Command-line entry point for performance execution and evidence rendering."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence

from benchmarks.report import (
    baseline_compatibility_reasons,
    executable_identity_differences,
    write_report,
)
from benchmarks.runner import RunnerError, run_scenario_set
from benchmarks.schema import (
    SchemaError,
    load_result,
    load_scenario_set,
)


def build_parser() -> argparse.ArgumentParser:
    """Build the performance evidence command parser."""
    parser = argparse.ArgumentParser(
        prog="python -m benchmarks.run",
        description="Execute, validate, compare, and render fdu performance evidence.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    execute = subparsers.add_parser("execute", help="execute an explicit scenario set")
    execute.add_argument("--scenarios", required=True, type=Path)
    execute.add_argument("--scenario", action="append", dest="scenario_ids")
    execute.add_argument(
        "--executable",
        action="append",
        required=True,
        help="adapter=/absolute/component; repeat to build an interpreter command",
    )
    execute.add_argument("--work-dir", required=True, type=Path)
    execute.add_argument("--output-dir", required=True, type=Path)
    execute.add_argument("--order-seed", required=True)

    validate = subparsers.add_parser("validate", help="validate a scenario or result")
    validate.add_argument("--kind", required=True, choices=("result", "scenarios"))
    validate.add_argument("--path", required=True, type=Path)

    render = subparsers.add_parser("render", help="render a result as Markdown")
    render.add_argument("--result", required=True, type=Path)
    render.add_argument("--output", required=True, type=Path)

    compare = subparsers.add_parser(
        "compare", help="check whether two result sets are baseline-compatible"
    )
    compare.add_argument("--current", required=True, type=Path)
    compare.add_argument("--baseline", required=True, type=Path)
    return parser


def main(arguments: Optional[Sequence[str]] = None) -> int:
    """Run one evidence command and emit one structured JSON response."""
    namespace = build_parser().parse_args(arguments)
    try:
        response, status = _run(namespace)
    except (OSError, RunnerError, SchemaError) as error:
        _write_json(
            sys.stderr,
            {
                "action": namespace.command,
                "error": str(error),
                "error_type": "performance_evidence_error",
                "ok": False,
            },
        )
        return 2
    _write_json(sys.stdout, response)
    return status


def _run(namespace: argparse.Namespace) -> tuple[Dict[str, Any], int]:
    if namespace.command == "execute":
        scenarios = load_scenario_set(namespace.scenarios)
        executables = _parse_executables(namespace.executable)
        result = run_scenario_set(
            scenarios,
            executables=executables,
            work_directory=namespace.work_dir,
            output_directory=namespace.output_dir,
            order_seed=namespace.order_seed,
            scenario_ids=namespace.scenario_ids,
        )
        result_path = next(
            namespace.output_dir.glob(f"run-*-{result['identity']['run_id']}.json")
        )
        timed = [
            trial for trial in result["trials"] if trial["sample_kind"] == "timed"
        ]
        return (
            {
                "action": "execute",
                "invalid_timed_trials": sum(
                    not trial["validation"]["valid"] for trial in timed
                ),
                "ok": True,
                "result_hash": result["result_hash"],
                "result_path": str(result_path.resolve()),
                "run_id": result["identity"]["run_id"],
                "valid_timed_trials": sum(
                    trial["validation"]["valid"] for trial in timed
                ),
            },
            0,
        )
    if namespace.command == "validate":
        if namespace.kind == "scenarios":
            document = load_scenario_set(namespace.path)
            version = document["schema"]
        else:
            document = load_result(namespace.path)
            version = document["schema"]
        return (
            {
                "action": "validate",
                "kind": namespace.kind,
                "ok": True,
                "schema": version,
            },
            0,
        )
    if namespace.command == "render":
        write_report(namespace.result, namespace.output)
        return (
            {
                "action": "render",
                "ok": True,
                "output_path": str(namespace.output.resolve()),
            },
            0,
        )
    if namespace.command == "compare":
        current = load_result(namespace.current)
        baseline = load_result(namespace.baseline)
        reasons = baseline_compatibility_reasons(current, baseline)
        identity_changes = executable_identity_differences(current, baseline)
        return (
            {
                "action": "compare",
                "compatible": not reasons,
                "executable_identity_changes": identity_changes,
                "ok": not reasons,
                "reasons": reasons,
            },
            0 if not reasons else 2,
        )
    raise RunnerError(f"unknown evidence command: {namespace.command!r}")


def _parse_executables(values: Sequence[str]) -> Mapping[str, Sequence[str]]:
    parsed: Dict[str, List[str]] = {}
    for value in values:
        if "=" not in value:
            raise RunnerError("--executable must use adapter=/absolute/component")
        adapter, component = value.split("=", 1)
        if not adapter or not component:
            raise RunnerError("--executable must use adapter=/absolute/component")
        parsed.setdefault(adapter, []).append(component)
    return parsed


def _write_json(output: Any, value: Mapping[str, Any]) -> None:
    json.dump(value, output, ensure_ascii=True, sort_keys=True)
    output.write("\n")


if __name__ == "__main__":
    raise SystemExit(main())
