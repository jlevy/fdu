---
name: fdu
description: >-
  Inspect directory trees with hierarchical file counts, apparent and allocated sizes,
  recency, and extension tallies. Use when investigating disk usage, finding large
  directories, summarizing file types, listing files by size or age, or collecting stable
  JSON filesystem roll-ups for scripts and coding agents.
---
# fdu Directory Roll-Ups

Use `fdu` to summarize a directory tree without modifying files in that tree.
Every report requires an explicit `PATH`; bare `fdu` prints help instead of scanning the
current directory.

## Run fdu

Start with the report that answers the question:

```bash
fdu --view languages PATH                 # detected language sizes; metadata only
fdu --analyze code --view languages PATH  # add standard LOC; reads content
fdu --view types PATH                     # all file types; metadata only
fdu PATH                                  # folder-size tree; metadata only
fdu --cache off --view summary PATH       # one totals row; no retained index
```

`--analyze` chooses what may be read and `--view` chooses what is printed.
The two language commands differ only on the analysis axis: the first uses byte shares
without content reads, while the second adds code, comment, and blank-line metrics.
Use `--size apparent` when logical file lengths are wanted instead of allocated bytes.

For a bounded machine-readable tree, use:

```bash
fdu --format json --view tree --depth 2 --limit 20 PATH
```

If no local command exists and this release is published on PyPI, use the exact reviewed
version. Never use an unversioned `uvx` runner or `latest` in agent instructions:

```bash
uvx --from fdu==__FDU_VERSION__ fdu --format json --view tree PATH
```

## Compose the Request From Five Axes

Every option belongs to exactly one axis, and any axis composes with any other.
There are no subcommands: the grammar is always “report on a path”.

| Axis | Question | Options |
| --- | --- | --- |
| Scope | What is scanned and cached? | `PATH`, `--scan-depth N` |
| Selection | Which entries does this query consider? | `--include`, `--exclude`, `--min-size`, `--modified-since`, `--modified-before`, `--kind`, `--depth`, `-n/--limit`, `--sort`, `--reverse`, `--size` |
| View | Which roll-up is reported? | `--view tree,extensions,types,families,languages,documents,files,summary` |
| Format | How is it serialized? | `--format text\|json\|jsonl\|yaml`, `--color` |
| Mode | How is work performed? | `--cache auto\|refresh\|read-only\|only\|off`, `--analyze none\|basic\|code\|documents\|full` |

Scope versus selection is the distinction that matters: scope decides what is scanned
and cached, so one cache serves every query, while selection filters the retained index
at query time. Narrowing a selection never costs a rescan.

Cost has three layers.
`--cache off --view summary PATH` is the one exact composition that retains only
aggregate tallies and no index.
Ordinary metadata requests retain the reusable index but never read regular-file
contents. Any non-`none` `--analyze` profile opts into streaming reads through every
eligible file and a separate profile-scoped sidecar.
A repeated run with the same profile and semantic settings reuses unchanged content
records. Coverage is profile-scoped too: an unsupported deeper analyzer leaves byte
metadata visible but does not retain a separate lower-level metric record for that file.

## Pick the View, Then Shape It

- `--view tree` (default) for per-directory roll-ups.
- `--view extensions` for the original raw-extension breakdown.
- `--view types` for stable detected file types and exact byte shares.
- `--view families` for code, prose, markup, data, binary, and unknown roll-ups.
- `--view languages` for code-family rows and byte shares from path-only detection.
- `--view documents` for prose metrics; it requires any enabled analysis profile.
- `--view files` for a flat listing.
  One-shot text adds the performance footer described below; use a machine format when
  output is consumed programmatically.
- `--view summary` for one aggregate row.
- Several views in one run share one scan: `--view summary,types,families`.

Add `--analyze basic` to stream physical, blank, and nonblank lines and raw prose words.
Add `--analyze code` to the language view for standard LOC, comment, and code-blank
partitions across supported common languages; the percentage column then uses code lines
instead of bytes. Use `--analyze documents --view documents` for normalized prose words,
paragraphs, aggregate-derived pages, and reader-visible Markdown that excludes
destinations and code.
`--analyze full` computes both families in one streaming pass.
Use `--analysis-workers` to bound concurrent reads and `--words-per-page` to control
page derivation. Analysis never truncates a file or excludes it because of size.
Invalid UTF-8, binary data, and unsupported SLOC languages remain visible as normal
coverage outcomes. Only I/O failures, files changed during a read, or stale commits make
analysis operationally partial.
Content analysis is currently one-shot and cannot be combined with `--watch`.

One-shot text reports end with a compact performance line.
It reports regular files and apparent bytes walked, content bytes actually read,
fresh-analysis file and byte rates, content-sidecar files and apparent bytes restored
from cache, the metadata cache tier, and total report time.
Known binary files can contribute walked bytes but zero read bytes.
Cache-only runs report zero walked files because they never consult the tree.
The line is gray only when color is active and has no ANSI escapes otherwise.
JSON, JSONL, YAML, skill output, lifecycle output, and watch streams omit it.

Common shapes are compositions rather than dedicated flags:

```bash
fdu --view files --sort size --limit 20 PATH          # largest files
fdu --view files --modified-since 2h PATH             # changed in the last two hours
fdu --view files --include '*.{rs,toml}' PATH         # by pattern
fdu --view tree --sort mtime PATH                     # an activity map
```

`--depth` and `--limit` bound only the rendered view; `--scan-depth` bounds what is
scanned and retained, so do not reach for it merely to shorten output.

## Value Grammars

- Sizes: `512`, `10k`, `10M`, `1.5GiB`. Decimal and binary units, case-insensitive.
- Times: `now`, a compound age (`45s`, `2h`, `1h30m`), an RFC 3339 timestamp with an
  offset (`2026-08-10T18:22:31Z`), or `@` epoch seconds.
  Calendar units and fractional ages are rejected with the spelling to use instead; a
  bare local date-time is rejected because resolving it needs a time-zone database.
- `--modified-since` is inclusive and `--modified-before` is exclusive.

## Use Timestamps as a Sync Watermark

Every report carries `scan_started_at`. Feeding it back selects exactly what changed
after that scan began, which is what makes incremental follow-up sound:

```bash
fdu --view summary --format json PATH                       # record scan_started_at
fdu --view files --format jsonl --modified-since <that> PATH
```

Use the scan’s *start*, not its end: a file modified mid-scan may have been observed
before the modification, so only the start bound is conservative.

## Validate Every Automated Result

Check the process exit status and these fields:

- `schema` before parsing anything else
- `complete` and `errors` before trusting totals
- `freshness` and `source` before presenting data as current
- `truncated` on a tree node before treating it as exhaustive
- `coverage` before presenting a metric summary as complete
- `detection.sources`, `detection.confidence`, and `detection.flags` before treating a
  deep-detected type or origin label as exact

`source` is `cold_scan`, `warm_revalidate`, or `cache_only`. Only `--cache only` can
return `freshness: stale`, and it says so rather than implying currency; it fails
outright when no usable snapshot exists rather than silently scanning.

Exit 0 is accepted success, exit 1 is a fatal failure, and exit 2 is incomplete data or
invalid usage. Do not discard useful stdout from exit 2; inspect the completeness fields
and use `--allow-partial` only when incomplete totals are acceptable.

## Cache Behavior

The snapshot is one file per root under the user cache directory.
`--cache-status` maps a hash-named file back to the tree it describes, and
`--cache-clear` removes it; both run without scanning and never touch files this build
cannot identify.

Verification cost follows the question asked.
Sizes and timestamps need one stat per entry, because an in-place edit changes a file
without changing any directory.
Questions answerable from names alone need only one stat per directory.
Adding metrics within a tier is free; crossing a tier boundary is what costs.

Exact names and ordinary extensions remain path-only classifications.
When analysis is enabled, unresolved files and ambiguous `.h` headers may use bounded
shebang, modeline, literal, or signature probes.
Do not collapse their provenance into an unqualified language claim; retain the report’s
source and confidence fields when summarizing or transforming machine output.

Run `fdu --help` for the complete flag, cache, color, scope, and exit contract.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
