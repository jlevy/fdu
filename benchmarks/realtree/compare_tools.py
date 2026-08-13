"""Paired live-tree comparison of fdu with external disk-usage tools.

This is calibration evidence, not the optimization accept/reject loop. External tools
make different semantic trade-offs, so each competitor runs immediately beside an fdu
anchor and is reported with an explicit work class. The emitted artifact structurally
redacts the subject path and command output.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import statistics
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Iterable, List, Mapping, Optional, Sequence, Tuple

from benchmarks.realtree import measure, tree

SCHEMA = "fdu-tool-comparison-v1"
DEFAULT_RESULTS = Path("benchmarks/results/realtree")


class ComparisonError(RuntimeError):
    """A requested comparison cannot produce trustworthy evidence."""


@dataclass(frozen=True)
class ToolContract:
    """The stable command and semantic class for one supported tool."""

    name: str
    work_class: str
    description: str
    argv: Tuple[str, ...]
    version_argv: Tuple[str, ...]


@dataclass(frozen=True)
class Tool:
    """A resolved tool binary with its declared contract."""

    contract: ToolContract
    binary: Path


CONTRACTS: Dict[str, ToolContract] = {
    "fdu": ToolContract(
        name="fdu",
        work_class="indexed-tree",
        description="complete scan, reusable metadata index, roll-ups, and 10-row tree",
        argv=(
            "{binary}",
            "--cache",
            "off",
            "--color",
            "never",
            "--depth",
            "1",
            "--limit",
            "10",
            "{root}",
        ),
        version_argv=("{binary}", "--version"),
    ),
    "dust": ToolContract(
        name="dust",
        work_class="rendered-tree",
        description="complete scan, size roll-ups, hard-link accounting, and 10-row tree",
        argv=(
            "{binary}",
            "-d",
            "1",
            "-n",
            "10",
            "--no-progress",
            "--no-colors",
            "{root}",
        ),
        version_argv=("{binary}", "--version"),
    ),
    "gdu": ToolContract(
        name="gdu",
        work_class="rendered-tree",
        description="complete parallel scan and non-interactive 10-row tree",
        argv=(
            "{binary}",
            "--non-interactive",
            "--depth",
            "1",
            "--top",
            "10",
            "--no-progress",
            "--no-color",
            "--config-file",
            "/dev/null",
            "{root}",
        ),
        version_argv=("{binary}", "--version"),
    ),
    "pdu": ToolContract(
        name="pdu",
        work_class="rendered-tree",
        description="complete parallel scan, size roll-ups, and depth-one chart",
        argv=(
            "{binary}",
            "--max-depth",
            "1",
            "--silent-errors",
            "{root}",
        ),
        version_argv=("{binary}", "--version"),
    ),
    "ncdu": ToolContract(
        name="ncdu",
        work_class="indexed-tree",
        description="complete scan and in-memory browseable tree, with its UI disabled",
        argv=(
            "{binary}",
            "-0",
            "--ignore-config",
            "-o",
            "/dev/null",
            "{root}",
        ),
        version_argv=("{binary}", "--version"),
    ),
    "dua": ToolContract(
        name="dua",
        work_class="total-only",
        description="complete parallel scan reduced to one aggregate total",
        argv=("{binary}", "{root}"),
        version_argv=("{binary}", "--version"),
    ),
    "diskus": ToolContract(
        name="diskus",
        work_class="total-only",
        description="complete parallel scan reduced to one aggregate total",
        argv=("{binary}", "{root}"),
        version_argv=("{binary}", "--version"),
    ),
    "bsd-du": ToolContract(
        name="bsd-du",
        work_class="total-only",
        description="system serial traversal reduced to one aggregate total",
        argv=("{binary}", "-sk", "{root}"),
        version_argv=(),
    ),
    "gnu-du": ToolContract(
        name="gnu-du",
        work_class="total-only",
        description="GNU serial traversal reduced to one aggregate total",
        argv=("{binary}", "-sk", "{root}"),
        version_argv=("{binary}", "--version"),
    ),
}


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(prog="benchmarks.realtree.compare_tools")
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--label", required=True)
    parser.add_argument(
        "--anchor",
        required=True,
        metavar="fdu=PATH",
        help="immutable fdu release binary used beside every competitor",
    )
    parser.add_argument(
        "--tool",
        action="append",
        required=True,
        metavar="NAME=PATH",
        help="supported competitor; repeat for each tool",
    )
    parser.add_argument("--trials", type=int, default=measure.DEFAULT_TRIALS)
    parser.add_argument("--warmups", type=int, default=measure.DEFAULT_WARMUPS)
    parser.add_argument("--baseline-fingerprint", type=Path)
    parser.add_argument(
        "--baseline-output",
        type=Path,
        help="write the immediate pre-run redacted fingerprint outside the subject",
    )
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_RESULTS)
    parser.add_argument("--name", default="")
    parser.add_argument("--storage", default="", help="non-sensitive storage description")
    parser.add_argument("--timeout", type=float, default=measure.DEFAULT_TIMEOUT_SECONDS)
    arguments = parser.parse_args(list(argv))

    try:
        _require_external_output(arguments.root, arguments.output_dir)
        if arguments.baseline_output is not None:
            _require_external_file(arguments.root, arguments.baseline_output)
        anchor = _parse_tool(arguments.anchor, expected="fdu")
        competitors = [_parse_tool(item) for item in arguments.tool]
        document = run(
            root=arguments.root,
            label=arguments.label,
            anchor=anchor,
            competitors=competitors,
            trials=arguments.trials,
            warmups=arguments.warmups,
            baseline_fingerprint=_load_baseline(arguments.baseline_fingerprint),
            baseline_output=arguments.baseline_output,
            storage=arguments.storage,
            timeout_seconds=arguments.timeout,
        )
    except (ComparisonError, OSError, tree.ReferenceTreeError) as error:
        parser.error(str(error))

    slug = arguments.name or time.strftime("%Y%m%d-%H%M%S", time.gmtime())
    arguments.output_dir.mkdir(parents=True, exist_ok=True)
    json_path = arguments.output_dir / f"run-{slug}.json"
    markdown_path = arguments.output_dir / f"run-{slug}.md"
    json_path.write_text(json.dumps(document, indent=2, sort_keys=True), encoding="utf-8")
    markdown_path.write_text(render(document), encoding="utf-8")
    print(f"\nwrote {json_path}\nwrote {markdown_path}", file=sys.stderr)
    print(render(document))

    if document["tree_mutated_during_run"] or document["baseline_drift"]:
        return 2
    if document["invalid_samples"]:
        return 3
    return 0


def run(
    *,
    root: Path,
    label: str,
    anchor: Tool,
    competitors: Sequence[Tool],
    trials: int,
    warmups: int,
    baseline_fingerprint: Optional[Mapping[str, Any]],
    baseline_output: Optional[Path],
    storage: str,
    timeout_seconds: float,
) -> Dict[str, Any]:
    """Run each competitor immediately beside the anchor and return redacted evidence."""
    if anchor.contract.name != "fdu":
        raise ComparisonError("the comparison anchor must use the fdu contract")
    if not competitors:
        raise ComparisonError("at least one competitor is required")
    if trials < 3:
        raise ComparisonError("at least three trials are required for a paired interval")
    if warmups < 0:
        raise ComparisonError("warmups cannot be negative")
    names = [tool.contract.name for tool in competitors]
    if len(names) != len(set(names)) or "fdu" in names:
        raise ComparisonError("competitor names must be unique and cannot be fdu")

    started = time.time()
    before = tree.fingerprint(root, label=label)
    if baseline_output is not None:
        baseline_output.parent.mkdir(parents=True, exist_ok=True)
        baseline_output.write_text(
            json.dumps(before, indent=2, sort_keys=True), encoding="utf-8"
        )
    drift = tree.compare(before, dict(baseline_fingerprint)) if baseline_fingerprint else []
    schedule = _schedule(competitors, trials=trials, warmups=warmups)
    samples: List[Dict[str, Any]] = []
    total = len(schedule) * 2
    position = 0
    for competitor, ordinal, warmup, anchor_first in schedule:
        ordered = (anchor, competitor) if anchor_first else (competitor, anchor)
        for tool in ordered:
            position += 1
            sample = _run_one(
                tool,
                pair=competitor.contract.name,
                ordinal=ordinal,
                warmup=warmup,
                root=root,
                timeout_seconds=timeout_seconds,
            )
            samples.append(sample)
            _progress(position, total, sample)

    after = tree.fingerprint(root, label=label)
    mutation = tree.compare(after, before)
    tools = [anchor, *competitors]
    document: Dict[str, Any] = {
        "schema": SCHEMA,
        "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(started)),
        "duration_seconds": round(time.time() - started, 3),
        "host": measure.host_facts(),
        "conditions": {
            "os_cache": "warm-steady",
            "storage": storage,
            "trials": trials,
            "warmups": warmups,
            "interleaved": True,
            "schedule": "competitor-adjacent-fdu-anchor-v1",
        },
        "tree": before,
        "tree_after_digest": after["engine_digest"],
        "tree_mutated_during_run": mutation,
        "baseline_drift": drift,
        "anchor": anchor.contract.name,
        "competitor_order": names,
        "tools": {tool.contract.name: _identity(tool) for tool in tools},
        "samples": samples,
    }
    document["invalid_samples"] = sum(
        1 for sample in samples if not sample["warmup"] and not sample["valid"]
    )
    document["statistics"] = _statistics(document)
    document["overall"] = _overall(document)
    return document


def _schedule(
    competitors: Sequence[Tool], *, trials: int, warmups: int
) -> List[Tuple[Tool, int, bool, bool]]:
    """Rotate pair order while keeping every competitor immediately beside fdu."""
    schedule: List[Tuple[Tool, int, bool, bool]] = []
    for ordinal in range(-warmups, trials):
        offset = (ordinal + warmups) % len(competitors)
        rotated = list(competitors[offset:]) + list(competitors[:offset])
        if (ordinal + warmups) % 2:
            rotated.reverse()
        for pair_position, competitor in enumerate(rotated):
            anchor_first = (ordinal + pair_position) % 2 == 0
            schedule.append((competitor, ordinal, ordinal < 0, anchor_first))
    return schedule


def _run_one(
    tool: Tool,
    *,
    pair: str,
    ordinal: int,
    warmup: bool,
    root: Path,
    timeout_seconds: float,
) -> Dict[str, Any]:
    argv = _expand(tool.contract.argv, tool.binary, root)
    result = measure._spawn(argv, timeout_seconds=timeout_seconds)
    resources = dict(result["resources"])
    user = resources.get("user_cpu_ns")
    system = resources.get("system_cpu_ns")
    metrics = {
        **resources,
        "wall_ns": result["wall_ns"],
        "cpu_ns": user + system if user is not None and system is not None else None,
    }
    reasons: List[str] = []
    if result["timed_out"]:
        reasons.append("command timed out")
    if result["exit_code"] != 0:
        reasons.append(f"command exited with {result['exit_code']}")
    stderr = result["stderr"].encode("utf-8", errors="replace")
    return {
        "pair": pair,
        "tool": tool.contract.name,
        "ordinal": ordinal,
        "warmup": warmup,
        "valid": not reasons,
        "reasons": reasons,
        "metrics": metrics,
        "stdout_bytes": len(result["stdout"]),
        "stdout_sha256": hashlib.sha256(result["stdout"]).hexdigest(),
        "stderr_bytes": len(stderr),
        "stderr_sha256": hashlib.sha256(stderr).hexdigest(),
    }


def _statistics(document: Mapping[str, Any]) -> Dict[str, Any]:
    anchor = str(document["anchor"])
    samples = list(document["samples"])
    statistics_document: Dict[str, Any] = {}
    for competitor in document["competitor_order"]:
        per_tool: Dict[str, Any] = {}
        for name in (anchor, competitor):
            selected = [
                sample
                for sample in samples
                if sample["pair"] == competitor
                and sample["tool"] == name
                and not sample["warmup"]
                and sample["valid"]
            ]
            per_tool[name] = {
                "samples": len(selected),
                "invalid": sum(
                    1
                    for sample in samples
                    if sample["pair"] == competitor
                    and sample["tool"] == name
                    and not sample["warmup"]
                    and not sample["valid"]
                ),
                "metrics": {
                    metric: measure.distribution(
                        [
                            sample["metrics"][metric]
                            for sample in selected
                            if sample["metrics"].get(metric) is not None
                        ]
                    )
                    for metric in measure.LOWER_IS_BETTER
                    if metric != "component_ns" and metric != "blocked_ns"
                },
            }
        statistics_document[competitor] = {
            "tools": per_tool,
            "competitor_vs_fdu": _paired(samples, pair=competitor, metric_names=per_tool[anchor]["metrics"]),
        }
    return statistics_document


def _overall(document: Mapping[str, Any]) -> Dict[str, Any]:
    """Summarize each executable once; fdu includes every adjacent anchor sample."""
    result: Dict[str, Any] = {}
    for name in [document["anchor"], *document["competitor_order"]]:
        selected = [
            sample
            for sample in document["samples"]
            if sample["tool"] == name and not sample["warmup"] and sample["valid"]
        ]
        result[name] = {
            "samples": len(selected),
            "metrics": {
                metric: measure.distribution(
                    [
                        sample["metrics"][metric]
                        for sample in selected
                        if sample["metrics"].get(metric) is not None
                    ]
                )
                for metric in measure.LOWER_IS_BETTER
                if metric != "component_ns" and metric != "blocked_ns"
            },
        }
    return result


def _paired(
    samples: Sequence[Mapping[str, Any]], *, pair: str, metric_names: Iterable[str]
) -> Dict[str, Any]:
    result: Dict[str, Any] = {}
    for metric in metric_names:
        indexed: Dict[str, Dict[int, int]] = {}
        for name in ("fdu", pair):
            indexed[name] = {
                int(sample["ordinal"]): int(sample["metrics"][metric])
                for sample in samples
                if sample["pair"] == pair
                and sample["tool"] == name
                and not sample["warmup"]
                and sample["valid"]
                and sample["metrics"].get(metric) is not None
            }
        ordinals = sorted(set(indexed["fdu"]) & set(indexed[pair]))
        ratios = [
            (indexed[pair][ordinal] - indexed["fdu"][ordinal]) / indexed["fdu"][ordinal]
            for ordinal in ordinals
            if indexed["fdu"][ordinal]
        ]
        if len(ratios) < 3:
            result[metric] = None
            continue
        low, high = measure._bootstrap_median_interval(ratios)
        result[metric] = {
            "pairs": len(ratios),
            "median_change_pct": round(statistics.median(ratios) * 100, 3),
            "ci95_change_pct": [round(low * 100, 3), round(high * 100, 3)],
            "direction": measure._direction(low, high),
        }
    return result


def render(document: Mapping[str, Any]) -> str:
    """Render a compact, path-redacted review of one comparison run."""
    tree_document = document["tree"]
    host = document["host"]
    lines = [
        "# Live Tool Comparison",
        "",
        (
            f"{tree_document['counts']['total']:,} entries on "
            f"{host.get('cpu_model') or host.get('arch')} with "
            f"{document['conditions'].get('storage') or 'unspecified storage'}."
        ),
        "",
        "| Tool | Work class | Median wall | Versus paired fdu | 95% interval | Peak RSS |",
        "| --- | --- | ---: | ---: | ---: | ---: |",
    ]
    anchor = document["anchor"]
    anchor_metrics = document["overall"][anchor]["metrics"]
    lines.append(
        "| "
        + " | ".join(
            [
                anchor,
                document["tools"][anchor]["work_class"],
                _seconds(anchor_metrics["wall_ns"]),
                "baseline",
                "—",
                _mib(anchor_metrics["peak_rss_bytes"]),
            ]
        )
        + " |"
    )
    for name in document["competitor_order"]:
        entry = document["statistics"][name]
        wall = entry["tools"][name]["metrics"]["wall_ns"]
        rss = entry["tools"][name]["metrics"]["peak_rss_bytes"]
        comparison = entry["competitor_vs_fdu"]["wall_ns"]
        contract = document["tools"][name]
        change = _comparison_change(comparison)
        lines.append(
            "| "
            + " | ".join(
                [
                    name,
                    contract["work_class"],
                    _seconds(wall),
                    change[0],
                    change[1],
                    _mib(rss),
                ]
            )
            + " |"
        )
    lines.extend(
        [
            "",
            "Positive percentages mean the competitor took more wall time than the immediately "
            "adjacent fdu run. External tools are calibration references, not optimization "
            "verdicts; work classes and command templates are retained in the JSON artifact.",
            "",
            f"Invalid timed samples: {document['invalid_samples']}. ",
            f"Baseline drift: {document['baseline_drift'] or 'none'}. ",
            f"Mutation during run: {document['tree_mutated_during_run'] or 'none'}.",
            "",
            "* * *",
            "",
            "*Part of the fdu project documentation.  ",
            "See [AGENTS.md](../../../AGENTS.md).*",
            "",
        ]
    )
    return "\n".join(lines)


def _parse_tool(specification: str, *, expected: Optional[str] = None) -> Tool:
    name, separator, raw_path = specification.partition("=")
    if not separator or not name or not raw_path:
        raise ComparisonError(f"tool {specification!r} must be NAME=PATH")
    if expected is not None and name != expected:
        raise ComparisonError(f"expected {expected}=PATH, got {specification!r}")
    contract = CONTRACTS.get(name)
    if contract is None:
        raise ComparisonError(
            f"unsupported tool {name!r}; choose from {', '.join(sorted(CONTRACTS))}"
        )
    binary = Path(raw_path).resolve()
    if not binary.is_file():
        raise ComparisonError(f"{name} binary does not exist: {binary}")
    return Tool(contract=contract, binary=binary)


def _identity(tool: Tool) -> Dict[str, Any]:
    content = tool.binary.read_bytes()
    return {
        "name": tool.contract.name,
        "work_class": tool.contract.work_class,
        "description": tool.contract.description,
        "command": list(tool.contract.argv),
        "version": _version(tool),
        "binary_sha256": hashlib.sha256(content).hexdigest(),
        "binary_size_bytes": len(content),
    }


def _version(tool: Tool) -> str:
    if not tool.contract.version_argv:
        return "version not reported by system binary"
    completed = subprocess.run(
        _expand(tool.contract.version_argv, tool.binary, Path(".")),
        stdin=subprocess.DEVNULL,
        capture_output=True,
        timeout=10,
        check=False,
    )
    text = (completed.stdout + completed.stderr).decode("utf-8", errors="replace").strip()
    return " ".join(text.split())[:300]


def _expand(template: Sequence[str], binary: Path, root: Path) -> List[str]:
    return [
        str(binary) if item == "{binary}" else str(root) if item == "{root}" else item
        for item in template
    ]


def _load_baseline(path: Optional[Path]) -> Optional[Mapping[str, Any]]:
    if path is None:
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def _require_external_output(root: Path, output_dir: Path) -> None:
    """Keep evidence writes out of the subject whose stability they certify."""
    _require_external_file(root, output_dir)


def _require_external_file(root: Path, destination: Path) -> None:
    """Reject any evidence destination nested in the measured tree."""
    resolved_root = root.resolve(strict=True)
    resolved_destination = destination.resolve(strict=False)
    if resolved_destination == resolved_root or resolved_root in resolved_destination.parents:
        raise ComparisonError(
            "evidence outputs must be outside --root so recording them cannot mutate "
            "the benchmark tree"
        )


def _progress(position: int, total: int, sample: Mapping[str, Any]) -> None:
    wall = sample["metrics"]["wall_ns"] / 1e9
    tag = "warmup" if sample["warmup"] else f"#{sample['ordinal']:02d}"
    validity = "" if sample["valid"] else f" INVALID: {'; '.join(sample['reasons'])}"
    print(
        f"[{position:3d}/{total}] {sample['pair']:<8} {sample['tool']:<8} "
        f"{tag:<6} {wall:7.3f} s{validity}",
        file=sys.stderr,
        flush=True,
    )


def _seconds(distribution: Optional[Mapping[str, Any]]) -> str:
    return "—" if not distribution else f"{distribution['median'] / 1e9:.3f} s"


def _mib(distribution: Optional[Mapping[str, Any]]) -> str:
    return "—" if not distribution else f"{distribution['median'] / 1024 / 1024:.1f} MiB"


def _percent(value: float) -> str:
    return f"{value:+.1f}%"


def _comparison_change(comparison: Optional[Mapping[str, Any]]) -> Tuple[str, str]:
    if comparison is None:
        return "invalid", "—"
    return (
        _percent(comparison["median_change_pct"]),
        " to ".join(_percent(value) for value in comparison["ci95_change_pct"]),
    )


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
