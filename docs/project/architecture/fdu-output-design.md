# fdu Output Design

## Purpose

A reader should immediately distinguish measured results, missing information, actions,
and execution cost.
Keep the output concise without removing the facts needed to debug an
unexpected result. This contract applies to the CLI, installed wheel, Python report
rendering, and watch reports.

The implementation rules are documented beside
[the shared renderer](../../../../crates/fdu-core/src/report_format.rs),
[the diagnostic collector](../../../../crates/fdu-core/src/report_format/report_epilogue.rs),
and `write_report_diagnostics` in [the CLI](../../../../crates/fdu/src/cli.rs).
Keep those comments and this guide consistent.

## Row Styling

Names that use cyan are bright cyan and bold; names shown in white or gray retain those
colors.
Directory names have a gray trailing `/`, except `.` and `..`; regular file names
do not. Path and structured formats keep their original path values.
Sizes of at least 1 GiB are bold, including gray parenthetical sizes and performance
details. The threshold uses exact bytes, before rounding.
Zero sizes such as `0 B` are gray.
Other smaller sizes and file counts use the ordinary foreground color.
Percentages below 1% are gray, using the exact ratio before rounding.
These styles apply wherever human output presents those values.
Tree rows put the share bar first, then the percentage of the selected root, size, and
indented filename.
Keep these columns aligned across directory, file, and remainder rows.
Write `attic/ 3,508 files (43 MiB gitignored)`: file counts are outside parentheses,
while embedded gitignored amounts are parenthesized and gray.
The gitignored amount is a subset already included in the row total, not additional
usage.
For example, `224 MiB ... (73 MiB gitignored)` means 224 MiB overall, including 73
MiB classified by `.gitignore` rules.
Secondary breakdowns follow the same convention, such as
`477,298 lines (439,949 nonblank, 37,349 blank)` with the parenthetical detail gray.
Apply these roles consistently across human report views.

## Number Formatting

Human integer counts use comma grouping consistently: `13,580 files`, `1,234 rules`, and
`205,709 files/s`. Reports, notes, progress, and performance use the shared
`human_count` formatter; its `human_count_u128` implementation also handles wide rate
calculations without losing precision.
This is the single policy point for future localization or ungrouped display.
Do not add grouping at individual call sites.
Structured formats retain numeric values, and executable flag values retain their parser
syntax. Durations, percentages, and scaled byte units keep their own precision rules.

## Streams and Categories

Formatted results belong on stdout.
Human diagnostics belong on stderr, including when stdout is redirected or contains
JSON, JSONL, YAML, paths, or long rows.
A library renderer returns only the formatted result; notes and tips are exposed
separately for the caller to route.

Use short lines with these prefixes, without category headings or empty categories:

| Category | Meaning | Terminal style |
| --- | --- | --- |
| `note:` | Coverage, interpretation, and limitations of the result | Gray, not bold |
| `warn:` | An operation failed or could not complete; useful results may remain | Yellow, not bold |
| `tip:` | A specific action or option that changes the result or reveals more detail | Gray, not bold |
| `perf:` | Observed execution work and elapsed time | Gray, not bold |
| Error | A fatal or usage error, using the command’s established error rendering | Red and bold |

For a successful or partial report, emit notes, warnings, tips, then performance.
Performance closes a human text/tree report; machine and path formats do not add an
unsolicited performance footer.
Watch repaints have no final one-shot performance total.
Preserve failure exit status and diagnostic details when output fails or a background
cache save fails.

Add brief interpretation notes by default when the displayed facts can be misread:
gitignored values are included in totals, and hidden file tallies are recursive subsets
already included in directory totals.
Emit each clarification once per report.
`--quiet` (`-q`) suppresses notes, tips, performance lines, and transient progress.
It preserves result stdout, warnings, errors, completeness facts, and exit status.
Machine reports keep their structured facts; quiet controls diagnostic presentation.

Apply color using stderr’s terminal state for diagnostics and stdout’s terminal state
for results.
Honor the existing color option and `NO_COLOR`. Redirected streams are plain
by default; explicit forced color remains supported.
Machine data never contains ANSI escapes.

## Omitted Rows

Colored tree bars use green solid blocks (`█`) for non-gitignored usage, green
dark-shade blocks (`▓`) for gitignored usage, and faint gray light-shade blocks (`░`)
for the unused width.
Bars default to ten cells; `--bar-size` sets the width.
Zero or negative values hide the bar and its following gutter.
The filled width is rounded against the selected root, then its cells are divided by the
row’s gitignored proportion.
This keeps a predominantly gitignored one-cell bar shaded instead of losing that
population to independent rounding.
Whole cells are an approximation: numeric amounts are exact at their displayed
precision; increase `--bar-size` for finer resolution.
Widths above 4,096 are rejected before allocating the decorative bar.
Rust `RenderOptions.bar_size` and Python `Report.render(bar_size=...)` expose the same
rendering capability.
Machine formats ignore bar width.
Unknown ignore classification uses green `▒` blocks for unclassified usage rather than
claiming either population.
Uncolored bars retain their plain block glyphs.

Each tree has at most one remainder line below its selected root.
Its annotation and values are gray, with the shared size emphasis and bar population
colors. It uses the same bar, percentage, and size columns as tree rows; its name column
reads `… and N more files`. The bar and percentage show the combined hidden share of the
selected root.

```text
█░░░░░░░░░    12%     1.2 MiB  … and 12,345 more files
```

The size and recursive file count cover unlisted immediate branches of the selected
root.
A listed directory represents its entire recursive total, even when descendants are
not expanded; those descendants do not contribute again to the remainder.
For a complete tree, listed root-child totals plus the remainder equal the root total.
The root row provides context and is not subtracted from itself.
When no root children are listed, the remainder can equal the entire root.
Nested omission boundaries still explain display limits without inflating this summary.
Count regular files inside hidden directories, not just their directory roots.
A machine omission boundary’s `entries` field counts direct hidden roots (files or
directories); it is distinct from `files`. Unknown measurements remain `null` in
structured output. Text uses `unknown` in the size column and
`… and more files (count unknown)` for an unknown count.
An unknown hidden size has a blank bar and `—` percentage.
As with ordinary rows, percentages use the observed root total; partial scan diagnostics
remain essential when coverage is incomplete.

The core `TreeRemainder` model supplies every format.
JSON, JSONL, and YAML tree sections expose `remainder` with `files`, `bytes`,
`allocated`, and the applicable `reasons` in stable order.
`remainder: null` means all root branches are represented; descendants may still be
collapsed under listed directories.
The existing `limits` fields provide the corresponding bound values; per-node
`omissions` retain detailed boundaries for debugging.
Report schema `fdu.report/10` uses this root-branch accounting for the remainder.

Explain accounting and applicable bounds once at the end, then offer remedies:

```text
note: more covers unlisted root branches; listed directory totals already include their descendants
note: display limits: below 1% of selected root, depth 5
tip: show smaller entries: --min-share=0%
tip: expand deeper: --depth=all
```

The share threshold applies to individual entries against the selected root, not to the
combined hidden amount.
Collect remedies from actual omissions, deduplicate across views, and never sum
remainders across views whose contents may overlap.

A complete recursive export is a normal use case:

```shell
fdu . --view tree --format json --full
```

`--full` expands to `--depth=all --breadth=all --limit=all --min-share=0%`. Explicit
bounds override the shorthand regardless of flag order.
It does not change selection, population, analysis, or scan scope.

On a fully readable tree with unrestricted discovery, this shows every directory and
regular file, keeps the same aggregate measurements, returns `remainder: null`, and
emits no omission notes or tips.
Display bounds do not control scan completeness: `--scan-depth`, unreadable directories,
and other discovery restrictions still apply.

## Performance Summary

Start with elapsed wall-clock time, followed by work and throughput:

```text
perf: took 66.0 ms to walk 13,580 files (225 MiB) at 205,709 files/s (3.583 GB/s); 675 gitignore rules (44 files); content read 0 B; analysis 0 fresh, 0 cached; cold scan
```

Use parentheses for associated quantities, such as bytes after a file count and ignore
files after their rule count.
Slashes belong only in rate units.
Total rates use the displayed wall-clock interval; GB/s describes represented file size,
not measured storage bandwidth.
Content-read rates describe bytes actually read.
When ignore rules were not read, say `gitignore not read` rather than presenting zero as
an observed count.

## Useful Diagnostics

Keep each diagnostic self-contained: identify what happened, where it happened, and the
relevant cause or limit.
Put a path and the operating-system error on the same warning line where available.
Keep facts separate from suggestions.

```text
note: ignore classification incomplete: 1 ignore file not applied (1 with a line over the 16 KiB line limit); affected: vendor
warn: vendor/private: Permission denied
tip: apply refused ignore files: raise --gitignore-line-limit above 16 KiB, or set it to all
```

Bound path lists and retained error details, stating how many were omitted.
When a structured format contains additional retained detail, suggest the exact existing
format option. Do not invent a verbose flag, promise unretained detail, or imply that
rerunning will necessarily resolve a permission or filesystem failure.

A refused ignore file makes classification incomplete; it does not by itself make size
measurements inaccurate.
Preserve that distinction in the note and structured result.
A display limit hides rows but does not reduce totals or filesystem work.

## Ownership and Enforcement

The engine owns facts, display omissions, and actionable suggestions.
Diagnostic categories are explicit values; frontends do not recover them by parsing
rendered text. Surface vocabulary supplies option names, so Python suggestions name
Python arguments. The CLI adds run telemetry and operational warnings, then routes each
category to stderr.

Use one shared golden corpus for CLI and installed Python results.
Expected stdout and stderr are recorded separately.
Preserve portable timestamp, allocation, and rate patterns; inspect golden changes
rather than accepting host-specific recordings.

Focused tests enforce tree-column alignment, exact omitted sizes, missing-size wording,
one tip per applicable bound, category ordering, terminal color roles, stdout/stderr
separation, clean machine parsing, and unchanged partial-result exit status.
Include multiple directories and multiple views so deduplication is exercised.

Validation belongs to the existing golden, parity, terminal, and full `make check`
gates.
A formatting change is not allowed to weaken the assertions about measured results
or to classify unexplained Python/CLI differences as harmless telemetry.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
