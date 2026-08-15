"""Render a run into Markdown, and decide whether a candidate earned its place.

The accept rule lives here rather than in a person's head, because the whole point of
a written loop is that "it felt faster" never becomes evidence. A candidate is
accepted only when the paired median improves by more than the noise floor *and* the
bootstrap interval says the improvement is not a coincidence.

The rule cannot decide the other half, which is whether the change is worth its
complexity. That is a judgment, so the ledger records it as one: every experiment
carries a written verdict and a reason, including the ones that were fast and got
rejected anyway.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence

#: A change has to beat this to be worth carrying. Below it, run-to-run noise on a
#: laptop is the same size as the effect, and the code is more complex for nothing.
ACCEPT_THRESHOLD_PCT = -3.0

#: Metrics shown in the per-job table, in the order a reader wants them: what the
#: user waits for, what the engine spent, then where that time went.
REPORT_METRICS = (
    ("wall_ns", "wall", "ms"),
    ("component_ns", "component", "ms"),
    ("cpu_ns", "cpu total", "ms"),
    ("user_cpu_ns", "user", "ms"),
    ("system_cpu_ns", "system", "ms"),
    ("blocked_ns", "blocked (I/O+sched)", "ms"),
    ("peak_rss_bytes", "peak rss", "MiB"),
    ("minor_faults", "minor faults", "count"),
    ("involuntary_context_switches", "invol. ctx switches", "count"),
)


def evidence_label(entry: Mapping[str, Any]) -> str:
    """What one metric's interval actually says, in three words or fewer.

    Derived from the interval rather than read from a stored flag, so artifacts
    recorded before the fields were split render with the same honesty as new ones.
    """
    interval = entry.get("ci95_change_pct") or [None, None]
    low, high = interval[0], interval[1]
    if low is None or high is None:
        return "—"
    if high < 0:
        return "improved"
    if low > 0:
        return "**regressed**"
    return "unclear"


def verdict(comparison: Mapping[str, Any], *, metric: str = "wall_ns") -> Dict[str, Any]:
    """Decide accept/reject for one comparison on one metric.

    Note this asks only the accept question. Use :func:`evidence_label` when reporting
    what a metric actually did: a metric can fail the accept rule by regressing, and
    printing that as "no" is how a regression gets read as noise.
    """
    entry = (comparison.get("metrics") or {}).get(metric)
    if not entry:
        return {
            "accepted": False,
            "reason": f"no paired {metric} samples",
            "change_pct": None,
        }
    change = entry["median_change_pct"]
    interval = entry["ci95_change_pct"]
    if change is None:
        return {"accepted": False, "reason": "no paired ratio", "change_pct": None}
    # `passes_acceptance` is the field this decision belongs to; `significant` is its
    # deprecated alias, still read so pre-split comparisons decide the same way. Neither
    # is required: the interval alone determines the answer.
    accepted = entry.get(
        "passes_acceptance",
        entry.get(
            "significant", interval is not None and interval[1] is not None and interval[1] < 0
        ),
    )
    if not accepted:
        # Two different failures, and saying "includes no change" for both is how a
        # measured regression gets filed as noise.
        regressed = interval and interval[0] is not None and interval[0] > 0
        detail = "is a regression" if regressed else "includes no change"
        return {
            "accepted": False,
            "reason": (
                f"{change:+.2f}% median, and the 95% interval "
                f"[{interval[0]:+.2f}%, {interval[1]:+.2f}%] {detail}"
            ),
            "change_pct": change,
        }
    if change > ACCEPT_THRESHOLD_PCT:
        return {
            "accepted": False,
            "reason": (
                f"{change:+.2f}% is significant but under the "
                f"{abs(ACCEPT_THRESHOLD_PCT):.0f}% bar worth added complexity"
            ),
            "change_pct": change,
        }
    return {
        "accepted": True,
        "reason": (
            f"{change:+.2f}% median, 95% interval [{interval[0]:+.2f}%, {interval[1]:+.2f}%]"
        ),
        "change_pct": change,
    }


def render(document: Mapping[str, Any], *, profiles: Sequence[Mapping[str, Any]] = ()) -> str:
    """Render one measurement run as a Markdown section."""
    lines: List[str] = []
    tree = document["tree"]
    host = document["host"]
    conditions = document["conditions"]

    lines.append(f"# Real-tree performance run — {document['started_utc']}")
    lines.append("")
    if document.get("note"):
        lines.append(document["note"])
        lines.append("")

    lines.append("## Conditions")
    lines.append("")
    lines.append(
        f"- Tree `{tree['label']}` (`root_id` `{tree['root_id'][:16]}…`): "
        f"{tree['counts']['total']:,} entries, "
        f"{tree['counts']['directories']:,} directories, "
        f"{tree['counts']['files']:,} files, max depth {tree['max_depth']}, "
        f"{tree['sizes']['apparent_bytes'] / 2**30:.2f} GiB apparent."
    )
    lines.append(
        f"- Host: {host.get('cpu_model') or host['system']} "
        f"({host['cpu_count']} logical cores"
        + (
            f", {host['performance_cores']}P/{host['efficiency_cores']}E"
            if host.get("performance_cores")
            else ""
        )
        + f"), {host['system']} {host['release']}, {host.get('filesystem') or 'unknown fs'}."
    )
    lines.append(
        f"- {conditions['trials']} timed trials per variant, "
        f"{conditions['warmups']} discarded warmups, variants interleaved "
        f"({conditions['schedule']})."
    )
    lines.append(f"- OS page cache: {conditions['os_cache']}.")
    if conditions.get("campaign_stage"):
        lines.append(
            f"- Campaign stage: `{conditions['campaign_stage']}`; "
            f"interval `{conditions.get('confidence_interval', 'unspecified')}`; "
            f"stopping rule `{conditions.get('stopping_rule', 'unspecified')}`."
        )
    lines.append(
        "- Tree verified unchanged across the run"
        if not document["tree_mutated_during_run"]
        else "- **Tree changed during the run; these numbers are not comparable.**"
    )
    if document.get("baseline_drift"):
        lines.append(
            "- **Drift from the recorded baseline tree: "
            + "; ".join(document["baseline_drift"])
            + "**"
        )
    corpus = document.get("corpus")
    if isinstance(corpus, Mapping):
        lines.append(
            f"- Generated corpus `{corpus.get('recipe_id')}`; manifest "
            f"`{str(corpus.get('manifest_hash', ''))[:16]}…`; semantic digest "
            f"`{str(corpus.get('semantic_digest', ''))[:16]}…`."
        )
        phases = (corpus.get("topology") or {}).get("phases") or []
        for phase in phases:
            counts = phase.get("counts") or {}
            lines.append(
                f"  - `{phase.get('path_prefix')}`: {counts.get('total', 0):,} entries, "
                f"depth {phase.get('max_depth')}, fanout "
                f"{phase.get('max_directory_fanout')}."
            )
    lines.append("")

    lines.append("## Variants")
    lines.append("")
    lines.append("| variant | kind | sha256 | notes |")
    lines.append("| --- | --- | --- | --- |")
    for name, identity in document["variants"].items():
        lines.append(
            f"| `{name}` | {identity['kind']} | `{identity['sha256'][:12]}` | "
            f"{identity['notes'] or '—'} |"
        )
    lines.append("")

    for job_id, job in document["jobs"].items():
        stats = document["statistics"][job_id]
        lines.append(f"## `{job_id}` ({job['start_state']} start)")
        lines.append("")
        lines.append(job["description"])
        lines.append("")
        variants = list(document["variants"])
        header = "| metric | " + " | ".join(f"`{name}`" for name in variants) + " |"
        lines.append(header)
        lines.append("| --- |" + " --- |" * len(variants))
        for metric, label, unit in REPORT_METRICS:
            cells = []
            for name in variants:
                summary = stats["variants"][name]["metrics"].get(metric)
                cells.append(_cell(summary, unit))
            if all(cell == "—" for cell in cells):
                continue
            lines.append(f"| {label} | " + " | ".join(cells) + " |")
        lines.append("")
        lines.append(
            "_median ± MAD; n = "
            + ", ".join(f"{name}:{stats['variants'][name]['samples']}" for name in variants)
            + "_"
        )
        invalid = {
            name: stats["variants"][name]["invalid"]
            for name in variants
            if stats["variants"][name]["invalid"]
        }
        if invalid:
            lines.append("")
            lines.append(
                "**Rejected samples (oracle disagreement or nonzero exit): "
                + ", ".join(f"`{name}` {count}" for name, count in invalid.items())
                + "**"
            )
        lines.append("")

        for key, comparison in stats["comparisons"].items():
            lines.append(f"### Paired comparison: {key}")
            lines.append("")
            lines.append("| metric | median change | 95% interval | evidence | +3% decision |")
            lines.append("| --- | --- | --- | --- | --- |")
            for metric, label, _unit in REPORT_METRICS:
                entry = comparison["metrics"].get(metric)
                if not entry or entry["median_change_pct"] is None:
                    continue
                interval = entry["ci95_change_pct"]
                lines.append(
                    f"| {label} | {entry['median_change_pct']:+.2f}% | "
                    + (f"[{interval[0]:+.2f}%, {interval[1]:+.2f}%]" if interval else "—")
                    + f" | {evidence_label(entry)} | {entry.get('noninferiority', '—')} |"
                )
            decision = verdict(comparison)
            lines.append("")
            lines.append(
                f"**Verdict on wall time: "
                f"{'ACCEPT' if decision['accepted'] else 'REJECT'}** — {decision['reason']}"
            )
            qualification = comparison.get("qualification")
            if isinstance(qualification, Mapping):
                confirmable = (
                    "confirmable" if qualification.get("confirmable") else "not confirmable"
                )
                lines.append("")
                lines.append(
                    f"**Adaptive qualification: "
                    f"{str(qualification.get('classification', 'inconclusive')).upper()}** "
                    f"(`{qualification.get('campaign_stage', 'exploratory')}`, {confirmable})."
                )
                reasons = qualification.get("reasons") or []
                if reasons:
                    lines.append("Reasons: " + "; ".join(str(reason) for reason in reasons) + ".")
            lines.append("")

    if profiles:
        lines.append("## Profiles")
        lines.append("")
        lines.append(
            "Collected separately from timing, on a `profiling` build with symbols "
            "and `--repeat`. Attribution only; no timing claim comes from these."
        )
        lines.append("")
        for entry in profiles:
            lines.append(f"### {entry['label']} ({entry['total_samples']:,} samples)")
            lines.append("")
            lines.append("| layer | self time |")
            lines.append("| --- | --- |")
            for layer in entry["by_layer"]:
                lines.append(f"| {layer['layer']} | {layer['percent']:.2f}% |")
            lines.append("")
            lines.append("| symbol | self time |")
            lines.append("| --- | --- |")
            for frame in entry["self_time"][:12]:
                lines.append(f"| `{frame['symbol'][:76]}` | {frame['percent']:.2f}% |")
            lines.append("")

    return "\n".join(lines) + "\n"


def _cell(summary: Optional[Mapping[str, Any]], unit: str) -> str:
    if not summary:
        return "—"
    median = summary["median"]
    mad = summary["mad"]
    if unit == "ms":
        return f"{median / 1e6:.1f} ± {mad / 1e6:.1f}"
    if unit == "MiB":
        return f"{median / 2**20:.1f}"
    return f"{median:,.0f}"


def write(document: Mapping[str, Any], destination: Path, *, profiles=()) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(render(document, profiles=profiles), encoding="utf-8")


def load(path: Path) -> Dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))
