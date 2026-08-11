"""Evaluate benchmark latency and resource gates from one paired comparison."""

from __future__ import annotations

from typing import Any, Dict, Mapping, Optional, Sequence

from benchmarks.realtree import ledger

RESOURCE_METRICS = ("cpu_ns", "peak_rss_bytes")
MIN_DEGENERATE_RESOURCE_SAMPLES = 4


def resource_measurement_errors(
    document: Mapping[str, Any], *, selected_variants: Sequence[str]
) -> Dict[str, str]:
    """Reject a resource signal that cannot distinguish selected child processes.

    On some Linux launch paths, ``wait4.ru_maxrss`` can be pinned to the Python
    launcher's pre-exec high-water mark. A single exact value across every job and both
    variants is therefore not evidence that their memory use is equal.
    """
    selected = set(selected_variants)
    peak_rss = [
        (sample.get("metrics") or {}).get("peak_rss_bytes")
        for sample in document.get("samples") or []
        if sample.get("variant") in selected
        and sample.get("valid") is True
        and sample.get("warmup") is False
    ]
    measured = [value for value in peak_rss if type(value) is int and value >= 0]
    if (
        len(measured) >= MIN_DEGENERATE_RESOURCE_SAMPLES
        and len(set(measured)) == 1
    ):
        return {
            "peak_rss_bytes": (
                "identical peak RSS for every selected non-warmup sample across the "
                "run; the process-launcher floor may dominate this host measurement"
            )
        }
    return {}


def resource_guardrail(
    comparison: Mapping[str, Any],
    *,
    metric: str,
    maximum_regression_pct: float,
    unmeasured_reason: Optional[str] = None,
) -> Dict[str, Any]:
    """Evaluate whether a paired resource regression is established above a limit."""
    if metric not in RESOURCE_METRICS:
        raise ValueError(f"unsupported resource guardrail metric {metric!r}")
    if maximum_regression_pct < 0:
        raise ValueError(f"{metric} regression threshold must be non-negative")
    if unmeasured_reason:
        return {
            "metric": metric,
            "maximum_regression_pct": maximum_regression_pct,
            "observed_change_pct": None,
            "ci95_low_pct": None,
            "ci95_high_pct": None,
            "status": "not-measured",
            "reason": unmeasured_reason,
        }
    entry = (comparison.get("metrics") or {}).get(metric)
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


def evaluate(
    comparison: Mapping[str, Any],
    *,
    invalid_samples: int = 0,
    tree_mutated: bool = False,
    maximum_cpu_regression_pct: float = 10.0,
    maximum_rss_regression_pct: float = 10.0,
    unmeasured_resources: Optional[Mapping[str, str]] = None,
) -> Dict[str, Any]:
    """Return the complete per-cell decision without granting a resource waiver."""
    measurement_errors = unmeasured_resources or {}
    latency = ledger.verdict(comparison)
    guardrails = [
        resource_guardrail(
            comparison,
            metric="cpu_ns",
            maximum_regression_pct=maximum_cpu_regression_pct,
            unmeasured_reason=measurement_errors.get("cpu_ns"),
        ),
        resource_guardrail(
            comparison,
            metric="peak_rss_bytes",
            maximum_regression_pct=maximum_rss_regression_pct,
            unmeasured_reason=measurement_errors.get("peak_rss_bytes"),
        ),
    ]
    failed = [
        gate["metric"]
        for gate in guardrails
        if gate["status"] in {"failed", "not-measured"}
    ]
    if tree_mutated or invalid_samples:
        overall = "invalid"
        reasons = []
        if tree_mutated:
            reasons.append("the workload changed during the run")
        if invalid_samples:
            reasons.append(f"{invalid_samples} selected sample(s) failed the oracle")
        reason = "; ".join(reasons)
    elif not latency["accepted"]:
        overall = "rejected"
        reason = f"latency gate: {latency['reason']}"
    elif failed:
        overall = "rejected"
        reason = "resource guardrail failed or was not measured: " + ", ".join(failed)
    else:
        overall = "accepted"
        reason = "latency gate and both resource guardrails passed"
    return {
        "latency_gate_passed": bool(latency["accepted"]),
        "latency_reason": latency["reason"],
        "resource_guardrails": guardrails,
        "overall_decision": overall,
        "reason": reason,
    }


__all__ = [
    "MIN_DEGENERATE_RESOURCE_SAMPLES",
    "RESOURCE_METRICS",
    "evaluate",
    "resource_guardrail",
    "resource_measurement_errors",
]
