---
sandbox: true
path:
  - $FDU_BIN
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
  PERF_TIME: '[\d.]+ (ns|µs|ms|s)'
---
# CLI Surface

## A Bare Invocation Is Safe and Shows the Complete Contract

The critical no-argument path is the golden itself: it must print help successfully and
must never infer the current directory.
A unit test separately proves that `--help` produces these exact bytes.

```console
$ fdu
A fast, incremental file roll-up engine: hierarchical tallies over large directory trees

Usage: fdu [OPTIONS] <PATH>
       fdu [PATH] --cache-status[=<SCOPE>] [--cache-clear[=<SCOPE>]]
       fdu [PATH] --cache-clear[=<SCOPE>]
       fdu --docs
       fdu --skill

ARGUMENTS
  [PATH]  Report root; optional only for the discovery and cache-lifecycle flags

SCOPE
      --scan-depth <N>     Limit scanning and retention to N entry levels
      --one-filesystem     Stay on the filesystem the root lives on
      --order <ORDER>      Directory visit order: breadth-first (default) or depth-first
      --threads <N>        Walker threads, or unset to choose automatically
      --type-rules <FILE>  Classify against a `[[kind]]` rule file instead of fdu's taxonomy

SELECTION
      --include <GLOB>          Report only entries matching this glob; repeatable
      --exclude <GLOB>          Exclude entries matching this glob; repeatable, and wins over
                                --include
      --min-size <SIZE>         Report only entries at least this large, as 512, 10M, or 1.5GiB
      --modified-since <WHEN>   Report only entries modified at or after this time, as 2h or an RFC
                                3339 stamp
      --modified-before <WHEN>  Report only entries modified before this time
      --kind <LIST>             Entry kinds to report: file, dir, symlink, other
  -d, --depth <N>               Directory levels to show; does not limit scanning. Accepts `all`
                                [tree default: 2]
  -n, --limit <N>               Rows to show, per group. Accepts `all`
      --sort <KEY>              Order results: size, count, mtime, or name
      --reverse                 Reverse the ordering
      --size <METRIC>           Which size metric to report: allocated or apparent [default:
                                allocated]

VIEWS
      --view <LIST>         Views: tree, extensions, types, families, languages, documents, largest,
                            recent, files, summary, or full. Defaults to the view that displays what
                            --analyze asked for
      --words-per-page <N>  Logical words per derived document page [default: 250]

CONTENT ANALYSIS
      --analyze <LIST>        Analyzers to run: none, lines, code, words, or all [default: none]
      --analysis-workers <N>  Content reader workers; zero selects available parallelism [default:
                              0]

OUTPUT
      --format <FORMAT>  Output format: text, json, jsonl, or yaml [default: text]
      --color <WHEN>     Colorize human output: auto, always, or never [default: auto]

EXECUTION
      --cache <POLICY>  Cache policy: auto, refresh, read-only, only, or off [default: auto]
      --allow-partial   Accept operationally partial results, including filesystem or analysis
                        failures
      --watch           Stream changes continuously instead of returning one report
      --interval <DUR>  How often aggregate views re-render while watching, as a duration [default:
                        2s]

CACHE MANAGEMENT
      --cache-status[=<SCOPE>]  Report cache contents instead of scanning: root (default) or all
      --cache-clear[=<SCOPE>]   Remove cached snapshots instead of scanning: root (default) or all

OTHER
  -h, --help     Print help
  -V, --version  Print version
      --docs     Print the usage guide: the report ladder, both axes, and the output contracts
      --skill    Print a portable agent skill to stdout

Run `fdu --docs` for more help and important usage examples.
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
`fdu --docs` prints the full usage guide -- the report ladder, both axes, and the output
contracts -- without a PATH and without scanning.
Every report requires an explicit `PATH`; bare `fdu` prints help instead of scanning the
current directory.

## Run fdu

Start with the report that answers the question:

```bash
fdu --view languages PATH                 # detected language sizes; metadata only
fdu --analyze code --view languages PATH  # add standard LOC; reads content
fdu --view types PATH                     # all file types; metadata only
fdu PATH                                  # folder-size tree; metadata only
fdu --view summary PATH                   # one totals row; no retained index
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
uvx --from fdu==0.1.0 fdu --format json --view tree PATH
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
A single unfiltered `--view summary PATH` is the one exact composition that retains only
aggregate tallies and no index, under every cache policy except `only` and `refresh`,
whose contracts are about the snapshot itself.
Under the rest a snapshot cannot save the walk that request is already doing, so it
neither reads nor writes one.
Ordinary metadata requests retain the reusable index but never read regular-file
contents. Any non-`none` `--analyze` profile opts into streaming reads through every
eligible file and a separate profile-scoped sidecar.
A repeated run with the same profile and semantic settings reuses unchanged content
records. Coverage is profile-scoped too: an unsupported deeper analyzer leaves byte
metadata visible but does not retain a separate lower-level metric record for that file.

## Pick the View, Then Shape It

- `--view tree` (default) for per-directory roll-ups.
- `--view extensions` for the original raw-extension breakdown.
  Rows partition the tree and so sum to its total; a derived extension always carries a
  leading dot, and names having none are tallied under the literal `(none)`.
- `--view types` for stable detected file types and exact byte shares.
- `--view families` for code, prose, markup, data, binary, and unknown roll-ups.
- `--view languages` for code-family rows and byte shares from path-only detection.
- `--view documents` for prose metrics; it requires any enabled analysis profile.
- `--view files` for a flat listing.
  One-shot text adds the performance footer described below; use a machine format when
  output is consumed programmatically.
- `--view summary` for one aggregate row.
- Several views in one run share one scan: `--view summary,types,families`. Text then
  labels each block with an all-caps header naming its view; a single-view text report
  has no header. Machine formats tag every report with `view` either way.

`--analyze` names a set of analyzers, comma-separated, from `lines`, `code`, and
`words`; `none` and `all` are totals and cannot be combined with anything else.
Anything but `none` opens and reads every eligible file, which is the only setting that
makes a run cost more than one metadata walk.

Add `--analyze lines` to stream physical, blank, and nonblank lines and raw word counts.
Add `--analyze code` for standard LOC, comment, and code-blank partitions across
supported common languages; the percentage column then uses code lines instead of bytes.
Use `--analyze words` for normalized word volume, paragraphs, aggregate-derived pages,
and reader-visible Markdown that excludes destinations and code.
`--analyze code,words` — or `all` — computes both in one streaming pass.

Requesting analysis without naming a view selects one that displays it: `code` reports
`languages`, `words` reports `documents`, and either both or `lines` alone reports
`families`. Naming `--view` overrides that; a view never enables an analyzer, so a
`--view` that displays no content metric prints a note saying what was read for nothing.
`--view all` reports every view the requested analyzers can answer and names any it
skipped. Use `--analysis-workers` to bound concurrent reads and `--words-per-page` to
control page derivation.
Analysis never truncates a file or excludes it because of size.
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

- `schema` before parsing anything else: a metadata-only report carries `fdu.report/4`,
  a report that ran content analysis carries `fdu.report/5`, and a `--watch` stream
  carries `fdu.stream/1`. Treat an unrecognized value as a version you cannot parse
  rather than guessing at the fields.
- `complete` and `errors` before trusting totals
- `freshness` and `source` before presenting data as current
- `truncated` on a tree node before treating it as exhaustive, and `remainder` for what
  it withheld: `rows`, `files`, `dirs`, `bytes`, and `allocated` for the child rows not
  emitted, or `null` when none were.
  Emitted children plus `remainder` account for every directory beneath the node, which
  is what makes an “other” row addable without a second query.
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

## The Guide Is Reachable Without a Root

`--docs` is a discovery surface like `--skill` and bare `fdu`: it answers without a
PATH, scans nothing, and exits 0. The body is pinned in full below so a flag rename or a
dropped section fails here rather than in somebody’s terminal.

```console
$ fdu --docs
fdu — a fast, incremental file roll-up engine.

THE LADDER
  Every report is one command, and they form a ladder. Each rung costs more than
  the one above it and tells you more, so stop at the cheapest answer that
  settles your question.

    fdu --view summary PATH             how big is this tree?        no reads
    fdu PATH                            which folders are big?       no reads
    fdu --view types PATH               what kinds of files?         no reads
    fdu --view languages PATH           which languages?             no reads
    fdu --analyze code PATH             how much code?               READS FILES
    fdu --analyze words PATH            how much writing?            READS FILES
    fdu --analyze all --view all PATH   everything there is          READS FILES

TWO FLAGS DO ALL OF IT
  --analyze decides what gets read. Anything but `none` opens and reads every
    eligible file, which is the only setting that makes a run cost more than a
    single metadata walk.
  --view decides what gets printed. It is free: every view is a projection over
    one walk, so asking for more views never touches the filesystem again.

  You rarely need both. Naming analyzers selects a view that displays them, so
  `fdu --analyze code PATH` already prints language rows with lines of code.
  Name --view yourself for a different projection; it always wins.

  A view never turns on an analyzer, because choosing how to look at a result
  should not quietly authorize reading every file in the tree. So a --view that
  displays none of what you asked to read says how much was read for nothing,
  and --view all names any view it had to skip.

MORE COMPOSITIONS
  fdu --view extensions ~/Downloads
  fdu --view types,families --format json .
  fdu --analyze words --view documents .
  fdu --view files --min-size 10M --sort size -n 100 PATH   largest files
  fdu --view files --modified-since 1h --sort mtime PATH    recent changes
  fdu --watch --view files --format jsonl PATH              a tail -f for a tree

  --interval throttles rendering only; change detection is event-driven and
  unaffected by it, so an idle tree costs nothing between changes.

SIX AXES, AND EVERY OPTION BELONGS TO EXACTLY ONE
  Scope      PATH, --scan-depth, --order, --threads, --type-rules
  Content    --analyze none|lines|code|words|all        which file bodies are read
  Selection  --include, --exclude, --depth, --limit     which entries are considered
  View       tree,extensions,types,families,languages,documents,files,summary,all
  Format     --format text|json|jsonl|yaml, --color
  Mode       --cache, --watch, --analysis-workers

CONTENT ANALYSIS
  none       metadata only; source files are never opened (default)
  lines      physical, blank, and nonblank lines plus raw word counts
  code       standard SLOC from the versioned common-language analyzer
  words      normalized and reader-visible word volume
  all        every shipped analyzer

  A comma-separated set: code,words runs both. none and all name the whole
  axis and cannot be combined. lines comes with any analyzer, free.
  languages is metadata-only by default; code adds standard LOC.
  documents requires any enabled analyzer.
  Analysis streams every eligible file through EOF; files are never size-truncated.
  --analysis-workers bounds concurrency.
  --words-per-page changes only report-time page derivation.
  Unchanged results are restored from a separate sidecar; a stored set answers
  any narrower request without re-reading.
  cache=only never opens source files and fails if requested content is absent.

OUTPUT AND AUTOMATION
  Metadata-only machine output remains fdu.report/4; metric summaries use fdu.report/5.
  Text language rows use canonical names; machine formats retain lowercase IDs.
  Metric rows include detection source, confidence, origin flags, and coverage.
  One-shot text reports end with a gray performance line; machine formats omit it.
  Results go to stdout; warnings and errors go to stderr.
  The command never prompts, pages, or animates progress.
  Reports require an explicit PATH; bare `fdu` prints help and scans nothing.
  `fdu --skill` prints a portable agent skill describing this same surface.

EXIT STATUS
  0  Complete result, or a partial result accepted with --allow-partial
  1  Fatal filesystem or cache error
  2  Partial result, or command-line usage error
? 0
```

## Version Is Exact

The semver is asserted exactly; the dev-build revision after it varies with the checkout
and is matched by pattern.

```console
$ fdu --version
fdu 0.1.0[DEV_REVISION]
? 0
```

## An Explicit Current Root Works for an Empty Sandbox

```console
$ fdu --cache off --color never --size apparent --depth 0 .
       0 B  ░░░░░░░░░░     0%  . (0 files)
Performance: walked 0 files / 0 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
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
!        fdu --docs
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
! fdu: watching requires full scope and cannot be combined with --scan-depth or --one-filesystem: a watcher cannot filter backend events against a narrowed boundary. Selection such as --depth, --include, and --modified-since does work while watching, because it filters the retained index rather than narrowing the scan
? 2
```

```console
$ fdu --watch --one-filesystem .
! fdu: watching requires full scope and cannot be combined with --scan-depth or --one-filesystem: a watcher cannot filter backend events against a narrowed boundary. Selection such as --depth, --include, and --modified-since does work while watching, because it filters the retained index rather than narrowing the scan
? 2
```

## A Missing Root Is a Fatal Filesystem Error

```console
$ fdu --cache off missing
! fdu: I/O error at missing: [OS_ERROR]
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
? 1
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
