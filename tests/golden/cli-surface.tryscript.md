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
  # Dev builds carry the git revision (and a dirty marker when the tree has local
  # edits); a build without git metadata reports the bare semver. The semver itself
  # is still asserted exactly — only the build metadata varies.
  DEV_REVISION: '(-dev\+g[0-9a-f]{7,12}(\.dirty)?)?'
---
# CLI Surface

## A Bare Invocation Is Safe and Shows the Complete Contract

The critical no-argument path is the golden itself: it must print help successfully and
must never infer the current directory.
A unit test separately proves that `--help` produces these exact bytes.

```console
$ fdu
Four common reports:
  Languages and LOC   fdu --analyze code --view languages PATH
                      Reads eligible files for code, comment, and blank lines.
  File types by name  fdu --view types PATH
                      Uses exact names and extensions; never reads file contents.
  Folder sizes        fdu PATH
                      Uses the metadata-only tree view and reusable index.
  Fast totals only    fdu --cache off --view summary PATH
                      Returns bytes plus file and directory counts;
                      retains no index or cache.

--analyze chooses what may be read; --view chooses what is printed.

A fast, incremental file roll-up engine: hierarchical tallies over large directory trees

Usage: fdu [OPTIONS] <PATH>
       fdu [PATH] --cache-status[=<SCOPE>] [--cache-clear[=<SCOPE>]]
       fdu [PATH] --cache-clear[=<SCOPE>]
       fdu --skill

Arguments:
  [PATH]
          Report root; optional only for cache lifecycle operations

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
          Views: tree, extensions, types, families, languages, documents, files, summary

          [default: tree]

      --analyze <PROFILE>
          Content depth: none, basic, code, documents, or full

          [default: none]

      --max-file-size <SIZE>
          Maximum bytes read from one analyzed file

          [default: 16MiB]

      --analysis-workers <N>
          Content reader workers; zero selects available parallelism

          [default: 0]

      --words-per-page <N>
          Logical words per derived document page

          [default: 250]

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

More compositions:
  fdu --view extensions ~/Downloads
  fdu --view types,families --format json .
  fdu --analyze documents --view documents .

Five axes, and every option belongs to exactly one:
  Scope      PATH, --scan-depth                         what is scanned and cached
  Selection  --include, --exclude, --depth, --limit    which entries are considered
  View       tree,extensions,types,families,languages,documents,files,summary
  Format     --format text|json|jsonl|yaml, --color
  Mode       --cache, --analyze, --max-file-size, --analysis-workers

Content analysis:
  none       metadata only; source files are never opened (default)
  basic      physical, blank, and nonblank lines plus raw prose words
  code       basic metrics plus the versioned common-language SLOC analyzer
  documents  basic metrics plus logical and reader-visible prose metrics
  full       every shipped analyzer

  languages requires code or full; documents requires any enabled profile.
  Views never enable content analysis implicitly.
  Content reads are bounded by --max-file-size and --analysis-workers.
  --words-per-page changes only report-time page derivation.
  Unchanged results for the same profile are restored from a separate sidecar.
  cache=only never opens source files and fails if requested content is absent.

Output and automation:
  Metadata-only machine output remains fdu.report/1; metric summaries use fdu.report/2.
  Metric rows include detection source, confidence, origin flags, and coverage.
  Results go to stdout; warnings and errors go to stderr.
  The command never prompts, pages, or animates progress.

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
Every report requires an explicit `PATH`; bare `fdu` prints help instead of scanning the
current directory.

## Run fdu

Start with the report that answers the question:

```bash
fdu --analyze code --view languages PATH  # language and standard LOC; reads content
fdu --view types PATH                     # file types from names; metadata only
fdu PATH                                  # folder-size tree; metadata only
fdu --cache off --view summary PATH       # one totals row; no retained index
```

`--analyze` chooses what may be read and `--view` chooses what is printed.
The language command needs both because a view never enables content reads implicitly.
Use `--size apparent` when logical file lengths are wanted instead of allocated bytes.

For a bounded machine-readable tree, use:

```bash
fdu --format json --view tree --depth 2 --limit 20 PATH
```

If no local command exists and this release is published on PyPI, use the exact reviewed
version. Never use an unversioned `uvx` runner or `latest` in agent instructions:

```bash
uvx --from fdu==0.0.1 fdu --format json --view tree PATH
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
contents. Any non-`none` `--analyze` profile opts into bounded content reads and a
separate profile-scoped sidecar.
A repeated run with the same profile and semantic settings reuses unchanged content
records. Coverage is profile-scoped too: an unsupported deeper analyzer leaves byte
metadata visible but does not retain a separate lower-level metric record for that file.

## Pick the View, Then Shape It

- `--view tree` (default) for per-directory roll-ups.
- `--view extensions` for the original raw-extension breakdown.
- `--view types` for stable detected file types and exact byte shares.
- `--view families` for code, prose, markup, data, binary, and unknown roll-ups.
- `--view languages` for code-family rows; it requires `--analyze code` or `full`.
- `--view documents` for prose metrics; it requires any enabled analysis profile.
- `--view files` for a flat listing; in text output it prints one path per line and
  nothing else, so it pipes directly into other commands.
- `--view summary` for one aggregate row.
- Several views in one run share one scan: `--view summary,types,families`.

Add `--analyze basic` to stream physical, blank, and nonblank lines and raw prose words.
Use `--analyze code --view languages` for standard LOC, comment, and code-blank
partitions across supported common languages.
Use `--analyze documents --view documents` for normalized prose words, paragraphs,
aggregate-derived pages, and reader-visible Markdown that excludes destinations and
code. `--analyze full` computes both families in one bounded pass.
Use `--max-file-size`, `--analysis-workers`, and `--words-per-page` to bound work and
control page derivation.
Content analysis is currently one-shot and cannot be combined with `--watch`.

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
? 0
````

## Version Is Exact

The semver is asserted exactly; the dev-build revision after it varies with the checkout
and is matched by pattern.

```console
$ fdu --version
fdu 0.0.1[DEV_REVISION]
? 0
```

## An Explicit Current Root Works for an Empty Sandbox

```console
$ fdu --cache off --color never --size apparent --depth 0 .
       0 B  ░░░░░░░░░░     0%  . (0 files)
? 0
```

## Unknown Options Are Usage Errors on Stderr

```console
$ fdu --definitely-not-an-option
! error: unexpected argument '--definitely-not-an-option' found
!
!   tip: to pass '--definitely-not-an-option' as a value, use '-- --definitely-not-an-option'
!
! Usage: fdu [OPTIONS] <PATH>
!        fdu [PATH] --cache-status[=<SCOPE>] [--cache-clear[=<SCOPE>]]
!        fdu [PATH] --cache-clear[=<SCOPE>]
!        fdu --skill
!
! For more information, try '--help'.
? 2
```

## A Time Bound the Index Cannot Represent Is Rejected

Silently dropping the bound would run the query with no time filter at all while the
user believed one was active, which is worse than refusing the flag.

```console
$ fdu --modified-since 2300-01-01T00:00:00Z .
! fdu: invalid --modified-since "2300-01-01T00:00:00Z": that time is outside the range fdu can represent (about 1677 to 2262)
? 2
```

## Watching Rejects a Narrowed Scan Scope

Both scope flags are refused under `--watch`, because events can land outside a narrowed
scan and the index would silently diverge from the tree.
Selection flags are not refused: they filter what a full index reports.

```console
$ fdu --watch --scan-depth 2 .
! fdu: --watch cannot be combined with --scan-depth or --one-filesystem: watching requires full scope. Selection flags such as --depth, --include, and --modified-since do work with --watch, because they filter the index rather than narrowing the scan
? 2
```

```console
$ fdu --watch --one-filesystem .
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
