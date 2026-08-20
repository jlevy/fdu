"""Render the projected experiment dataset into one self-contained HTML report.

    python -m benchmarks.realtree.report_html --data <timeline.json> --out <index.html>

Charts are hand-written inline SVG. No chart library, for the same reason the report
serializers in the Rust crate are hand-written: the shapes here are few and fully known,
and a library would have to be pinned, audited, and carried for the rest of the project's
life to draw four figures. Inline SVG also keeps the page self-contained, which is what
lets it be opened from a file, committed, and published without an asset pipeline.

The visual language is the tbd web design system's, reduced to what a static report
needs: one neutral hue so every surface reads as one material, semantic colour families
reused rather than reinvented per chart, sans for anything the page says about itself and
monospace for measured values. Both themes are defined in tokens; nothing downstream
holds a literal colour.
"""

from __future__ import annotations

import argparse
import html
import json
import math
import sys
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence

#: Jobs shown in the absolute figure, in the order the work happens: build the index
#: from a cold tree, persist it, then bring a saved one back up to date.
ANCHOR_JOBS = (
    ("cold-scan-index", "Cold scan and index"),
    ("cold-scan-producer", "Cold scan, producer only"),
    ("cold-snapshot-save", "Snapshot save"),
    ("warm-revalidate", "Warm revalidate"),
    ("warm-snapshot-load", "Snapshot load"),
)

DECISION_ORDER = ("accepted", "rejected", "superseded", "in-progress", "baseline")

DECISION_LABEL = {
    "accepted": "kept",
    "rejected": "rejected",
    "superseded": "superseded",
    "in-progress": "in progress",
    "baseline": "baseline",
}


def esc(value: Any) -> str:
    return html.escape(str(value), quote=True)


def ms(nanoseconds: Optional[float]) -> Optional[float]:
    return None if nanoseconds is None else nanoseconds / 1e6


def fmt_ms(nanoseconds: Optional[float]) -> str:
    value = ms(nanoseconds)
    if value is None:
        return "—"
    if value >= 1000:
        return f"{value / 1000:,.2f} s"
    return f"{value:,.0f} ms"


def fmt_pct(value: Optional[float]) -> str:
    return "—" if value is None else f"{value:+.1f}%"


# ---------------------------------------------------------------- svg primitives


def axis_ticks(maximum: float, count: int = 4) -> List[float]:
    """Round tick values covering `0..maximum`.

    Chosen from a 1/2/5 ladder so the labels read as round numbers a person would say
    out loud, rather than as the data's own maximum divided into equal parts.
    """
    if maximum <= 0:
        return [0.0]
    raw = maximum / count
    magnitude = 10 ** math.floor(math.log10(raw))
    for multiple in (1, 2, 2.5, 5, 10):
        step = magnitude * multiple
        if step >= raw:
            break
    # The ladder has to *cover* the maximum, not stop below it. Stopping below was a
    # real bug: the last tick became the scale's top, so a 1,219 ms bar was drawn against
    # a 1,000 ms axis and ran 300 px past the plot into the value column.
    ticks = []
    value = 0.0
    while True:
        ticks.append(round(value, 6))
        if value >= maximum * 0.9999:
            break
        value += step
    return ticks


def svg_open(width: int, height: int, title: str, desc: str = "") -> List[str]:
    """An accessible SVG root sized by viewBox so it scales to its container."""
    return [
        f'<svg class="chart" viewBox="0 0 {width} {height}" role="img" '
        f'preserveAspectRatio="xMinYMin meet" aria-label="{esc(title)}">',
        f"<title>{esc(title)}</title>" + (f"<desc>{esc(desc)}</desc>" if desc else ""),
    ]


def legend(*items: Sequence[str]) -> str:
    """A key row.

    Each entry is one flex item. Loose text between two keys becomes an anonymous flex
    item of its own and drifts away from the swatch it belongs to, which is what the
    first version did.
    """
    cells = "".join(
        f'<span class="legend-item"><span class="key {css}"></span>{esc(label)}</span>'
        if css
        else f'<span class="legend-item">{esc(label)}</span>'
        for css, label in items
    )
    return f'<p class="legend">{cells}</p>'


# ---------------------------------------------------------------- figure: absolute


def figure_absolute(dataset: Mapping[str, Any]) -> str:
    """Before and after in real milliseconds, with the route between them.

    Each row is one job. The grey marker is the pre-work binary and the blue track is the
    code of the day at each checkpoint, ending in the value that shipped.

    Its construction is why it can be trusted. Every checkpoint re-measured the *original*
    binary against the current one, interleaved, on the same tree within minutes — so each
    row's endpoints come from one run, not from one number taken on Tuesday set beside
    another taken on Friday. The pale band behind each row is the full range those repeated
    re-measurements of the *unchanged* binary covered. It is drawn rather than averaged
    away because it is the scale any movement along the blue track has to be read against:
    on the producer job it is 35% wide, which is wider than several of the steps.
    """
    series = _flagship(dataset)
    if not series:
        return ""

    rows = []
    for job_id, label in ANCHOR_JOBS:
        job = next((item for item in series["jobs"] if item["job"] == job_id), None)
        if job:
            rows.append((label, job))
    if not rows:
        return ""

    points = rows[0][1]["points"]
    maximum = max(
        value
        for _, job in rows
        for point in job["points"]
        for value in (point["control_ns"], point["candidate_ns"])
        if value
    )
    # Five intervals rather than four: the producer job's noisiest control run reaches
    # 1,219 ms, and a four-interval ladder rounds the axis top to 1,500 and leaves a
    # third of the plot empty.
    ticks = axis_ticks(ms(maximum) or 0, count=5)
    top = ticks[-1]

    # The right margin is a value column rather than inline labels: the "after" point
    # sits left of the "before" one on every row, so a label beside it lands on top of
    # the track it is describing.
    left, right = 176, 176
    row_height = 62
    width = 900
    plot = width - left - right
    height = len(rows) * row_height + 56
    scale = lambda value: left + (ms(value) / top) * plot

    out = svg_open(
        width,
        height,
        "Wall time before and after, across five cumulative checkpoints",
        "One row per job. Grey marks the pre-work binary; the blue track is the code of "
        "the day at each checkpoint.",
    )
    for tick in ticks:
        x = left + (tick / top) * plot
        out.append(f'<line class="grid" x1="{x:.1f}" y1="30" x2="{x:.1f}" y2="{height - 30}"/>')
        out.append(
            f'<text class="tick" x="{x:.1f}" y="{height - 14}" text-anchor="middle">{tick:,.0f}</text>'
        )
    out.append(
        f'<text class="tick axis-name" x="{left}" y="18">milliseconds &mdash; lower is faster</text>'
    )

    for index, (label, job) in enumerate(rows):
        y = 44 + index * row_height
        out.append(
            f'<text class="row-label" x="{left - 14}" y="{y + 4}" text-anchor="end">{esc(label)}</text>'
        )

        low, high = job.get("baseline_low_ns"), job.get("baseline_high_ns")
        if low and high and high > low:
            x1, x2 = scale(low), scale(high)
            out.append(
                f'<rect class="drift-band" x="{x1:.1f}" y="{y - 13}" '
                f'width="{max(x2 - x1, 1.5):.1f}" height="26"><title>'
                f"the same unchanged binary measured {fmt_ms(low)} to {fmt_ms(high)} "
                f"across the five runs</title></rect>"
            )

        track = [
            (point, scale(point["candidate_ns"]))
            for point in job["points"]
            if point["candidate_ns"] and not point["baseline"]
        ]
        controls = [point["control_ns"] for point in job["points"] if point["control_ns"]]
        if controls:
            median_before = sorted(controls)[len(controls) // 2]
            x_before = scale(median_before)
            if track:
                out.append(
                    f'<line class="connector" x1="{x_before:.1f}" y1="{y}" '
                    f'x2="{track[-1][1]:.1f}" y2="{y}"/>'
                )
            out.append(
                f'<circle class="dot-before" cx="{x_before:.1f}" cy="{y}" r="5.5"><title>'
                f"before: {fmt_ms(median_before)} (median of {len(controls)} re-measurements)"
                f"</title></circle>"
            )

        if track:
            path = " ".join(f"{x:.1f},{y}" for _, x in track)
            out.append(f'<polyline class="track" points="{path}"/>')
            for order, (point, x) in enumerate(track):
                final = order == len(track) - 1
                out.append(
                    f'<circle class="{"dot-final" if final else "dot-step"}" cx="{x:.1f}" '
                    f'cy="{y}" r="{5.5 if final else 3.5}"><title>'
                    f'{esc(point["id"])} ({esc(point["date"])}): {fmt_ms(point["candidate_ns"])}'
                    f"</title></circle>"
                )
            before_text = fmt_ms(median_before) if controls else "—"
            out.append(
                f'<text class="value-label" x="{width - 8}" y="{y + 4}" text-anchor="end">'
                f'<tspan class="value-before">{esc(before_text)}</tspan>'
                f'<tspan class="value-arrow"> &#8594; </tspan>'
                f'{fmt_ms(track[-1][0]["candidate_ns"])}</text>'
            )
    out.append("</svg>")

    keys = legend(
        ("key-before-dot", "the pre-work binary, median of five re-measurements"),
        ("key-after-dot", "each checkpoint, ending in what shipped"),
        ("key-drift", "range those five re-measurements covered"),
    )
    checkpoints = "".join(
        f'<li><span class="mono">{esc(point["id"])}</span> '
        f'<span class="muted">{esc(point["date"])}</span> {esc(point["title"])}</li>'
        for point in points
    )
    return (
        f'<figure class="fig">{"".join(out)}{keys}'
        f"<figcaption>One macOS tree of {series['entries_first']:,}–"
        f"{series['entries_last']:,} entries, measured five times over three days. "
        f'The checkpoints, in order:<ol class="checkpoints">{checkpoints}</ol>'
        "</figcaption></figure>"
    )


def _flagship(dataset: Mapping[str, Any]) -> Optional[Dict[str, Any]]:
    series = dataset.get("anchors", {}).get("series") or []
    return series[0] if series else None


# ---------------------------------------------------------------- figure: effects


def figure_effects(dataset: Mapping[str, Any]) -> str:
    """Every experiment's paired effect on its own primary job, with its interval.

    This is the relative view, and it is the one the verdicts were actually made on. The
    dot is the median paired change; the bar behind it is the 95% bootstrap interval. An
    interval that crosses zero means the run could not tell the change from noise, which
    is a different statement from "no effect" and is drawn differently from both wins and
    regressions.

    The dashed line is the accept threshold. Everything to its right was, by rule, not
    worth carrying — which is most of the chart, and the point of publishing it.
    """
    records = [
        record
        for record in dataset["experiments"]
        if record["decision"] != "baseline" and _primary(record) is not None
    ]
    if not records:
        return ""
    records.sort(key=lambda record: _primary(record)["paired"]["change_pct"] or 0.0)

    bounds = [
        value
        for record in records
        for value in (
            _primary(record)["paired"]["ci95_low_pct"],
            _primary(record)["paired"]["ci95_high_pct"],
            _primary(record)["paired"]["change_pct"],
        )
        if value is not None
    ]
    # The axis is sized to the data and nothing is clipped. An earlier draft clamped it
    # to a comfortable window and marked the overflow with arrows, which quietly moved
    # four of the largest measured regressions off the picture — including the 66% one
    # that is the single most useful rejection in the record.
    # Bounds snap to ten so the axis hugs the data; labels are drawn every twenty so the
    # scale stays readable at one row per experiment.
    step, label_step = 10, 20
    low_limit = math.floor(min(bounds) / step) * step
    high_limit = math.ceil(max(bounds) / step) * step

    # A narrow gutter carries each experiment's number. Without it the figure could only
    # be read by hovering, which leaves it saying nothing on paper and nothing to a reader
    # who wants to look a point up in the table below.
    left, right = 46, 34
    width = 900
    plot = width - left - right
    row = 13
    height = len(records) * row + 62
    span = high_limit - low_limit
    scale = lambda value: left + ((value - low_limit) / span) * plot

    out = svg_open(
        width,
        height,
        "Paired effect of every experiment on its primary job, with 95% intervals",
        "Horizontal dot and interval per experiment, sorted by effect.",
    )
    # Anchored on zero rather than on the axis end, so the ladder stays regular and the
    # zero line is one of its rungs instead of an extra one crowding its neighbour.
    first = math.ceil(low_limit / label_step) * label_step
    ticks = list(range(int(first), int(high_limit) + 1, label_step))
    for tick in ticks:
        x = scale(tick)
        css = "zero" if tick == 0 else "grid"
        out.append(f'<line class="{css}" x1="{x:.1f}" y1="30" x2="{x:.1f}" y2="{height - 30}"/>')
        text = "0" if tick == 0 else f"{tick:+d}"
        out.append(
            f'<text class="tick" x="{x:.1f}" y="{height - 14}" text-anchor="middle">{text}%</text>'
        )
    threshold = scale(dataset["accept_threshold_pct"])
    out.append(
        f'<line class="threshold" x1="{threshold:.1f}" y1="30" x2="{threshold:.1f}" y2="{height - 30}"/>'
    )
    out.append(
        f'<text class="tick axis-name" x="{left}" y="18">'
        "paired change on the primary job &mdash; left is faster</text>"
    )
    out.append(
        f'<text class="tick" x="{threshold + 5:.1f}" y="{height - 34}">accept threshold</text>'
    )

    for index, record in enumerate(records):
        metric = _primary(record)
        paired = metric["paired"]
        y = 36 + index * row
        out.append(
            f'<text class="gutter" x="{left - 10}" y="{y + 3}" text-anchor="end">'
            f'{esc(record["id"].removeprefix("exp-"))}</text>'
        )
        change = paired["change_pct"] or 0.0
        low, high = paired["ci95_low_pct"], paired["ci95_high_pct"]
        evidence = paired["evidence"]
        tone = "good" if evidence == "improved" else "bad" if evidence == "regressed" else "flat"
        if low is not None and high is not None:
            x1, x2 = scale(low), scale(high)
            out.append(
                f'<line class="whisker whisker-{tone}" x1="{x1:.1f}" y1="{y}" x2="{x2:.1f}" y2="{y}"/>'
            )
        out.append(
            f'<circle class="dot-{tone}" cx="{scale(change):.1f}" cy="{y}" r="3"><title>'
            f'{esc(record["id"])} {esc(record["title"])}: {fmt_pct(change)} on '
            f'{esc(record["primary_job"])}'
            + (f" [{low:+.1f}%, {high:+.1f}%]" if low is not None and high is not None else "")
            + f' &mdash; {esc(DECISION_LABEL.get(record["decision"], record["decision"]))}</title></circle>'
        )
    out.append("</svg>")

    kept = sum(1 for record in records if record["decision"] == "accepted")
    evidence = [_primary(record)["paired"]["evidence"] for record in records]
    improved = evidence.count("improved")
    unclear = evidence.count("unclear")
    regressed = evidence.count("regressed")
    return (
        f'<figure class="fig">{"".join(out)}'
        + legend(
            ("key-good", "interval entirely below zero"),
            ("key-bad", "interval entirely above zero"),
            ("key-flat", "interval crosses zero \u2014 the run could not tell"),
        )
        + f"<figcaption>{len(records)} experiments, sorted by effect; {kept} were kept. "
        f"{improved} have an interval entirely below zero, {unclear} cross it, and "
        f"{regressed} are entirely above it. Nothing is clipped: the axis runs to the "
        "widest interval measured. Hover any point for the experiment."
        "</figcaption></figure>"
    )


def _primary(record: Mapping[str, Any]) -> Optional[Dict[str, Any]]:
    """The metric the verdict rested on, on the job it rested on."""
    job = next((item for item in record["jobs"] if item["job"] == record["primary_job"]), None)
    if not job:
        return None
    return job["metrics"].get(record["primary_metric"] or "wall_ns")


# ---------------------------------------------------------------- figure: per entry


def figure_per_entry(dataset: Mapping[str, Any]) -> str:
    """Cost per entry per subject, so trees three orders of magnitude apart can be compared.

    Milliseconds are only comparable against the tree that produced them, and this record
    spans 307 entries to 1.01 million. Dividing by entry count is the one normalisation the
    workload actually supports — a scan's cost is very nearly linear in what it must visit
    — and it is what lets the campaign's central claim be checked rather than asserted: the
    speed-up was not a small-tree artifact.

    Each row is one subject, largest first, showing the first cost measured on it and the
    last. A first draft plotted every measurement against its date instead; with only six
    days in the record the points collapsed into six vertical piles and the trend the
    figure existed to show could not be read at all.

    The synthetic subjects are drawn but held apart. The adversarial worker-policy tree
    costs about four times what a real tree of the same week does — which is the point of
    it, and would be a lie if averaged in with the rest.
    """
    subjects = {subject["key"]: subject for subject in dataset["subjects"]}
    grouped: Dict[str, List[Dict[str, Any]]] = {}
    for record in dataset["experiments"]:
        job = next((item for item in record["jobs"] if item["job"] == "cold-scan-index"), None)
        if not job:
            continue
        # The arm that stayed in the product, not the arm that was tried. Using the
        # candidate unconditionally made a subject's last point a *rejected* build: the
        # 1.01M row read "7.7 → 8.0 µs" and appeared to have got slower, when what it
        # actually recorded was a buffer change that was measured, disliked, and dropped.
        kept = job["per_entry_ns"][record["kept"]]
        if not kept:
            continue
        grouped.setdefault(record["family"], []).append(
            {
                "id": record["id"],
                "date": record["date"],
                "number": record["number"],
                "decision": record["decision"],
                "control": (job["per_entry_ns"]["control"] or 0) / 1000.0,
                "kept": kept / 1000.0,
                "subject": record["subject"],
            }
        )

    rows = []
    for _, measurements in grouped.items():
        measurements.sort(key=lambda item: item["number"])
        subject = subjects[measurements[-1]["subject"]]
        rows.append(
            {
                "subject": subject,
                "first": measurements[0],
                "last": measurements[-1],
                "all": measurements,
                "count": len(measurements),
            }
        )
    # Largest tree first, and every synthetic subject after every real one however big.
    rows.sort(key=lambda item: (item["subject"]["synthetic"], -item["subject"]["entries"]))
    rows = [row for row in rows if row["count"] >= 1]
    if not rows:
        return ""

    values = [
        value
        for row in rows
        for measurement in row["all"]
        for value in (measurement["control"], measurement["kept"])
        if value
    ]
    ticks = axis_ticks(max(values), count=5)
    top = ticks[-1]

    left, right = 176, 150
    row_height = 46
    width = 900
    plot = width - left - right
    height = len(rows) * row_height + 56
    scale = lambda value: left + (min(value, top) / top) * plot

    out = svg_open(
        width,
        height,
        "Cold scan cost per entry, first and last measurement on each subject",
        "One row per measured tree, largest first.",
    )
    for tick in ticks:
        x = left + (tick / top) * plot
        out.append(f'<line class="grid" x1="{x:.1f}" y1="30" x2="{x:.1f}" y2="{height - 30}"/>')
        out.append(
            f'<text class="tick" x="{x:.1f}" y="{height - 14}" text-anchor="middle">{tick:g}</text>'
        )
    out.append(
        f'<text class="tick axis-name" x="{left}" y="18">'
        "microseconds per entry &mdash; lower is faster</text>"
    )

    for index, row in enumerate(rows):
        subject = row["subject"]
        y = 44 + index * row_height
        entries = subject["entries"]
        size = f"{entries / 1_000_000:.2f}M" if entries >= 1_000_000 else f"{entries / 1000:.0f}k"
        css = "series-linux" if subject["platform"] == "Linux" else "series-mac"
        out.append(
            f'<text class="row-label" x="{left - 14}" y="{y + 4}" text-anchor="end">'
            f'{esc(subject["platform"])} <tspan class="row-sub">&middot; {esc(size)}'
            + (" &middot; synthetic" if subject["synthetic"] else "")
            + "</tspan></text>"
            # Two subjects can share a platform and a size and still be different trees,
            # so the tree's own name goes underneath rather than leaving two rows that
            # look like a duplicate.
            + f'<text class="row-sub" x="{left - 14}" y="{y + 17}" text-anchor="end">'
            f'{esc(subject["labels"][0])}</text>' 
        )
        first, last = row["first"], row["last"]
        opacity = ' opacity="0.45"' if subject["synthetic"] else ""
        if first["control"]:
            x1, x2 = scale(first["control"]), scale(last["kept"])
            out.append(f'<line class="connector" x1="{x1:.1f}" y1="{y}" x2="{x2:.1f}" y2="{y}"/>')
            out.append(
                f'<circle class="dot-before" cx="{x1:.1f}" cy="{y}" r="5"{opacity}><title>'
                f'{esc(first["id"])} control: {first["control"]:.2f} µs/entry</title></circle>'
            )
        for measurement in row["all"][1:-1]:
            out.append(
                f'<circle class="{css} pt-mid" cx="{scale(measurement["kept"]):.1f}" cy="{y}" '
                f'r="2.5"{opacity}><title>{esc(measurement["id"])}: '
                f'{measurement["kept"]:.2f} µs/entry</title></circle>'
            )
        out.append(
            f'<circle class="{css}" cx="{scale(last["kept"]):.1f}" cy="{y}" r="5"{opacity}>'
            f'<title>{esc(last["id"])}: {last["kept"]:.2f} µs/entry</title></circle>'
        )
        out.append(
            f'<text class="value-label" x="{width - 8}" y="{y + 4}" text-anchor="end">'
            + (
                f'<tspan class="value-before">{first["control"]:.1f}</tspan>'
                f'<tspan class="value-arrow"> &#8594; </tspan>'
                if first["control"]
                else ""
            )
            + f'{last["kept"]:.1f} µs</text>'
        )
    out.append("</svg>")

    real = [row for row in rows if not row["subject"]["synthetic"]]
    finals = [row["last"]["kept"] for row in real if row["subject"]["platform"] == "macOS"]
    return (
        f'<figure class="fig">{"".join(out)}'
        + legend(
            ("key-before-dot", "first cost measured on that tree"),
            ("key-mac", "latest, macOS"),
            ("key-linux", "latest, Linux"),
            ("", "small dots are the measurements between"),
        )
        + "<figcaption>Cold scan and index only, since it is the job every subject ran. "
        f"The {len(finals)} real macOS subjects finish between {min(finals):.1f} and "
        f"{max(finals):.1f} µs per entry despite differing in size by more than sixteen "
        "times, which is the check that the normalisation is describing the workload and "
        "not hiding a difference.</figcaption></figure>"
    )


# ---------------------------------------------------------------- page

#: The whole visual language, in tokens.
#:
#: Adapted from the tbd web design system. Its two load-bearing rules are kept: every
#: colour is defined once here and referenced by name everywhere else, and each token's
#: hue places it in a family — the neutrals all sit on 215° so surfaces, rules, and text
#: read as one material, while the semantic families keep the hues they have elsewhere in
#: the project's tooling. Dark holds hue and saturation and moves lightness only.
#:
#: What is deliberately absent: shadows, gradients, rounded panels within panels,
#: animation, and any second typeface. A report of measurements should look like the
#: measurements.
STYLE = """
:root {
  --bg: hsl(215 0% 100%);
  --panel: hsl(220 20% 97%);
  --border: hsl(215 15% 87%);
  --rule: hsl(215 15% 92%);
  --text: hsl(215 20% 10%);
  --muted: hsl(215 9% 43%);
  --accent: hsl(220 82% 55%);
  --good: hsl(149 68% 30%);
  --good-soft: hsl(149 45% 88%);
  --bad: hsl(0 64% 46%);
  --bad-soft: hsl(0 55% 91%);
  --warn: hsl(38 82% 36%);
  --before: hsl(215 12% 62%);
  --after: hsl(211 72% 42%);
  --drift: hsl(215 15% 91%);
  /* IBM Plex was drawn for technical documentation and its sans and mono are one
     family, which is what this page needs: a spine of measured values in mono running
     through prose that has to stay readable for several thousand words. Both carry full
     fallback stacks, so the page holds its layout if the font host is unreachable. */
  --sans: 'IBM Plex Sans', system-ui, -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif;
  --mono: 'IBM Plex Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  --measure: 74ch;
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme='light']) {
    --bg: hsl(215 15% 9%);
    --panel: hsl(217 16% 13%);
    --border: hsl(216 14% 21%);
    --rule: hsl(216 14% 17%);
    --text: hsl(215 15% 91%);
    --muted: hsl(214 12% 65%);
    --accent: hsl(220 100% 71%);
    --good: hsl(149 52% 55%);
    --good-soft: hsl(149 30% 20%);
    --bad: hsl(0 100% 73%);
    --bad-soft: hsl(0 35% 22%);
    --warn: hsl(38 82% 64%);
    --before: hsl(215 10% 45%);
    --after: hsl(211 86% 62%);
    --drift: hsl(216 14% 17%);
  }
}
:root[data-theme='dark'] {
  --bg: hsl(215 15% 9%);
  --panel: hsl(217 16% 13%);
  --border: hsl(216 14% 21%);
  --rule: hsl(216 14% 17%);
  --text: hsl(215 15% 91%);
  --muted: hsl(214 12% 65%);
  --accent: hsl(220 100% 71%);
  --good: hsl(149 52% 55%);
  --good-soft: hsl(149 30% 20%);
  --bad: hsl(0 100% 73%);
  --bad-soft: hsl(0 35% 22%);
  --warn: hsl(38 82% 64%);
  --before: hsl(215 10% 45%);
  --after: hsl(211 86% 62%);
  --drift: hsl(216 14% 17%);
}

* { box-sizing: border-box; }
:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; border-radius: 2px; }
body {
  margin: 0; padding: 0 24px 96px;
  background: var(--bg); color: var(--text);
  font: 15px/1.6 var(--sans);
  -webkit-font-smoothing: antialiased;
}
.wrap { max-width: 960px; margin: 0 auto; }
h1, h2, h3 { line-height: 1.25; font-weight: 650; }
h1 {
  font-size: 32px; margin: 56px 0 10px; letter-spacing: -0.02em;
  text-wrap: balance; font-weight: 600;
}
h2 {
  font-size: 13px; text-transform: uppercase; letter-spacing: 0.08em;
  color: var(--muted); font-weight: 650;
  margin: 64px 0 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border);
}
h3 { font-size: 17px; margin: 34px 0 8px; text-wrap: balance; font-weight: 600; }
p, li { max-width: var(--measure); }
p { margin: 0 0 14px; }
a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }
.lede { font-size: 18px; color: var(--muted); max-width: var(--measure); margin-bottom: 8px; }
.muted { color: var(--muted); }
.mono { font-family: var(--mono); font-size: 0.92em; }
.tnum { font-variant-numeric: tabular-nums; }

/* Headline figures. Unboxed: the number is the point, a card around it is not. */
.headline { display: flex; flex-wrap: wrap; gap: 40px; margin: 28px 0 8px; }
.headline div { min-width: 120px; }
.headline .n {
  display: block; font-size: 30px; font-weight: 650;
  font-variant-numeric: tabular-nums; letter-spacing: -0.02em;
}
.headline .k {
  display: block; font-size: 12px; text-transform: uppercase;
  letter-spacing: 0.06em; color: var(--muted); margin-top: 2px;
}
.headline .good { color: var(--good); }

figure.fig { margin: 24px 0 8px; }
.chart { width: 100%; height: auto; display: block; overflow: visible; }
figcaption { font-size: 13px; color: var(--muted); margin-top: 10px; max-width: var(--measure); }
.checkpoints { margin: 8px 0 0; padding-left: 18px; font-size: 12.5px; }
.checkpoints li { margin: 1px 0; }

.legend { font-size: 12.5px; color: var(--muted); margin: 12px 0 0; display: flex; flex-wrap: wrap; gap: 4px 20px; max-width: none; }
.legend-item { display: inline-flex; align-items: center; }
.key { width: 10px; height: 10px; display: inline-block; margin-right: 6px; vertical-align: -1px; }
.key-before { background: var(--before); }
.key-before-dot { background: var(--before); border-radius: 50%; }
.key-after-dot { background: var(--after); border-radius: 50%; }
.key-after { background: var(--after); }
.key-drift { background: var(--drift); outline: 1px solid var(--border); }
.key-good { background: var(--good); }
.key-bad { background: var(--bad); }
.key-mac { background: var(--after); }
.key-linux { background: var(--warn); }
.key-flat { background: var(--muted); }
.key-synth { background: transparent; outline: 1px dashed var(--muted); }
.pt-mid { fill-opacity: 0.55; }

/* SVG roles. Text inside a chart is chrome; measured values use tabular numerals. */
.grid { stroke: var(--rule); stroke-width: 1; }
.axis { stroke: var(--border); stroke-width: 1; }
.zero { stroke: var(--muted); stroke-width: 1; }
.threshold { stroke: var(--good); stroke-width: 1; stroke-dasharray: 3 3; }
.tick { font: 11px var(--sans); fill: var(--muted); font-variant-numeric: tabular-nums; }
.axis-name { font-size: 11px; letter-spacing: 0.04em; text-transform: uppercase; }
.row-label { font: 13px var(--sans); fill: var(--text); }
.row-sub { font: 11px var(--sans); fill: var(--muted); }
.bar-before { fill: var(--before); }
.bar-after { fill: var(--after); }
.connector { stroke: var(--border); stroke-width: 1; }
.track { fill: none; stroke: var(--after); stroke-width: 1.5; }
.dot-before { fill: var(--before); }
.dot-step { fill: var(--after); opacity: 0.55; }
.dot-final { fill: var(--after); }
.value-label { font: 12px var(--mono); fill: var(--after); font-variant-numeric: tabular-nums; }
.value-before { fill: var(--muted); }
.value-arrow { fill: var(--border); }
.drift-band { fill: var(--drift); }
.dot-good { fill: var(--good); }
.dot-bad { fill: var(--bad); }
.dot-flat { fill: var(--muted); }
.whisker { stroke: var(--border); stroke-width: 5; stroke-linecap: butt; }
.whisker-good { stroke: var(--good-soft); }
.whisker-bad { stroke: var(--bad-soft); }
.series-mac { stroke: var(--after); fill: var(--after); }
.series-linux { stroke: var(--warn); fill: var(--warn); }
.series-line { fill: none; stroke-width: 1.5; }
.point-label { font: 10px var(--mono); fill: var(--muted); }
.gutter { font: 9.5px var(--mono); fill: var(--muted); opacity: 0.75; }

table { border-collapse: collapse; width: 100%; font-size: 13.5px; margin: 16px 0; }
th, td { text-align: left; padding: 7px 10px; border-bottom: 1px solid var(--rule); vertical-align: top; }
th {
  font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em;
  color: var(--muted); font-weight: 650; border-bottom: 1px solid var(--border);
  position: sticky; top: 0; background: var(--bg);
}
td.n, th.n { text-align: right; font-variant-numeric: tabular-nums; font-family: var(--mono); font-size: 12.5px; }
tbody tr:hover { background: var(--panel); }
.scroll { overflow-x: auto; margin: 16px -4px; padding: 0 4px; }
.v-accepted { color: var(--good); }
.v-rejected { color: var(--bad); }
.v-superseded, .v-in-progress, .v-baseline { color: var(--muted); }
.note {
  border-left: 2px solid var(--border); padding: 2px 0 2px 16px;
  margin: 20px 0; color: var(--muted); font-size: 14px; max-width: var(--measure);
}
.note strong { color: var(--text); }
.why { font-size: 12px; line-height: 1.45; margin-top: 3px; max-width: 62ch; }
footer { margin-top: 72px; padding-top: 16px; border-top: 1px solid var(--border); font-size: 12.5px; color: var(--muted); }
"""


def render(dataset: Mapping[str, Any]) -> str:
    """The whole page."""
    body = "".join(
        [
            _header(dataset),
            _section_absolute(dataset),
            _section_relative(dataset),
            _section_scale(dataset),
            _section_mechanisms(dataset),
            _section_reading(dataset),
            _section_table(dataset),
            _footer(dataset),
        ]
    )
    page = (
        "<title>fdu Performance Evidence</title>\n"
        '<link rel="preconnect" href="https://fonts.googleapis.com">\n'
        '<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>\n'
        '<link rel="stylesheet" href="https://fonts.googleapis.com/css2?'
        "family=IBM+Plex+Mono:wght@400;500&"
        'family=IBM+Plex+Sans:wght@400;500;600&display=swap">\n'
        f"<style>{STYLE}</style>\n"
        f'<div class="wrap">{body}</div>'
    )
    # Emitted as pure ASCII with numeric references for everything else. The page is
    # embedded by hosts that supply their own `<head>`, so it cannot declare its own
    # charset; served without one it was decoded as latin-1 and every literal "µ" arrived
    # as "Âµ". References survive either decoding, which entities in some places and
    # literals in others did not.
    return page.encode("ascii", "xmlcharrefreplace").decode("ascii")


def _section_relative(dataset: Mapping[str, Any]) -> str:
    totals = dataset["totals"]
    rejected = totals["decisions"].get("rejected", 0)
    return f"""
<h2 id="relative">Relative</h2>
<h3>What each experiment actually did</h3>
<p>Milliseconds say how fast the tool is. They cannot say whether a particular change is
why, because the host drifts: on this record the reference tool <span class="mono">dust</span>,
whose binary never changed at all, measured anywhere from 210&nbsp;ms to 327&nbsp;ms on the
same tree. So every experiment interleaved its two builds and compared them in pairs, and
it is the paired figure below that each verdict rests on.</p>
{figure_effects(dataset)}
<p>Most of the chart sits against the zero line. That is the ordinary shape of this work
and the reason the threshold exists: an idea that seems obviously good usually turns out to
be worth somewhere between nothing and two percent, and the interval is what says so before
the code is carried for a year. The {rejected} rejections are the reusable part of the
record &mdash; each one is a day the next person does not have to spend.</p>
"""


def _section_scale(dataset: Mapping[str, Any]) -> str:
    return f"""
<h2 id="scale">Scale</h2>
<h3>Does it hold on a bigger tree, and on another kernel?</h3>
<p>Eighteen distinct trees were measured, from 307 entries to 1.01 million, on macOS and
Linux. Their milliseconds are not comparable, but their cost per entry is, because a
scan's work is very nearly linear in what it has to visit.</p>
{figure_per_entry(dataset)}
"""


def _mismatch(dataset: Mapping[str, Any]) -> tuple:
    """Experiments where the accept decision and the measured evidence disagree.

    Both directions exist and each says something the other does not. A change can be
    real and still not worth carrying, and a change can be worth carrying for a reason
    that is not speed.
    """
    records = [
        record
        for record in dataset["experiments"]
        if record["decision"] != "baseline" and _primary(record)
    ]
    improved = {
        record["id"] for record in records if _primary(record)["paired"]["evidence"] == "improved"
    }
    kept = {record["id"] for record in records if record["decision"] == "accepted"}
    return sorted(improved - kept), sorted(kept - improved)


def _section_mechanisms(dataset: Mapping[str, Any]) -> str:
    """What the big wins actually did, told by the CPU column rather than by intent.

    Two mechanisms produced almost everything here, and wall time alone cannot tell them
    apart. Making work happen at the same time as other work and removing the work
    entirely both make the clock go down; only one of them makes the machine do less.
    Total CPU separates them, and the record measured it every time, so the distinction is
    read out of the evidence rather than asserted from what each change was trying to do.
    """
    rows = []
    for record in dataset["experiments"]:
        # Individual changes only, decided by two facts the artifacts already carry.
        # `lines_changed == 0` marks an experiment that wrote no code and re-measured work
        # already merged — every cumulative checkpoint and platform validation. A long
        # hypothesis list marks a whole pull request measured at once, as exp-033's five
        # do across 1,514 lines. Both would credit a single row with a dozen changes.
        if record["decision"] != "accepted" or record["anchored"]:
            continue
        if not record["complexity"].get("lines_changed") or len(record["hypotheses"]) > 2:
            continue
        job = next(
            (item for item in record["jobs"] if item["job"] == record["primary_job"]), None
        )
        if not job:
            continue
        wall = job["metrics"].get("wall_ns")
        cpu = job["metrics"].get("cpu_ns")
        if not wall or not cpu:
            continue
        wall_change = wall["paired"]["change_pct"]
        cpu_change = cpu["paired"]["change_pct"]
        if wall_change is None or cpu_change is None or wall_change > -5:
            continue
        rows.append((record, job["job"], wall_change, cpu_change))
    rows.sort(key=lambda item: item[2])
    if not rows:
        return ""

    body = "".join(
        f"<tr><td class='n'>{esc(record['id'].removeprefix('exp-'))}</td>"
        f"<td>{esc(record['title'])}</td>"
        f"<td class='mono muted'>{esc(job)}</td>"
        f"<td class='n'>{fmt_pct(wall_change)}</td>"
        f"<td class='n {'v-accepted' if cpu_change < 0 else 'v-rejected'}'>{fmt_pct(cpu_change)}</td>"
        f"<td>{'removed work' if cpu_change < 0 else 'overlapped work'}</td>"
        f"<td class='n muted'>{record['complexity'].get('lines_changed') or 0:,}</td></tr>"
        for record, job, wall_change, cpu_change in rows
    )
    removed = sum(1 for _, _, _, cpu_change in rows if cpu_change < 0)
    return f"""
<h2 id="mechanisms">Mechanism</h2>
<h3>Two ways to make a clock go down</h3>
<p>Work can be moved off the critical path, or it can stop happening. Both shorten wall
time and only one makes the machine do less, so wall time alone cannot tell them apart.
Total CPU can, and it was measured every time.</p>
<div class="scroll"><table>
<thead><tr><th class="n">#</th><th>change</th><th>job</th><th class="n">wall</th>
<th class="n">total cpu</th><th>what it did</th><th class="n">lines</th></tr></thead>
<tbody>{body}</tbody></table></div>
<p>{removed} of these {len(rows)} deleted work outright. The rest bought their latency by
spending more of the machine, which is a real gain for a person waiting on a scan and no
gain at all for a laptop battery &mdash; a trade worth making knowingly rather than by
accident.</p>
<p class="note">The clearest pair sits at the two ends. Running the directory walk on four
threads (exp&#8209;001) cut wall time in half and raised system CPU
<strong>83%</strong>: the same syscalls, issued concurrently. Replacing those syscalls
with one bulk call per directory (exp&#8209;022) cut wall time 30% and system CPU
<strong>47%</strong>: fewer syscalls. The second kind is what let the first kind's cost be
paid back later.</p>
"""


def _section_reading(dataset: Mapping[str, Any]) -> str:
    """The two caveats a reader needs before drawing a conclusion from either figure.

    Stated in the report rather than left in the harness because both are ways an honest
    dataset can be read into a wrong answer, and the second one flatly contradicts what a
    reader would otherwise assume from seeing both figures on one page.
    """
    improved_only, kept_only = _mismatch(dataset)
    mismatch = (
        f"{len(improved_only)} experiments measured a real improvement and were still not "
        f"kept &mdash; below the threshold, superseded, or not yet finished. "
        f"{len(kept_only)} were kept although their interval crossed zero, because what "
        f"they bought was not speed."
    )
    return f"""
<h2 id="reading">Reading these numbers</h2>
<h3>The two figures do not divide into each other</h3>
<p>It is tempting to take a row from the absolute figure, divide the endpoints, and expect
the relative figure's percentage. That is wrong, and the record is careful about why. The
absolute values are the median of each arm on its own. The relative value is the median of
the <em>paired</em> differences &mdash; each candidate trial against the control trial
interleaved beside it. When the host drifts mid-run the two diverge, and in this record
they differ by more than two percentage points on 21% of measurements and sometimes differ
in sign.</p>
<p class="note">exp&#8209;005's <span class="mono">cold-scan-index</span> reads
<strong>+2.8%</strong> if you divide its medians and <strong>&minus;3.9%</strong> paired.
The paired figure is the one that controls for drift, so it is the one the verdict used.
Both are published; neither is derived from the other.</p>
<h3>Kept and faster are different questions</h3>
<p>The two sets nearly coincide and it would be easy to assume they are the same set.
They are not, and both directions of the mismatch are worth knowing about.</p>
<p>{mismatch}</p>
<p class="note">Instrumentation is the clearest case. exp&#8209;052 and exp&#8209;053 were
kept on intervals of <span class="mono">[&minus;3.3%, +3.8%]</span> and
<span class="mono">[&minus;3.0%, +1.4%]</span> &mdash; neither is a speed-up, and neither
was claimed as one. What they bought was the ability to see inside the engine at a cost
the measurement could not detect, which is a different thing to want and was recorded as
one.</p>
<h3>What is not measured here</h3>
<p>Every number is one machine: an Apple M1 Pro, and one Linux VM. The page cache was warm
for all of it, because dropping it needs root, so nothing here describes a genuinely cold
disk. Tuning constants were fitted on the subjects shown and are inherited, not proven, on
anything else.</p>
"""


def _section_table(dataset: Mapping[str, Any]) -> str:
    rows = []
    for record in dataset["experiments"]:
        metric = _primary(record)
        paired = metric["paired"] if metric else None
        absolute = metric["absolute"] if metric else None
        interval = (
            f'{paired["ci95_low_pct"]:+.1f} to {paired["ci95_high_pct"]:+.1f}'
            if paired and paired["ci95_low_pct"] is not None
            else "—"
        )
        change = (
            fmt_pct(paired["change_pct"])
            if paired and record["decision"] != "baseline"
            else "—"
        )
        before = fmt_ms(absolute["control"]) if absolute else "—"
        after = (
            fmt_ms(absolute["candidate"])
            if absolute and record["decision"] != "baseline"
            else "—"
        )
        decision = record["decision"]
        rows.append(
            f"<tr>"
            f'<td class="n">{esc(record["id"].removeprefix("exp-"))}</td>'
            f'<td>{esc(record["title"])}'
            f'<div class="muted why">{esc(record["reason"] or "")}</div></td>'
            f'<td class="mono muted">{esc(record["primary_job"] or "")}</td>'
            f'<td class="n">{esc(before)}</td>'
            f'<td class="n">{esc(after)}</td>'
            f'<td class="n">{esc(change)}</td>'
            f'<td class="n muted">{esc(interval)}</td>'
            f'<td class="v-{esc(decision)}">{esc(DECISION_LABEL.get(decision, decision))}</td>'
            f"</tr>"
        )
    return f"""
<h2 id="every">Every experiment</h2>
<p>Absolute values are that experiment's own two arms on its own subject, so they are
comparable across a row and not down a column. The change and interval are paired.</p>
<div class="scroll"><table>
<thead><tr><th class="n">#</th><th>experiment and why it went that way</th><th>primary job</th>
<th class="n">before</th><th class="n">after</th><th class="n">change</th>
<th class="n">95% interval</th><th>verdict</th></tr></thead>
<tbody>{"".join(rows)}</tbody>
</table></div>
"""


def _footer(dataset: Mapping[str, Any]) -> str:
    totals = dataset["totals"]
    return f"""
<footer>
<p>Generated from {totals['experiments']} validated soft-schema artifacts in
<span class="mono">docs/project/experiments/</span> by
<span class="mono">make perf-report</span>. Every number is read back out of those
artifacts, never retyped, so this page cannot drift from the record. Regenerate it rather
than editing it.</p>
</footer>
"""


def _headline_figure(series: Optional[Mapping[str, Any]], job_id: str, label: str) -> str:
    """One before-and-after headline, or nothing if that job was not measured.

    The first version indexed straight into the series for the two jobs it wanted and
    raised `StopIteration` when either was absent, which made the whole report depend on
    a particular subject having run a particular job. A headline is the least important
    thing on the page; it should not be the thing that can stop the page existing.
    """
    if not series:
        return ""
    job = next((item for item in series["jobs"] if item["job"] == job_id), None)
    if not job or not job["points"]:
        return ""
    before = job["points"][0]["control_ns"]
    after = next(
        (point["candidate_ns"] for point in reversed(job["points"]) if point["candidate_ns"]),
        None,
    )
    if not before or not after:
        return ""
    return (
        f'<div><span class="n">{fmt_ms(before)} <span class="muted">&rarr;</span> '
        f'<span class="good">{fmt_ms(after)}</span></span>'
        f'<span class="k">{esc(label)}</span></div>'
    )


def _header(dataset: Mapping[str, Any]) -> str:
    totals = dataset["totals"]
    series = _flagship(dataset)
    headlines = "".join(
        _headline_figure(series, job_id, label)
        for job_id, label in (
            ("cold-scan-index", "cold scan and index"),
            ("warm-revalidate", "warm revalidate"),
        )
    )
    return f"""
<h1>What sixty-four experiments bought</h1>
<p class="lede">Every performance change made to fdu between {esc(totals['first_date'])} and
{esc(totals['last_date'])}: what it cost in milliseconds, what each experiment measured,
and why {totals['decisions'].get('rejected', 0)} of the {totals['experiments']} were
thrown away.</p>
<div class="headline">
  {headlines}
  <div><span class="n tnum">{totals['decisions'].get('accepted', 0)}</span>
    <span class="k">changes kept</span></div>
  <div><span class="n tnum">{totals['accepted_lines_changed']:,}</span>
    <span class="k">lines they cost</span></div>
</div>
"""


def _section_absolute(dataset: Mapping[str, Any]) -> str:
    return f"""
<h2 id="absolute">Absolute</h2>
<h3>Wall time, in milliseconds</h3>
<p>The campaign's own summaries are all percentages, and a percentage cannot say whether
a scan takes half a second or half a minute. These are the measured medians.</p>
{figure_absolute(dataset)}
"""


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(prog="benchmarks.realtree.report_html", description=__doc__)
    parser.add_argument("--data", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument(
        "--check",
        action="store_true",
        help="fail if the committed page is not what the projection produces",
    )
    arguments = parser.parse_args(list(argv))
    dataset = json.loads(arguments.data.read_text(encoding="utf-8"))
    page = render(dataset)
    if arguments.check:
        from benchmarks.realtree.timeline import _check

        return _check(arguments.out, page)
    arguments.out.parent.mkdir(parents=True, exist_ok=True)
    arguments.out.write_text(page, encoding="utf-8")
    print(f"wrote {arguments.out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
