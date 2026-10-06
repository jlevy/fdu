"""Render the projected experiment dataset into one self-contained HTML report.

    python -m benchmarks.realtree.report_html --data <timeline.json> --out <index.html>

Charts are hand-written inline SVG. No chart library, for the same reason the report
serializers in the Rust crate are hand-written: the shapes here are few and fully known,
and a library would have to be pinned, audited, and carried for the rest of the project's
life to draw a handful of figures. Inline SVG also keeps the page self-contained, so it
can be opened from a file, committed, and published without an asset pipeline.

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

from benchmarks.atomic_write import write_text_atomic
from benchmarks.realtree.perf_index import CONTROLLED_REGIMES, QUOTABLE_ROUNDS, score_ratio
from benchmarks.realtree.timeline import METRICS

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
    "accepted": "accepted",
    "rejected": "rejected",
    "superseded": "superseded",
    "in-progress": "in progress",
    "baseline": "baseline",
}


def decision_label(record: Mapping[str, Any]) -> str:
    """Describe the verdict without conflating acceptance with a retained arm."""
    decision = record["decision"]
    if decision != "accepted":
        return DECISION_LABEL.get(decision, decision)
    kept = record.get("kept")
    if kept == "candidate":
        return "candidate kept"
    if kept == "control":
        return "control kept"
    return "accepted evidence"


def compares(record: Mapping[str, Any]) -> bool:
    """Whether a record's two arms are different builds, so it has a change to show.

    The projection records it as `compares`. A projection without that field falls back
    to the decision, which treats every baseline as one build measured against itself.
    """
    return bool(record.get("compares", record["decision"] != "baseline"))


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


def fmt_bytes(value: Optional[float]) -> str:
    if value is None:
        return "—"
    mib = value / (1024 * 1024)
    if mib >= 1024:
        return f"{mib / 1024:,.2f} GiB"
    return f"{mib:,.1f} MiB"


def fmt_primary(value: Optional[float], metric: Optional[str]) -> str:
    """Format the verdict's primary metric in that metric's unit.

    A peak-RSS primary prints in MiB; formatting it with [`fmt_ms`] would print a byte
    count as milliseconds.
    """
    unit = METRICS.get(metric or "wall_ns", "ns")
    if unit == "bytes":
        return fmt_bytes(value)
    return fmt_ms(value)


def fmt_pct(value: Optional[float]) -> str:
    return "—" if value is None else f"{value:+.1f}%"


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
    # The ladder covers the maximum rather than stopping below it: the last tick is the
    # scale's top, so a bar longer than it would run past the plot into the value column.
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

    Each entry is one flex item. Loose text between two keys would become an anonymous
    flex item of its own and drift away from the swatch it belongs to.
    """
    cells = "".join(
        f'<span class="legend-item"><span class="key {css}"></span>{esc(label)}</span>'
        if css
        else f'<span class="legend-item">{esc(label)}</span>'
        for css, label in items
    )
    return f'<p class="legend">{cells}</p>'


def tip(text: str) -> str:
    """A `data-tip` attribute for the delegated tooltip.

    The text keeps real newlines rather than an escape sequence. An attribute value may
    contain them and the parser preserves them, so the script splits on `\n` directly and
    no second encoding has to agree with it.
    """
    return 'data-tip="' + esc(text) + '"'


def hover_row(x: float, y: float, width: float, height: float, text: str) -> str:
    """An invisible full-width band that makes a whole chart row hoverable.

    Without it the only hover target is the marker itself, which on the effects figure is
    a dot a few pixels wide in a row the width of the chart.
    """
    return (
        f'<rect class="hit" x="{x:.1f}" y="{y - height / 2:.1f}" width="{width:.1f}" '
        f'height="{height:.1f}" {tip(text)}/>'
    )


def figure_absolute(dataset: Mapping[str, Any]) -> str:
    """Before and after in real milliseconds, with the route between them.

    Each row is one job. The grey marker is the pre-work binary and the blue track is the
    build at each checkpoint, ending at the last checkpoint.

    Every checkpoint remeasured the *original* binary against the current one,
    interleaved, on the same tree within minutes, so each checkpoint's pair comes from one
    run rather than from two numbers taken days apart. The pale band behind each row is the
    full range those remeasurements of the *unchanged* binary covered. It is drawn rather
    than averaged away because it is the scale any movement along the blue track has to be
    read against: on the producer job it is 35% wide, wider than several of the steps.
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
    # One interval more than the default: the producer job's noisiest control run reaches
    # 1,219 ms, which the default ladder rounds up to a 1,500 ms axis, leaving the right
    # fifth of the plot empty.
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
        "One row per job. Grey marks the pre-work binary; the blue track is the build at "
        "each checkpoint.",
    )
    for tick in ticks:
        x = left + (tick / top) * plot
        out.append(f'<line class="grid" x1="{x:.1f}" y1="30" x2="{x:.1f}" y2="{height - 30}"/>')
        out.append(
            f'<text class="tick" x="{x:.1f}" y="{height - 14}" text-anchor="middle">{tick:,.0f}</text>'
        )
    out.append(
        f'<text class="tick axis-name" x="{left}" y="18">milliseconds, lower is faster</text>'
    )

    for index, (label, job) in enumerate(rows):
        y = 44 + index * row_height
        out.append(
            f'<text class="row-label" x="{left - 14}" y="{y + 4}" text-anchor="end">{esc(label)}</text>'
        )

        controls_all = [point["control_ns"] for point in job["points"] if point["control_ns"]]
        finals = [point["candidate_ns"] for point in job["points"] if point["candidate_ns"]]
        if controls_all and finals:
            out.append(
                hover_row(
                    left,
                    y,
                    plot,
                    row_height - 12,
                    f"{label}\n"
                    f"Grey: the pre-work binary, measured {len(controls_all)} times at "
                    f"{fmt_ms(min(controls_all))} to {fmt_ms(max(controls_all))}. Its code "
                    f"never changed, so that range is the host's noise.\n"
                    f"Band: that range, to scale.\n"
                    f"Blue: the build at each checkpoint, ending at "
                    f"{fmt_ms(finals[-1])}.\n"
                    f"Each checkpoint timed both binaries in one interleaved run.",
                )
            )

        low, high = job.get("baseline_low_ns"), job.get("baseline_high_ns")
        if low and high and high > low:
            x1, x2 = scale(low), scale(high)
            out.append(
                f'<rect class="drift-band" x="{x1:.1f}" y="{y - 13}" '
                f'width="{max(x2 - x1, 1.5):.1f}" height="26"><title>'
                f"the unchanged pre-work binary, {fmt_ms(low)} to {fmt_ms(high)} "
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
                f"before: {fmt_ms(median_before)} (median of {len(controls)} remeasurements)"
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
                    f'{esc(point["id"])}: {fmt_ms(point["candidate_ns"])}'
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
        ("key-before-dot", "the pre-work binary, median of five remeasurements"),
        ("key-after-dot", "the build at each checkpoint"),
        ("key-drift", "range of those five remeasurements"),
    )
    checkpoints = "".join(
        f'<li><span class="mono">{esc(point["id"])}</span> {esc(point["title"])}</li>'
        for point in points
    )
    return (
        f'<figure class="fig">{"".join(out)}{keys}'
        f"<figcaption>Campaign 1, the first optimization campaign, on one macOS tree of "
        f"{series['entries_first']:,}–{series['entries_last']:,} entries, measured five "
        f"times over three days. The pre-work binary is the code before any experiment. "
        f'The checkpoints, in order:<ol class="checkpoints">{checkpoints}</ol>'
        "</figcaption></figure>"
    )


def _flagship(dataset: Mapping[str, Any]) -> Optional[Dict[str, Any]]:
    series = dataset.get("anchors", {}).get("series") or []
    return series[0] if series else None


#: The record the header states as the current standing: the latest release measured
#: end to end against the release before it, both builds in one interleaved cell. Move
#: it when a later release cell is recorded; the header leaves the figure out while the
#: record is absent, so a projection without it still renders.
STANDING_EXPERIMENT = "exp-202"
STANDING_LABEL = "Linux default tree, 0.2.1 to 0.3.0"


def _record(dataset: Mapping[str, Any], experiment_id: str) -> Optional[Mapping[str, Any]]:
    return next(
        (record for record in dataset["experiments"] if record["id"] == experiment_id), None
    )


def _wall_arms(record: Mapping[str, Any]) -> Optional[tuple]:
    """The control and candidate wall medians of a record's primary job, if measured."""
    job = next((item for item in record["jobs"] if item["job"] == record.get("primary_job")), None)
    wall = (job or {}).get("metrics", {}).get("wall_ns", {}).get("absolute") or {}
    control, candidate = wall.get("control"), wall.get("candidate")
    return (control, candidate) if control and candidate else None


def _subject_label(dataset: Mapping[str, Any], key: Optional[str]) -> str:
    subject = next((item for item in dataset["subjects"] if item["key"] == key), None)
    labels = (subject or {}).get("labels") or []
    return labels[0] if labels else "an unlabelled subject"


def end_to_end_cells(dataset: Mapping[str, Any], platform: str) -> List[Mapping[str, Any]]:
    """Baselines that compare two builds on the default tree: the end-to-end cells.

    Each one measured an older engine against a newer one in a single interleaved cell,
    which is what lets its two arms be drawn as absolute milliseconds. Cells from
    different sessions are not drawn as one track: on one virtualized host an unchanged
    binary drifted by up to 70% between cells over a night.
    """
    return [
        record
        for record in dataset["experiments"]
        if record["platform"] == platform
        and record["decision"] == "baseline"
        and compares(record)
        and record.get("primary_job") == "default-tree"
        and _wall_arms(record)
    ]


def figure_end_to_end(dataset: Mapping[str, Any], platform: str = "Linux") -> str:
    """One row per end-to-end cell: the older engine's bar above the newer one's."""
    cells = end_to_end_cells(dataset, platform)
    if not cells:
        return ""
    maximum = max(value for record in cells for value in _wall_arms(record))
    ticks = axis_ticks(ms(maximum) or 0, count=4)
    top = ticks[-1]
    left, right = 200, 170
    row_height = 58
    width = 900
    plot = width - left - right
    height = len(cells) * row_height + 50
    scale = lambda value: (ms(value) / top) * plot

    out = svg_open(
        width,
        height,
        f"{platform} default tree, older engine against newer, one cell per row",
        "Each row is one interleaved cell. Grey is the older engine and blue the newer one.",
    )
    for tick in ticks:
        x = left + (tick / top) * plot
        out.append(f'<line class="grid" x1="{x:.1f}" y1="24" x2="{x:.1f}" y2="{height - 30}"/>')
        out.append(
            f'<text class="tick" x="{x:.1f}" y="{height - 14}" text-anchor="middle">{tick:,.0f}</text>'
        )
    out.append(
        f'<text class="tick axis-name" x="{left}" y="14">milliseconds, lower is faster</text>'
    )
    for index, record in enumerate(cells):
        control, candidate = _wall_arms(record)
        y = 30 + index * row_height
        subject = _subject_label(dataset, record.get("subject"))
        out.append(
            hover_row(
                left,
                y + (row_height - 10) / 2,
                plot,
                row_height - 10,
                f"{record['id']}: {record['title']}\n"
                f"Older: {record.get('control') or 'control'}, {fmt_ms(control)}\n"
                f"Newer: {record.get('candidate') or 'candidate'}, {fmt_ms(candidate)}\n"
                f"Paired change {fmt_pct(record.get('change_pct'))}, on {subject}.",
            )
        )
        out.append(
            f'<text class="row-label" x="{left - 14}" y="{y + 16}" text-anchor="end">'
            f"{esc(record['id'])}</text>"
        )
        out.append(
            f'<text class="row-sub" x="{left - 14}" y="{y + 31}" text-anchor="end">'
            f"{esc(subject)}</text>"
        )
        out.append(
            f'<rect class="bar-before" x="{left}" y="{y + 6}" '
            f'width="{max(scale(control), 1.5):.1f}" height="14"/>'
        )
        out.append(
            f'<rect class="bar-after" x="{left}" y="{y + 24}" '
            f'width="{max(scale(candidate), 1.5):.1f}" height="14"/>'
        )
        out.append(
            f'<text class="value-label" x="{width - 8}" y="{y + 26}" text-anchor="end">'
            f'<tspan class="value-before">{fmt_ms(control)}</tspan>'
            f'<tspan class="value-arrow"> &#8594; </tspan>{fmt_ms(candidate)}'
            f'<tspan class="value-before"> {esc(fmt_pct(record.get("change_pct")))}</tspan>'
            f"</text>"
        )
    out.append("</svg>")
    keys = legend(
        ("key-before", "the older engine, in the same cell"),
        ("key-after", "the newer engine"),
    )
    listed = "".join(
        f'<li><span class="mono">{esc(record["id"])}</span> {esc(record["title"])}</li>'
        for record in cells
    )
    return (
        f'<figure class="fig">{"".join(out)}{keys}'
        f"<figcaption>Every {esc(platform)} cell that measured one engine against a later "
        f"one end to end on the default tree. The percentage is the paired change. Rows "
        f"are separate sessions on a virtualized host whose absolute speed drifted between "
        f"them, so each row compares only its own two bars."
        f'<ol class="checkpoints">{listed}</ol></figcaption></figure>'
    )


#: Accepted records that measure a change already counted, with the reason, so each kept
#: change counts once: cumulative checkpoints, validations after a merge or on another
#: platform, the same candidate measured on a second tree, and determinations. The first
#: record of a change keeps it. A record's fields cannot tell these apart (checkpoints
#: record changed lines, and real changes sometimes record none, exp-015 and exp-190), so
#: the list is explicit; a test ties every entry to a committed record.
REMEASUREMENTS = {
    "exp-006": "a cumulative checkpoint against the pre-work binary",
    "exp-023": "a cumulative checkpoint against the pre-work binary",
    "exp-027": "a cumulative checkpoint against the pre-work binary",
    "exp-032": "a cumulative checkpoint against the pre-work binary",
    "exp-033": "a validation after the composable command line merged",
    "exp-034": "a validation after the composable command line merged",
    "exp-035": "a validation after the composable command line merged",
    "exp-054": "the Linux campaign's changes validated on macOS",
    "exp-065": "exp-064's change validated on a second tree",
    "exp-071": "a rewrite measured against its own regression",
    "exp-126": "a leftover determination, no code change",
    "exp-134": "a leftover determination, no code change",
    "exp-136": "a leftover determination, no code change",
    "exp-138": "macOS changes validated on Linux",
    "exp-140": "macOS changes validated on Linux",
    "exp-148": "a screen that kept neither arm",
    "exp-154": "a PGO screen whose kept arm is the control",
    "exp-171": "exp-170's change measured on a second tree",
    "exp-181": "exp-180's change measured on a second tree",
    "exp-184": "exp-183's change measured on a second tree",
    "exp-186": "exp-185's change measured on a second tree",
    "exp-187": "exp-170's change validated on Linux",
}


def iteration_kind(record: Mapping[str, Any]) -> str:
    """How the iterations figure colours one experiment.

    `kept` is an accepted change, not a remeasurement, whose primary metric improved by
    at least the accept threshold and whose candidate stayed; `rejected` is a change
    tried and not kept; everything else (baselines, checkpoints, validations,
    determinations, noninferiority steps, unfinished work) is `measured`.
    """
    change = record.get("change_pct")
    if (
        record["decision"] == "accepted"
        and change is not None
        and change <= -3
        and record["id"] not in REMEASUREMENTS
        and record.get("kept") != "control"
    ):
        return "kept"
    if record["decision"] == "rejected":
        return "rejected"
    return "measured"


#: Builds that changed what a job does, not only how fast it does it, keyed by build
#: label, each note in the lines it is drawn on. The top panel annotates the build on
#: every metric that draws it, so a step there reads as more work rather than as a
#: slowdown. A test ties every entry to a projected build.
BUILD_NOTES = {
    "v0.1.0": (".gitignore read by default:", "more work per entry on trees that have them"),
}

#: The iterations figure's vertical range, as percent faster. An effect beyond it is
#: drawn at the edge, and its tooltip carries the real figure.
ITERATION_CLAMP = (-30.0, 60.0)


def _chronological(dataset: Mapping[str, Any]) -> List[Mapping[str, Any]]:
    return sorted(dataset["experiments"], key=lambda record: (record.get("date") or "", record["number"]))




#: The platform the top panel draws, chosen by name: the projection sorts platforms by
#: name, so taking the first would let Linux's cells silently replace macOS's chart.
#: The headline states every platform's score.
CHART_PLATFORM = "macOS"


def projected_platform(
    dataset: Mapping[str, Any], platform: Optional[str] = None
) -> Optional[Mapping[str, Any]]:
    """One platform's projected index: the named one, else the chart's, else the only one."""
    platforms = dataset.get("index") or []
    wanted = platform or CHART_PLATFORM
    for projected in platforms:
        if projected["platform"] == wanted:
            return projected
    if platform is None and len(platforms) == 1:
        return platforms[0]
    return None


def score_label(projected: Mapping[str, Any], components: int) -> str:
    """A platform's score with its coverage: "macOS score, 12 of 12 components".

    A platform's score is never the full index, which weights every platform; the
    coverage counts the platform's own components.
    """
    total = len(projected["measured"]) + len(projected["missing"])
    return f"{projected['platform']} score, {components} of {total} components"


def regime_note(projected: Mapping[str, Any], link: bool = False) -> str:
    """"exploratory, uncontrolled host, 12 rounds" when a score cannot be quoted.

    The loop's regime table limits an uncontrolled host to exploration and discovery, and
    a quoted score needs `QUOTABLE_ROUNDS` rounds, so a score short of either is labelled
    with each reason wherever it is stated: the host when any cell's was not controlled,
    and the fewest rounds when any cell ran short. An exploratory stage on a controlled
    host with enough rounds is just "exploratory".

    Plain text by default, for a tooltip or the chooser. With `link` it is markup whose
    "rounds" links to the loop section that defines a round, since the headline that
    states it comes before that definition, and a reader could otherwise take twelve
    rounds for twelve optimization campaigns.
    """
    if not projected.get("exploratory"):
        return ""
    regimes = projected.get("host_regimes") or []
    loose = [regime for regime in regimes if regime not in CONTROLLED_REGIMES]
    reasons = ["exploratory"]
    if loose:
        reasons.append("uncontrolled host" if "uncontrolled" in loose else f"{'/'.join(loose)} host")
    if link:
        reasons = [esc(reason) for reason in reasons]
    fewest = projected.get("fewest_rounds")
    if fewest is not None and fewest < QUOTABLE_ROUNDS:
        reasons.append(f'{fewest} <a href="#loop">rounds</a>' if link else f"{fewest} rounds")
    return ", ".join(reasons)


def gate_note(projected: Mapping[str, Any]) -> str:
    """The range of the score's cells' shares of sample boundaries above the quiet gate.

    Markup, not plain text: the term links to the loop section that defines it, since the
    headline that states it comes before that definition.
    """
    shares = projected.get("above_quiet_gate_range")
    if not shares:
        return ""
    low, high = (round(value * 100) for value in shares)
    span = f"{low}%" if low == high else f"{low}% to {high}%"
    return f'{span} of each cell\'s sample boundaries above the <a href="#loop">quiet gate</a>'


def metric_series(
    dataset: Mapping[str, Any], position: Mapping[str, int], platform: Optional[str] = None
) -> List[Dict[str, Any]]:
    """The chooser's metrics: the platform's score first, then every measured component.

    Every value is relative to the reference build, so every line ends at 1.0 and lines
    that start at different builds still compare: the score, a millisecond benchmark,
    and peak memory read on one axis. The score has two lines: solid, from the first
    build that has every measured component, and a partial score, dashed, over the
    components every build has, each labelled with its coverage.
    """
    projected = projected_platform(dataset, platform)
    if projected is None:
        return []
    builds = [build for build in projected["builds"] if build.get("after_experiment") in position]
    reference = projected["reference_build"]

    def points(values: Sequence[tuple]) -> List[Dict[str, Any]]:
        present = [(build, value, detail) for build, value, detail in values if value]
        if len(present) < 2:
            return []
        return [
            {
                "after_experiment": build["after_experiment"],
                "label": build.get("label") or "",
                "short": build.get("short") or build["label"],
                "commit": build.get("commit") or "",
                "date": build.get("date") or "",
                "includes": build.get("includes") or "",
                "share": value,
                "detail": detail,
            }
            for build, value, detail in present
        ]

    measured = len(projected["measured"])
    total = measured + len(projected["missing"])
    measured_weight = sum(
        build["coverage"] for build in builds[-1:]
    ) if builds else 0.0
    full_line = points(
        [
            (
                build,
                (build.get("measured_full") or {}).get("index"),
                f"index {build['measured_full']['index']:.3f} of {reference}, all {measured} "
                f"measured components" if build.get("measured_full") else "",
            )
            for build in builds
        ]
    )
    partial_line = points(
        [
            (
                build,
                (build.get("common") or {}).get("index"),
                f"index {build['common']['index']:.3f} of {reference}, "
                f"{len(projected['common'])} components every build has"
                if build.get("common")
                else "",
            )
            for build in builds
        ]
    )
    lines = []
    if full_line:
        lines.append(
            {
                "points": full_line,
                "dashed": False,
                "label": score_label(projected, measured),
            }
        )
    if partial_line and len(projected["common"]) < measured:
        lines.append(
            {
                "points": partial_line,
                "dashed": True,
                "label": f"partial: {score_label(projected, len(projected['common']))} "
                f"({projected['common_weight'] * 100:.0f}% of the weight)",
            }
        )
    series = []
    if lines:
        coverage = (
            "" if measured == total else f", {measured_weight * 100:.0f}% of the suite measured"
        )
        regime = regime_note(projected)
        series.append(
            {
                "id": "score",
                "title": score_label(projected, measured)
                + coverage
                + (f"; {regime}" if regime else ""),
                "verb": "better",
                "worse": "worse",
                "unit": "index",
                # Kept apart from the title so each tooltip, which states the score, can
                # carry it too.
                "regime": regime,
                "lines": lines,
            }
        )
    for component in projected["measured"]:
        # Memory's own line uses the mix every build has, so it compares builds over one
        # set of components; the full score's memory reads every measured component.
        key = "memory_common" if component == "memory" else None
        values = points(
            [
                (
                    build,
                    build.get(key) if key else build["components"].get(component),
                    (
                        f"{build[key]:.3f} of {reference}, over components every build has"
                        if key and build.get(key)
                        else f"{build['components'][component]:.3f} of {reference}"
                        if component in build["components"]
                        else ""
                    ),
                )
                for build in builds
            ]
        )
        if values:
            title = projected["titles"].get(component, component)
            memory = component == "memory"
            series.append(
                {
                    "id": component,
                    "title": title,
                    "verb": "less memory" if memory else "faster",
                    "worse": "more memory" if memory else "slower",
                    "unit": "peak memory" if memory else "time",
                    "lines": [{"points": values, "dashed": False, "label": title.lower()}],
                }
            )
    return series


def trend_label(first: float, last: float, verb: str, worse: str) -> str:
    """How a line moved from its first build to its last, in the direction it moved.

    A line that ends above where it started would otherwise read "0.8x faster".
    """
    ratio = first / last
    if ratio >= 1:
        return f"{ratio:.1f}x {verb}"
    return f"{1 / ratio:.1f}x {worse}"


def _numbered_builds(
    points: Sequence[Mapping[str, Any]], position: Mapping[str, int]
) -> Dict[str, str]:
    """Every build the chart places, numbered in the order the builds landed.

    The full score's line starts at the first fully covered build and the partial
    line at the first build, so numbering in the order the lines list their points
    would number a late build first.
    """
    named: Dict[str, str] = {}
    for point in sorted(points, key=lambda point: position[point["after_experiment"]]):
        named.setdefault(point["after_experiment"], point["short"])
    return named


def _describe_cells(cells: Sequence[Mapping[str, Any]]) -> str:
    """The history cells the chart reads, one phrase per tree rather than per cell.

    A tree timed once per component job would otherwise repeat its title for every
    cell; the phrase says how many cells and paired rounds each tree has.
    """
    groups: Dict[tuple, List[Mapping[str, Any]]] = {}
    for cell in cells:
        key = (
            cell.get("title") or cell["subject"],
            cell.get("entries"),
            cell.get("cpu") or "",
            cell.get("storage") or "",
            cell.get("regime") or "",
        )
        groups.setdefault(key, []).append(cell)
    phrases = []
    for (title, entries, cpu, storage, regime), members in groups.items():
        rounds = sorted({cell.get("trials") for cell in members if cell.get("trials")})
        rounds_text = (
            f"{rounds[0]} paired rounds"
            if len(rounds) == 1
            else f"{rounds[0]} to {rounds[-1]} paired rounds"
            if rounds
            else "paired rounds"
        )
        count = f"{len(members)} cells, one per component job, " if len(members) > 1 else ""
        size = f"{entries:,} entries, " if isinstance(entries, int) else ""
        phrases.append(
            f"{esc(title)} ({size}{count}{rounds_text}, {esc(cpu)}, {esc(storage)}, "
            f"{esc(regime)} host)"
        )
    return "; ".join(phrases)


def figure_timeline(dataset: Mapping[str, Any]) -> str:
    """Two stacked panels on one experiment axis, in the order the experiments ran.

    The top panel is the chosen metric, the platform's score or one component, for every
    milestone build as a multiple of the reference build, each job timed in one
    interleaved session, so its steps compare directly. The bottom panel is every
    experiment's paired change on its own primary job, green where a change was kept, red
    where it was tried and dropped, grey for remeasurements and other verdicts. The bottom
    panel's effects are not multiplied into a runtime: each was measured on its own job
    and tree, and compounding them would claim a speed-up no build shows.
    """
    records = _chronological(dataset)
    if not records:
        return ""
    position = {record["id"]: index for index, record in enumerate(records)}
    cells = []
    for cell in dataset.get("history") or []:
        placed = [
            item
            for item in cell.get("milestones", [])
            if item.get("wall_ms") and item.get("after_experiment") in position
        ]
        if len(placed) >= 2:
            cells.append((cell, placed))
    chart = projected_platform(dataset)
    series = metric_series(dataset, position)
    milestones = [point for item in series for line in item["lines"] for point in line["points"]]

    left, right = 64, 150
    width = 900
    plot = width - left - right
    step = plot / len(records)
    x_of = lambda index: left + (index + 0.5) * step
    top_y0, top_h = 70, 210 if milestones else 0
    gap = 64 if milestones else 0
    bottom_y0 = top_y0 + top_h + gap
    bottom_h = 220
    height = bottom_y0 + bottom_h + 44
    out = svg_open(
        width,
        height,
        "Performance score by build above, every experiment's effect below",
        "Top: the chosen metric for each milestone build, as a multiple of the reference "
        "build's, on a log scale. Bottom: one bar per experiment, its paired change on its "
        "own primary job; green kept, red not kept, grey other verdicts.",
    )

    # Shared date ticks along the experiment axis: the first experiment of each date,
    # labelled only where the label has room.
    months = {"07": "Jul", "08": "Aug", "09": "Sep", "10": "Oct", "11": "Nov", "12": "Dec"}
    last_label_x = -1e9
    previous_date = None
    for index, record in enumerate(records):
        date = record.get("date") or ""
        if not date or date == previous_date:
            continue
        previous_date = date
        x = left + index * step
        if x - last_label_x >= 64:
            last_label_x = x
            label = f"{months.get(date[5:7], date[5:7])} {int(date[8:10])}"
            out.append(
                f'<line class="grid" x1="{x:.1f}" y1="{top_y0}" x2="{x:.1f}" '
                f'y2="{bottom_y0 + bottom_h}"/>'
            )
            out.append(
                f'<text class="tick" x="{x + 3:.1f}" y="{bottom_y0 + bottom_h + 16}">'
                f"{label}</text>"
            )
    out.append(
        f'<text class="tick axis-name" x="{left}" y="{height - 6}">experiments in the '
        f"order they ran</text>"
    )

    if series:
        # Log scale of each metric relative to the reference build, so a tree timed in
        # seconds, one timed in milliseconds, the score, and memory read on one axis,
        # and every line ends at 1x.
        values = [point["share"] for item in series for line in item["lines"] for point in line["points"]]
        floor = min(1.0, min(values)) * 0.85
        ceiling = max(1.0, max(values)) * 1.15
        span = math.log10(ceiling) - math.log10(floor)
        y_of = lambda share: top_y0 + (math.log10(ceiling) - math.log10(share)) / span * top_h
        for tick in (0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0):
            if floor <= tick <= ceiling:
                y = y_of(tick)
                out.append(f'<line class="grid" x1="{left}" y1="{y:.1f}" x2="{width - right}" y2="{y:.1f}"/>')
                out.append(
                    f'<text class="tick" x="{left - 6}" y="{y + 4:.1f}" text-anchor="end">'
                    f"{tick:g}&times;</text>"
                )
        reference = (chart or {}).get("reference_build", "the reference build")
        out.append(
            f'<text class="tick axis-name" x="{left}" y="{top_y0 - 46}">the chosen metric as a '
            f"multiple of {esc(reference)}&rsquo;s, log scale, lower is better; "
            f"builds numbered</text>"
        )
        # Milestones numbered once along the top, staggered over rows so builds that
        # landed close together stay legible; the caption names them. Every cell times
        # the same builds.
        named = _numbered_builds(milestones, position)
        for number, after in enumerate(named, start=1):
            x = x_of(position[after])
            label_y = top_y0 - 30 + (number - 1) % 3 * 10
            out.append(
                f'<line class="grid" x1="{x:.1f}" y1="{label_y + 3}" x2="{x:.1f}" '
                f'y2="{top_y0 + top_h}"/>'
            )
            out.append(
                f'<text class="point-label" x="{x:.1f}" y="{label_y}" text-anchor="middle">'
                f"{number}</text>"
            )
        for item in series:
            # Not the platform colours: every line here is one platform's metric.
            css = "series-cell-a" if item["id"] == "score" else "series-cell-b"
            group = [f'<g class="metric{" on" if item["id"] == "score" else ""}" data-metric="{esc(item["id"])}">']
            for line in item["lines"]:
                points = [
                    (x_of(position[point["after_experiment"]]), y_of(point["share"]), point)
                    for point in line["points"]
                ]
                path = []
                for index, (x, y, _) in enumerate(points):
                    if index:
                        path.append(f"{x:.1f},{points[index - 1][1]:.1f}")
                    path.append(f"{x:.1f},{y:.1f}")
                path.append(f"{width - right:.1f},{points[-1][1]:.1f}")
                dashed = " dashed" if line["dashed"] else ""
                group.append(f'<polyline class="{css} series-line{dashed}" points="{" ".join(path)}"/>')
                first, last = line["points"][0], line["points"][-1]
                for x, y, point in points:
                    group.append(f'<circle class="{css}" cx="{x:.1f}" cy="{y:.1f}" r="3.5"/>')
                    group.append(
                        f'<rect class="hit" x="{x - 7:.1f}" y="{y - 7:.1f}" width="14" height="14" '
                        + tip(
                            f"{line['label']}: {point['short']} "
                            f"({point['commit']}, {point['date']})\n"
                            f"{point['includes']}\n"
                            # The score is an index, memory a peak, a component a time.
                            f"{point['share']:.2f}x {reference}'s {item['unit']}"
                            + (f"; {point['detail']}" if point.get("detail") else "")
                            + (f"; {item['regime']}" if item.get("regime") else "")
                        )
                        + "/>"
                    )
                # Every line ends at the reference build, so each is labelled at its start.
                x0, y0 = points[0][0], points[0][1]
                group.append(
                    f'<text class="value-label {css}" x="{x0 + 8:.1f}" y="{y0 - 8:.1f}" '
                    f'stroke="none">'
                    f'{esc(trend_label(first["share"], last["share"], item["verb"], item["worse"]))} '
                    f"since {esc(first['short'])}</text>"
                )
                group.append(
                    f'<text class="point-label" x="{x0 + 8:.1f}" y="{y0 + 14:.1f}">'
                    f"{esc(line['label'])}</text>"
                )
            # An annotated build is labelled once per metric that draws it, beside its
            # numbered grid line just inside the panel's top: above every data point
            # except a steep line's first, which sits at the far left.
            noted = {
                point["label"]: point["after_experiment"]
                for line in item["lines"]
                for point in line["points"]
                if point["label"] in BUILD_NOTES
            }
            for label, after in noted.items():
                x = x_of(position[after])
                inward = x > left + plot / 2
                anchor_x = x - 6 if inward else x + 6
                rows = "".join(
                    f'<tspan x="{anchor_x:.1f}" dy="{12 if row else 0}">{esc(text)}</tspan>'
                    for row, text in enumerate(BUILD_NOTES[label])
                )
                group.append(
                    f'<text class="tick build-note" y="{top_y0 + 10}" '
                    f'text-anchor="{"end" if inward else "start"}">{rows}</text>'
                )
            group.append("</g>")
            out.extend(group)

    # Bottom panel: every experiment, coloured by what it did.
    low, high = ITERATION_CLAMP
    y_of_change = lambda faster: bottom_y0 + (high - faster) / (high - low) * bottom_h
    zero = y_of_change(0.0)
    for faster in (-30, 0, 30, 60):
        y = y_of_change(faster)
        out.append(f'<line class="grid" x1="{left}" y1="{y:.1f}" x2="{width - right}" y2="{y:.1f}"/>')
        out.append(
            f'<text class="tick" x="{left - 6}" y="{y + 4:.1f}" text-anchor="end">'
            + ("0" if faster == 0 else f"{faster:+d}%")
            + "</text>"
        )
    threshold = y_of_change(3.0)
    out.append(
        f'<line class="threshold" x1="{left}" y1="{threshold:.1f}" x2="{width - right}" y2="{threshold:.1f}"/>'
    )
    out.append(f'<line class="zero" x1="{left}" y1="{zero:.1f}" x2="{width - right}" y2="{zero:.1f}"/>')
    out.append(
        f'<text class="tick axis-name" x="{left}" y="{bottom_y0 - 14}">each experiment: % better '
        f"on its own job, paired</text>"
    )
    css = {"kept": "dot-good", "rejected": "dot-bad", "measured": "dot-flat"}
    bar = max(min(step * 0.7, 6.0), 1.2)
    jobs = dataset.get("index_jobs") or {}
    titles = {}
    for platform in dataset.get("index") or []:
        titles.update(platform.get("titles") or {})
    for index, record in enumerate(records):
        change = record.get("change_pct")
        faster = -change if change is not None else 0.0
        drawn = max(low, min(high, faster))
        x = left + index * step + (step - bar) / 2
        y1, y2 = sorted((zero, y_of_change(drawn)))
        kind = iteration_kind(record)
        metric = record.get("primary_metric")
        component = "memory" if metric == "peak_rss_bytes" else jobs.get(record.get("primary_job") or "", "")
        out.append(
            f'<rect class="{css[kind]} bar" x="{x:.1f}" y="{y1:.1f}" width="{bar:.1f}" '
            f'height="{max(y2 - y1, 1.0):.1f}" data-component="{esc(component)}" '
            f'data-platform="{esc(record.get("platform") or "")}"/>'
        )
        label = {"kept": ", at least 3% better", "rejected": "", "measured": ""}[kind]
        counts_toward = titles.get(component) or component or "no component"
        out.append(
            f'<rect class="hit" x="{left + index * step:.1f}" y="{bottom_y0}" '
            f'width="{step:.1f}" height="{bottom_h}" '
            + tip(
                f"{record['id']}: {record['title']}\n"
                f"{decision_label(record)}{label}: "
                f"{fmt_pct(change) if change is not None else 'no paired change'} "
                f"on {record.get('primary_job') or 'its job'}"
                + (f" ({metric})" if metric not in (None, "wall_ns") else "")
                + f"\nCounts toward: {counts_toward}, {record.get('platform') or 'its platform'}"
                + (
                    f"\nNot a new change: {REMEASUREMENTS[record['id']]}"
                    if record["id"] in REMEASUREMENTS
                    else ""
                )
            )
            + "/>"
        )
    out.append("</svg>")

    counts = {kind: sum(iteration_kind(record) == kind for record in records) for kind in css}
    keys = legend(
        ("key-cell-a", f"the {(chart or {}).get('platform', '')} score".replace("the  ", "the ")),
        ("key-cell-b", "a single component, when chosen"),
        ("key-good", f"kept, at least 3% better ({counts['kept']})"),
        ("key-bad", f"tried, not kept ({counts['rejected']})"),
        ("key-flat", f"other verdicts and remeasurements ({counts['measured']})"),
    )
    caption = ""
    if cells:
        caption = (
            f"Top: each cell timed every milestone build in one interleaved session, on "
            f"{_describe_cells([cell for cell, _ in cells])}. Builds before 0.1.0 do not "
            f"read .gitignore, so on a tree with .gitignore files they do less work. "
        )
        named = _numbered_builds(milestones, position)
        caption += (
            "Builds: "
            + "; ".join(f"{number} {esc(short)}" for number, short in enumerate(named.values(), start=1))
            + ". "
        )
    chooser = ""
    if series:
        options = "".join(
            f'<option value="{esc(item["id"])}"{" selected" if item["id"] == "score" else ""}>'
            f"{esc(item['title'])}</option>"
            for item in series
        )
        chooser = (
            '<p class="metric-chooser"><label>Metric <select id="metric" '
            f'data-platform="{esc((chart or {}).get("platform", ""))}">'
            f"{options}</select></label> <span class=\"muted\">Bars that do not count "
            "toward the chosen metric are faded.</span></p>"
        )
    return (
        f'<figure class="fig">{chooser}{"".join(out)}{keys}<figcaption>{caption}Bottom: bars beyond '
        "the axis stop at its edge, and their tooltips give the real figure. The dashed line "
        "is the 3% accept threshold.</figcaption></figure>"
    )

def figure_effects(dataset: Mapping[str, Any]) -> str:
    """Every experiment's paired effect on its own primary job, with its interval.

    This is the relative view, the one the verdicts were made on. The dot is the median
    paired change; the bar behind it is the 95% bootstrap interval. An interval that
    crosses zero means the run could not tell the change from noise, which is a different
    statement from "no effect" and is drawn differently from both wins and regressions.

    The dashed line is the accept threshold. Nothing to its right could be accepted as a
    speed-up, and publishing those experiments is the point of the figure.

    A baseline that compares two builds is drawn too, since its change is what it
    measured: the end-to-end and release cells (exp-194, exp-195, exp-201, exp-202) are
    how the record says what a campaign added up to. A baseline of one build against itself
    has nothing to draw.
    """
    records = [
        record
        for record in dataset["experiments"]
        if compares(record) and _primary(record) is not None
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
    # The axis is sized to the data so nothing is clipped: a fixed window would push the
    # largest measured regressions, among the most useful rejections in the record, off
    # the picture. Bounds snap to `step` so the axis hugs the data, and labels fall every
    # `label_step` so the scale stays readable at one row per experiment.
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
        "One dot and interval per experiment, sorted by effect.",
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
        "paired change on the primary job, left is faster</text>"
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
        low_text, high_text = paired["ci95_low_pct"], paired["ci95_high_pct"]
        detail = [
            f'{record["id"]}: {record["title"]}',
            f'{decision_label(record).upper()}'
            f'  |  {record["primary_job"]}  |  {fmt_pct(change)}'
            + (
                f" [{low_text:+.1f}%, {high_text:+.1f}%]"
                if low_text is not None and high_text is not None
                else ""
            ),
        ]
        if compares(record) and record["candidate"]:
            detail.append(f'Tried: {record["candidate"]}')
        if record["reason"]:
            detail.append(f'Why: {record["reason"]}')
        detail.append(
            f'{record["entries"]:,} entries, {record["platform"]}'
            f'  |  {record["trials"]} paired rounds'
        )
        out.append(hover_row(left, y, plot, row, "\n".join(detail)))
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
            + f"; {esc(decision_label(record))}</title></circle>"
        )
    out.append("</svg>")

    accepted = sum(1 for record in records if record["decision"] == "accepted")
    baselines = sum(1 for record in records if record["decision"] == "baseline")
    evidence = [_primary(record)["paired"]["evidence"] for record in records]
    improved = evidence.count("improved")
    unclear = evidence.count("unclear")
    regressed = evidence.count("regressed")
    return (
        f'<figure class="fig">{"".join(out)}'
        + legend(
            ("key-good", "interval entirely below zero"),
            ("key-bad", "interval entirely above zero"),
            ("key-flat", "interval crosses zero: the run could not tell"),
        )
        + f"<figcaption>{len(records)} experiments, sorted by effect"
        + (
            f", {baselines} of them baselines that compare two builds or configurations"
            " and decide nothing"
            if baselines
            else ""
        )
        + f"; {accepted} were accepted. "
        f"{improved} intervals lie entirely below zero, {unclear} cross zero, and "
        f"{regressed} lie entirely above it. Nothing is clipped: the axis runs to the "
        "widest interval measured."
        "</figcaption></figure>"
    )


def _primary(record: Mapping[str, Any]) -> Optional[Dict[str, Any]]:
    """The metric the verdict rested on, on the job it rested on."""
    job = next((item for item in record["jobs"] if item["job"] == record["primary_job"]), None)
    if not job:
        return None
    return job["metrics"].get(record["primary_metric"] or "wall_ns")


def figure_per_entry(dataset: Mapping[str, Any]) -> str:
    """Cost per entry per subject, so trees three orders of magnitude apart can be compared.

    Milliseconds are only comparable against the tree that produced them, and this record
    spans 307 entries to 1.01 million. A scan's cost grows roughly linearly with the
    entries it visits, so dividing by entry count is the normalization the workload
    supports, and it lets a reader check that the speed-up was not a small-tree artifact.

    Each row is one subject, largest first, showing the first cost measured on it and the
    last. Plotting every measurement against its date would pile the points into one
    column per day of the record, hiding the trend.

    The synthetic subjects are drawn but held apart. Some are built to be adversarial, so
    averaging them in would misstate what a real tree costs.
    """
    subjects = {subject["key"]: subject for subject in dataset["subjects"]}
    grouped: Dict[str, List[Dict[str, Any]]] = {}
    for record in dataset["experiments"]:
        job = next((item for item in record["jobs"] if item["job"] == "cold-scan-index"), None)
        if not job:
            continue
        # The arm that stayed in the product, not the arm that was tried, so a rejected
        # candidate never appears as a subject's current cost. A record whose verdict says
        # `kept: neither` names no arm, because its verdict decided a claim about code that
        # ships regardless, so it has no point here.
        if record["kept"] is None:
            continue
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
        "One row per measured tree: real trees largest first, then generated ones.",
    )
    for tick in ticks:
        x = left + (tick / top) * plot
        out.append(f'<line class="grid" x1="{x:.1f}" y1="30" x2="{x:.1f}" y2="{height - 30}"/>')
        out.append(
            f'<text class="tick" x="{x:.1f}" y="{height - 14}" text-anchor="middle">{tick:g}</text>'
        )
    out.append(
        f'<text class="tick axis-name" x="{left}" y="18">'
        "microseconds per entry, lower is faster</text>"
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
            + (" &middot; generated" if subject["synthetic"] else "")
            + "</tspan></text>"
            # Two subjects can share a platform and a size and still be different trees,
            # so the tree's own name goes underneath rather than leaving two rows that
            # look like a duplicate.
            + f'<text class="row-sub" x="{left - 14}" y="{y + 17}" text-anchor="end">'
            f'{esc(subject["labels"][0])}</text>' 
        )
        first, last = row["first"], row["last"]
        out.append(
            hover_row(
                left,
                y + (row_height - 8) / 2,
                plot,
                row_height - 8,
                f'{subject["labels"][0]}: {subject["platform"]}, {entries:,} entries\n'
                + ("A generated subject: screening evidence, held apart from the real "
                   "trees rather than averaged with them.\n" if subject["synthetic"] else "")
                + f'First measured at {first["control"]:.2f} µs per entry, latest '
                f'{last["kept"]:.2f} µs, across {row["count"]} '
                f'{"experiment" if row["count"] == 1 else "experiments"}.',
            )
        )
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
        + "<figcaption>Cold scan and index only, the one job every subject ran. Each later "
        "point is the arm its verdict kept, so a rejected candidate never appears as a "
        f"tree's current cost. The {len(finals)} real macOS subjects finish between "
        f"{min(finals):.1f} and {max(finals):.1f} µs per entry.</figcaption></figure>"
    )


#: The whole visual language, in tokens.
#:
#: Adapted from the tbd web design system, keeping two of its rules: every colour is
#: defined once here and referenced by name everywhere else, and each token's hue places
#: it in a family. The neutrals all sit on hue 215 so surfaces, rules, and text read as
#: one material, while the semantic families keep the hues they have elsewhere in the
#: project's tooling. Dark holds hue and saturation and moves lightness only.
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
  --cell-b: hsl(268 52% 52%);
  color-scheme: light;
  /* The system stack, and nothing else. A webfont would be a network dependency inside a
     committed document, in a repository that pins everything else it depends on, and this
     file has to render from a file:// URL on a machine that has never opened it before.
     Naming a preferred face ahead of the stack would also make the page look different
     from one reader to the next depending on what they happen to have installed.
     Monospace is reserved for measured values, which is the one distinction the type
     needs to carry. */
  --sans: system-ui, -apple-system, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
  --mono: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
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
    --cell-b: hsl(268 70% 74%);
    color-scheme: dark;
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
  --cell-b: hsl(268 70% 74%);
  color-scheme: dark;
}

* { box-sizing: border-box; }
:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; border-radius: 2px; }
html { -webkit-text-size-adjust: 100%; }
body {
  margin: 0; padding: 0 24px 96px;
  background: var(--bg); color: var(--text);
  font: 15px/1.6 var(--sans);
  -webkit-font-smoothing: antialiased;
}
.wrap { max-width: 960px; margin: 0 auto; }
h1, h2, h3 { line-height: 1.25; font-weight: 650; }
h1 {
  font-size: 28px; margin: 56px 0 12px; letter-spacing: -0.015em;
  text-wrap: balance; font-weight: 600;
}
h2 {
  font-size: 12px; text-transform: uppercase; letter-spacing: 0.08em;
  color: var(--muted); font-weight: 650;
  margin: 64px 0 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border);
}
h3 { font-size: 16px; margin: 34px 0 8px; text-wrap: balance; font-weight: 600; }
p, li { max-width: var(--measure); }
p { margin: 0 0 14px; }
a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }
.lede { max-width: var(--measure); }
.muted { color: var(--muted); }
.mono { font-family: var(--mono); font-size: 0.92em; }
.tnum { font-variant-numeric: tabular-nums; }

/* Headline figures. Unboxed: the number is the point, a card around it is not. They
   are sized to hold one row at the widths this page is read at, and wrap rather than
   strand a single stat below the others. */
.headline { display: flex; flex-wrap: wrap; gap: 14px 30px; margin: 28px 0 8px; }
.headline div { min-width: 92px; }
.headline .n {
  display: block; font-size: 22px; font-weight: 650;
  font-variant-numeric: tabular-nums; letter-spacing: -0.02em;
}
.headline .k {
  display: block; font-size: 12px; text-transform: uppercase;
  letter-spacing: 0.06em; color: var(--muted); margin-top: 2px;
}
.headline .good { color: var(--good); }
.headline .bad { color: var(--bad); }

figure.fig { margin: 24px 0 8px; }
.chart { width: 100%; height: auto; display: block; overflow: visible; }
figcaption { font-size: 12px; color: var(--muted); margin-top: 10px; max-width: var(--measure); }
.checkpoints { margin: 8px 0 0; padding-left: 18px; }
.checkpoints li { margin: 1px 0; }

.legend { font-size: 12px; color: var(--muted); margin: 12px 0 0; display: flex; flex-wrap: wrap; gap: 4px 20px; max-width: none; }
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
.key-cell-a { background: var(--after); }
.key-cell-b { background: var(--cell-b); }
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
.metric { display: none; }
.metric.on { display: inline; }
.bar.faded { opacity: 0.15; }
.metric-chooser { font-size: 13px; margin: 0 0 8px; max-width: none; }
.metric-chooser select { font: inherit; margin-left: 6px; padding: 2px 4px; color: var(--text);
  background: var(--bg); border: 1px solid var(--border); border-radius: 4px; }
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
.series-cell-a { stroke: var(--after); fill: var(--after); }
.series-cell-b { stroke: var(--cell-b); fill: var(--cell-b); }
.series-line { fill: none; stroke-width: 1.5; }
.series-line.dashed { stroke-dasharray: 5 4; }
.point-label { font: 10px var(--mono); fill: var(--muted); }
.gutter { font: 9.5px var(--mono); fill: var(--muted); opacity: 0.75; }
/* Every mark is inert to the pointer, so the only hover target inside a chart is the row
   band underneath it. The marks are drawn above the band, so a hoverable dot would make
   the pointer leave and re-enter the band, flickering the tooltip; and a hoverable mark
   fires its own native `title` bubble, a second tooltip saying something different. */
.chart circle, .chart polyline, .chart line, .chart text,
.chart rect:not(.hit) { pointer-events: none; }

/* Invisible, but it is the hover target for the whole row. `pointer-events: all` is
   required: a fill of `none` would otherwise make it untouchable. */
.hit { fill: transparent; pointer-events: all; }
.hit:hover { fill: var(--panel); fill-opacity: 0.55; }

#tip {
  /* `max-content` with a cap, rather than leaving it to shrink-to-fit. A fixed-position
     box with no width is sized against the space left before the viewport edge, so it
     would be measured at its previous position and could wrap into a narrow column.
     Sizing it from its own content makes the placement code's measurement independent
     of where the box sits. The cap is in `ch` rather than `vw` because viewport units
     are zero inside some embedded frames, which would collapse the cap to nothing. */
  position: fixed; z-index: 50; padding: 8px 10px;
  width: max-content; max-width: 52ch;
  background: var(--panel); color: var(--text);
  border: 1px solid var(--border); border-radius: 3px;
  font: 12px/1.5 var(--sans); white-space: pre-line;
  pointer-events: none; visibility: hidden;
}
#tip.on { visibility: visible; }
#tip b { font-weight: 600; }

table { border-collapse: collapse; width: 100%; font-size: 13px; margin: 16px 0; }
th, td { text-align: left; padding: 7px 10px; border-bottom: 1px solid var(--rule); vertical-align: top; }
th {
  font-size: 12px; text-transform: uppercase; letter-spacing: 0.06em;
  color: var(--muted); font-weight: 650; border-bottom: 1px solid var(--border);
  position: sticky; top: 0; background: var(--bg);
}
td.n, th.n { text-align: right; font-variant-numeric: tabular-nums; font-family: var(--mono); font-size: 12px; }
tbody tr:hover { background: var(--panel); }
.scroll { overflow-x: auto; margin: 16px -4px; padding: 0 4px; }
.v-accepted { color: var(--good); }
.v-rejected { color: var(--bad); }
.v-superseded, .v-in-progress, .v-baseline { color: var(--muted); }
.note {
  border-left: 2px solid var(--border); padding: 2px 0 2px 16px;
  margin: 20px 0; color: var(--muted); max-width: var(--measure);
}
.note strong { color: var(--text); }
/* Disclosure rows. A marker and a hover tint are the whole affordance; the table has a
   row per experiment, and anything heavier turns it into a wall of boxes. */
details > summary {
  cursor: pointer; list-style: none; font-weight: 500;
  padding: 1px 0; border-radius: 2px;
}
details > summary::-webkit-details-marker { display: none; }
details > summary::before {
  content: "+"; display: inline-block; width: 12px;
  color: var(--muted); font-family: var(--mono); font-weight: 400;
}
details[open] > summary::before { content: "-"; }
details > summary:hover { color: var(--accent); }
dl.detail {
  margin: 8px 0 4px 12px; padding-left: 12px; border-left: 1px solid var(--rule);
  display: grid; grid-template-columns: max-content 1fr; gap: 4px 14px;
  font-size: 13px; line-height: 1.55; max-width: 78ch;
}
dl.detail dt {
  color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em;
  font-size: 12px; padding-top: 1px; white-space: nowrap;
}
dl.detail dd { margin: 0; }
.arrow { color: var(--border); }
.source { margin: 6px 0 4px 24px; font-size: 12px; color: var(--muted); }
footer { margin-top: 72px; padding-top: 16px; border-top: 1px solid var(--border); font-size: 12px; color: var(--muted); }
"""


#: The whole of the page's behaviour.
#:
#: One delegated listener and one reused node, which is how the tbd design system handles
#: tooltips and why this needs no framework. Native `title` bubbles were the alternative:
#: they take about a second to appear, cannot render more than one line well, and on the
#: effects figure would have had a three-pixel dot as their only target.
#:
#: ASCII only, for the same reason the stylesheet is: script content is raw text, so a
#: character reference here would be shown literally rather than decoded.
SCRIPT = """
(function () {
  var tip = document.getElementById('tip');
  var shown = null;
  // Anchored to the row, not to the pointer. A tooltip that tracks the mouse is
  // unreadable while the eye is trying to read it, so it is placed once when a row is
  // entered and stays there until another row is entered or it is dismissed.
  function place(host) {
    var pad = 10;
    var row = host.getBoundingClientRect();
    var box = tip.getBoundingClientRect();
    // Some embeddings report a zero viewport. Clamping against that would put every
    // tooltip in the top-left corner, so a zero bound is not applied.
    var vw = window.innerWidth || document.documentElement.clientWidth || 0;
    var vh = window.innerHeight || document.documentElement.clientHeight || 0;
    var x = row.left + pad;
    var y = row.bottom + pad;
    if (vw > 0 && x + box.width > vw - 8) x = vw - box.width - 8;
    if (vh > 0 && y + box.height > vh - 8) y = row.top - box.height - pad;
    tip.style.left = Math.max(8, x) + 'px';
    tip.style.top = Math.max(8, y) + 'px';
  }
  function hide() {
    shown = null;
    tip.classList.remove('on');
  }
  document.addEventListener('mouseover', function (event) {
    var host = event.target.closest ? event.target.closest('[data-tip]') : null;
    if (!host) { if (shown) hide(); return; }
    if (host === shown) return;
    shown = host;
    var text = host.getAttribute('data-tip').split('\\n');
    tip.textContent = '';
    text.forEach(function (line, index) {
      if (index === 0) {
        var strong = document.createElement('b');
        strong.textContent = line;
        tip.appendChild(strong);
      } else {
        tip.appendChild(document.createTextNode(line));
      }
      if (index < text.length - 1) tip.appendChild(document.createTextNode('\\n'));
    });
    tip.classList.add('on');
    place(host);
  });
  document.addEventListener('mouseleave', hide, true);
  document.addEventListener('keydown', function (event) {
    if (event.key === 'Escape') hide();
  });
  window.addEventListener('scroll', hide, { passive: true });
})();
"""


#: The theme chooser, borrowed from tbd's web view: a gear opening a segmented
#: System / Light / Dark group. "system" follows the reader's OS; the other two force a
#: theme through `data-theme`, which the style blocks above already honour. The choice is
#: stored per browser, and the page renders correctly without storage.
THEME_PREPAINT = """
try {
  var mode = localStorage.getItem('fdu.report.themeMode') || 'system';
  if (mode !== 'light' && mode !== 'dark') mode = 'system';
  document.documentElement.setAttribute('data-theme-mode', mode);
  if (mode !== 'system') document.documentElement.setAttribute('data-theme', mode);
} catch (_error) {
  // Private mode or blocked storage: the system theme remains active.
}
"""

SETTINGS = """
<div class="topbar"><span id="settings">
<button type="button" id="gear" aria-label="Settings" aria-expanded="false" aria-controls="menu">
<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915"/><circle cx="12" cy="12" r="3"/></svg>
</button>
<div id="menu" hidden>
<div class="menu-label">Theme</div>
<div class="chooser" role="group" aria-label="Theme">
<button type="button" class="seg" data-theme-choice="system" aria-pressed="false" title="System theme" aria-label="System theme"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect width="20" height="14" x="2" y="3" rx="2"/><line x1="8" x2="16" y1="21" y2="21"/><line x1="12" x2="12" y1="17" y2="21"/></svg></button>
<button type="button" class="seg" data-theme-choice="light" aria-pressed="false" title="Light theme" aria-label="Light theme"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/></svg></button>
<button type="button" class="seg" data-theme-choice="dark" aria-pressed="false" title="Dark theme" aria-label="Dark theme"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401"/></svg></button>
</div>
</div>
</span></div>
"""

THEME_STYLE = """
.topbar { position: relative; height: 0; }
#settings { position: absolute; right: 0; top: 14px; }
#gear { display: inline-flex; align-items: center; justify-content: center; width: 32px; height: 32px;
  padding: 0; border: 1px solid transparent; border-radius: 6px; background: transparent;
  color: var(--muted); cursor: pointer; }
#gear:hover, #gear[aria-expanded='true'] { color: var(--text); background: var(--panel); border-color: var(--border); }
#gear svg { width: 18px; height: 18px; }
#menu { position: absolute; right: 0; top: calc(100% + 6px); background: var(--bg);
  border: 1px solid var(--border); border-radius: 6px; padding: 8px; z-index: 10; min-width: 150px; }
#menu[hidden] { display: none; }
.menu-label { font-size: 11px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--muted); margin-bottom: 5px; }
.chooser { display: flex; gap: 2px; }
.seg { flex: 1; display: inline-flex; align-items: center; justify-content: center; height: 30px;
  padding: 0; border: 0; border-radius: 4px; background: transparent; color: var(--muted); cursor: pointer; }
.seg:hover, .seg:focus-visible { color: var(--text); background: var(--panel); }
.seg svg { width: 16px; height: 16px; }
.seg[aria-pressed='true'] { color: var(--accent); background: var(--panel); }
"""

THEME_SCRIPT = """
(function () {
  var root = document.documentElement;
  var gear = document.getElementById('gear');
  var menu = document.getElementById('menu');
  var buttons = Array.prototype.slice.call(document.querySelectorAll('[data-theme-choice]'));
  function mode(value) { return value === 'light' || value === 'dark' ? value : 'system'; }
  function apply(choice, persist) {
    root.setAttribute('data-theme-mode', choice);
    if (choice === 'system') root.removeAttribute('data-theme');
    else root.setAttribute('data-theme', choice);
    buttons.forEach(function (button) {
      button.setAttribute('aria-pressed', String(button.getAttribute('data-theme-choice') === choice));
    });
    if (persist) {
      try { localStorage.setItem('fdu.report.themeMode', choice); } catch (_error) {}
    }
  }
  function close() { menu.hidden = true; gear.setAttribute('aria-expanded', 'false'); }
  gear.addEventListener('click', function (event) {
    event.stopPropagation();
    menu.hidden = !menu.hidden;
    gear.setAttribute('aria-expanded', String(!menu.hidden));
  });
  menu.addEventListener('click', function (event) { event.stopPropagation(); });
  document.addEventListener('click', close);
  document.addEventListener('keydown', function (event) { if (event.key === 'Escape') close(); });
  buttons.forEach(function (button) {
    button.addEventListener('click', function () { apply(mode(button.getAttribute('data-theme-choice')), true); });
  });
  apply(mode(root.getAttribute('data-theme-mode')), false);
})();
"""

#: The metric chooser: shows the chosen series on the top panel and fades every
#: experiment bar that does not count toward it. The score counts every component, so
#: under it only bars from a platform the score does not cover fade.
METRIC_SCRIPT = """
(function () {
  var chooser = document.getElementById('metric');
  if (!chooser) return;
  var platform = chooser.getAttribute('data-platform');
  var groups = Array.prototype.slice.call(document.querySelectorAll('.metric'));
  var bars = Array.prototype.slice.call(document.querySelectorAll('.bar'));
  function show(metric) {
    groups.forEach(function (group) {
      group.classList.toggle('on', group.getAttribute('data-metric') === metric);
    });
    bars.forEach(function (bar) {
      var counts = bar.getAttribute('data-platform') === platform &&
        (metric === 'score' || bar.getAttribute('data-component') === metric);
      bar.classList.toggle('faded', !counts);
    });
  }
  chooser.addEventListener('change', function () { show(chooser.value); });
  show(chooser.value);
})();
"""


def render(dataset: Mapping[str, Any]) -> str:
    """The whole page."""
    body = "".join(
        [
            _header(dataset),
            _section_iterations(dataset),
            _section_loop(dataset),
            _section_absolute(dataset),
            _section_relative(dataset),
            _section_scale(dataset),
            _section_platforms(dataset),
            _section_mechanisms(dataset),
            _section_reading(dataset),
            _section_table(dataset),
            _footer(dataset),
        ]
    )
    page = f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>fdu Performance Evidence</title>
<meta name="description" content="The fdu performance score by build, with the absolute times and paired effects of every fdu performance experiment, including rejected ones.">
<script>{THEME_PREPAINT}</script>
<style>{STYLE}{THEME_STYLE}</style>
</head>
<body>
<div class="wrap">{SETTINGS}{body}</div>
<div id="tip" role="tooltip"></div>
<script>{SCRIPT}{THEME_SCRIPT}{METRIC_SCRIPT}</script>
</body>
</html>
"""
    # The document declares UTF-8 and is also written as pure ASCII, with numeric
    # references for everything else, so it renders the same opened from disk, served by
    # something that guesses a charset, or pasted into a host that supplies its own head,
    # where a literal micro sign could otherwise arrive as two mojibake characters.
    return page.encode("ascii", "xmlcharrefreplace").decode("ascii")


#: The peer tool whose unchanged binary shows how far the host's speed drifts between
#: runs on one tree.
DRIFT_TOOL = "dust"


def peer_drift(dataset: Mapping[str, Any], tool: str = DRIFT_TOOL) -> Optional[Dict[str, Any]]:
    """One peer binary's range on the tree it was timed on most, and its run count.

    Runs are counted by run artifact, not by experiment: experiments that shared a
    session (exp-007 and exp-009, exp-010 and exp-011) share its reading, so counting
    calibration rows would count one run twice. A row without an artifact counts alone.
    """
    subjects: Dict[str, List[Mapping[str, Any]]] = {}
    for row in dataset.get("calibration") or []:
        if row.get("tool") == tool and row.get("wall_ns"):
            subjects.setdefault(row["subject"], []).append(row)

    def runs(rows: Sequence[Mapping[str, Any]]) -> int:
        return len({row.get("run") or row["id"] for row in rows})

    if not subjects:
        return None
    subject, rows = max(subjects.items(), key=lambda item: runs(item[1]))
    walls = [row["wall_ns"] for row in rows]
    return {"subject": subject, "runs": runs(rows), "low_ns": min(walls), "high_ns": max(walls)}


def _section_relative(dataset: Mapping[str, Any]) -> str:
    totals = dataset["totals"]
    rejected = totals["decisions"].get("rejected", 0)
    drift = peer_drift(dataset)
    spread = (
        f': the peer tool <span class="mono">{esc(DRIFT_TOOL)}</span>, whose binary never '
        f"changed, measured {fmt_ms(drift['low_ns']).replace(' ', '&nbsp;')} to "
        f"{fmt_ms(drift['high_ns']).replace(' ', '&nbsp;')} on the same tree across "
        f"{drift['runs']} runs"
        if drift
        else ""
    )
    return f"""
<h2 id="relative">Relative</h2>
<h3>What each experiment did</h3>
<p>Milliseconds say how fast the tool is, but not whether a particular change made it so,
because the host's speed drifts{spread}.
Every experiment therefore interleaved its two builds and compared them in pairs, and each
verdict rests on the paired change below.</p>
{figure_effects(dataset)}
<p>A change that looks promising often measures within a few percent of zero, which is why
a speed-up is accepted only when it clears the threshold with its interval below zero. The
{rejected} rejections are the reusable part of the record: each is an idea the
next person need not try again.</p>
"""


def _section_scale(dataset: Mapping[str, Any]) -> str:
    subjects = dataset["subjects"]
    families = {
        record["family"]
        for record in dataset["experiments"]
        if record.get("family") is not None
    }
    entry_counts = [subject["entries"] for subject in subjects]
    return f"""
<h2 id="scale">Scale</h2>
<h3>Does it hold on a bigger tree, and on another kernel?</h3>
<p>The record measured {len(subjects)} fingerprinted states of {len(families)} trees
(subject families), from {min(entry_counts):,} to {max(entry_counts):,} entries, on macOS
and Linux. Their milliseconds are not comparable. Cost per entry is closer to comparable,
since a scan's work grows roughly linearly with the entries it visits.</p>
{figure_per_entry(dataset)}
"""


def kept_improvements(dataset: Mapping[str, Any], platform: str) -> List[Dict[str, Any]]:
    """Accepted changes still in the product whose deciding run measured an improvement.

    The kept arm must be the candidate, so an accepted screen the release never adopted
    (exp-154) and a verdict that decided a claim rather than code (`kept: neither`) stay
    out. A cumulative checkpoint or a whole pull request measured at once is left out, as
    in the mechanism table, because it would credit one row with a dozen changes. The
    primary interval has to lie below zero, so instrumentation and leftover
    determinations, which are accepted on noninferiority, are not presented as speed-ups.

    A validation run on a second subject or platform stays in although it wrote no code:
    it is the evidence this section exists to show.
    """
    rows = []
    for record in dataset["experiments"]:
        if record["platform"] != platform:
            continue
        if record["decision"] != "accepted" or record["kept"] != "candidate":
            continue
        if record["anchored"] or len(record["hypotheses"]) > 2:
            continue
        metric = _primary(record)
        if not metric or metric["paired"]["evidence"] != "improved":
            continue
        rows.append(record)
    rows.sort(key=lambda record: record["number"])
    return rows


def _entries_short(entries: int) -> str:
    return f"{entries / 1_000_000:.2f}M" if entries >= 1_000_000 else f"{entries / 1000:.0f}k"


def _section_platforms(dataset: Mapping[str, Any]) -> str:
    """What each platform's evidence consists of, and what it kept.

    The rest of the page mixes platforms on purpose, because the loop is one method. A
    reader deciding what fdu does on their machine needs the opposite cut: the changes a
    platform's own runs decided, with that run's absolute arms on its own subject, and how
    many of those runs were on a generated tree, which the loop treats as screening.
    """
    subjects = {subject["key"]: subject for subject in dataset["subjects"]}
    counts: Dict[str, Dict[str, int]] = {}
    families: Dict[str, Dict[str, bool]] = {}
    for record in dataset["experiments"]:
        platform = record["platform"]
        tally = counts.setdefault(platform, {})
        tally[record["decision"]] = tally.get(record["decision"], 0) + 1
        synthetic = subjects[record["subject"]]["synthetic"]
        seen = families.setdefault(platform, {})
        family = record.get("family") or record["subject"]
        seen[family] = seen.get(family, False) or synthetic
    if not counts:
        return ""
    platforms = sorted(counts, key=lambda name: (-sum(counts[name].values()), name))

    summary = []
    for platform in platforms:
        tally = counts[platform]
        other = sum(tally.values()) - tally.get("accepted", 0) - tally.get("rejected", 0)
        generated = sum(1 for synthetic in families[platform].values() if synthetic)
        summary.append(
            f"<tr><td>{esc(platform)}</td>"
            f"<td class='n'>{sum(tally.values())}</td>"
            f"<td class='n'>{tally.get('accepted', 0)}</td>"
            f"<td class='n'>{tally.get('rejected', 0)}</td>"
            f"<td class='n'>{other}</td>"
            f"<td class='n'>{len(families[platform]) - generated}</td>"
            f"<td class='n'>{generated}</td></tr>"
        )

    sections = []
    for platform in platforms:
        rows = kept_improvements(dataset, platform)
        if not rows:
            continue
        on_generated = sum(1 for record in rows if subjects[record["subject"]]["synthetic"])
        body = []
        for record in rows:
            metric = _primary(record)
            paired, absolute = metric["paired"], metric["absolute"]
            subject = subjects[record["subject"]]
            interval = (
                f'{paired["ci95_low_pct"]:+.1f} to {paired["ci95_high_pct"]:+.1f}'
                if paired["ci95_low_pct"] is not None
                else "—"
            )
            body.append(
                f"<tr><td class='n'>{esc(record['id'].removeprefix('exp-'))}</td>"
                f"<td>{esc(record['title'])}</td>"
                f"<td class='mono muted'>{esc(', '.join(record['hypotheses']) or '—')}</td>"
                f"<td>{esc(subject['labels'][0])} "
                f"<span class='muted'>&middot; {esc(_entries_short(subject['entries']))}"
                + (" &middot; generated" if subject["synthetic"] else "")
                + "</span></td>"
                f"<td class='mono muted'>{esc(record['primary_job'] or '')}</td>"
                f"<td class='n'>{esc(fmt_primary(absolute['control'], record['primary_metric']))}</td>"
                f"<td class='n'>{esc(fmt_primary(absolute['candidate'], record['primary_metric']))}</td>"
                f"<td class='n'>{esc(fmt_pct(paired['change_pct']))}</td>"
                f"<td class='n muted'>{esc(interval)}</td></tr>"
            )
        noun = "accepted run" if len(rows) == 1 else "accepted runs"
        sections.append(
            f"<h3>{esc(platform)}: {len(rows)} {noun} that improved</h3>"
            f"<p>Decided on a generated tree: {on_generated} of {len(rows)}.</p>"
            '<div class="scroll"><table>'
            '<thead><tr><th class="n">#</th><th>experiment</th><th>hypothesis</th>'
            '<th>subject</th><th>primary job</th><th class="n">before</th>'
            '<th class="n">after</th><th class="n">change</th>'
            '<th class="n">95% interval</th></tr></thead>'
            f"<tbody>{''.join(body)}</tbody></table></div>"
        )

    return f"""
<h2 id="platforms">By Platform</h2>
<h3>What each platform's own runs decided</h3>
<p>A result is evidence about the platform, host, and tree it was measured on. A change
kept on one platform's evidence is inherited, not proven, on the other, and most
hypotheses here were measured on one platform only. Subjects are counted as families, one
tree measured in several states, and the loop treats a generated tree as screening rather
than as a sample of ordinary work.</p>
<div class="scroll"><table>
<thead><tr><th>platform</th><th class="n">experiments</th><th class="n">accepted</th>
<th class="n">rejected</th><th class="n">other verdicts</th>
<th class="n">real subjects</th><th class="n">generated subjects</th></tr></thead>
<tbody>{"".join(summary)}</tbody></table></div>
<p>Each platform's table lists the accepted changes still in the product whose deciding
run measured an improvement on its primary metric, oldest first, with that run's two arms
on its own subject. A validation on a second subject or platform is its own row, so the
tables count runs, not changes, and total more than the kept changes in the header.
Rejected and noninferiority verdicts are in <a href="#every">the full table</a>.</p>
{"".join(sections)}
"""


def _mismatch(dataset: Mapping[str, Any]) -> tuple:
    """Experiments where acceptance and primary speed evidence differ.

    Both directions exist and each says something the other does not. A change can be
    real and still not be accepted, and an experiment can be accepted for a reason that
    is not a primary wall-time improvement.
    """
    records = [
        record
        for record in dataset["experiments"]
        if record["decision"] != "baseline" and _primary(record)
    ]
    improved = {
        record["id"] for record in records if _primary(record)["paired"]["evidence"] == "improved"
    }
    accepted = {record["id"] for record in records if record["decision"] == "accepted"}
    return sorted(improved - accepted), sorted(accepted - improved)


def _section_mechanisms(dataset: Mapping[str, Any]) -> str:
    """What the large wins did, read from total CPU rather than from intent.

    Overlapping work with other work and removing it both shorten wall time; only removing
    it makes the machine do less. Total CPU separates the two, so the mechanism is read
    from the evidence rather than asserted from what each change was trying to do.
    """
    rows = []
    for record in dataset["experiments"]:
        # Individual changes only, decided by two facts the artifacts carry.
        # `lines_changed == 0` marks an experiment that wrote no code and remeasured
        # merged work, such as a cumulative checkpoint or a platform validation. A long
        # hypothesis list marks a whole pull request measured at once, as exp-033 lists
        # five across 1,514 lines. Either would credit a single row with a dozen changes.
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
time, but only the second makes the machine do less, so wall time alone cannot tell them
apart; total CPU can. The table lists each accepted single change that cut wall time by at
least 5%.</p>
<div class="scroll"><table>
<thead><tr><th class="n">#</th><th>change</th><th>job</th><th class="n">wall</th>
<th class="n">total CPU</th><th>what it did</th><th class="n">lines</th></tr></thead>
<tbody>{body}</tbody></table></div>
<p>{removed} of these {len(rows)} also cut total CPU: they removed work. The rest spent
more CPU to finish sooner, a gain for a person waiting on a scan and a cost to a laptop
battery.</p>
<p class="note">The clearest pair: running the directory walk on four threads
(exp&#8209;001) cut wall time in half and raised system CPU <strong>83%</strong>, the same
syscalls issued concurrently. Replacing per-entry metadata calls with bulk calls per
directory (exp&#8209;022) cut wall time 30% and system CPU <strong>47%</strong>, with
fewer syscalls.</p>
"""


def _section_reading(dataset: Mapping[str, Any]) -> str:
    """The caveats a reader needs before drawing a conclusion from the figures.

    Each is a way to read an honest dataset into a wrong answer, and the first contradicts
    what a reader would assume from seeing the absolute and relative figures on one page,
    so the page states them rather than leaving them in the harness.
    """
    improved_only, accepted_only = _mismatch(dataset)
    mismatch = (
        f"{len(improved_only)} experiments measured an improvement and were not accepted, "
        f"for reasons such as falling below the threshold, being superseded, or being "
        f"unfinished. {len(accepted_only)} were accepted without a measured improvement on "
        f"their primary metric, on grounds such as noninferiority, instrumentation, or "
        f"correctness evidence."
    )
    return f"""
<h2 id="reading">Reading These Numbers</h2>
<h3>The two figures do not divide into each other</h3>
<p>Dividing a row's endpoints in the absolute figure does not give the relative figure's
percentage. The absolute values are each arm's median on its own; the relative value is
the median of the <em>paired</em> changes, the candidate against the control within each
round. When the host drifts during a run the two diverge, in this record by several
percentage points and sometimes in sign.</p>
<p class="note">exp&#8209;005's <span class="mono">cold-scan-index</span> reads
<strong>+2.8%</strong> from its medians and <strong>&minus;3.9%</strong> paired. The paired
figure controls for drift, so verdicts use it. The page publishes both, and neither is
derived from the other.</p>
<h3>Accepted and faster are different questions</h3>
<p>{mismatch}</p>
<p class="note">Instrumentation is the clearest case: exp&#8209;052 and exp&#8209;053 were
accepted on intervals of <span class="mono">[&minus;3.3%, +3.8%]</span> and
<span class="mono">[&minus;3.0%, +1.4%]</span>. Neither was claimed as a speed-up; each
made the engine observable at a cost the measurement could not detect.</p>
<h3>What is not measured here</h3>
<p>Every number comes from one Apple M1 Pro or a handful of virtualized 4-vCPU Linux
guests; nothing is from bare-metal Linux, and Windows is not benchmarked. The page cache
was warm throughout because dropping it needs root, so nothing here describes a cold disk.
Tuning constants were fitted on the subjects shown.</p>
"""


def _experiment_detail(record: Mapping[str, Any]) -> str:
    """Everything the artifacts say about one experiment, short of its body prose.

    A title and the verdict's one-line reason leave a row like "Borrowed path components"
    saying almost nothing about what was done, so every field below is shown; each is
    validated frontmatter the ledger also renders.

    Body prose stays out. The frontmatter is the data path by design, and a report that
    scraped Markdown sections would be reading something no contract validates.
    """
    complexity = record["complexity"] or {}
    rows = []

    if compares(record) and (record["control"] or record["candidate"]):
        rows.append(
            (
                "Compared",
                f'<span class="muted">{esc(record["control"] or "—")}</span>'
                f'<span class="arrow"> &rarr; </span>{esc(record["candidate"] or "—")}',
            )
        )
    if complexity.get("notes"):
        rows.append(("How", esc(complexity["notes"])))

    cost = []
    lines_changed = complexity.get("lines_changed")
    if lines_changed:
        cost.append(f"{lines_changed:,} lines changed")
    for dependency in complexity.get("new_dependencies") or []:
        cost.append(f"new dependency: {esc(dependency)}")
    if complexity.get("new_unsafe_blocks"):
        cost.append(f"{complexity['new_unsafe_blocks']} new unsafe block(s)")
    for mode in complexity.get("new_failure_modes") or []:
        cost.append(f"new failure mode: {esc(mode)}")
    if cost:
        rows.append(("Cost to carry", "<br>".join(cost)))

    if record["reason"]:
        rows.append(("Verdict", esc(record["reason"])))

    facts = [f'{record["trials"]} paired rounds']
    if record["interleaved"]:
        facts.append("interleaved")
    facts.append(f'{record["entries"]:,} entries, {esc(record["platform"])}')
    if record["hypotheses"]:
        facts.append("hypothesis " + ", ".join(esc(h) for h in record["hypotheses"]))
    if record["commit"]:
        facts.append(f'commit {esc(record["commit"])}')
    rows.append(("Run", " &middot; ".join(facts)))

    body = "".join(f"<dt>{label}</dt><dd>{value}</dd>" for label, value in rows)
    return (
        f"<details><summary>{esc(record['title'])}</summary>"
        f"<dl class='detail'>{body}</dl>"
        f'<p class="source">Full record: <span class="mono">{esc(record["path"])}</span></p>'
        "</details>"
    )


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
        change = fmt_pct(paired["change_pct"]) if paired and compares(record) else "—"
        before = (
            fmt_primary(absolute["control"], record["primary_metric"]) if absolute else "—"
        )
        after = (
            fmt_primary(absolute["candidate"], record["primary_metric"])
            if absolute and compares(record)
            else "—"
        )
        decision = record["decision"]
        rows.append(
            f"<tr>"
            f'<td class="n">{esc(record["id"].removeprefix("exp-"))}</td>'
            f"<td>{_experiment_detail(record)}</td>"
            f'<td class="mono muted">{esc(record["primary_job"] or "")}</td>'
            f'<td class="n">{esc(before)}</td>'
            f'<td class="n">{esc(after)}</td>'
            f'<td class="n">{esc(change)}</td>'
            f'<td class="n muted">{esc(interval)}</td>'
            f'<td class="v-{esc(decision)}">{esc(decision_label(record))}</td>'
            f"</tr>"
        )
    return f"""
<h2 id="every">Every Experiment</h2>
<p>Each row opens to show what was compared, how it was built, its cost to carry, and its
verdict. Before and after are that experiment's own two arms on its own subject, so they
are comparable across a row but not down a column. The change and interval are
paired.</p>
<div class="scroll"><table>
<thead><tr><th class="n">#</th><th>experiment</th><th>primary job</th>
<th class="n">before</th><th class="n">after</th><th class="n">change</th>
<th class="n">95% interval</th><th>verdict</th></tr></thead>
<tbody>{"".join(rows)}</tbody>
</table></div>
"""


def _footer(dataset: Mapping[str, Any]) -> str:
    totals = dataset["totals"]
    # The one date worth printing. Passed in through the dataset rather than read from the
    # clock here, so regenerating the page without changing an artifact does not change
    # the page.
    prepared = dataset.get("prepared") or "an unrecorded date"
    return f"""
<footer>
<p>Prepared {esc(prepared)} by <span class="mono">make perf-report</span> from
{totals['experiments']} validated experiment records in
<span class="mono">docs/project/experiments/</span>. Every number is read from those
records, never retyped, and <span class="mono">make perf-report-check</span> fails when the
page differs from what they produce, so regenerate it rather than editing it.</p>
<p>The table leaves out experiment dates, which record when the work happened rather than
anything about the evidence. What a result depends on is its subject, machine, and cache
state, and each record states those.</p>
</footer>
"""


def _headline_figure(series: Optional[Mapping[str, Any]], job_id: str, label: str) -> str:
    """One before-and-after headline, or nothing if that job was not measured.

    A missing job yields no headline rather than an error, so the page never depends on a
    particular subject having run a particular job.
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


def _standing_figure(dataset: Mapping[str, Any]) -> str:
    """The release standing's two arms, or nothing if that record is absent."""
    record = _record(dataset, STANDING_EXPERIMENT)
    arms = _wall_arms(record) if record else None
    if not arms:
        return ""
    control, candidate = arms
    return (
        f'<div><span class="n">{fmt_ms(control)} <span class="muted">&rarr;</span> '
        f'<span class="good">{fmt_ms(candidate)}</span></span>'
        f'<span class="k">{esc(STANDING_LABEL)}</span></div>'
    )


def _score_figure(dataset: Mapping[str, Any]) -> str:
    """Each platform's score from its first build to its last, with its interval.

    The interval is the combined interval of the two index values, both relative to the
    reference build, so it is the width of the ratio between them. Each figure
    names its platform and coverage (a platform's score is never the full index), says
    how the cells behind it were measured, and names any platform not yet measured.
    """
    figures = []
    recorded = {record["id"] for record in dataset["experiments"]}
    for projected in dataset.get("index") or []:
        regime = regime_note(projected, link=True)
        gate = gate_note(projected)
        context = "; ".join(
            part
            for part in (
                f"{regime} ({gate})" if regime and gate else regime,
                "; ".join(
                    f"{esc(name)} not yet measured ({weight * 100:.0f}% of the full index)"
                    for name, weight in sorted((projected.get("unmeasured_platforms") or {}).items())
                ),
            )
            if part
        )
        for key, components, partial in (
            ("measured_full", projected["measured"], False),
            ("common", projected["common"], True),
        ):
            if partial and len(projected["common"]) == len(projected["measured"]):
                continue
            # Only builds the chart can place, so the headline and the chart agree.
            builds = [
                build
                for build in projected["builds"]
                if build.get(key) and build.get("after_experiment") in recorded
            ]
            if len(builds) < 2:
                continue
            first, last = builds[0][key], builds[-1][key]
            speedup, low, high = score_ratio(first, last)
            better = speedup >= 1
            shown = speedup if better else 1 / speedup
            bounds = (low, high) if better else (1 / high, 1 / low)
            label = score_label(projected, len(components))
            if partial:
                label = f"partial: {label} ({projected['common_weight'] * 100:.0f}% of the weight)"
            figures.append(
                f'<div><span class="n {"good" if better else "bad"}">{shown:.2f}&times; '
                f'{"better" if better else "worse"}</span>'
                f'<span class="k">{esc(label)}, '
                f'{esc(builds[0].get("short") or builds[0]["label"])} to '
                f'{esc(builds[-1].get("short") or builds[-1]["label"])} '
                f"[{bounds[0]:.2f}&times;, {bounds[1]:.2f}&times;]"
                + (f"; {context}" if context else "")
                + "</span></div>"
            )
    return "".join(figures)


def _history_figure(dataset: Mapping[str, Any]) -> str:
    """The unified score, then any history cell's own first-to-last speedup.

    A cell that is one job of an index component speaks through the score and its
    component's line, so only a cell outside the index gets a headline of its own: one
    tree timed once per component job would otherwise repeat its title for every cell.
    The ratio is of the medians the chart draws, so the headline and the chart agree;
    the paired figure stays in the record.
    """
    figures = [_score_figure(dataset)]
    # The same builds the chart can place, so the headline never states a ratio between
    # builds the chart does not draw.
    recorded = {record["id"] for record in dataset["experiments"]}
    for cell in dataset.get("history") or []:
        if cell.get("component"):
            continue
        milestones = [
            item
            for item in cell.get("milestones", [])
            if item.get("wall_ms") and item.get("after_experiment") in recorded
        ]
        if len(milestones) < 2:
            continue
        first, last = milestones[0], milestones[-1]
        figures.append(
            f'<div><span class="n good">{first["wall_ms"] / last["wall_ms"]:.1f}&times; '
            f'faster</span><span class="k">{esc(cell.get("title") or cell["subject"])}, '
            f'first build to {esc(last.get("short") or last["label"])}</span></div>'
        )
    return "".join(figures)


def _header(dataset: Mapping[str, Any]) -> str:
    totals = dataset["totals"]
    decisions = totals["decisions"]
    kept = sum(iteration_kind(record) == "kept" for record in dataset["experiments"])
    headlines = _history_figure(dataset) + _standing_figure(dataset) + _headline_figure(
        _flagship(dataset), "cold-scan-index", "macOS cold scan, campaign 1"
    )
    return f"""
<h1>Making fdu Faster, One Measured Experiment at a Time</h1>
<p class="lede"><a href="https://github.com/jlevy/fdu">fdu</a> is a disk-usage and file
roll-up tool written in Rust. Its performance work is a loop of experiments, each testing
one change against the code it came from, and a change is kept only if it passes
<a href="#loop">the accept rule</a>. The record holds {totals['experiments']} experiments,
including {decisions.get('rejected', 0)} rejected changes.</p>
<div class="headline">
  {headlines}
  <div><span class="n tnum">{totals['experiments']}</span>
    <span class="k">experiments</span></div>
  <div><span class="n tnum good">{kept}</span>
    <span class="k">changes kept, at least 3% better</span></div>
  <div><span class="n tnum">{decisions.get('rejected', 0)}</span>
    <span class="k">tried, not kept</span></div>
</div>
<p class="muted">How the loop was run, phase by phase, is in
<a href="../report-2026-08-14-performance-campaign-status.md">the loop history</a>; fdu
against other tools is in the comparisons
<a href="../report-2026-09-26-fdu-live-tool-comparison.md">on macOS</a> and
<a href="../report-2026-09-27-fdu-linux-tool-comparison.md">on Linux</a>.</p>
"""


def _section_iterations(dataset: Mapping[str, Any]) -> str:
    return f"""
<h2 id="iterations">Over Time</h2>
<h3>The performance score by build, and every experiment</h3>
<p>The top panel is fdu&rsquo;s performance score
(<a href="../../specs/active/plan-2026-10-05-fdu-performance-index.md">the index
spec</a>): a weighted combination of benchmark components, including a run with
fdu&rsquo;s caches empty, warm content analysis, an opened root, a million-entry tree, and
peak memory. Every milestone build was timed against the reference build, 0.3.0, in one
interleaved session per benchmark job, so each step is measured directly rather than
compounded from the experiment effects below. Those sessions ran on a desktop in use, so
differences of about 10% between adjacent builds are within the noise.</p>
<p>The solid line is the platform&rsquo;s score over every component, from the first build
that has them all; the dashed line is the partial score, over only the components every
build has. A platform&rsquo;s score is not the full index, which weights every platform.
The chooser shows any single component. The bottom panel is every
experiment in the order it ran, each bar its paired change on its own primary job.</p>
{figure_timeline(dataset)}
"""


def _section_loop(dataset: Mapping[str, Any]) -> str:
    return """
<h2 id="loop">The Loop</h2>
<p>An experiment names a hypothesis and a predicted effect, builds exactly one change,
checks that every answer is unchanged, and times the change (the candidate) against the
code it came from (the control): its two <strong>arms</strong>. Both binaries alternate in
one interleaved session, a <strong>cell</strong>, on a tree pinned by content digest, for at
least 12 <strong>rounds</strong>, or 20 when the predicted effect is small; a round is one
paired trial of each arm (the harness&rsquo;s <span class="mono">--trials</span>). The
<strong>paired change</strong> is the median over the rounds of the candidate&rsquo;s change
from the control in the same round, so drift in the host&rsquo;s speed reaches both arms
alike; its 95% interval is a bootstrap of that median. A change is kept when it is at
least 3% faster with its 95% interval below zero. Some are accepted instead on
<strong>noninferiority</strong>, when the interval rules out a slowdown beyond a stated
margin.</p>
<p>The harness reads host CPU load before and after every sample. On a quiet host a sample
with a reading above 25% busy, the <strong>quiet gate</strong>, is discarded; an
uncontrolled host keeps every sample and records the load. A score is
<strong>exploratory</strong>, for discovery rather than for quoting, when any cell behind
it ran on an uncontrolled host, ran fewer than 20 rounds, or belongs to an exploratory
stage (<a href="../../guides/performance-loop.md#host-pressure-regimes">host-pressure
regimes</a>).</p>
"""


def _section_absolute(dataset: Mapping[str, Any]) -> str:
    end_to_end = figure_end_to_end(dataset, "Linux")
    linux = ""
    if end_to_end:
        linux = f"""
<h4>Linux, end to end</h4>
<p>Later Linux work was measured the same way at its milestones: an older engine against a
newer one in a single interleaved cell. The 0.3.0 release cell is the comparison to
quote.</p>
{end_to_end}
"""
    return f"""
<h2 id="absolute">Absolute</h2>
<h3>Wall time, in milliseconds</h3>
<p>A percentage cannot say whether a scan takes half a second or half a minute. These are
the measured medians.</p>
<h4>macOS, campaign 1</h4>
{figure_absolute(dataset)}
{linux}
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
    write_text_atomic(arguments.out, page, encoding="utf-8")
    print(f"wrote {arguments.out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
