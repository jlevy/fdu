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
  PERF_RATE: '[0-9]{1,3}(?:,[0-9]{3})* files/s \(\d+\.\d{3} GiB/s\)'
---
# CLI Surface

## A Bare Invocation Is Safe and Shows the Complete Contract

The critical no-argument path is the golden itself: it must print help successfully and
must never infer the current directory.
A unit test separately proves that `--help` produces these exact bytes.

```console
$ fdu
Fastest du replacement, with .gitignore-aware sizes and code and document counts, for the command
line, Python, and Rust

Usage: fdu [OPTIONS] <PATH>...
       fdu [PATH] --cache-status[=<SCOPE>] [--cache-clear[=<SCOPE>]]
       fdu [PATH] --cache-clear[=<SCOPE>]
       fdu --docs
       fdu --skill
       fdu --install-skill [--agent-base <DIR>]

ARGUMENTS
  [PATH]...  Report roots, disjoint; optional only for the discovery and cache-lifecycle flags

SCOPE
      --scan-depth <N>               Limit scanning and retention to N entry levels
      --one-filesystem               Stay on the filesystem the root lives on
      --gitignore-budget <SIZE>      Bytes of .gitignore rules to retain before refusing more files
                                     [default: 4MiB]. Accepts `all`, which also reads each
                                     .gitignore whole
      --gitignore-line-limit <SIZE>  Longest .gitignore line to apply before refusing its file
                                     [default: 16KiB]. Accepts `all`
      --no-gitignore                 Read no .gitignore files: rows lose their gitignored share, and
                                     the snapshot scope differs
      --ignored <MODE>               Ignored population: include, exclude, or only [default:
                                     include]

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
                                [tree default: 5]
  -n, --limit <N>               Maximum data rows per section. Accepts `all` [default: all;
                                largest/recent: 20]
      --breadth <N>             Maximum children per directory. Accepts `all` [default: all]
      --min-share <PERCENT>     Minimum percentage of the selected root measure [tree default: 1%]
      --full                    Show all rows: --depth=all --breadth=all --limit=all --min-share=0%.
                                Explicit bounds override this shorthand
      --sort <KEY>              Order results: size, count, mtime, name, or a requested metric such
                                as `code_lines`
      --reverse                 Reverse the ordering
      --size <METRIC>           Which size metric to report: allocated or apparent [default:
                                allocated]

VIEWS
      --view <LIST>         Views: list, tree, files, extensions, types, families, languages, code,
                            documents, largest, recent, summary, or full. Defaults to list with no
                            analysis, otherwise to a view that displays the requested analysis. code
                            and documents read file contents: each runs its analyzer
      --words-per-page <N>  Logical words per derived document page [default: 250]

CONTENT ANALYSIS
      --analyze <LIST>  Analyzers to run beyond what the views imply: none, lines, code, words, or
                        all [default: none]
      --workers <N>     Content-analysis workers; zero selects available parallelism [default: 0]

OUTPUT
      --format <FORMAT>  Format: tree (list default), paths, long (size/age/path), json, jsonl,
                         yaml, or automatic text [default: text]
      --tree             Display the list as the default directory tree
      --long             Display flat matching paths with size and modification age
      --color <WHEN>     Colorize human output: auto, always, or never [default: auto]
      --progress <WHEN>  Show a progress line on stderr while a report runs: auto, always, or never.
                         Drawn only at an interactive terminal: always, unlike --color, never draws
                         into a pipe or file [default: auto]
      --bar-size <N>     Width of tree bars in cells (at most 4096); zero or a negative value hides
                         the bar [default: 10]
  -q, --quiet            Suppress notes, tips, performance summaries, and progress; keep warnings
                         and errors

EXECUTION
      --cache <POLICY>  Cache policy: auto (where it pays for this run), on, or off [default: auto]
      --stale-ok        Answer from the cached snapshot without scanning; the answer may be stale
      --allow-partial   Accept operationally partial results, including filesystem or analysis
                        failures
      --watch           Stream changes continuously instead of returning one report
      --interval <DUR>  How often aggregate views re-render while watching, as a duration [default:
                        2s]

DELIVERY
      --cache-dir <DIR>  Cache directory; overrides `FDU_CACHE_DIR` and the platform default

CACHE MANAGEMENT
      --cache-status[=<SCOPE>]  Report cache contents instead of scanning: root (default) or all
      --cache-clear[=<SCOPE>]   Remove cached snapshots instead of scanning: root (default) or all

OTHER
  -h, --help              Print help
  -V, --version           Print version
      --docs              Print setup, common commands, cache behavior, and the complete usage guide
      --skill             Print a portable agent skill to stdout
      --install-skill     Write that skill under the project root, to .agents/skills/fdu/ and
                          .claude/skills/fdu/
      --agent-base <DIR>  With --install-skill, write DIR/skills/fdu/SKILL.md instead: one agent's
                          user scope, such as ~/.claude

Agent setup:
  uvx --no-build fdu@latest --install-skill

Examples:
  fdu .                         directory sizes (metadata only)
  fdu . --view=code,documents   lines of code by language, words by document type
  fdu . --ignored=exclude       omit entries covered by .gitignore
  fdu . --view=summary          one total for the tree
  fdu . --kind dir --include .venv --modified-before 7d --long
  fdu . --kind dir --include node_modules --modified-before 30d --long
  fdu . --kind dir --include target --modified-before 30d --format paths

Run `fdu --docs` for setup, libraries, more commands, cache behavior, and the full usage guide.
? 0
```

## The Portable Skill Is Complete and Names the Build That Wrote It

````console
$ fdu --skill
---
name: fdu
description: >-
  Inspect disk usage and codebases: directory sizes, file counts, recency, file types,
  lines of code by language, and words by document type. Use to find large or stale
  build directories, size up the source and prose in a codebase, or collect structured
  filesystem roll-ups for scripts and agents.
---
<!-- generated by fdu; re-run fdu --install-skill to update -->

# fdu Directory Roll-Ups

This is the complete fdu usage contract; it needs no setup chat or prior session
context.

Use `fdu` to summarize a directory tree without modifying files in that tree.
It reports size, file count, recency, and file kinds for every directory at once, and on
request lines of code by language and words by document type.
It walks the tree on several threads through each platform’s native directory interface,
and on a generated million-entry tree (875,000 files) it finished ahead of `du` and the
seven other disk-usage tools measured on Linux and macOS. Every report requires an
explicit `PATH`, `.` for the current directory; bare `fdu` prints help instead of
scanning. `fdu --docs` prints common commands, cache behavior, and the full usage
contract without a PATH and without scanning.

## Run fdu

Start with the report that answers the question:

```bash
fdu .                          # directory-size tree; metadata only
fdu . --view=code,documents    # code lines by language, words by document type
fdu . --format=json --view=tree --depth=2 --limit=20
```

The tree reads only metadata.
`--view=code,documents` also reads file contents, both analyses in one scan, and caches
the results, so a repeated run reads only changed files.
The third is a shallower tree for a script: `--format=json` (or `jsonl` or `yaml`) gives
exact values under a versioned `schema`, so use it whenever the output is parsed.

Prefer an `fdu` already on `PATH`. When there is none, run the latest release through
`uvx`, which needs no install step:

```bash
if command -v fdu >/dev/null 2>&1; then
  fdu . --format=json --view=tree --depth=2 --limit=20
else
  uvx --no-build fdu@latest . --format=json --view=tree --depth=2 --limit=20
fi
```

`--no-build` makes the fallback use a published wheel instead of compiling Rust.
To keep fdu installed, run `uv tool install --no-build fdu` (later
`uv tool upgrade --no-build fdu`) or `cargo install --locked fdu`.

Generated by fdu `0.4.0[DEV_REVISION]`; if `fdu --version` differs, re-run
`fdu --install-skill` so this file describes the installed command.

More reports, each from one scan:

```bash
fdu . --view=code                          # code overview with population and coverage
fdu . --view=documents                     # words and pages by document format
fdu . --view=code --ignored=exclude        # source overview without ignored content
fdu . --ignored=exclude                    # skip ignored trees and their contents
fdu . --view=summary                       # one total with its ignored share
fdu . --view=languages                     # detected language sizes; metadata only
fdu . --view=families,types,extensions     # three file-kind breakdowns
fdu . --view=largest --limit=10            # ten largest files
fdu . --view=recent --limit=10             # ten most recently modified files
```

`--view` chooses what is printed, and `--analyze` adds what may be read beyond it.
`code` and `documents` are the views that read file contents: they show nothing without
analysis, so naming one runs its analyzer.
Every other view reads only metadata unless `--analyze` names an analyzer, as in these
two controls:

```bash
fdu . --analyze=code --view=languages      # code lines in the language rows
fdu . --analyze=lines --view=languages     # physical lines and raw words by language
```

Without code analysis, language percentages are byte shares; with it, the rows add code,
comment, and blank-line metrics and use code-line shares.
Use `--size apparent` when logical file lengths are wanted instead of allocated bytes.

Several disjoint paths are one report, what each would report added:

```bash
fdu docs src                               # a (total) row, then each root as a row
fdu docs src --view=largest --limit=10     # the ten largest files across both
```

Every size, row, share, and bound is taken over the union, once.
Text prints each path after its root’s label (`docs/guide.md`); JSON keeps paths
relative to their root, sets `root` to null, names the roots in `roots`, and gives rows,
errors, and refusals a `root` index into it.
A tree section over several roots has `tree: null`, a `total` row, and `trees`, one tree
per root, each with its own `remainder` and a `root` index.
Every PATH is a directory: `fdu */` names only the directories here, where `fdu *` stops
at the first file. A root inside another, or the same directory twice, is refused.
`--watch`, `--cache-status`, and `--cache-clear` take one PATH.

## Read the Result and Its Notes

Stdout holds only the result: rows, column headings, and, when several views are shown,
an all-caps header per view, such as `LANGUAGES (3 of 15)` when a row limit hid some.
A grouped row (`families`, `types`, `languages`, `documents`) ends its first line at its
file count; its other measures, such as lines, words, and files not analyzed, are
indented lines under that count, not rows of their own.
Every explanation follows on stderr, one prefixed line each, in this order: `note:` for
what the result covers and how to read it, `warn:` for an operation that failed or an
answer nothing verified, `tip:` for a runnable change, and, after text reports, `perf:`
for the work done and the elapsed time.
Notes are consolidated: one says what totals include, one lists every display limit that
hid rows, and one tip lifts them all:

```text
note: totals include gitignored sizes and descendants
note: display limits: below 1% of root, depth 5
tip: show more: --min-share=0% --depth=all
```

Add the tip’s flags to the same command to see what was hidden, or use `--full` to lift
every display bound.
Display limits never change totals.
Notes also name a percentage denominator other than bytes, with the section it applies
to when several views are shown
(`note: percentages are shares of code lines (CODE), document words (DOCUMENTS)`), and
the code view’s coverage (`note: 15 languages analyzed`,
`note: not analyzed: 2 unsupported`). Machine formats send the same lines to stderr, so
stdout stays parseable; check their structured fields, not the notes, before trusting a
result. Use `--quiet` (`-q`) to hide notes, tips, performance lines, and progress.
Result stdout, warnings, errors, and exit status are unchanged; structured facts are
retained.

The default is `list` in `tree` format, in allocated bytes, largest first, to depth 5.
It shows directory subtrees and file leaves contributing at least 1% of the selected
root. Breadth and total rows are unbounded unless requested; `--depth`, `--min-share`,
`--breadth`, and `--limit` compose independently.
Hidden and ignored entries are included.
`.gitignore` is read to label ignored shares, not to exclude matching entries.
A parenthetical amount such as `(73 MiB gitignored)` is included in the row total.
Directories get a `/` suffix, except `.` and `..`; file names and structured paths do
not change.
Between size and name each row shows its age: how long ago anything it counts
last changed, files, directories, and symlinks alike, never the scanned root’s own time.
Ages read `42s`, `59m`, `23h`, `30d`, `11mo` (months of 30.44 days), and `3y` (years of
365.25 days), floored.
An age is `unknown` where part of the subtree was not listed, and `—` on a row that
counts nothing.
With no filter a directory’s age matches its `--long` age; under a filter
each row ages only what it counts.
`--sort mtime` orders tree rows by this age.
Time bounds take days: `--modified-before 60d`, not `2mo`.

Color is on only when the output is a terminal or `--color=always` is given, so piped
tree bars are plain: `█` for usage and `░` for unused width.
Colored bars use green `█` for non-gitignored usage, green `▓` for gitignored usage, and
green `▒` when classification is unknown; dim green `░` fills unused width.
Zero sizes and percentages below 1% are gray; sizes at least 1 GiB are bold.
Cyan names are bright and bold.
Gitignored directories use regular, nonbold cyan; containing ignored files is not
enough. The directory `/` suffix is gray.
`--bar-size=20` widens tree bars; zero or negative values hide the bar column.
The default is 10 characters; the maximum is 4,096. This does not change structured
output or measurements.

## Complete Inventories and File Search

```bash
fdu . --view tree --full --format json             # full recursive hierarchy
fdu . --kind dir --full --sort name --format json  # directory rows with recursive usage
fdu . --view files --kind file --full --format json # individual regular files
fdu . --kind file --include '*.rs' --full --format paths
fdu . --kind dir --include node_modules --full --long
```

`--full` means `--depth=all --breadth=all --limit=all --min-share=0%`. Explicit bounds
override it, regardless of order.
`--view full` chooses multiple views; `--full` expands whichever views were chosen.
Neither changes scan scope or analysis.

Use these commands for find/fd-style filename searches and complete usage inventories.
fdu includes hidden and ignored content; fd needs `--unrestricted` for that population.
fdu selection is glob-based.
It does not implement find expressions or `-exec`, and its ignore sources differ from
fd’s. Paths output escapes control characters; use structured output for arbitrary
native filenames. Directory rows include descendants and overlap; add
`--view list,summary` for a path-union total rather than summing rows.

A tree’s `remainder` contains recursive `files`, `bytes`, `allocated`, and applicable
`reasons` outside its displayed root-level rows; `null` means nothing is hidden there.
Over several roots the section’s `tree` is null: read `total` and each tree in `trees`,
whose `remainder` is that root’s own.
A displayed directory already represents its whole subtree, including descendants whose
rows were bounded away.
Per-boundary `entries` counts hidden roots, while `files` counts regular files
recursively. Unknown amounts are null.
The text equivalent is one root-level line with the same bar, percentage, and size
columns as the tree rows, a blank age, and `… and N more files`. These columns represent
the remaining share of the selected root.
Unknown size or count stays unknown; an unknown hidden size has no numeric share.
Fully expanded, fully observed trees have no remainder line and no display-limit notes.
Check completeness separately: unreadable directories and scan-depth restrictions still
apply.

## Compose the Request From Six Axes

The six axes separate discovery, measurement, selection, and presentation.
Unsupported combinations fail before scanning.
There are no subcommands: the grammar is always “report on a path”.

| Axis | Question | Options |
| --- | --- | --- |
| Scope | What is scanned and cached? | `PATH...`, `--scan-depth N`, `--one-filesystem`, `--gitignore-budget SIZE\|all`, `--gitignore-line-limit SIZE\|all`, `--no-gitignore`, `--ignored=include\|exclude\|only` |
| Content | Which file bodies are read beyond what the views imply? | `--analyze none\|lines\|code\|words\|all` |
| Selection | Which entries does this query consider? | `--include`, `--exclude`, `--min-size`, `--modified-since`, `--modified-before`, `--kind`, `--depth`, `--min-share`, `--breadth`, `-n/--limit`, `--full`, `--sort`, `--reverse`, `--size` |
| View | Which roll-up is reported? | `--view list,summary,tree,families,types,extensions,languages,code,documents,largest,recent,files`, or `--view full` |
| Format | How is it serialized? | `--format text\|tree\|paths\|long\|json\|jsonl\|yaml`, `--color`, `--progress` |
| Mode | How is work performed? | `--cache auto\|on\|off`, `--stale-ok`, `--cache-dir DIR`, `--watch`, `--workers N` |

Scope determines what is scanned and cached.
In a one-shot report, the ignored population also determines which subtrees and file
bodies may be skipped.
A retained index can answer narrower queries when it holds the required facts.

Work has three layers.
A single unfiltered `--view summary PATH` is the one exact composition that retains only
aggregate tallies and no index, except under `--cache on` and `--stale-ok`, whose
contracts are about the snapshot itself.
Otherwise a snapshot cannot save the walk that request is already doing, so it neither
reads nor writes one.
Reading `.gitignore`, the summary keeps the rules and classifies each entry as it counts
it, so its ignored share needs no index either.
An unfiltered tree with a share floor, such as the default `fdu PATH`, keeps every
directory but only the files large enough to show as rows; other metadata requests
retain the reusable index.
None of them reads regular-file contents.
One-shot metadata reports under `auto` neither load a snapshot, which cannot avoid the
current metadata walk, nor write one; `--cache on` writes one.
Any analyzer, named by `--analyze` or implied by `--view code` or `--view documents`,
opts into a separate content sidecar.
fdu reads eligible files whose requested result is absent or stale; a compatible
repeated run reuses unchanged records.
Coverage is scoped to the analyzers too: an unsupported deeper analyzer leaves byte
metadata visible but does not retain a separate lower-level metric record for that file.

## Pick the View, Then Shape It

- `--view list` (default), with `--format tree` for directory roll-ups, `--format paths`
  for complete flat matching paths, or `--long` for size, age, and path.
- `--view tree` for the directory hierarchy, in machine output too.
- `--view extensions` for the raw-extension breakdown.
  Rows partition the tree and so sum to its total; a derived extension always carries a
  leading dot, and names having none are tallied under the literal `(none)`.
- `--view types` for stable detected file types and exact byte shares.
- `--view families` for code, prose, markup, data, binary, and unknown roll-ups.
- `--view languages` for code-family rows and byte shares from path-only detection.
- `--view code` for source-line totals, coverage, and a language breakdown whose TOTAL
  row includes languages a display bound hid.
  It runs code analysis itself and is the default view for that analyzer.
- `--view documents` for words and pages by document format; it runs words analysis
  itself and is the default view for that analyzer.
- `--view largest` for the 20 largest regular files, and `--view recent` for the 20 most
  recently modified. Both are presets over `files`, not separate machinery: `largest` is
  `files --sort size --limit 20` and `recent` is `files --sort mtime --limit 20`, each
  restricted to regular files, and `--sort` and `--limit` still override them.
- `--view files` for a complete flat listing: every matching entry, in name order.
  One-shot text adds the performance footer described below; use a machine format when
  output is consumed programmatically.
- `--view summary` for one aggregate row.
- `--view full` for the bounded digest of every view but List and Files.
- Several views in one run share one scan: `--view summary,types,families`. Text then
  labels each block with an all-caps header naming its view; a single-view text report
  has no header. Machine formats tag every report with `view` either way.

`--analyze` names a set of analyzers, comma-separated, from `lines`, `code`, and
`words`; `none` and `all` are totals and cannot be combined with anything else.
It runs in union with what the views imply, so `--analyze none --view code` still runs
code analysis. Any analyzer may open eligible files and adds content work beyond the
metadata walk. Compatible cached results prevent unchanged bodies from being reread.

Add `--analyze lines` to stream physical, blank, and nonblank lines and raw word counts.
`--view code`, or `--analyze code` with another view, gives standard LOC, comment, and
code-blank partitions across supported common languages.
The Code table aligns code, comment, and blank lines with analyzed-file coverage for
each language and a bold TOTAL row.
TOTAL includes languages hidden by display bounds, and a note says so.
Each row’s gray parenthetical, such as `(0 gitignored)`, is its gitignored contribution,
already included in the row; coverage follows the table as notes.
Languages retains byte sizes and uses code-line shares when code analysis is enabled.
`--view documents`, or `--analyze words` with another view, gives normalized word
volume, paragraphs, aggregate-derived pages, and reader-visible Markdown that excludes
destinations and code.
The `documents` percentage column is document-word share, and a note says so in text.
`--view code,documents` — like `--analyze code,words` or `all` — computes both in one
streaming pass.
Both include `lines`, so adding it explicitly changes neither the metrics
nor the work. `lines` alone measures physical text volume without code counting or word
normalization.
Unsupported code languages still have physical-line metrics; their SLOC is
unavailable.

Requesting analysis without naming a view selects one that displays it: `code` selects
`code`, `words` selects `documents`, and `lines` selects `families`. `code,words` or
`all` selects both `code` and `documents`. Naming `--view` overrides that.
The converse holds only for the two views with no metadata meaning: `--view code`
implies `code` and `--view documents` implies `words`. Every other view, `full`
included, never enables an analyzer: a display choice with a cheap meaning must not
quietly authorize reading every file.
Headers name views; columns name metrics.
`words` is an analyzer, while `documents` is the prose/markup population.
Use `--analyze=words --view=types` to include word metrics for other text types.
Use `--analyze=lines --view=languages` for physical line counts across code languages.

When the selected views show none or only part of the requested analysis, a note names
what is not shown and a tip gives the views that show it:
`--analyze=code --view=summary` ends with `note: code analysis not shown by summary` and
`tip: show it: --view code`, and `--analyze=code --view=documents` with
`note: code analysis not shown by documents` and `tip: show it: --view documents,code`.
`--view full` includes Code only with code analysis and Documents only with words
analysis, naming the views it skipped and `--analyze all` to include them.
Use `--workers` to bound concurrent reads and `--words-per-page` to control page
derivation. Analysis never truncates a file or excludes it because of size.
One fixed bound changes a method: `words` counts a Markdown file over 64 MiB as plain
text, read whole but not rendered, and says so (`counted as text`, `text_only`, a note).
Invalid UTF-8, binary data, and unsupported SLOC languages remain visible as normal
coverage outcomes. Only I/O failures, files changed during a read, or stale commits make
analysis operationally partial.
Content analysis is one-shot and cannot be combined with `--watch`.

One-shot text reports end with a `perf:` line on stderr.
It reports regular files walked and their represented bytes, ignore files and accepted
rules, actual content bytes read, fresh and cached analysis, the cache tier, and elapsed
time. Total files/s and binary GiB/s use that elapsed time; represented GiB/s is not
content-read bandwidth.
Content-read throughput uses the analysis duration.
Known binary files can contribute walked bytes but zero read bytes.
`--stale-ok` runs report zero walked files because they never consult the tree.
The line is gray only when color is active and has no ANSI escapes otherwise.
Paths, Long, JSON, JSONL, YAML, skill output, lifecycle output, and watch streams omit
it.

A progress line can appear on stderr for a person at a terminal.
It is never drawn when stderr is not a terminal, `TERM` is `dumb`, or `CI` is set, so
agents need no flag; `fdu --docs` states the full rule.

Common shapes are compositions rather than dedicated flags:

```bash
fdu --view largest -n 100 PATH                        # the 100 largest files
fdu --kind file --modified-since 2h PATH              # files changed in the last two hours
fdu --view files --include '*.{rs,toml}' PATH         # by pattern
fdu --view tree --sort mtime PATH                     # an activity map
```

Tree output defaults to depth 5 and a minimum share of 1% of the selected root size.
Significant files appear alongside directories.
`--depth`, `--min-share`, `--breadth`, and `--limit` compose: depth bounds levels, share
hides smaller branches, breadth caps children per directory, and limit caps data rows
per section. Breadth and limit default to `all`; largest and recent retain their 20-row
presets. A `note: display limits:` line names each bound that hid rows, and one
`tip: show more:` names the flags that lift them; neither changes totals.

Use `--full`, or `--min-share=0% --depth=all --breadth=all --limit=all`, to remove every
display bound. Numeric content sorts such as `--sort=code_lines` require their analyzer.
Extensions supports metadata sorts only; use a metric-capable view for content ranking.
`--scan-depth` changes what is scanned and retained; display bounds only shorten output.

## Find Environments and Build Outputs

List all matching directories, then tally the covered paths from the same snapshot:

```bash
fdu PATH --kind dir --include .venv --include node_modules --include target --long --cache on
fdu PATH --kind dir --include .venv --include node_modules --include target --view summary --stale-ok
fdu PATH --kind dir --include .venv --include node_modules --include target --view files,summary --sort size --format json
```

The second command reads the snapshot the first one left with `--cache on`, without
revalidating it. Keep the root, cache destination, and scan population the same.
The third command combines flat rows and Summary in one machine report.
Ignored directories are included by default.

To select old environments by modification activity:

```bash
fdu PATH --kind dir --include .venv --modified-before 7d --long
fdu PATH --kind dir --include node_modules --modified-before 30d --long
fdu PATH --kind dir --include target --modified-before 30d --format paths
fdu PATH --kind dir --include .venv --include venv --include node_modules --include target --modified-before 30d --long --sort mtime --reverse
fdu PATH --kind dir --include .venv --modified-before 30d --format json
```

Kind, basename/relative-path patterns, size, and modification age are filters.
Repeated includes form a union.
Directory bytes sum eligible regular-file contents; recency is the newest root or
eligible descendant mtime, including directories and symlinks.
Empty directories use their own mtime.
This is modification activity, not access or last use.
Directory names such as Cargo’s `target` are conventions, not proof of ownership.
Allocated bytes are counted per path; separate hard links are not deduplicated and
clones can share physical blocks.
Windows currently reports apparent size as allocated.
These measurements do not estimate space freed by deletion.
Symlinks are not followed.

Exclusions win throughout selected subtrees before size/age bounds; ignored-only queries
traverse structural ancestors.
Flat output lists matching entries, including nested roots whose sizes overlap.
Aggregate views count the covered union once.
The default is the directory tree; `--tree` makes its format explicit.
List’s flat formats are complete and size-ranked by default; Files retains name order
unless `--sort` changes it.
A row limit caps each section, including a tree; breadth caps each directory and depth
bounds displayed levels.
None changes measured totals.
Paths escapes control characters only and keeps stdout to paths; bound and rule notices
go to stderr. Long adds size and signed age.
Machine rows retain exact `mtime_ns`, `age_ns`, directory `files`/`dirs`, and the
report’s `age_reference_ns`; unknown ages are null.
Tree nodes carry the same `mtime_ns`, `complete`, and `age_ns` as the text column,
beside `newest_mtime_ns`, the newest regular file alone.
`modified_at` and the envelope’s `age_reference_at` are the same instants in RFC 3339
UTC; `modified_at` is null wherever the time is only a lower bound.
A `--watch` repaint measures its ages from its own instant.

Tree/Paths/Long require one compatible list view.
Use automatic Text or machine output for grouped/mixed views and Full.
Largest/recent retain regular-file ranks with Paths or Long.
Explicit Paths/Long overrides the Tree view’s presentation.
Format flags conflict.
Rust/Python callers select format on the query before reading; a detached Report cannot
turn a folded tree into a complete flat inventory.
Request another report from the retained index for that change, without scanning again.

## Read What `.gitignore` Covers

Fresh scans read applicable per-directory `.gitignore` files by default.
`--stale-ok` reports use retained rule state, and `--no-gitignore` disables the rules.
Summary, tree, and extension rows show ignored size as a gray parenthetical such as
`(128 B gitignored)` when color is enabled.
The performance line counts ignore files and accepted rules.

```bash
fdu PATH --ignored=exclude                              # folders by what the rules leave
fdu PATH --view=files --ignored=only --format=jsonl     # every entry the rules cover
fdu PATH --no-gitignore                                 # read no rules, show no share
```

Selecting a side changes sizes, ordering, and `--min-size` together, because they follow
the entries shown. `--ignored=exclude` avoids enumerating safely ignored subtrees and
reading ignored bodies.
`--ignored=only` traverses the directories needed to discover ignored entries and
analyzes only ignored bodies.
The default `include` measures both populations and reports their contributions
separately. Unknown classifications cannot justify pruning.
`--no-gitignore` with either selection is a usage error.
Only per-directory `.gitignore` files apply, not `core.excludesFile`,
`.git/info/exclude`, or a global ignore file.
Each is found as git opens it, so a `.GITIGNORE` counts on a case-insensitive volume;
matching itself is case-sensitive.
Unignored does not mean tracked: `.git` is unignored unless a rule names it.
For recent working files, add both `--ignored=exclude` and `--exclude='.git/**'`. An
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
fdu --view files --kind file --format jsonl --modified-since <that> PATH
```

Use the scan’s *start*, not its end: a file modified mid-scan may have been observed
before the modification, so only the start bound is conservative.

## Validate Every Automated Result

Check the process exit status and these fields:

- `schema` before parsing anything else: a report carries `fdu.report/11`, a `--watch`
  stream carries `fdu.stream/2`, and `--cache-status` carries `fdu.cache/3`. Treat an
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
`--stale-ok` can return `provenance.freshness: stale`, and it says so rather than
implying currency: every format also prints a `warn: stale answer` line on stderr, which
`--quiet` keeps. It fails outright when no usable snapshot exists rather than silently
scanning.

Exit 0 is accepted success, exit 1 is a fatal failure, and exit 2 is incomplete data or
invalid usage. Do not discard useful stdout from exit 2; inspect the completeness fields
and use `--allow-partial` only when incomplete totals are acceptable.

## Cache Behavior

No ordinary view requires a preexisting cache.
`--cache auto`, the default, uses the cache only where the kind of run gains from it.
Metadata-only one-shot reports include current sizes or timestamps, so they must inspect
every entry; under `auto` they neither load a snapshot, which cannot make that
verification cheaper, nor write one, which no later report reads.
Content analysis, `--watch`, and a retained index from the Rust or Python `open` read,
revalidate, and write it.
`--cache on` also writes after a one-shot report, which is how to leave a snapshot for a
later `--stale-ok` answer.

Content analysis is where ordinary repeated runs benefit most.
The first compatible run reads eligible bodies; a later run restores unchanged records
from the content sidecar and reads only changed or newly eligible files.
A stored analyzer set answers only the same set: a different one, wider or narrower,
reads the files again and replaces it.
The performance footer reports fresh and cached analysis separately.

`--stale-ok` is a distinct contract: it never verifies the source tree, labels the
answer stale, and fails unless compatible metadata and any requested content analysis
already exist. `--cache=off` neither reads nor writes fdu cache data.

The cache defaults to `~/.cache/fdu` on macOS and Linux, and `%LOCALAPPDATA%/fdu` on
Windows. Set an exact destination with `--cache-dir DIR` or `FDU_CACHE_DIR`; the flag
wins. Otherwise `XDG_CACHE_HOME/fdu` overrides the platform default.
Status and clear use the same destination.

Each root has a `<16-hex-key>.metadata.bin` filesystem snapshot and, when analyzed, a
matching `<16-hex-key>.analysis.bin` file of derived metrics.
The key identifies the canonical root.
Analysis files store counts and classifications, not copies of source bodies.
These binary files are disposable; use cache status to inspect them.
`--cache-status` maps a hash-named file back to the tree it describes, and
`--cache-clear` removes it; both run without scanning.
Cache status is its own document, carrying the `fdu.cache/3` schema in every machine
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
fdu — the fastest du replacement, with .gitignore-aware sizes and code and
document counts, for the command line, Python, and Rust.

  For every directory in a tree at once, fdu reports its size, file count,
  recency, and file kinds, and on request lines of code by language and words
  by document type.
  It walks the tree on several threads through each platform's native
  directory interface, and on a generated million-entry tree (875,000 files)
  it finished ahead of du and the seven other disk-usage tools measured on
  Linux and macOS.
  Content metrics are cached between runs, and every capability is also a
  Rust and Python API.
  Performance measurements: https://github.com/jlevy/fdu/blob/main/docs/performance-measurements.md

SET UP WITH ANY CODING AGENT
  Install fdu's self-contained skill for current and future agent sessions:

    uvx --no-build fdu@latest --install-skill

  Run it from the project root. The generated SKILL.md needs no prior session
  context. If fdu is on PATH, `fdu --install-skill` is equivalent.

INSTALL THE COMMAND LINE
  Run the latest release once, or keep it on PATH:

    uvx --no-build fdu@latest PATH
    uv tool install --no-build fdu
    fdu PATH

  The wheel requires GIL-enabled Python 3.12 or newer. uv normally selects a
  matching interpreter; if it selects a free-threaded build such as 3.14t,
  add `--python 3.14` to the uvx or uv tool command.

USE AS A LIBRARY
  Python: `uv add fdu` (or `pip install fdu`)
  Rust:   `cargo add fdu` (or `cargo add fdu-core` for the engine alone)

START HERE
  A report requires a PATH. Use `.` for the current directory.

    fdu .                                      directory sizes (the default)
    fdu . --view=code,documents                lines of code and words, one scan
    fdu . --view=code                          standard lines of code by language
    fdu . --view=documents                     words and pages by document format
    fdu . --ignored=exclude                    omit entries covered by .gitignore
    fdu . --view=summary                       one total for the tree
    fdu . --view=languages                     languages by byte size
    fdu . --view=families,types,extensions     three file-kind breakdowns
    fdu . --view=recent --limit=10             ten most recently modified files
    fdu docs src                               several paths as one report, with a total

  `fdu .` is metadata-only. It prints a tree in allocated bytes, largest first,
  to depth 5, showing contents with at least 1% of the selected root size, each
  with its age: how long ago anything it counts last changed. Hidden
  and gitignored entries are included; .gitignore is read to label gitignored shares, not to exclude them.

  Several paths are what each would report, added: every size, row, share, and
  bound is over the union, once. The tree starts with a (total) row, each root
  is a row named as given, and flat paths are printed after their root's label.
  Every PATH is a directory; `fdu */` names only the directories here, and is
  refused where one is a symlink to another (lib64 -> lib). Each root is walked
  separately and pays a walk's fixed cost: 625 small roots took 1.72 s, one
  walk of their parent 0.23 s, about 7x (exp-216, macOS, uncontrolled host).
  One walk shows every entry one level down: `fdu --depth 1 --min-share 0% .`
  A root inside another, or the same directory twice, is refused (exit 2).
  --cache-status and --cache-clear take one PATH, as does --watch.

  code and documents read file contents; --analyze is the extra control for
  analysis a view does not imply:

    fdu . --analyze=code --view=languages      code lines in the language rows
    fdu . --analyze=lines --view=languages     physical lines and raw words by language

VIEWS AND ANALYSIS
  --view chooses the question the report answers. Several views share one scan
    and one analysis; adding a view does not run a second scan.
  code and documents are the views that read file contents: they show nothing
    without analysis, so naming one runs its analyzer, code or words. Every
    other view, full included, opens no regular file on its own.
  --analyze runs analyzers beyond what the views imply: code lines in the
    languages rows, physical lines in families and types, or a run that only
    warms the cache. Compatible cached results avoid rereading unchanged bodies,
    so a repeated content analysis can be much cheaper.

  Naming analyzers alone selects a view that displays them: code selects code,
  words selects documents, code,words selects both, and lines selects families.
  Name --view for a different projection; it always wins. Headers name views;
  columns name measurements: words is an analyzer, and documents is its view of
  prose and markup. --analyze words --view types counts words in every text type.

  A view with a metadata meaning never turns on an analyzer, because choosing how
  to look at a result should not quietly authorize reading every file in the
  tree. If a selected view cannot display requested analysis, fdu still performs
  the analysis and prints a note. --view=full names any view it had to skip.

MORE COMPOSITIONS
  fdu ~/Downloads --view=extensions
  fdu . --view=types,families --format=json
  fdu . --analyze=words --view=types
  fdu PATH --view tree --full --format json                 complete recursive tree
  fdu PATH --kind dir --full --format json                  recursive directory totals
  fdu PATH --kind file --full --format paths                find/fd-style file inventory
  fdu PATH --view=largest --limit=100                        the 100 largest files
  fdu PATH --view=files --kind=file --modified-since=1h      files changed lately
  fdu PATH --view=files --ignored=only --format=jsonl        what .gitignore covers
  fdu --watch --view files --format jsonl PATH              a tail -f for a tree

  --interval throttles rendering only; change detection is event-driven and
  unaffected by it, so an idle tree costs nothing between changes.
  The duration uses the age grammar: `2s`, `200ms`, `1h30m`.

  largest and recent are presets over files, not more views to learn:
    largest = files --sort size --limit 20, regular files only
    recent  = files --sort mtime --limit 20, regular files only
  --full expands to --depth=all --breadth=all --limit=all --min-share=0%.
  Explicit bounds override it regardless of order. It leaves view, scan scope,
  population, and analysis unchanged. --view=full chooses views instead.
  --sort and --limit still override them. files alone is complete: every
  matching entry, in name order. full combines applicable views, without list/files.

LIST FORMATS AND OLD BUILD DIRECTORIES
  The metadata default view is list; its default format is tree. These agree:
    fdu PATH
    fdu PATH --view list --format tree
  Tree shows directories and significant files to depth 5. --min-share 1% compares
  every row to the selected root total. --min-share 0% shows even zero-size rows.
  --breadth caps children per directory; --limit caps data rows per section.
  Both default to all. --depth all expands levels. Omission notes name each bound.
  Display bounds never change totals or limit the filesystem scan.
  A tree row's age is its newest counted change, of files, directories, and symlinks;
  unknown where a subtree was not listed. Ages read s, m, h, d, mo (30.44 days), and
  y (365.25 days); time bounds take days. --sort mtime orders rows by that age.

  --format paths gives matching paths only; --long adds size, age, and path.
  Flat lists are complete and size-ranked by default; --sort name lists by name.
  JSON, JSONL, and YAML give exact metrics. text keeps automatic human tables.
  Tree/paths/long require a single list view; use text or machine formats for
  grouped/mixed views and full. largest/recent accept paths/long, keeping file ranks.
  files keeps name order; the tree view keeps structured tree output.
  Explicit paths/long overrides the tree presentation. Format flags conflict.

  fdu PATH --kind dir --include .venv --modified-before 7d --long
  fdu PATH --kind dir --include node_modules --modified-before 30d --format long
  fdu PATH --kind dir --include target --modified-before 30d --format paths
  fdu PATH --kind dir --include .venv --include venv --include node_modules --include target --modified-before 30d --long --sort mtime --reverse
  fdu PATH --kind dir --include .venv --modified-before 30d --format json

  Kind, name/path, size, and age are filters. Repeated includes form a union.
  Directory sizes sum eligible regular-file contents, excluding inode/symlink bytes.
  Age uses the newest modification of the root or an eligible descendant, including
  directories and symlinks. Empty directories use their own time; future age is negative.
  This is modification activity, not access or last use. target is a naming convention.
  Exclusions win throughout the subtree before size/age filtering. Nested matching
  roots can overlap; aggregate views count the covered contents once. --size apparent
  selects logical bytes. Paths/long omit the footer and send bound notices to stderr.

SIX AXES, AND EVERY OPTION BELONGS TO EXACTLY ONE
  Scope      PATH..., --scan-depth, --one-filesystem    what is scanned and cached
             --gitignore-budget, --gitignore-line-limit, --no-gitignore, --ignored
  Content    --analyze none|lines|code|words|all        which file bodies are read
                                                        beyond what the views imply
  Selection  --include, --exclude, --depth, --limit     which entries are considered
             --breadth, --min-share
  View       list,summary,tree,families,types,extensions,languages,code,documents,
             largest,recent,files,full
  Format     --format text|tree|paths|long|json|jsonl|yaml, --tree, --long
             --color, --progress
  Mode       --cache, --watch, --workers

CONTENT ANALYSIS
  none       no analyzer beyond what the views imply (default)
  lines      physical, blank, and nonblank lines plus raw word counts
  code       standard SLOC from the versioned common-language analyzer
  words      normalized and reader-visible word volume
  all        every shipped analyzer

  A comma-separated set: code,words runs both. none and all name the whole
  axis and cannot be combined. code and words already include lines; adding lines
  explicitly changes neither measurements nor work. lines alone measures physical
  text volume without language-specific code counting or word normalization.
  languages is metadata-only by default; --view code runs code analysis itself.
  Code reports show source lines, language shares, population columns, and coverage.
  --view documents runs words analysis itself.
  Analysis streams every eligible file through EOF; files are never size-truncated.
  --workers bounds content-analysis concurrency; directory scanning uses its own pool.
  --words-per-page changes only report-time page derivation.
  Unchanged results are restored from a separate sidecar written by the same
  analyzer set; any other set, wider or narrower, reads the files again.
  --stale-ok never opens source files and fails if requested content is absent.

CACHE BEHAVIOR
  --cache=auto, the default, uses the cache only where the kind of run gains from
  it. A metadata report neither reads nor writes one: checking a snapshot costs as
  much as the scan it would save, and no later report reads it. Content analysis
  and long-lived sessions read, revalidate, and write it. --cache=on also writes
  after a one-shot report, leaving a snapshot for a later --stale-ok answer.

  Content analysis is where repeated-run caching pays most. The first run reads
  eligible file bodies. A compatible later run reuses results for unchanged files
  and reads only changed or newly eligible bodies; the performance footer reports
  fresh and cached analysis separately. Repeat a --view=code run to see it.

  --stale-ok answers from the snapshot alone: it does no filesystem verification,
  requires a compatible snapshot and content sidecar for the requested analysis,
  and labels its answer stale with a warn: line on stderr that --quiet keeps.
  --cache=off neither reads nor writes fdu's cache.

  macOS and Linux default to ~/.cache/fdu; Windows uses %LOCALAPPDATA%/fdu.
  --cache-dir overrides FDU_CACHE_DIR, then XDG_CACHE_HOME/fdu and native defaults.
  Each root has <key>.metadata.bin and optional <key>.analysis.bin. The latter
  stores derived metrics, not source bodies. Status and clear use the same directory.

IGNORE RULES
  Fresh scans read applicable .gitignore files by default; --stale-ok uses retained
  rules. Summary, tree, and extension rows show gitignored size as `(128 B gitignored)`.
  Ignoring a directory covers its descendants. Non-gitignored does not mean Git-tracked:
  .git is non-gitignored unless a rule names it. --ignored=include is default.
  --ignored=exclude prunes safely gitignored subtrees and skips gitignored body reads.
  --ignored=only discovers gitignored matches through ordinary ancestors, reading
  only gitignored bodies for analysis. Sort and --min-size follow the size shown.
  --no-gitignore reads no rules and shows no share. Only per-directory .gitignore
  files apply, not core.excludesFile, .git/info/exclude, or a global ignore file.
  Each is found as git opens it, so a .GITIGNORE counts on a case-insensitive
  volume; matching itself is case-sensitive on every platform. An unreadable
  .gitignore makes the result partial, like any unreadable path. A .gitignore past
  --gitignore-budget or --gitignore-line-limit is refused whole and named in a note:
  sizes stay exact, gitignored shares under that directory do not.

OUTPUT AND AUTOMATION
  Every machine report uses fdu.report/11; watch changes use fdu.stream/2.
  Cache status is its own document in every machine format: fdu.cache/3.
  Summary, tree, extension, and file rows carry `ignored`: null under --no-gitignore.
  Text language rows use canonical names; machine formats retain lowercase IDs.
  Metric rows include detection source, confidence, origin flags, and coverage.
  Tree remainder totals are shared by every format: recursive files, apparent and
  allocated bytes, and applicable reasons. Null means nothing hidden; unknown counts
  or sizes stay null. Text shows one root-level row, `… and N more files`, with the
  hidden share and size in the tree's columns.
  Text results hold only rows, column headings, and multi-view headers. Human
  diagnostics use note:, warn:, tip:, and perf: on stderr, in that order; one
  `tip: show more:` names the flags that lift every display limit that hid rows.
  One-shot text reports end with gray perf: on stderr; machine formats omit it.
  It counts ignore files and accepted rules, including repeated governing sources.
  Total files/s and binary GiB/s use the displayed elapsed duration. GiB/s represents
  walked file size; actual body-read throughput is reported separately.
  JSON numbers above 2^53 (fingerprints, option hashes, nanosecond timestamps)
  lose precision in IEEE 754 binary64 parsers such as JavaScript JSON.parse.
  Results go to stdout; all diagnostics, including warnings and errors, go to stderr.
  The command never prompts or pages. A progress line is drawn on stderr only for a
  person at an interactive terminal; --progress never draws into a pipe, a file, or CI.
  Reports require an explicit PATH; bare `fdu` prints help and scans nothing.
  `fdu --install-skill` writes a portable agent skill describing this same surface
  under the project root, and `fdu --skill` prints it.

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
fdu 0.4.0[DEV_REVISION]
? 0
```

## An Explicit Current Root Works for an Empty Sandbox

```console
$ fdu --cache off --color never --size apparent .
░░░░░░░░░░      —         0 B  —  . 0 files
! note: root size is zero, so shares are undefined
! tip: show more: --min-share=0%
! perf: took [PERF_TIME] to walk 0 files (0 B) at [PERF_RATE]; 0 gitignore rules (0 files); content read 0 B; analysis 0 fresh, 0 cached; cold scan
? 0
```

## Unknown Options Are Usage Errors on Stderr

```console
$ fdu --definitely-not-an-option
! error: unexpected argument '--definitely-not-an-option' found
!
!   tip: to pass '--definitely-not-an-option' as a value, use '-- --definitely-not-an-option'
!
! Usage: fdu [OPTIONS] <PATH>...
!        fdu [PATH] --cache-status[=<SCOPE>] [--cache-clear[=<SCOPE>]]
!        fdu [PATH] --cache-clear[=<SCOPE>]
!        fdu --docs
!        fdu --skill
!        fdu --install-skill [--agent-base <DIR>]
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

## Watching Keeps One Root

A watch keeps one tree current, so a second PATH is refused before anything is read.

```console
$ node -e "for (const d of ['one', 'two']) require('node:fs').mkdirSync(d)"
? 0
```

```console
$ fdu --watch one two
! fdu: --watch takes one PATH; 2 were given
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
! fdu: plain-file is a file; fdu reports on directories (to name only the directories here: fdu */)
? 1
```

## The Skill Installs Where Agents Look, and a Rerun Changes Nothing

`--install-skill` writes the document `--skill` prints to `.agents/skills/fdu/` and
`.claude/skills/fdu/` under the git root of the current directory, or under the current
directory itself when no ancestor is a repository.
The sandbox is made a repository root first, so the install lands here whatever lies
above the temporary directory.
Each file is reported once, and a second run finds nothing to do.

```console
$ node -e "require('node:fs').mkdirSync('.git')"
? 0
```

```console
$ fdu --install-skill
installed .agents/skills/fdu/SKILL.md
installed .claude/skills/fdu/SKILL.md
? 0
```

```console
$ fdu --install-skill
unchanged .agents/skills/fdu/SKILL.md
unchanged .claude/skills/fdu/SKILL.md
? 0
```

## A Skill fdu Did Not Generate Is Refused, Not Overwritten

Only a file carrying fdu’s own marker is ever replaced.
A `SKILL.md` written by hand at the same path is a usage error, and nothing is written,
not even the other file.

```console
$ node -e "require('node:fs').writeFileSync('.claude/skills/fdu/SKILL.md', '---\nname: fdu\n---\n# Written by hand\n')"
? 0
```

```console
$ fdu --install-skill
! fdu: refusing to overwrite .claude/skills/fdu/SKILL.md: fdu did not generate it (no `<!-- generated by fdu` marker); move it aside, then re-run fdu --install-skill
? 2
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
