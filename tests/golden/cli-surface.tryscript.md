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
      --scan-depth <N>               Limit scanning and retention to N entry levels
      --one-filesystem               Stay on the filesystem the root lives on
      --gitignore-budget <SIZE>      Bytes of .gitignore rules to retain before refusing more files
                                     [default: 4MiB]. Accepts `all`, which also reads each
                                     .gitignore whole
      --gitignore-line-limit <SIZE>  Longest .gitignore line to apply before refusing its file
                                     [default: 16KiB]. Accepts `all`
      --no-gitignore                 Read no .gitignore files: rows lose their ignored share, and
                                     the snapshot scope differs

SELECTION
      --include <GLOB>          Report only entries matching this glob; repeatable
      --exclude <GLOB>          Exclude entries matching this glob; repeatable, and wins over
                                --include
      --min-size <SIZE>         Report only entries at least this large, as 512, 10M, or 1.5GiB
      --modified-since <WHEN>   Report only entries modified at or after this time, as 2h or an RFC
                                3339 stamp
      --modified-before <WHEN>  Report only entries modified before this time
      --kind <LIST>             Entry kinds to report: file, dir, symlink, other
      --exclude-ignored         Report only entries no .gitignore rule ignores; sizes and ordering
                                follow
      --only-ignored            Report only entries a .gitignore rule ignores
  -d, --depth <N>               Directory levels to show; does not limit scanning. Accepts `all`
                                [tree default: 2]
  -n, --limit <N>               Rows to show, per group. Accepts `all`
      --sort <KEY>              Order results: size, count, mtime, or name
      --reverse                 Reverse the ordering
      --size <METRIC>           Which size metric to report: allocated or apparent [default:
                                allocated]

VIEWS
      --view <LIST>         Views: tree, extensions, types, families, languages, documents, largest,
                            recent, files, summary, or full. Defaults to tree with no analysis,
                            otherwise to a view that displays the requested analysis
      --words-per-page <N>  Logical words per derived document page [default: 250]

CONTENT ANALYSIS
      --analyze <LIST>        Analyzers to run: none, lines, code, words, or all [default: none]
      --analysis-workers <N>  Content reader workers; zero selects available parallelism [default:
                              0]

OUTPUT
      --format <FORMAT>  Output format: text, json, jsonl, or yaml [default: text]
      --color <WHEN>     Colorize human output: auto, always, or never [default: auto]

EXECUTION
      --cache <POLICY>  Cache policy: auto, refresh, read-only, only (unverified), or off [default:
                        auto]
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
      --docs     Print common commands, cache behavior, and the complete usage guide
      --skill    Print a portable agent skill to stdout

Examples:
  fdu .                     directory sizes (metadata only)
  fdu . --exclude-ignored   omit entries covered by .gitignore
  fdu . --view=summary      one total for the tree
  fdu . --analyze=code      standard lines of code by language

Run `fdu --docs` for more commands, cache behavior, and the full usage guide.
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
`fdu --docs` prints common commands, cache behavior, and the full usage contract without
a PATH and without scanning.
Every report requires an explicit `PATH`; bare `fdu` prints help instead of scanning the
current directory.

## Run fdu

Use `.` for the current directory.
Start with the report that answers the question:

```bash
fdu .                                      # directory-size tree; metadata only
fdu . --exclude-ignored                    # select what .gitignore rules leave
fdu . --view=summary                       # one total with its ignored share
fdu . --view=languages                     # detected language sizes; metadata only
fdu . --view=families,types,extensions     # three file-kind breakdowns
fdu . --view=recent --limit=10             # ten most recently modified files
fdu . --analyze=lines                      # physical lines and raw words
fdu . --analyze=lines --view=languages     # those metrics by language
fdu . --analyze=code                       # standard LOC by language
fdu . --analyze=words                      # prose volume by document type
```

The default is `tree` in allocated bytes, largest first, to depth 2, with at most ten
children per directory.
Hidden and ignored entries are included.
`.gitignore` is read to label ignored shares, not to exclude matching entries.

`--analyze` chooses what may be read and `--view` chooses what is printed.
The language commands differ only on the analysis axis: without code analysis
percentages are byte shares; with it, the report adds code, comment, and blank-line
metrics and uses code-line shares.
Text labels a non-byte percentage denominator explicitly.
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

## Compose the Request From Six Axes

Every option belongs to exactly one axis, and any axis composes with any other.
There are no subcommands: the grammar is always “report on a path”.

| Axis | Question | Options |
| --- | --- | --- |
| Scope | What is scanned and cached? | `PATH`, `--scan-depth N`, `--one-filesystem`, `--gitignore-budget SIZE\|all`, `--gitignore-line-limit SIZE\|all`, `--no-gitignore` |
| Content | Which file bodies are read? | `--analyze none\|lines\|code\|words\|all` |
| Selection | Which entries does this query consider? | `--include`, `--exclude`, `--min-size`, `--modified-since`, `--modified-before`, `--kind`, `--exclude-ignored`, `--only-ignored`, `--depth`, `-n/--limit`, `--sort`, `--reverse`, `--size` |
| View | Which roll-up is reported? | `--view summary,tree,families,types,extensions,languages,documents,largest,recent,files`, or `--view full` |
| Format | How is it serialized? | `--format text\|json\|jsonl\|yaml`, `--color` |
| Mode | How is work performed? | `--cache auto\|refresh\|read-only\|only\|off`, `--watch`, `--analysis-workers N` |

Scope versus selection is the distinction that matters: scope decides what is scanned
and cached, so one cache serves every query, while selection filters the retained index
at query time. Narrowing a selection never costs a rescan.

Work has three layers.
A single unfiltered `--no-gitignore --view summary PATH` is the one exact composition
that retains only aggregate tallies and no index, under every cache policy except `only`
and `refresh`, whose contracts are about the snapshot itself.
Under the rest a snapshot cannot save the walk that request is already doing, so it
neither reads nor writes one.
Without `--no-gitignore` the summary reads `.gitignore` to report its ignored share,
which needs the index.
Ordinary metadata requests retain the reusable index but never read regular-file
contents. One-shot metadata reports under `auto` skip loading a snapshot when it cannot
avoid the current metadata walk, though a complete indexed report may write one.
Any `--analyze` value other than `none` opts into a separate content sidecar.
fdu reads eligible files whose requested result is absent or stale; a compatible
repeated run reuses unchanged records.
Coverage is scoped to the analyzers too: an unsupported deeper analyzer leaves byte
metadata visible but does not retain a separate lower-level metric record for that file.

## Pick the View, Then Shape It

- `--view tree` (default) for per-directory roll-ups.
- `--view extensions` for the original raw-extension breakdown.
  Rows partition the tree and so sum to its total; a derived extension always carries a
  leading dot, and names having none are tallied under the literal `(none)`.
- `--view types` for stable detected file types and exact byte shares.
- `--view families` for code, prose, markup, data, binary, and unknown roll-ups.
- `--view languages` for code-family rows and byte shares from path-only detection.
- `--view documents` for prose metrics; it requires any enabled analyzer.
- `--view largest` for the 20 largest regular files, and `--view recent` for the 20 most
  recently modified. Both are presets over `files`, not separate machinery: `largest` is
  `files --sort size --limit 20` and `recent` is `files --sort mtime --limit 20`, each
  restricted to regular files, and `--sort` and `--limit` still override them.
- `--view files` for a complete flat listing: every matching entry, in name order.
  One-shot text adds the performance footer described below; use a machine format when
  output is consumed programmatically.
- `--view summary` for one aggregate row.
- `--view full` for every view except `files`.
- Several views in one run share one scan: `--view summary,types,families`. Text then
  labels each block with an all-caps header naming its view; a single-view text report
  has no header. Machine formats tag every report with `view` either way.

`--analyze` names a set of analyzers, comma-separated, from `lines`, `code`, and
`words`; `none` and `all` are totals and cannot be combined with anything else.
Anything but `none` may open eligible files and adds content work beyond the metadata
walk. Compatible cached results prevent unchanged bodies from being reread.

Add `--analyze lines` to stream physical, blank, and nonblank lines and raw word counts.
Add `--analyze code` for standard LOC, comment, and code-blank partitions across
supported common languages.
Text labels its code-line percentage denominator while retaining bytes in the first
column. Use `--analyze words` for normalized word volume, paragraphs, aggregate-derived
pages, and reader-visible Markdown that excludes destinations and code.
The `documents` percentage column is document-word share and is also labeled in text.
`--analyze code,words` — or `all` — computes both in one streaming pass.

Requesting analysis without naming a view selects one that displays it: `code` reports
`languages`, `words` reports `documents`, and either both or `lines` alone reports
`families`. Naming `--view` overrides that; a view never enables an analyzer, so a
`--view` that displays no content metric prints a note saying what was read for nothing.
`--view full` includes `documents` only when an analyzer ran, and otherwise names it as
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
fdu --view largest -n 100 PATH                        # the 100 largest files
fdu --view files --modified-since 2h PATH             # changed in the last two hours
fdu --view files --include '*.{rs,toml}' PATH         # by pattern
fdu --view tree --sort mtime PATH                     # an activity map
```

`--depth` and `--limit` bound only the rendered view; `--scan-depth` bounds what is
scanned and retained, so do not reach for it merely to shorten output.

## Read What `.gitignore` Covers

Every report reads the tree’s `.gitignore` files.
Summary, tree, and extension rows end with the part of their size those rules ignore, as
`(128 B ignored)`, left off a row with no ignored file; the performance line counts the
rule files read.

```bash
fdu PATH --exclude-ignored                              # folders by what the rules leave
fdu PATH --view=files --only-ignored --format=jsonl     # every entry the rules cover
fdu PATH --no-gitignore                                 # read no rules, show no share
```

Selecting a side changes sizes, ordering, and `--min-size` together, because they follow
the entries shown.
It filters retained results after the scan; it does not prune metadata
work or content analysis.
`--no-gitignore` with either selection is a usage error.
Only per-directory `.gitignore` files apply, not `core.excludesFile`,
`.git/info/exclude`, or a global ignore file, and matching is case-sensitive.
Unignored does not mean tracked: `.git` is unignored unless a rule names it.
For recent working files, add both `--exclude-ignored` and `--exclude='.git/**'`. An
unreadable `.gitignore` makes the result partial (exit 2), while one past
`--gitignore-budget` or `--gitignore-line-limit` is refused whole and named in a note:
sizes stay exact, the ignored shares under that directory do not.

Under `--watch`, a rule edit that moves an entry into either selection streams the
upsert that draws it and one that moves it out streams the removal, so the stream holds
the entry set the flag names.
An upsert carries `ignored`, and so does a removal a rule edit caused; an ordinary
removal, an invalidation, and every record of a run that read no rules omit it.
Without either flag the stream maintains membership rather than each row’s bit, so
re-read a listing after a rule edit if the bit matters.

## Value Grammars

- Sizes: `512`, `10k`, `10M`, `1.5GiB`. Decimal and binary units, case-insensitive.
- Times: `now`, a compound age (`200ms`, `45s`, `2h`, `1h30m`), an RFC 3339 timestamp
  with an offset (`2026-08-10T18:22:31Z`), or `@` epoch seconds.
  Calendar units and fractional ages are rejected with the spelling to use instead; a
  bare local date-time is rejected because resolving it needs a time-zone database.
- `--modified-since` is inclusive and `--modified-before` is exclusive.

## Use Timestamps as a Sync Watermark

Every report carries `provenance.scan_started_at`. Feeding it back selects exactly what
changed after that scan began, which is what makes incremental follow-up sound:

```bash
fdu --view summary --format json PATH              # record provenance.scan_started_at
fdu --view files --format jsonl --modified-since <that> PATH
```

Use the scan’s *start*, not its end: a file modified mid-scan may have been observed
before the modification, so only the start bound is conservative.

## Validate Every Automated Result

Check the process exit status and these fields:

- `schema` before parsing anything else: a report carries `fdu.report/7`, a `--watch`
  stream carries `fdu.stream/2`, and `--cache-status` carries `fdu.cache/2`. Treat an
  unrecognized value as a version you cannot parse rather than guessing at the fields.
- Integer fields that exceed 2^53 (fingerprints, option hashes, nanosecond timestamps)
  lose precision in IEEE 754 binary64 parsers such as JavaScript `JSON.parse`
- `status.complete`, `status.errors`, and `status.errors_omitted` before trusting totals
- `provenance.freshness` and `provenance.source` before presenting data as current
- `truncated` on a tree node before treating it as exhaustive
- Each requested unit in a metric row’s `coverage` before presenting its metrics as
  complete
- `ignored` on a row before calling anything ignored or not: an object, or `true` and
  `false` on a file row, where rules were read, and `null` where none were, which never
  means nothing is ignored
- `ignore_rules.refused` before trusting an ignored share: below a refused `.gitignore`
  the split is not exact, though sizes are
- `detection.sources`, `detection.confidence`, and `detection.flags` before treating a
  deep-detected type or origin label as exact

`provenance.source` is `cold_scan`, `warm_revalidate`, or `cache_only`. Only
`--cache only` can return `provenance.freshness: stale`, and it says so rather than
implying currency; it fails outright when no usable snapshot exists rather than silently
scanning.

Exit 0 is accepted success, exit 1 is a fatal failure, and exit 2 is incomplete data or
invalid usage. Do not discard useful stdout from exit 2; inspect the completeness fields
and use `--allow-partial` only when incomplete totals are acceptable.

## Cache Behavior

No ordinary view requires a preexisting cache.
Metadata-only one-shot reports include current sizes or timestamps, so they must inspect
every entry; under `auto` they skip loading a snapshot when it cannot make that
verification cheaper.
A complete indexed scan may still write one.

Content analysis is where ordinary repeated runs benefit most.
The first compatible run reads eligible bodies; a later run restores unchanged records
from the content sidecar and reads only changed or newly eligible files.
A stored analyzer set answers only the same set: a different one, wider or narrower,
reads the files again and replaces it.
The performance footer reports fresh and cached analysis separately.

`--cache=only` is a distinct contract: it never verifies the source tree, labels the
answer stale, and fails unless compatible metadata and any requested content analysis
already exist. `--cache=off` neither reads nor writes fdu cache data.

The snapshot is one file per root under the user cache directory.
`--cache-status` maps a hash-named file back to the tree it describes, and
`--cache-clear` removes it; both run without scanning.
Cache status is its own document, carrying the `fdu.cache/2` schema in every machine
format rather than a report schema.
A current snapshot’s row carries the `identity` of the entry and `.gitignore` tiers it
holds, and every row a `content` object for the sidecar beside it, with its own `state`
and a current sidecar’s `identity`, or `null` when there is none.
Each status row carries a `state`: `current`, `stale` for a snapshot another fdu version
wrote or one this build cannot read, `leftover` for a file fdu left behind, with a
`leftover_kind`, `unrecognized` for a file that is not fdu’s, or `absent`. Clearing
removes current and stale snapshots, so `--cache-clear=all` reclaims what an upgrade
leaves behind; it also reclaims leftovers, and it never removes an unrecognized file.

All shipped metadata views need current per-file attributes because they report sizes or
timestamps. An in-place edit changes no directory timestamp, so a cached directory
fingerprint cannot prove those answers current.
Content reuse still pays because an unchanged file fingerprint avoids the much more
expensive body read and analysis.

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

START HERE
  A report requires a PATH. Use `.` for the current directory.

    fdu .                                      directory sizes (the default)
    fdu . --exclude-ignored                    omit entries covered by .gitignore
    fdu . --view=summary                       one total for the tree
    fdu . --view=languages                     languages by byte size
    fdu . --view=families,types,extensions     three file-kind breakdowns
    fdu . --view=recent --limit=10             ten most recently modified files
    fdu . --analyze=lines                      physical lines and raw words
    fdu . --analyze=lines --view=languages     those metrics by language
    fdu . --analyze=code                       standard lines of code by language
    fdu . --analyze=words                      prose volume by document type

  `fdu .` is metadata-only. It prints a tree in allocated bytes, largest first,
  to depth 2, with at most 10 children per directory. Hidden and ignored entries
  are included; .gitignore is read to label ignored shares, not to exclude them.

VIEWS AND ANALYSIS
  --view chooses the question the report answers. Several views share one scan
    and one requested analysis; adding a view does not run a second scan.
  --analyze opts into reading eligible file bodies. Without it, regular file
    contents are not opened. Compatible cached results avoid rereading unchanged
    bodies, so a repeated content analysis can be much cheaper.

  Naming analyzers selects a view that displays them: code selects languages,
  words selects documents, and lines or a multi-analyzer set selects families.
  Name --view for a different projection; it always wins.

  A view never turns on an analyzer, because choosing how to look at a result
  should not quietly authorize reading every file in the tree. If a selected
  view cannot display requested analysis, fdu still performs the analysis and
  prints a note. --view=full names any view it had to skip.

MORE COMPOSITIONS
  fdu ~/Downloads --view=extensions
  fdu . --view=types,families --format=json
  fdu . --analyze=words --view=documents
  fdu PATH --view=largest --limit=100                        the 100 largest files
  fdu PATH --view=files --modified-since=1h --sort=mtime     recent changes
  fdu PATH --view=files --only-ignored --format=jsonl        what .gitignore covers
  fdu --watch --view files --format jsonl PATH              a tail -f for a tree

  --interval throttles rendering only; change detection is event-driven and
  unaffected by it, so an idle tree costs nothing between changes.
  The duration uses the age grammar: `2s`, `200ms`, `1h30m`.

  largest and recent are presets over files, not more views to learn:
    largest = files --sort size --limit 20, regular files only
    recent  = files --sort mtime --limit 20, regular files only
  --sort and --limit still override them. files alone is complete: every
  matching entry, in name order. full is every view except files.

SIX AXES, AND EVERY OPTION BELONGS TO EXACTLY ONE
  Scope      PATH, --scan-depth, --one-filesystem       what is scanned and cached
             --gitignore-budget, --gitignore-line-limit, --no-gitignore
  Content    --analyze none|lines|code|words|all        which file bodies are read
  Selection  --include, --exclude, --depth, --limit     which entries are considered
             --exclude-ignored, --only-ignored
  View       summary,tree,families,types,extensions,languages,documents,
             largest,recent,files,full
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
  Unchanged results are restored from a separate sidecar written by the same
  analyzer set; any other set, wider or narrower, reads the files again.
  --cache=only never opens source files and fails if requested content is absent.

CACHE BEHAVIOR
  No ordinary view requires a preexisting cache. Metadata-only one-shot reports
  still inspect current metadata; under --cache=auto they skip loading a snapshot
  that cannot make that work cheaper, though a complete indexed scan may write one.

  Content analysis is where repeated-run caching pays most. The first run reads
  eligible file bodies. A compatible later run reuses results for unchanged files
  and reads only changed or newly eligible bodies; the performance footer reports
  fresh and cached analysis separately. Repeat the same --analyze command to see it.

  --cache=only is different: it does no filesystem verification, requires a
  compatible snapshot and content sidecar for the requested analysis, and labels
  its answer stale. --cache=off neither reads nor writes fdu's cache.

IGNORE RULES
  Every report reads each .gitignore in the tree, and summary, tree, and extension
  rows end with how much of their size its rules ignore, as `(128 B ignored)`.
  A directory a rule ignores is ignored with everything below it. Unignored is not
  tracked: .git is unignored unless a rule names it. --exclude-ignored and
  --only-ignored select one side after the scan; they do not prune metadata work
  or content analysis. Sort and --min-size follow the size shown.
  --no-gitignore reads no rules and shows no share. Only per-directory .gitignore
  files apply, not core.excludesFile, .git/info/exclude, or a global ignore file,
  and matching is case-sensitive on every platform. An unreadable .gitignore makes
  the result partial, like any unreadable path. A .gitignore past --gitignore-budget
  or --gitignore-line-limit is refused whole and named in a note: sizes stay exact,
  ignored shares under that directory do not.

OUTPUT AND AUTOMATION
  Every machine report uses fdu.report/7; watch changes use fdu.stream/2.
  Cache status is its own document in every machine format: fdu.cache/2.
  Summary, tree, extension, and file rows carry `ignored`: null under --no-gitignore.
  Text language rows use canonical names; machine formats retain lowercase IDs.
  Metric rows include detection source, confidence, origin flags, and coverage.
  One-shot text reports end with a gray performance line; machine formats omit it.
  JSON numbers above 2^53 (fingerprints, option hashes, nanosecond timestamps)
  lose precision in IEEE 754 binary64 parsers such as JavaScript JSON.parse.
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
Performance: walked 0 files / 0 B; ignore rules 0 files; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
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

`--one-filesystem` is refused by the same rule, and is asserted in `query_request.rs`
rather than here: where a build cannot honor it at all, such as a Windows host with no
device identity, the request is refused for that reason first, so the message is not the
same on every platform.

```console
$ fdu --watch --scan-depth 2 .
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
