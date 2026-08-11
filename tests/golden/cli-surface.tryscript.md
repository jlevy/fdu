---
sandbox: true
path:
  - $TRYSCRIPT_GIT_ROOT/target/debug
env:
  FORCE_COLOR: "0"
  LANG: C
  LC_ALL: C
  NO_COLOR: "1"
  TZ: UTC
patterns:
  OS_ERROR: '[^\r\n]+'
  SCAN_PATH: '[^\r\n]+'
---
# CLI Surface

## Help Is the Complete Invocation Contract

```console
$ fdu --help
A fast, incremental file roll-up engine: hierarchical tallies over large directory trees

Usage: fdu [OPTIONS] [PATH]

Arguments:
  [PATH]
          Directory to summarize
          
          [default: .]

Options:
      --scan-depth <N>
          Limit scanning and retention to N entry levels

      --one-filesystem
          Stay on the filesystem the root lives on

      --include <GLOB>
          Report only entries matching this glob; repeatable

      --exclude <GLOB>
          Exclude entries matching this glob; repeatable, and wins over --include

      --min-size <SIZE>
          Report only entries at least this large, as 512, 10M, or 1.5GiB

      --modified-since <WHEN>
          Report only entries modified at or after this time, as 2h or an RFC 3339 stamp

      --modified-before <WHEN>
          Report only entries modified before this time

      --kind <LIST>
          Entry kinds to report: file, dir, symlink, other

  -d, --depth <N>
          Directory levels to show; does not limit scanning. Accepts `all`
          
          [default: 2]

  -n, --limit <N>
          Entries to show per directory. Accepts `all`
          
          [default: 10]

      --sort <KEY>
          Order results: size, count, mtime, or name

      --reverse
          Reverse the ordering

      --size <METRIC>
          Which size metric to report: allocated or apparent
          
          [default: allocated]

      --view <LIST>
          Views to report: tree, types, files, summary
          
          [default: tree]

      --format <FORMAT>
          Output format: text, json, jsonl, or yaml
          
          [default: text]

      --color <WHEN>
          Colorize human output: auto, always, or never
          
          [default: auto]

      --cache <POLICY>
          Cache policy: auto, refresh, read-only, only, or off
          
          [default: auto]

      --allow-partial
          Accept incomplete totals when paths cannot be read

      --cache-status[=<SCOPE>]
          Report cache contents instead of scanning: root (default) or all

      --cache-clear[=<SCOPE>]
          Remove cached snapshots instead of scanning: root (default) or all

      --watch
          Stream changes continuously instead of returning one report

      --interval <DUR>
          How often aggregate views re-render while watching, as a duration.
          
          Throttles rendering only; change detection is event-driven and unaffected.
          
          [default: 2s]

      --skill
          Print a portable agent skill to stdout

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  fdu
  fdu --view types ~/Downloads
  fdu --view files --sort size --limit 20 ~/src
  fdu --view files --modified-since 2h --format jsonl .
  fdu --view summary,types --format json .

Five axes, and every option belongs to exactly one:
  Scope      PATH, --scan-depth        what is scanned and cached
  Selection  --include, --exclude, --min-size, --modified-since, --modified-before,
             --kind, --depth, --limit, --sort, --reverse, --size
  View       --view tree,types,files,summary
  Format     --format text|json|jsonl|yaml, --color
  Mode       --cache auto|refresh|read-only|only|off

Scope versus selection:
  --scan-depth limits what is scanned and retained; one cache then serves every query.
  --depth and --limit bound only the rendered view, and never cost a rescan.
  --depth 0 reports totals for the root and nothing beneath it.
  --depth and --limit accept `all` for no bound.

Values:
  SIZE   512, 10k, 10M, 1.5GiB (decimal and binary units, case-insensitive)
  WHEN   now, an age (45s, 2h, 1h30m), RFC 3339 with an offset, or @epoch seconds
  --modified-since is inclusive; --modified-before is exclusive
  --include and --exclude are repeatable globs; --view and --kind are comma lists

Cache:
  auto       read, revalidate, and write back when complete (default)
  refresh    ignore any snapshot, scan cold, and rewrite it
  read-only  read and revalidate, but never write
  only       answer from the snapshot without touching the tree; labeled stale,
             and fails when no usable snapshot exists rather than scanning
  off        ignore the snapshot and leave nothing behind

Output and automation:
  Results go to stdout; warnings and errors go to stderr.
  Machine formats are schema-versioned and never colorized.
  Every report carries schema, source, freshness, complete, errors, and both timestamps.
  Feed a report's scan_started_at back as --modified-since to list what changed since.
  The command never prompts, pages, or animates progress.

Color:
  --color overrides NO_COLOR and FORCE_COLOR. In auto mode, NO_COLOR disables color,
  FORCE_COLOR enables it, and otherwise the destination must be a terminal.

Exit status:
  0  Complete result, or a partial result accepted with --allow-partial
  1  Fatal filesystem or cache error
  2  Partial result, or command-line usage error
? 0
```

## The Portable Skill Is Complete and Version-Pinned

````console
$ fdu --skill
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

## Run fdu

Use the local command when it is available:

```bash
fdu --format json --view tree --depth 2 --limit 20 PATH
```

If no local command exists and this release is published on PyPI, use the exact reviewed
version. Never use an unversioned `uvx` runner or `latest` in agent instructions:

```bash
uvx --from fdu==0.0.1 fdu --format json --view tree PATH
```

## Compose the Request From Five Axes

Every option belongs to exactly one axis, and any axis composes with any other. There
are no subcommands: the grammar is always "report on a path".

| Axis | Question | Options |
| --- | --- | --- |
| Scope | What is scanned and cached? | `PATH`, `--scan-depth N` |
| Selection | Which entries does this query consider? | `--include`, `--exclude`, `--min-size`, `--modified-since`, `--modified-before`, `--kind`, `--depth`, `-n/--limit`, `--sort`, `--reverse`, `--size` |
| View | Which roll-up is reported? | `--view tree,types,files,summary` |
| Format | How is it serialized? | `--format text\|json\|jsonl\|yaml`, `--color` |
| Mode | How is the cache used? | `--cache auto\|refresh\|read-only\|only\|off` |

Scope versus selection is the distinction that matters: scope decides what is scanned
and cached, so one cache serves every query, while selection filters the retained index
at query time. Narrowing a selection never costs a rescan.

## Pick the View, Then Shape It

- `--view tree` (default) for per-directory roll-ups.
- `--view types` for an extension breakdown.
- `--view files` for a flat listing; in text output it prints one path per line and
  nothing else, so it pipes directly into other commands.
- `--view summary` for one aggregate row.
- Several views in one run share one scan: `--view summary,types`.

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
  offset (`2026-08-10T18:22:31Z`), or `@` epoch seconds. Calendar units and fractional
  ages are rejected with the spelling to use instead; a bare local date-time is rejected
  because resolving it needs a time-zone database.
- `--modified-since` is inclusive and `--modified-before` is exclusive.

## Use Timestamps as a Sync Watermark

Every report carries `scan_started_at`. Feeding it back selects exactly what changed
after that scan began, which is what makes incremental follow-up sound:

```bash
fdu --view summary --format json PATH                       # record scan_started_at
fdu --view files --format jsonl --modified-since <that> PATH
```

Use the scan's *start*, not its end: a file modified mid-scan may have been observed
before the modification, so only the start bound is conservative.

## Validate Every Automated Result

Check the process exit status and these fields:

- `schema` before parsing anything else
- `complete` and `errors` before trusting totals
- `freshness` and `source` before presenting data as current
- `truncated` on a tree node before treating it as exhaustive

`source` is `cold_scan`, `warm_revalidate`, or `cache_only`. Only `--cache only` can
return `freshness: stale`, and it says so rather than implying currency; it fails
outright when no usable snapshot exists rather than silently scanning.

Exit 0 is accepted success, exit 1 is a fatal failure, and exit 2 is incomplete data or
invalid usage. Do not discard useful stdout from exit 2; inspect the completeness fields
and use `--allow-partial` only when incomplete totals are acceptable.

## Cache Behavior

The snapshot is one file per root under the user cache directory. `--cache-status` maps a
hash-named file back to the tree it describes, and `--cache-clear` removes it; both run
without scanning and never touch files this build cannot identify.

Verification cost follows the question asked. Sizes and timestamps need one stat per
entry, because an in-place edit changes a file without changing any directory. Questions
answerable from names alone need only one stat per directory. Adding metrics within a
tier is free; crossing a tier boundary is what costs.

Run `fdu --help` for the complete flag, cache, color, scope, and exit contract.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
? 0
````

## Version Is Exact

```console
$ fdu --version
fdu 0.0.1
? 0
```

## The Default Root Works for an Empty Sandbox

```console
$ fdu --cache off --color never --size apparent --depth 0
       0 B  . (0 files)
? 0
```

## Unknown Options Are Usage Errors on Stderr

```console
$ fdu --definitely-not-an-option
! error: unexpected argument '--definitely-not-an-option' found
!
!   tip: to pass '--definitely-not-an-option' as a value, use '-- --definitely-not-an-option'
!
! Usage: fdu [OPTIONS] [PATH]
!
! For more information, try '--help'.
? 2
```

## A Time Bound the Index Cannot Represent Is Rejected

Silently dropping the bound would run the query with no time filter at all while the user
believed one was active, which is worse than refusing the flag.

```console
$ fdu --modified-since 2300-01-01T00:00:00Z
! fdu: invalid --modified-since "2300-01-01T00:00:00Z": that time is outside the range fdu can represent (about 1677 to 2262)
? 2
```

## Watching Rejects a Narrowed Scan Scope

Both scope flags are refused under `--watch`, because events can land outside a narrowed
scan and the index would silently diverge from the tree. Selection flags are not
refused: they filter what a full index reports.

```console
$ fdu --watch --scan-depth 2
! fdu: --watch cannot be combined with --scan-depth or --one-filesystem: watching requires full scope. Selection flags such as --depth, --include, and --modified-since do work with --watch, because they filter the index rather than narrowing the scan
? 2
```

```console
$ fdu --watch --one-filesystem
! fdu: --watch cannot be combined with --scan-depth or --one-filesystem: watching requires full scope. Selection flags such as --depth, --include, and --modified-since do work with --watch, because they filter the index rather than narrowing the scan
? 2
```

## A Missing Root Is a Fatal Filesystem Error

```console
$ fdu --cache off missing
! fdu: I/O error at missing: [OS_ERROR]
!   caused by: [OS_ERROR]
? 1
```

## A File Cannot Be Used as the Scan Root

### Create a Regular File

```console
$ node -e "require('node:fs').writeFileSync('plain-file', 'x')"
? 0
```

### Reject It as the Root

```console
$ fdu --cache off plain-file
! fdu: I/O error at [SCAN_PATH]: scan root is not a directory
!   caused by: scan root is not a directory
? 1
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
