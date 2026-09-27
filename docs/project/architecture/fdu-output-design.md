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

Names use cyan; totals and file counts use the ordinary foreground color.
Write `attic 3508 files (43 MiB gitignored)`: file counts are outside parentheses, while
embedded gitignored amounts are parenthesized and gray.
The gitignored amount is a subset already included in the row total, not additional
usage.
For example, `224 MiB ... (73 MiB gitignored)` means 224 MiB overall, including 73
MiB classified by `.gitignore` rules.
Secondary breakdowns follow the same convention, such as
`477298 lines (439949 nonblank, 37349 blank)` with the parenthetical detail gray.
Apply these roles consistently across human report views.

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

Apply color using stderr’s terminal state for diagnostics and stdout’s terminal state
for results.
Honor the existing color option and `NO_COLOR`. Redirected streams are plain
by default; explicit forced color remains supported.
Machine data never contains ANSI escapes.

## Omitted Rows

Each tree has at most one gray remainder line, aligned below its selected root in the
filename column. Leave size, bar, and percentage columns blank.

```text
                                 … and 1.2 MiB (12,345 files) more
```

The size and recursive file count cover disjoint hidden subtrees across the entire tree.
They are already included in directory totals.
Count regular files inside hidden directories, not just their directory roots.
A machine omission boundary’s `entries` field counts direct hidden roots (files or
directories); it is distinct from `files`. Unknown measurements remain `null` in
structured output and read `unknown size` or `unknown file count` in text.
Never infer exact totals from incomplete coverage.

The core `TreeRemainder` model supplies every format.
JSON, JSONL, and YAML tree sections expose `remainder` with `files`, `bytes`,
`allocated`, and the applicable `reasons` in stable order.
`remainder: null` means nothing was hidden.
The existing `limits` fields provide the corresponding bound values; per-node
`omissions` retain detailed boundaries for debugging.
Report schema `fdu.report/9` includes these fields.

Explain accounting and applicable bounds once at the end, then offer remedies:

```text
note: more includes hidden subtrees already counted in directory totals; files are counted recursively
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

Focused tests enforce filename-column alignment, exact omitted sizes, missing-size
wording, one tip per applicable bound, category ordering, terminal color roles,
stdout/stderr separation, clean machine parsing, and unchanged partial-result exit
status. Include multiple directories and multiple views so deduplication is exercised.

Validation belongs to the existing golden, parity, terminal, and full `make check`
gates.
A formatting change is not allowed to weaken the assertions about measured results
or to classify unexplained Python/CLI differences as harmless telemetry.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
