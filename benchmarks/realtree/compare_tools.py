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
import re
import statistics
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Iterable, List, Mapping, Optional, Sequence, Tuple

from benchmarks.realtree import installed_command, measure, provenance, tree

SCHEMA = "fdu-tool-comparison-v3"
DEFAULT_RESULTS = Path(tempfile.gettempdir()) / "fdu-tool-comparison" / "results"
MIN_TOOL_COMPARISON_WARMUPS = 1
PRE_RUN_FINGERPRINT_PASSES = 1
FDU_SUMMARY_CONTRACTS = frozenset({"fdu-index-summary", "fdu-transient-summary"})
FDU_SCAN_DIAGNOSTICS_PREFIX = "__FDU_SCAN_DIAGNOSTICS__="


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

    name: str
    contract: ToolContract
    binary: Path


CONTRACTS: Dict[str, ToolContract] = {
    "fdu": ToolContract(
        name="fdu",
        work_class="indexed-tree",
        description=(
            "complete scan, reusable exact metadata index, roll-ups, and 10-row tree; "
            "bytes are attributed to path entries"
        ),
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
    # These summary contracts intentionally use the same public CLI arguments. The
    # immutable binary supplied by the operator determines whether the request takes
    # the indexed or transient execution plan; the harness records that declared work
    # class but cannot infer retained engine state from process output.
    "fdu-index-summary": ToolContract(
        name="fdu-index-summary",
        work_class="indexed-summary",
        description=(
            "complete scan and reusable exact metadata index reduced to one summary; "
            "the persisted snapshot cache is disabled"
        ),
        argv=(
            "{binary}",
            "--cache",
            "off",
            "--view",
            "summary",
            "--format",
            "json",
            "--color",
            "never",
            "{root}",
        ),
        version_argv=("{binary}", "--version"),
    ),
    "fdu-transient-summary": ToolContract(
        name="fdu-transient-summary",
        work_class="transient-summary",
        description=(
            "complete scan reduced directly to exact summary tallies; no path index or "
            "persisted snapshot is retained"
        ),
        argv=(
            "{binary}",
            "--cache",
            "off",
            "--view",
            "summary",
            "--format",
            "json",
            "--color",
            "never",
            "{root}",
        ),
        version_argv=("{binary}", "--version"),
    ),
    "dust": ToolContract(
        name="dust",
        work_class="allocated-total",
        description=(
            "complete scan reduced to one exact allocated-byte total; hard links are "
            "deduplicated and any reported traversal error invalidates the sample"
        ),
        argv=(
            "{binary}",
            "-d",
            "0",
            "-n",
            "1",
            "--no-progress",
            "--no-colors",
            "--no-percent-bars",
            "--print-errors",
            "--output-format",
            "b",
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
    "dumac": ToolContract(
        name="dumac",
        work_class="total-only",
        description=(
            "macOS getattrlistbulk scan reduced to one hard-link-deduplicated allocated-byte total"
        ),
        argv=("{binary}", "{root}"),
        version_argv=(),
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
        metavar="[LABEL:]FDU_CONTRACT=PATH",
        help="immutable fdu release binary used beside every competitor",
    )
    parser.add_argument(
        "--tool",
        action="append",
        required=True,
        metavar="[LABEL:]CONTRACT=PATH",
        help="supported competitor; repeat for each tool",
    )
    parser.add_argument("--trials", type=int, default=measure.DEFAULT_TRIALS)
    parser.add_argument("--warmups", type=int, default=measure.DEFAULT_WARMUPS)
    parser.add_argument(
        "--stage",
        choices=("exploratory", "discovery", "held-out"),
        default="exploratory",
        help="evidence stage; only a held-out matrix can support confirmation",
    )
    parser.add_argument(
        "--host-regime",
        choices=sorted(measure.HOST_REGIMES),
        default="uncontrolled",
        help="predeclared host-pressure cell; held-out evidence must be controlled",
    )
    parser.add_argument(
        "--background-load-workers",
        type=int,
        default=2,
        help="CPU load workers for the controlled-interactive host regime",
    )
    parser.add_argument("--baseline-fingerprint", type=Path)
    parser.add_argument(
        "--provenance-manifest",
        type=Path,
        help="verified build/host manifest; required for held-out release evidence",
    )
    parser.add_argument(
        "--installation-attestation",
        type=Path,
        help="verified native or wheel-installed fdu resolution and smoke attestation",
    )
    parser.add_argument(
        "--supporting-artifact",
        action="append",
        default=[],
        metavar="LABEL=PATH",
        help="native payload belonging to an artifact, such as the Python extension",
    )
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
        anchor = _parse_tool(arguments.anchor)
        competitors = [_parse_tool(item) for item in arguments.tool]
        tools = [anchor, *competitors]
        supporting: Dict[str, List[Path]] = {}
        for specification in arguments.supporting_artifact:
            label, separator, raw_path = specification.partition("=")
            if not separator or not label or not raw_path:
                raise ComparisonError("--supporting-artifact must be LABEL=PATH")
            supporting.setdefault(label, []).append(Path(raw_path))
        if arguments.stage == "held-out" and arguments.provenance_manifest is None:
            raise ComparisonError("held-out release evidence requires --provenance-manifest")
        if arguments.stage == "held-out" and arguments.installation_attestation is None:
            raise ComparisonError("held-out release evidence requires --installation-attestation")
        provenance_document = (
            provenance.load_and_verify(
                arguments.provenance_manifest,
                source_root=provenance.PROJECT_ROOT,
                subject_root=arguments.root,
                artifacts={tool.name: tool.binary for tool in tools},
                supporting_files=supporting,
                require_claim_grade=arguments.stage == "held-out",
            )
            if arguments.provenance_manifest is not None
            else None
        )
        installation_document = (
            installed_command.load_and_verify(
                arguments.installation_attestation,
                executable=anchor.binary,
                root=arguments.root,
                provenance_document=provenance_document or {},
                require_claim_grade=arguments.stage == "held-out",
            )
            if arguments.installation_attestation is not None
            else None
        )
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
            campaign_stage=arguments.stage,
            provenance_document=provenance_document,
            installation_document=installation_document,
            host_regime=arguments.host_regime,
            background_load_workers=arguments.background_load_workers,
            timeout_seconds=arguments.timeout,
        )
    except (
        ComparisonError,
        installed_command.InstallationError,
        measure.MeasureError,
        OSError,
        provenance.ProvenanceError,
        tree.ReferenceTreeError,
    ) as error:
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
    campaign_stage: str = "exploratory",
    provenance_document: Optional[Mapping[str, Any]] = None,
    installation_document: Optional[Mapping[str, Any]] = None,
    host_regime: str = "uncontrolled",
    background_load_workers: int = 2,
    timeout_seconds: float = measure.DEFAULT_TIMEOUT_SECONDS,
) -> Dict[str, Any]:
    """Run each competitor immediately beside the anchor and return redacted evidence."""
    if anchor.contract.name not in {
        "fdu",
        "fdu-index-summary",
        "fdu-transient-summary",
    }:
        raise ComparisonError("the comparison anchor must use an fdu contract")
    if not competitors:
        raise ComparisonError("at least one competitor is required")
    if trials < 3:
        raise ComparisonError("at least three trials are required for a paired interval")
    if host_regime not in measure.HOST_REGIMES:
        raise ComparisonError(f"unknown host regime {host_regime!r}")
    if campaign_stage == "held-out" and host_regime == "uncontrolled":
        raise ComparisonError(
            "held-out release evidence requires a quiet or controlled-interactive host regime"
        )
    if campaign_stage == "held-out" and anchor.contract.name not in FDU_SUMMARY_CONTRACTS:
        raise ComparisonError(
            "held-out release evidence requires an fdu summary contract with a "
            "complete installed-command policy trace"
        )
    if campaign_stage == "held-out":
        if provenance_document is None or provenance_document.get("claim_grade") is not True:
            raise ComparisonError("held-out release evidence requires claim-grade provenance")
        if installation_document is None or installation_document.get("claim_grade") is not True:
            raise ComparisonError(
                "held-out release evidence requires a claim-grade installation attestation"
            )
        scope_reasons = provenance.apple_silicon_apfs_scope_reasons(provenance_document)
        if scope_reasons:
            raise ComparisonError(
                "held-out Apple Silicon/APFS scope is not established: " + "; ".join(scope_reasons)
            )
    cache_evidence = _warm_cache_evidence(warmups)
    names = [tool.name for tool in competitors]
    if len(names) != len(set(names)) or anchor.name in names:
        raise ComparisonError("competitor names must be unique and cannot repeat the anchor")

    tools = [anchor, *competitors]
    identities = {tool.name: _identity(tool) for tool in tools}
    started = time.time()
    before = tree.fingerprint(root, label=label)
    if baseline_output is not None:
        baseline_output.parent.mkdir(parents=True, exist_ok=True)
        baseline_output.write_text(json.dumps(before, indent=2, sort_keys=True), encoding="utf-8")
    drift = tree.compare(before, dict(baseline_fingerprint)) if baseline_fingerprint else []
    schedule = _schedule(competitors, trials=trials, warmups=warmups)
    samples: List[Dict[str, Any]] = []
    total = len(schedule) * 2
    position = 0
    with measure._host_regime(host_regime, background_load_workers) as regime:
        for competitor, ordinal, warmup, anchor_first in schedule:
            ordered = (anchor, competitor) if anchor_first else (competitor, anchor)
            for tool in ordered:
                position += 1
                sample = _run_one(
                    tool,
                    pair=competitor.name,
                    ordinal=ordinal,
                    warmup=warmup,
                    root=root,
                    summary_oracle=before,
                    host_regime=regime,
                    timeout_seconds=timeout_seconds,
                )
                samples.append(sample)
                _progress(position, total, sample)
        final_host_pressure = measure._host_pressure_snapshot(regime)

    after = tree.fingerprint(root, label=label)
    mutation = tree.compare(after, before)
    semantic_mismatches = _invalidate_semantic_mismatches(samples, anchor=anchor.name)
    oracle_mismatches = [
        {
            "pair": sample["pair"],
            "tool": sample["tool"],
            "ordinal": sample["ordinal"],
            "warmup": sample["warmup"],
            "reason": sample["summary_oracle_error"],
        }
        for sample in samples
        if sample.get("summary_oracle_error") is not None
    ]
    global_reasons: List[str] = []
    if drift:
        global_reasons.append("the measured tree disagreed with its preregistered baseline")
    if mutation:
        global_reasons.append("the measured tree changed during the run")
    if global_reasons:
        for sample in samples:
            sample["valid"] = False
            for reason in global_reasons:
                if reason not in sample["reasons"]:
                    sample["reasons"].append(reason)
    document: Dict[str, Any] = {
        "schema": SCHEMA,
        "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(started)),
        "duration_seconds": round(time.time() - started, 3),
        "host": measure.host_facts(),
        "conditions": {
            "campaign_stage": campaign_stage,
            "confidence_interval": "paired-bootstrap-median-95-v1",
            "os_cache": "warm-steady",
            "os_cache_evidence": cache_evidence,
            "storage": storage,
            "trials": trials,
            "warmups": warmups,
            "interleaved": True,
            "host_regime": host_regime,
            "host_acceptance": {
                "controlled_load_workers": regime.load_workers,
                "initial": regime.initial,
                "final": final_host_pressure,
                "quiet_max_load_per_cpu": measure.QUIET_MAX_LOAD_PER_CPU,
                "controlled_max_excess_load_per_cpu": (measure.CONTROLLED_MAX_EXCESS_LOAD_PER_CPU),
                "quiet_max_cpu_busy_pct": measure.QUIET_MAX_CPU_BUSY_PCT,
                "controlled_max_excess_cpu_busy_pct": (measure.CONTROLLED_MAX_EXCESS_CPU_BUSY_PCT),
            },
            "schedule": "competitor-adjacent-fdu-anchor-v1",
            "scan_diagnostics": {
                "environment": {"FDU_SCAN_DIAGNOSTICS": "1"},
                "required_contracts": sorted(FDU_SUMMARY_CONTRACTS),
                "schema": "fdu-scan-diagnostics-v1",
                "stderr_transport": FDU_SCAN_DIAGNOSTICS_PREFIX,
            },
            "stopping_rule": "fixed-N-no-optional-stopping-v1",
        },
        "tree": before,
        "tree_after_digest": after["engine_digest"],
        "tree_mutated_during_run": mutation,
        "baseline_drift": drift,
        "anchor": anchor.name,
        "competitor_order": names,
        "tools": identities,
        "samples": samples,
        "semantic_mismatches": semantic_mismatches,
        "summary_oracle_mismatches": oracle_mismatches,
    }
    if provenance_document is not None:
        document["provenance"] = dict(provenance_document)
    if installation_document is not None:
        document["installation"] = dict(installation_document)
    document["invalid_samples"] = sum(
        1 for sample in samples if not sample["warmup"] and not sample["valid"]
    )
    document["statistics"] = _statistics(document)
    document["overall"] = _overall(document)
    return document


def _invalidate_semantic_mismatches(
    samples: Sequence[Dict[str, Any]], *, anchor: str
) -> List[Dict[str, Any]]:
    """Invalidate fdu pairs whose stable report semantics differ.

    Timestamp, generator, and absolute-root fields legitimately differ between two
    adjacent processes.  ``_summary_semantic_digest`` removes only those envelope
    fields; a digest mismatch therefore means the candidate changed the answer and its
    timing cannot support a performance verdict.
    """
    indexed: Dict[Tuple[str, int, bool, str], Dict[str, Any]] = {
        (
            str(sample["pair"]),
            int(sample["ordinal"]),
            bool(sample["warmup"]),
            str(sample["tool"]),
        ): sample
        for sample in samples
    }
    mismatches: List[Dict[str, Any]] = []
    for sample in samples:
        competitor = str(sample["pair"])
        if sample["tool"] != competitor or sample.get("semantic_sha256") is None:
            continue
        anchor_sample = indexed.get(
            (competitor, int(sample["ordinal"]), bool(sample["warmup"]), anchor)
        )
        if anchor_sample is None or anchor_sample.get("semantic_sha256") is None:
            continue
        if sample["semantic_sha256"] == anchor_sample["semantic_sha256"]:
            continue
        reason = f"stable report semantics differ from {anchor}"
        for mismatched in (anchor_sample, sample):
            mismatched["valid"] = False
            mismatched["reasons"].append(reason)
        mismatches.append(
            {
                "pair": competitor,
                "ordinal": int(sample["ordinal"]),
                "warmup": bool(sample["warmup"]),
                "anchor_sha256": anchor_sample["semantic_sha256"],
                "competitor_sha256": sample["semantic_sha256"],
            }
        )
    return mismatches


def _apple_silicon_apfs_scope_reasons(
    provenance_document: Mapping[str, Any],
) -> List[str]:
    """Compatibility wrapper around the authoritative provenance scope check."""
    return provenance.apple_silicon_apfs_scope_reasons(provenance_document)


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
    summary_oracle: Mapping[str, Any],
    host_regime: measure.HostRegime,
    timeout_seconds: float,
) -> Dict[str, Any]:
    argv = _expand(tool.contract.argv, tool.binary, root)
    requires_scan_diagnostics = tool.contract.name in FDU_SUMMARY_CONTRACTS
    pressure_before = measure._host_pressure_snapshot(host_regime)
    result = measure._spawn(
        argv,
        timeout_seconds=timeout_seconds,
        environment_overrides={"FDU_SCAN_DIAGNOSTICS": "1"} if requires_scan_diagnostics else None,
    )
    pressure_after = measure._host_pressure_snapshot(host_regime)
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
    reasons.extend(measure._host_pressure_reasons(host_regime, pressure_before, pressure_after))
    semantic_sha256: Optional[str] = None
    semantic_total_bytes: Optional[int] = None
    summary_oracle_error: Optional[str] = None
    scan_diagnostics: Optional[Mapping[str, Any]] = None
    effective_stderr = result["stderr"]
    if requires_scan_diagnostics:
        scan_diagnostics, effective_stderr, diagnostic_reasons = _extract_scan_diagnostics(
            effective_stderr
        )
        reasons.extend(diagnostic_reasons)
        reasons.extend(
            measure._validate_scan_diagnostics(
                scan_diagnostics,
                require_macos_backend=sys.platform == "darwin",
                expected_dirs_read=(summary_oracle.get("counts") or {}).get("directories"),
            )
        )
        if effective_stderr.strip():
            reasons.append("fdu reported unexpected stderr outside the policy trace")

    if tool.contract.name in FDU_SUMMARY_CONTRACTS:
        semantic_sha256, summary, semantic_error = _summary_semantics(result["stdout"])
        if semantic_error is not None:
            reasons.append(semantic_error)
        elif summary is not None:
            summary_oracle_error = _summary_oracle_error(summary, summary_oracle)
            if summary_oracle_error is not None:
                reasons.append(summary_oracle_error)
    elif tool.contract.name == "dust":
        semantic_total_bytes, dust_error = _dust_semantics(
            result["stdout"], effective_stderr, summary_oracle
        )
        if dust_error is not None:
            summary_oracle_error = dust_error
            reasons.append(dust_error)
    stderr = effective_stderr.encode("utf-8", errors="replace")
    claim_metrics = metrics if not reasons else {key: None for key in metrics}
    return {
        "pair": pair,
        "tool": tool.name,
        "ordinal": ordinal,
        "warmup": warmup,
        "valid": not reasons,
        "reasons": reasons,
        "metrics": claim_metrics,
        "host_pressure": {"after": pressure_after, "before": pressure_before},
        "stdout_bytes": len(result["stdout"]),
        "stdout_sha256": hashlib.sha256(result["stdout"]).hexdigest(),
        "semantic_sha256": semantic_sha256,
        "semantic_total_bytes": semantic_total_bytes,
        "scan_diagnostics": scan_diagnostics,
        "summary_oracle_error": summary_oracle_error,
        "stderr_bytes": len(stderr),
        "stderr_sha256": hashlib.sha256(stderr).hexdigest(),
    }


def _extract_scan_diagnostics(
    stderr: str,
) -> Tuple[Optional[Mapping[str, Any]], str, List[str]]:
    """Remove and parse exactly one repository-owned policy-trace transport line."""
    encoded: List[str] = []
    residual: List[str] = []
    for line in stderr.splitlines():
        if line.startswith(FDU_SCAN_DIAGNOSTICS_PREFIX):
            encoded.append(line.removeprefix(FDU_SCAN_DIAGNOSTICS_PREFIX))
        elif line.strip():
            residual.append(line)

    reasons: List[str] = []
    diagnostics: Optional[Mapping[str, Any]] = None
    if len(encoded) != 1:
        reasons.append(
            f"installed fdu emitted {len(encoded)} policy traces; exactly one is required"
        )
    else:
        try:
            document = json.loads(encoded[0])
        except json.JSONDecodeError as error:
            reasons.append(f"installed fdu policy trace was not JSON: {error}")
        else:
            if not isinstance(document, dict):
                reasons.append("installed fdu policy trace was not a JSON object")
            else:
                diagnostics = document
    return diagnostics, "\n".join(residual), reasons


_DUST_TOTAL = re.compile(r"^\s*(?P<bytes>[0-9]+)B\s+.+$")


def _dust_semantics(
    stdout: bytes, stderr: str, oracle: Mapping[str, Any]
) -> Tuple[Optional[int], Optional[str]]:
    """Parse dust's exact byte total and prove it matches the independent walk."""
    if stderr.strip():
        return None, "dust reported traversal warnings or errors"
    try:
        lines = [line for line in stdout.decode("utf-8").splitlines() if line.strip()]
    except UnicodeDecodeError as error:
        return None, f"dust output was not UTF-8: {error}"
    if len(lines) != 1:
        return None, "dust output did not contain exactly one total row"
    matched = _DUST_TOTAL.fullmatch(lines[0])
    if matched is None:
        return None, "dust total row was not an exact byte value"
    if oracle["counts"].get("symlinks", 0):
        return None, "dust claim-grade contract excludes symlink-bearing subjects"
    observed = int(matched.group("bytes"))
    duplicate_allocated = (oracle.get("hardlinks") or {}).get("duplicate_allocated_bytes", 0)
    expected = int(oracle["sizes"]["allocated_bytes"]) - int(duplicate_allocated)
    if observed != expected:
        return (
            observed,
            f"dust allocated total {observed} disagrees with independent oracle {expected}",
        )
    return observed, None


def _summary_semantic_digest(stdout: bytes) -> Tuple[Optional[str], Optional[str]]:
    """Hash stable report semantics while excluding timestamps and the absolute root."""
    digest, _summary, error = _summary_semantics(stdout)
    return digest, error


def _summary_semantics(
    stdout: bytes,
) -> Tuple[Optional[str], Optional[Mapping[str, Any]], Optional[str]]:
    """Return stable report identity and independently checkable summary values."""
    try:
        document = json.loads(stdout)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        return None, None, f"summary output was not JSON: {error}"
    if not isinstance(document, dict):
        return None, None, "summary output was not a JSON object"
    if document.get("source") != "cold_scan" or document.get("freshness") != "fresh":
        return None, None, "cache-off summary was not a fresh cold scan"
    if document.get("complete") is not True or document.get("errors") != []:
        return None, None, "summary output was partial or reported errors"
    reports = document.get("reports")
    if (
        not isinstance(reports, list)
        or len(reports) != 1
        or not isinstance(reports[0], dict)
        or reports[0].get("view") != "summary"
    ):
        return None, None, "summary output did not contain exactly one summary report"
    summary = reports[0].get("summary")
    if not isinstance(summary, dict):
        return None, None, "summary report did not contain a summary object"
    stable = {
        "schema": document.get("schema"),
        "source": document.get("source"),
        "freshness": document.get("freshness"),
        "complete": document.get("complete"),
        "errors": document.get("errors"),
        "reports": reports,
    }
    encoded = json.dumps(stable, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest(), summary, None


def _summary_oracle_error(summary: Mapping[str, Any], oracle: Mapping[str, Any]) -> Optional[str]:
    """Return why an fdu summary disagrees with the independent Python walk."""
    counts = oracle["counts"]
    sizes = oracle["sizes"]
    expected = {
        "files": counts["files"],
        # fdu's summary describes descendants; the fingerprint also counts the root.
        "dirs": counts["directories"] - 1,
        "bytes": sizes["apparent_bytes"],
        "allocated": sizes["allocated_bytes"],
        "newest_mtime_ns": oracle["newest_file_mtime_ns"],
    }
    for field, wanted in expected.items():
        observed = summary.get(field)
        if observed != wanted:
            return f"summary {field}={observed!r} disagrees with independent oracle {wanted!r}"
    return None


def _statistics(document: Mapping[str, Any]) -> Dict[str, Any]:
    anchor = str(document["anchor"])
    samples = list(document["samples"])
    conditions = document.get("conditions")
    conditions = conditions if isinstance(conditions, Mapping) else {}
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
            "competitor_vs_fdu": _paired(
                samples, pair=competitor, metric_names=per_tool[anchor]["metrics"]
            ),
        }
        fdu_vs_competitor = _paired_named(
            samples,
            pair=competitor,
            control=competitor,
            candidate=anchor,
            metric_names=per_tool[anchor]["metrics"],
        )
        policy_stability = _installed_policy_stability(
            samples,
            pair=competitor,
            anchor=anchor,
            required_samples=int(conditions.get("trials", 3)),
        )
        fdu_vs_competitor["policy_stability"] = policy_stability
        fdu_vs_competitor["qualification"] = _release_qualification(
            fdu_vs_competitor,
            campaign_stage=str(conditions.get("campaign_stage", "exploratory")),
            required_pairs=int(conditions.get("trials", 3)),
            policy_stability=policy_stability,
        )
        statistics_document[competitor]["fdu_vs_competitor"] = fdu_vs_competitor
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
            "stdout_hashes": len(
                {sample["stdout_sha256"] for sample in selected if sample.get("stdout_sha256")}
            ),
            "semantic_hashes": len(
                {sample["semantic_sha256"] for sample in selected if sample.get("semantic_sha256")}
            ),
            "stderr_nonempty_samples": sum(
                1 for sample in selected if sample.get("stderr_bytes", 0)
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
    return result


def _paired(
    samples: Sequence[Mapping[str, Any]], *, pair: str, metric_names: Iterable[str]
) -> Dict[str, Any]:
    anchor = next(
        str(sample["tool"])
        for sample in samples
        if sample["pair"] == pair and sample["tool"] != pair
    )
    return _paired_named(
        samples,
        pair=pair,
        control=anchor,
        candidate=pair,
        metric_names=metric_names,
    )


def _paired_named(
    samples: Sequence[Mapping[str, Any]],
    *,
    pair: str,
    control: str,
    candidate: str,
    metric_names: Iterable[str],
) -> Dict[str, Any]:
    """Compare ``candidate`` with ``control``; positive change is worse."""
    result: Dict[str, Any] = {}
    for metric in metric_names:
        indexed: Dict[str, Dict[int, int]] = {}
        for name in (control, candidate):
            indexed[name] = {
                int(sample["ordinal"]): int(sample["metrics"][metric])
                for sample in samples
                if sample["pair"] == pair
                and sample["tool"] == name
                and not sample["warmup"]
                and sample["valid"]
                and sample["metrics"].get(metric) is not None
            }
        ordinals = sorted(set(indexed[control]) & set(indexed[candidate]))
        pairs = [(indexed[control][ordinal], indexed[candidate][ordinal]) for ordinal in ordinals]
        deltas = [candidate_value - control_value for control_value, candidate_value in pairs]
        ratios = [
            (candidate_value - control_value) / control_value
            for control_value, candidate_value in pairs
            if control_value
        ]
        if len(pairs) < 3:
            result[metric] = None
            continue
        low, high = measure._bootstrap_median_interval(ratios) if ratios else (None, None)
        delta_low, delta_high = measure._bootstrap_median_interval(deltas)
        median_change_pct = round(statistics.median(ratios) * 100, 3) if ratios else None
        interval_pct = (
            [round(low * 100, 3), round(high * 100, 3)]
            if low is not None and high is not None
            else None
        )
        result[metric] = {
            "pairs": len(pairs),
            "median_delta": statistics.median(deltas),
            "ci95_delta": [round(delta_low, 3), round(delta_high, 3)],
            "median_change_pct": median_change_pct,
            "ci95_change_pct": interval_pct,
            "direction": measure._direction(low, high),
            "noninferiority": measure._noninferiority_classification(
                median_change_pct, interval_pct
            ),
        }
    return result


def _release_qualification(
    comparison: Mapping[str, Any],
    *,
    campaign_stage: str,
    required_pairs: int,
    policy_stability: Optional[Mapping[str, Any]] = None,
) -> Dict[str, Any]:
    """Apply the pre-registered wall and resource gates to fdu versus a comparator."""
    reasons: List[str] = []
    wall = comparison.get("wall_ns")
    wall_classification = (
        wall.get("noninferiority") if isinstance(wall, Mapping) else "inconclusive"
    )
    if not isinstance(wall, Mapping) or wall.get("pairs") != required_pairs:
        reasons.append("the held-out wall comparison does not contain every fixed-N pair")
    if campaign_stage == "held-out" and (
        not isinstance(policy_stability, Mapping) or policy_stability.get("stable") is not True
    ):
        reasons.append(
            "installed fdu policy histories are missing, structurally harmful, or unstable"
        )

    resources: Dict[str, str] = {}
    for metric, limit in measure.RESOURCE_REGRESSION_LIMITS_PCT.items():
        entry = comparison.get(metric)
        if campaign_stage == "held-out" and (
            not isinstance(entry, Mapping) or entry.get("pairs") != required_pairs
        ):
            resources[metric] = "inconclusive"
            reasons.append(f"{metric} does not contain every fixed-N pair")
            continue
        interval = entry.get("ci95_change_pct") if isinstance(entry, Mapping) else None
        delta_interval = entry.get("ci95_delta") if isinstance(entry, Mapping) else None
        if not isinstance(interval, list) or len(interval) != 2:
            if (
                isinstance(delta_interval, list)
                and len(delta_interval) == 2
                and delta_interval[1] <= 0
            ):
                resources[metric] = "accepted"
            elif (
                isinstance(delta_interval, list)
                and len(delta_interval) == 2
                and delta_interval[0] > 0
            ):
                resources[metric] = "rejected"
                reasons.append(
                    f"{metric} increased from a zero baseline, so its relative limit is undefined"
                )
            else:
                resources[metric] = "inconclusive"
                reasons.append(f"{metric} is missing or has no paired interval")
        elif interval[1] <= limit:
            resources[metric] = "accepted"
        elif interval[0] > limit:
            resources[metric] = "rejected"
            reasons.append(f"{metric} exceeds its +{limit:g}% resource limit")
        else:
            resources[metric] = "inconclusive"
            reasons.append(f"{metric} does not establish its +{limit:g}% resource limit")

    major = comparison.get("major_faults")
    if campaign_stage == "held-out" and (
        not isinstance(major, Mapping) or major.get("pairs") != required_pairs
    ):
        resources["major_faults"] = "inconclusive"
        reasons.append("major_faults does not contain every fixed-N pair")
        major_interval = None
    else:
        major_interval = major.get("ci95_delta") if isinstance(major, Mapping) else None
    if not isinstance(major_interval, list) or len(major_interval) != 2:
        resources["major_faults"] = "inconclusive"
        reasons.append("major_faults is missing or has no paired delta interval")
    elif major_interval[1] <= measure.MAJOR_FAULT_DELTA_LIMIT:
        resources["major_faults"] = "accepted"
    elif major_interval[0] > measure.MAJOR_FAULT_DELTA_LIMIT:
        resources["major_faults"] = "rejected"
        reasons.append("major_faults exceeds its zero-increase limit")
    else:
        resources["major_faults"] = "inconclusive"
        reasons.append("major_faults does not establish its zero-increase limit")

    classification = wall_classification
    if wall_classification == "inferior" or "rejected" in resources.values():
        classification = "inferior"
    elif reasons or wall_classification == "inconclusive":
        classification = "inconclusive"
    confirmable = (
        campaign_stage == "held-out"
        and classification in {"superior", "noninferior"}
        and not reasons
    )
    return {
        "campaign_stage": campaign_stage,
        "classification": classification,
        "confirmable": confirmable,
        "major_fault_delta_limit": measure.MAJOR_FAULT_DELTA_LIMIT,
        "noninferiority_margin_pct": measure.NONINFERIORITY_MARGIN_PCT,
        "reasons": reasons,
        "resource_limits_pct": dict(measure.RESOURCE_REGRESSION_LIMITS_PCT),
        "resources": resources,
    }


def _installed_policy_stability(
    samples: Sequence[Mapping[str, Any]],
    *,
    pair: str,
    anchor: str,
    required_samples: int,
) -> Dict[str, Any]:
    """Require one reproducible trace signature and no structurally harmful history."""
    histories: List[Tuple[Any, Optional[str], Optional[str]]] = []
    missing = 0
    selected = [
        sample
        for sample in samples
        if sample.get("pair") == pair and sample.get("tool") == anchor and not sample.get("warmup")
    ]
    for sample in selected:
        diagnostics = sample.get("scan_diagnostics")
        policy = diagnostics.get("worker_policy") if isinstance(diagnostics, Mapping) else None
        windows = policy.get("windows") if isinstance(policy, Mapping) else None
        if not sample.get("valid") or not isinstance(windows, list):
            missing += 1
            continue
        outcome = policy.get("outcome")
        histories.append(
            (
                outcome,
                measure._order_sensitive_policy_history(outcome, windows),
                measure._harmful_policy_history(outcome, windows),
            )
        )
    signatures = sorted(set(histories), key=repr)
    harmful = sum(harm is not None for _outcome, _sensitivity, harm in histories)
    order_sensitive = sum(sensitivity is not None for _outcome, sensitivity, _harm in histories)
    return {
        "expected_histories": required_samples,
        "harmful_histories": harmful,
        "order_sensitive_histories": order_sensitive,
        "order_sensitivity_frequency": (
            round(order_sensitive / len(histories), 6) if histories else None
        ),
        "histories": len(histories),
        "missing_histories": missing,
        "rule": "zero-structurally-harmful-and-one-outcome-sensitivity-signature-v2",
        "signatures": [
            {"harm": harm, "outcome": outcome, "sensitivity": sensitivity}
            for outcome, sensitivity, harm in signatures
        ],
        "stable": (
            len(selected) == required_samples
            and len(histories) == required_samples
            and missing == 0
            and harmful == 0
            and len(signatures) == 1
        ),
    }


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
        _cache_note(document["conditions"]),
        "",
        _hardlink_note(tree_document),
        "",
        "| Tool | Work class | Median wall | Versus paired anchor | 95% interval | Peak RSS |",
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
            "## Release qualification",
            "",
            "| Comparator | fdu classification | Confirmable | Policy stable | Reasons |",
            "| --- | --- | --- | --- | --- |",
        ]
    )
    for name in document["competitor_order"]:
        comparison = document["statistics"][name]["fdu_vs_competitor"]
        qualification = comparison["qualification"]
        policy = comparison["policy_stability"]
        reasons = qualification.get("reasons") or []
        lines.append(
            "| "
            + " | ".join(
                [
                    name,
                    str(qualification.get("classification", "inconclusive")),
                    "yes" if qualification.get("confirmable") else "no",
                    "yes" if policy.get("stable") else "no",
                    "; ".join(map(str, reasons)) or "none",
                ]
            )
            + " |"
        )
    lines.extend(
        [
            "",
            "Positive percentages mean the competitor took more wall time than the immediately "
            "adjacent anchor run. External tools are calibration references, not optimization "
            "verdicts; work classes and command templates are retained in the JSON artifact.",
            "",
            f"Invalid timed samples: {document['invalid_samples']}. ",
            f"Semantic mismatches: {len(document.get('semantic_mismatches', []))}. ",
            (
                "Independent summary-oracle mismatches: "
                f"{len(document.get('summary_oracle_mismatches', []))}. "
            ),
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


def _warm_cache_evidence(warmups: int) -> Dict[str, Any]:
    """Describe what establishes the repeated-workload filesystem-cache state."""
    if warmups < MIN_TOOL_COMPARISON_WARMUPS:
        raise ComparisonError("a warm-steady tool comparison requires at least one warmup per tool")
    return {
        "pre_run_full_tree_fingerprints": PRE_RUN_FINGERPRINT_PASSES,
        "minimum_full_tree_warmups_per_tool": warmups,
        "residency_claim": "repeated-workload-steady-state",
    }


def _cache_note(conditions: Mapping[str, Any]) -> str:
    """Render the cache claim without implying that all metadata remained resident."""
    state = conditions.get("os_cache", "unspecified")
    evidence = conditions.get("os_cache_evidence")
    if not isinstance(evidence, Mapping):
        return f"OS filesystem cache: {state}."
    fingerprints = evidence.get("pre_run_full_tree_fingerprints", 0)
    warmups = evidence.get("minimum_full_tree_warmups_per_tool", 0)
    return (
        f"OS filesystem cache: {state}, after {fingerprints} complete independent "
        f"fingerprint and at least {warmups} full-tree warmups per tool. This is a "
        "repeated-workload state, not a claim that every metadata object remained "
        "resident."
    )


def _parse_tool(specification: str, *, expected: Optional[str] = None) -> Tool:
    descriptor, separator, raw_path = specification.partition("=")
    if not separator or not descriptor or not raw_path:
        raise ComparisonError(f"tool {specification!r} must be [LABEL:]CONTRACT=PATH")
    label, alias, contract_name = descriptor.partition(":")
    if not alias:
        label = descriptor
        contract_name = descriptor
    if not label or not contract_name:
        raise ComparisonError(f"tool {specification!r} must be [LABEL:]CONTRACT=PATH")
    if expected is not None and contract_name != expected:
        raise ComparisonError(f"expected {expected}=PATH, got {specification!r}")
    contract = CONTRACTS.get(contract_name)
    if contract is None:
        raise ComparisonError(
            f"unsupported tool contract {contract_name!r}; "
            f"choose from {', '.join(sorted(CONTRACTS))}"
        )
    binary = Path(raw_path).resolve()
    if not binary.is_file():
        raise ComparisonError(f"{label} binary does not exist: {binary}")
    return Tool(name=label, contract=contract, binary=binary)


def _identity(tool: Tool) -> Dict[str, Any]:
    content = tool.binary.read_bytes()
    return {
        "name": tool.name,
        "contract": tool.contract.name,
        "work_class": tool.contract.work_class,
        "description": tool.contract.description,
        "command": list(tool.contract.argv),
        "version": _version(tool),
        "binary_sha256": hashlib.sha256(content).hexdigest(),
        "binary_size_bytes": len(content),
    }


def _version(tool: Tool) -> str:
    if not tool.contract.version_argv:
        return "version command unavailable; binary hash is authoritative"
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
    wall_ns = sample["metrics"].get("wall_ns")
    wall = f"{wall_ns / 1e9:7.3f} s" if isinstance(wall_ns, int) else "    n/a"
    tag = "warmup" if sample["warmup"] else f"#{sample['ordinal']:02d}"
    validity = "" if sample["valid"] else f" INVALID: {'; '.join(sample['reasons'])}"
    print(
        f"[{position:3d}/{total}] {sample['pair']:<8} {sample['tool']:<8} "
        f"{tag:<6} {wall}{validity}",
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


def _hardlink_note(tree_document: Mapping[str, Any]) -> str:
    hardlinks = tree_document.get("hardlinks", {})
    duplicates = int(hardlinks.get("duplicate_file_entries", 0))
    allocated = int(hardlinks.get("duplicate_allocated_bytes", 0))
    if duplicates == 0:
        return (
            "The fingerprint found no duplicate in-tree hard-link entries, so hard-link "
            "accounting does not distinguish tool totals on this subject."
        )
    return (
        f"The fingerprint found {duplicates:,} duplicate in-tree hard-link entries "
        f"representing {allocated:,} path-counted allocated bytes. Tools differ in "
        "hard-link attribution; this is a traversal comparison, not an assertion that "
        "their reported byte totals are identical."
    )


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
