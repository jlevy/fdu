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

import gzip
import json
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence

from benchmarks.atomic_write import open_atomic, write_text_atomic

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


def job_invalid_samples(job_statistics: Mapping[str, Any]) -> int:
    """Timed samples the harness invalidated in one job, summed over every variant.

    A sample is invalid when the oracle disagreed, the command failed or timed out, the
    quiet-host gate saw pressure at its boundary, or the tree moved. Every variant
    counts, not only the pair being compared: the arms share one interleaved fixed-N
    cell, so a cell with any invalid sample is rerun whole rather than read around.
    """
    return sum(
        int((entry or {}).get("invalid") or 0)
        for entry in (job_statistics.get("variants") or {}).values()
    )


def verdict_label(decision: Mapping[str, Any]) -> str:
    """The one word printed for a verdict: ACCEPT, REJECT, or INCONCLUSIVE."""
    if decision.get("inconclusive"):
        return "INCONCLUSIVE"
    return "ACCEPT" if decision.get("accepted") else "REJECT"


def verdict(
    comparison: Mapping[str, Any],
    *,
    invalid_samples: int,
    metric: str = "wall_ns",
) -> Dict[str, Any]:
    """Decide accept/reject for one comparison on one metric.

    Note this asks only the accept question. Use :func:`evidence_label` when reporting
    what a metric actually did: a metric can fail the accept rule by regressing, and
    printing that as "no" is how a regression gets read as noise.

    ``invalid_samples`` is the job's count from :func:`job_invalid_samples`. Any nonzero
    count makes the verdict inconclusive, neither accept nor reject: the valid pairs
    that remain are a subset the host chose, and the accept rule requires that no
    sample was invalidated. Exit code 3 already said so; this makes the printed word
    say it too, so nobody reads ACCEPT off a cell that has to be run again. It has no
    default because a caller that left it out would do exactly that: the surviving
    pairs of an inconclusive cell can clear the accept arithmetic on their own.
    """
    if invalid_samples:
        entry = (comparison.get("metrics") or {}).get(metric) or {}
        plural = "" if invalid_samples == 1 else "s"
        return {
            "accepted": False,
            "inconclusive": True,
            "reason": f"{invalid_samples} invalid sample{plural}; rerun the cell whole",
            "change_pct": entry.get("median_change_pct"),
        }
    decision = _accept_rule(comparison, metric=metric)
    decision["inconclusive"] = False
    return decision


def _accept_rule(comparison: Mapping[str, Any], *, metric: str) -> Dict[str, Any]:
    """The accept arithmetic on a cell whose every sample was valid."""
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
                "**Invalid samples (oracle, exit, timeout, host pressure, or tree change): "
                + ", ".join(f"`{name}` {count}" for name, count in invalid.items())
                + ". The cell is inconclusive; rerun it whole.**"
            )
        lines.append("")

        job_invalid = job_invalid_samples(stats)
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
            decision = verdict(comparison, invalid_samples=job_invalid)
            lines.append("")
            lines.append(
                f"**Verdict on wall time: {verdict_label(decision)}** — {decision['reason']}"
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
    write_text_atomic(destination, render(document, profiles=profiles), encoding="utf-8")


def load(path: Path) -> Dict[str, Any]:
    """Read a stored run document, plain (``*.json``) or gzipped (``*.json.gz``).

    Committed run artifacts are stored gzipped: a run is hundreds of kilobytes of JSON
    that nobody reviews line by line. Older records, and a run the harness has just
    written, are plain. Every reader of a run comes through here, so each accepts both
    forms. The suffix decides, so a ``.gz`` that is not gzip fails loudly instead of
    being read as something else.
    """
    if path.suffix == ".gz":
        with gzip.open(path, "rt", encoding="utf-8") as source:
            return json.load(source)
    return json.loads(path.read_text(encoding="utf-8"))


def store(run: Path, destination: Path) -> None:
    """Commit the run document at ``run`` as ``destination`` (``*.json.gz``).

    The bytes are the run's own, compressed with no file name and an mtime of 0, so
    nothing about when or where the file was written lands in it, and compressing an
    unchanged run again reproduces the committed bytes. The file is written whole: a
    crash leaves no truncated artifact for a record to name.
    """
    if destination.suffix != ".gz":
        raise ValueError(f"a stored run is gzipped, so {destination} must end in .gz")
    payload = run.read_bytes()
    destination.parent.mkdir(parents=True, exist_ok=True)
    with (
        open_atomic(destination, "wb") as raw,
        gzip.GzipFile(filename="", mode="wb", fileobj=raw, compresslevel=9, mtime=0) as out,
    ):
        out.write(payload)
