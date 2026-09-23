"""Read `fdu.report/7` JSON for the correctness runbook scripts.

Both comparison scripts compare an answer with a cold oracle and check separately how the
answer was produced. The answer is the report without the fields that describe a single
delivery: `provenance` (source, freshness, and timings) and the report's reference
instant with the ages measured from it. Every row also carries `mtime_ns`, so dropping
`age_ns` loses nothing.
"""

from __future__ import annotations

import json

# Fields that legitimately differ between two runs of one request.
DELIVERY_FIELDS = frozenset({"provenance", "age_reference_ns", "age_ns", "elapsed_ns"})


def parse(out: str) -> dict | None:
    try:
        doc = json.loads(out)
    except json.JSONDecodeError:
        return None
    return doc if isinstance(doc, dict) else None


def answer(out: str) -> object:
    """The report with its delivery fields removed, or the raw text if it is not JSON."""
    doc = parse(out)
    if doc is None:
        return out

    def scrub(node: object) -> object:
        if isinstance(node, dict):
            return {k: scrub(v) for k, v in node.items() if k not in DELIVERY_FIELDS}
        if isinstance(node, list):
            return [scrub(item) for item in node]
        return node

    return scrub(doc)


def _provenance(out: str) -> dict:
    return (parse(out) or {}).get("provenance") or {}


def source_of(out: str) -> str | None:
    """How the answer was produced: `cold_scan`, `warm_revalidate`, or `cache_only`."""
    return _provenance(out).get("source")


def freshness_of(out: str) -> str | None:
    return _provenance(out).get("freshness")


def is_complete(out: str) -> bool | None:
    """Whether the report says its tree was fully observed; None if it cannot be read."""
    status = (parse(out) or {}).get("status") or {}
    complete = status.get("complete")
    return complete if isinstance(complete, bool) else None
