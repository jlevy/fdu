"""Read `fdu.report/7` JSON for the correctness runbook scripts.

Both comparison scripts compare an answer with a cold oracle and check separately how the
answer was produced. The answer is the report without the two root fields that describe a
single delivery: `provenance` (source, freshness, and timings) and `age_reference_ns`, the
instant ages are measured from. Row ages move with that instant, so instead of comparing
them across runs, each report's ages are checked against its own reference instant.
"""

from __future__ import annotations

import json

# Root fields that legitimately differ between two runs of one request.
DELIVERY_FIELDS = frozenset({"provenance", "age_reference_ns"})


def parse(out: str) -> dict | None:
    try:
        doc = json.loads(out)
    except json.JSONDecodeError:
        return None
    return doc if isinstance(doc, dict) else None


def answer(out: str) -> object:
    """The report without its delivery fields and row ages, or the raw text if not JSON."""
    doc = parse(out)
    if doc is None:
        return out

    def strip_ages(node: object) -> object:
        if isinstance(node, dict):
            return {k: strip_ages(v) for k, v in node.items() if k != "age_ns"}
        if isinstance(node, list):
            return [strip_ages(item) for item in node]
        return node

    return strip_ages({k: v for k, v in doc.items() if k not in DELIVERY_FIELDS})


def age_problems(out: str) -> list[str]:
    """Rows whose `age_ns` is not their reference instant minus their `mtime_ns`.

    An age is unknown (null) when the report has no reference instant, when the row's
    subtree was not listed in full (`complete: false`), or when the row has no mtime.
    """
    doc = parse(out)
    if doc is None:
        return []
    reference = doc.get("age_reference_ns")
    problems: list[str] = []

    def visit(node: object) -> None:
        if isinstance(node, dict):
            if "age_ns" in node:
                mtime = node.get("mtime_ns")
                known = reference is not None and mtime is not None
                expected = (
                    reference - mtime if known and node.get("complete") is not False else None
                )
                if node["age_ns"] != expected:
                    problems.append(f"{node.get('path')}: age {node['age_ns']} != {expected}")
            for value in node.values():
                visit(value)
        elif isinstance(node, list):
            for item in node:
                visit(item)

    visit(doc)
    return problems


def reference_outside(out: str, started_ns: int, finished_ns: int) -> str | None:
    """Why the report's `age_reference_ns` is not the instant of this run, if it is not.

    Ages are checked against the report's own reference instant, so a report measuring
    from a stale instant (a snapshot's, say) with ages to match would pass that check.
    The reference must fall within the invocation that produced the report.
    """
    reference = (parse(out) or {}).get("age_reference_ns")
    if reference is None or started_ns <= reference <= finished_ns:
        return None
    return f"age_reference_ns {reference} outside the run [{started_ns}, {finished_ns}]"


def _provenance(out: str) -> dict:
    return (parse(out) or {}).get("provenance") or {}


def source_of(out: str) -> str | None:
    """How the answer was produced: `cold_scan`, `warm_revalidate`, or `cache_only`."""
    return _provenance(out).get("source")


def content_source_of(out: str) -> str | None:
    """How the content tier was produced: `scanned`, `revalidated`, or a cache label.

    The report-level source says only how the entries were produced: after any metadata
    run has stored a snapshot, a content request reports `warm_revalidate` even when every
    content record was read again. Whether content was served is this tier's answer.
    """
    return ((_provenance(out).get("tiers") or {}).get("content") or {}).get("source")


def freshness_of(out: str) -> str | None:
    return _provenance(out).get("freshness")


def is_complete(out: str) -> bool | None:
    """Whether the report says its tree was fully observed; None if it cannot be read."""
    status = (parse(out) or {}).get("status") or {}
    complete = status.get("complete")
    return complete if isinstance(complete, bool) else None


def reject_unknown_flags(args: list[str], known: set[str]) -> None:
    unknown = [arg for arg in args if arg.startswith("--") and arg not in known]
    if unknown:
        raise SystemExit(f"unknown option: {' '.join(unknown)}")
