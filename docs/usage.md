# Using fdu

fdu scans one path and can answer several questions about it: directory sizes, totals,
recent or large files, file kinds and languages, physical lines, standard lines of code,
and prose volume. The command keeps the scan, analysis, selection, and presentation
choices separate so each option has one meaning.

## Start with the Current Directory

```shell
fdu .
```

This is the default report:

- `list` view in `tree` format
- allocated filesystem bytes
- largest entries first
- up to five entry levels, including significant file leaves
- entries contributing at least 1% of the selected root size
- no breadth or section-row cap
- metadata only; regular file bodies are not opened
- hidden and ignored entries included

fdu reads per-directory `.gitignore` files by default so it can annotate ignored byte
shares. Reading the rules does not exclude the entries they match.

These commands cover the most common questions:

```shell
fdu . --ignored=exclude
fdu . --view=summary
fdu . --view=languages
fdu . --view=families,types,extensions
fdu . --view=recent --limit=10
fdu . --analyze=lines
fdu . --analyze=lines --view=languages
fdu . --analyze=code
fdu . --analyze=words
```

## Choose a View

`--view` is a comma-separated list.
Multiple views share one scan and one requested content analysis; they do not cause
repeated filesystem walks.

| View | Answer |
| --- | --- |
| `list` | Matching entries; directory tree by default, flat paths or details on request |
| `tree` | The directory hierarchy, in structured output too |
| `summary` | One total for the selected tree |
| `families` | Broad code, prose, markup, data, binary, and unknown groups |
| `types` | Detected file types |
| `extensions` | Raw filename extensions; extensionless names use `(none)` |
| `languages` | Programming languages, metadata-only unless code analysis is enabled |
| `code` | Source-line overview, language and population totals, and coverage; requires code analysis |
| `documents` | Prose metrics; requires an enabled analyzer |
| `largest` | Twenty largest regular files by default |
| `recent` | Twenty most recently modified regular files by default |
| `files` | Every selected entry in name order |
| `full` | Every applicable view but `list` and `files` |

`largest` and `recent` are presets over `files`. `--sort` and `--limit` override their
defaults. Use `--limit=all` where a bounded view should print every row.
`--depth` limits reported tree levels, not scan depth; `--scan-depth` limits what can be
scanned and therefore changes the cache scope.

Numeric content sorts such as `--sort=code_lines` require their analyzer.
Use `files`, `list`, `tree`, or a metric grouping such as `languages` or `code` for
these rankings. `extensions` groups metadata only and rejects content-metric sorts.
The Code overview shows combined language totals with non-ignored and ignored
contributions in parentheses; unknown classification is separate.

Sizes use allocated bytes by default.
Add `--size=apparent` for logical file lengths.

The percentage column in grouped views normally shows byte share.
When code analysis is shown by language, it shows code-line share instead.
In `documents`, it shows document-word share.
Text output labels those two non-byte denominators; machine output always carries the
exact `share_metric`, numerator, and denominator.

## Choose a Format

The default output is the directory tree: `fdu .`, `fdu . --view list`, and
`fdu . --format tree` print the same bounded directory roll-ups.
Significant files appear as leaves alongside directory totals.
Each tree row shows a share bar, percentage of the selected root, size, then its
indented name. The single remainder row, when present, uses those same columns for all
hidden descendants together and ends `… and N more files`. Unknown quantities are
labeled unknown rather than estimated.
`--bar-size=20` widens the bar; `--bar-size=0` or a negative value hides it.
The default is 10 characters.
This affects human trees only.

| Format | List output |
| --- | --- |
| `tree` or `--tree` | Directories and significant files; default depth 5 and 1% of selected root size |
| `paths` | Every matching path, safely escaped, one per line |
| `long` or `--long` | Every match with its size (allocated unless `--size` says otherwise), modification age, and path |
| `json`, `jsonl`, `yaml` | Structured matching entries with exact metrics and bounds |
| `text` | Automatic human presentation: tree for List, tables for grouped views |

Flat lists use size-descending order with deterministic path ties; `--sort name` gives
an alphabetic inventory.
`--sort mtime --reverse` puts oldest entries first.
`--limit N` caps data rows per section, including the root in a tree.
`--breadth N` caps immediate children per directory.
Both default to `all`, except the 20-row largest/recent presets.
`--depth all` expands all levels and `--min-share 0%` admits all sizes.
Display bounds compose and leave aggregate measurements unchanged.
On a partial scan, verified small files and complete subtrees still obey the share
threshold against the observed root total.
Incomplete subtrees remain visible because their unseen contents could be significant; a
note explains this exception.
An unrelated scan error does not disable pruning.

Shares compare exact values against the selected root total, including threshold
equality. A child at 1% of its parent but below 1% of the root is hidden by the default.
Omission rows name share, breadth, depth, or row limits; they do not consume data rows.
`--limit 0` shows no data rows, while `--depth 0` shows only the root.
Explicit hierarchy controls require a hierarchical view; `--scan-depth` independently
limits discovery. Paths and Long omit the performance footer; bounds, rule coverage,
cache-only status, and watch invalidations are reported on stderr.
Paths is a lossy line-oriented listing: control characters are escaped so one row stays
one line, undecodable bytes become U+FFFD, and every other character, the platform
separator and a literal backslash included, is written verbatim.
Use machine output for exact native path identity when names contain undecodable bytes.

Tree, Paths, and Long require one compatible list view; grouped/mixed views and Full
accept automatic Text and machine formats.
Largest/recent keep regular-file ranking and support Paths and Long; use List with Tree
for directory roll-ups.
Conflicting format flags are usage errors.
Files keeps name ordering in flat formats; explicit Tree uses tree ordering.
The Tree view keeps its structured hierarchy output; explicit Paths/Long overrides it.
Full keeps its bounded digest and does not acquire an unbounded listing.

## Select Entries

Selection changes which retained entries contribute to a report.
It does not narrow the scan or cache scope.

Useful selections include:

```shell
fdu . --include='*.{rs,toml}'
fdu . --exclude='target/**'
fdu . --min-size=10MiB
fdu . --kind=file --modified-since=1h --view=files --sort=mtime
fdu . --kind=file --view=largest
```

Quote glob patterns so the shell does not expand them before fdu sees them.
Size and time bounds test a directory’s eligible subtree, not its inode, and a directory
they match covers its contents in aggregate views.
Without `--kind`, `fdu . --modified-since 7d` therefore lists every directory with
activity this week at its full size beside the files that changed, and `--view summary`
counts everything inside those directories; `--kind file` asks only for the files.

### Find Old Environments and Build Directories

Directory kind is a filter, just like name/path and modification time:

```shell
fdu ~/projects --kind dir --include .venv --modified-before 7d
fdu ~/projects --kind dir --include node_modules --modified-before 30d --long
fdu ~/projects --kind dir --include target --modified-before 30d --format paths
fdu ~/projects --kind dir --include .venv --include venv \
  --include node_modules --include target --modified-before 30d \
  --format long --sort mtime --reverse
fdu ~/projects --kind dir --include .venv --modified-before 30d --format json
```

Repeated includes form a union.
A pattern without a slash matches a basename; one with a slash matches the path relative
to the scan root. These names are conventions: `target` is Cargo’s default build
directory, but its name alone does not prove ownership.

Directory size sums eligible regular-file contents, excluding directory-inode and
symlink bytes. Its modification time is the newest timestamp on the root or an eligible
descendant, including directories and symlinks; an empty directory uses its own time.
Long displays age relative to one request instant (`30d`, `2h`, or a negative age for a
future timestamp). This measures modification activity, not access time or last use.
`--size apparent` switches to logical file lengths without changing matching semantics.

Exclusions apply throughout a matching directory’s subtree before size/age predicates.
Excluding a directory excludes its contents; a matching parent cannot bring them back.
`--ignored=exclude` subtracts ignored contents, while `--ignored=only` still traverses
structural unignored ancestors to discover ignored matches.
Matching a directory covers its eligible contents in summary and grouped views without
requiring descendant names to match.
Flat output emits matching entries only.
Nested matching roots are all shown, so their sizes can overlap; summary/grouped totals
count the covered union once.
The scan root provides context and is not itself a selectable descendant.
Hard links and shared extents are counted once for each path; sizes do not promise
uniquely reclaimable space.
Symlinks are never followed.

A directory whose subtree was not listed in full, at a `--scan-depth` boundary or in a
scan that finished with errors, reports a lower-bound size and an `unknown` age in
`long`, carries `complete: false` in machine output, and matches no modification bound.
See the [machine-output reference](machine-output.md) for exact size, age, timestamp,
and completeness fields.
A retained index can answer narrower selections when its scope contains the needed
facts. One-shot ignored-population choices determine discovery and cache identity.

### Select by `.gitignore`

```shell
fdu . --ignored=exclude
fdu . --view=files --ignored=only --format=jsonl
```

`--ignored=include` is the default and reports both populations.
`exclude` prunes safely ignored subtrees and avoids reading ignored bodies.
`only` discovers ignored matches through non-ignored ancestors and reads only ignored
bodies for analysis.
Unknown classification cannot justify pruning.
These choices change totals, ordering, and `--min-size` together.

Unignored means what the observed rules leave.
fdu does not read trackedness, `.git/info/exclude`, `core.excludesFile`, or a global
ignore file. `.git` itself is unignored unless a rule names it.

For the most recent working files without ignored entries or repository internals:

```shell
fdu . --view=recent --limit=10 --ignored=exclude --exclude='.git/**'
```

Human rows label this subset as `(73 MiB gitignored)`: the amount is already included in
the row total, not additional usage.
`--no-gitignore` disables reading and applying the rules, so ignored shares are unknown.
It cannot be combined with either ignored-state selection.

## Analyze File Contents

Without `--analyze`, fdu classifies paths and metadata without opening regular file
bodies. Content analysis is explicit:

| Analyzer | Metrics |
| --- | --- |
| `none` | No content metrics; the default |
| `lines` | Physical, blank, and nonblank lines plus raw words |
| `code` | `lines` plus standard code, comment, and code-blank lines for supported languages |
| `words` | `lines` plus normalized and reader-visible prose words, paragraphs, and pages |
| `all` | Every shipped analyzer |

`code,words` runs both deeper analyzers in one streaming pass.
`none` and `all` name the whole axis and cannot be combined with another value.
Files are analyzed through EOF, not truncated by size.
`--analysis-workers` bounds concurrent reads.
Under `code`, a code file in a language without a line-of-code counter is reported as
`unsupported` coverage and contributes no line metrics to that request.

Naming analysis without a view chooses one that displays it: `code` selects `code`,
`words` selects `documents`, `lines` selects `families`, and `code,words` or `all`
select both `code` and `documents`. An explicit `--view` always wins and never enables
an analyzer. If that view displays no content metric, fdu still performs the requested
analysis and prints a note explaining the mismatch.

## Understand the Cache

No ordinary view requires a cache.
The first command on a root can always scan it.

Metadata-only one-shot reports still have to inspect current metadata: an in-place file
edit changes no parent-directory timestamp.
Under `--cache=auto`, fdu therefore skips loading a metadata snapshot when loading it
cannot make that report cheaper, although a complete indexed scan may write a snapshot
that opened, watch, cache-only, or later analysis work can consume.

Content results are different.
They live in a sidecar keyed by analyzer identities and semantic options.
After the metadata check, fdu restores compatible results for unchanged files and opens
only changed or newly eligible bodies.
The sidecar answers only the analyzer set that wrote it: a different set, wider or
narrower, reads the files again and replaces it.

Run the same analysis twice to see the distinction:

```shell
fdu . --analyze=code
fdu . --analyze=code
```

The gray performance footer reports metadata source, content bytes read, fresh and
cached analysis counts, total ignore files, and accepted rules.
Rules count each governing location, including repeated rule sources and negations, and
exclude comments, blank lines, and rejected patterns.
Refused files are named separately.

Total files/s and decimal GB/s divide walked files and represented size by the same
elapsed wall-clock duration shown first in the `perf:` line.
Represented size uses the selected apparent or allocated measure; it is not disk-read
bandwidth. Actual content-read throughput uses bytes read and the content-analysis
duration. On an unchanged tree the second run can show zero content bytes read and every
analysis record cached, while metadata verification still occurs.

| Policy | Behavior |
| --- | --- |
| `auto` | Use a compatible cache where the execution plan benefits; write complete indexed results |
| `refresh` | Ignore existing cache, scan, and write a complete indexed result |
| `read-only` | Read compatible cache where useful but never write it |
| `only` | Read cache without touching the source tree; explicitly stale and fails on a miss |
| `off` | Neither read nor write fdu cache data |

`--cache=only` also requires compatible content data when analysis is requested.
It never silently falls back to scanning.
Use `fdu --cache-status=all` to inspect cache files and `fdu --cache-clear=all` to
remove current, stale, and recognized leftover fdu data.
Unrecognized files are never removed.

The cache defaults to `~/.cache/fdu` on macOS and Linux, and `%LOCALAPPDATA%/fdu` on
Windows. `--cache-dir DIR` wins over `FDU_CACHE_DIR`; otherwise `XDG_CACHE_HOME/fdu`
overrides the platform default.
Overrides name the exact directory.
Status and clear use this same resolution.

Each canonical root has `<16-hex-key>.metadata.bin`, holding filesystem facts and ignore
controls, and optional `<16-hex-key>.analysis.bin`, holding derived counts and
classifications. These disposable binary files do not store source-file bodies.
Use cache status to inspect their roots, scope, and freshness.

The [cache design](project/guides/cache-design.md) explains snapshot scopes,
verification costs, atomic writes, and cleanup rules.

## Output and Automation

```shell
fdu . --view=summary,types --format=json
fdu . --view=files --format=jsonl
fdu . --watch --view=files --format=jsonl
```

Text is for people. JSON, JSON Lines, and YAML carry versioned schemas and omit the
text-only performance footer.
Integer fields that exceed 2^53 — fingerprints, option hashes, and nanosecond timestamps
— lose precision in JavaScript `JSON.parse` and any other IEEE 754 binary64 consumer.
Read them as strings, or use a parser that preserves integers, if exact identity
matters. Results go to stdout; diagnostics go to stderr.
The command never prompts or pages.

A run that takes longer than half a second shows one progress line on stderr while it
works, and erases it before anything else is written:

```text
⠼ ~/wrk/github  Scanning        412,309 files ·    12,041 dirs ·   38 GiB  3.1 s
⠧ ~/wrk/github  Analyzing      24%  12,044 / 50,110 files  7.9 s
```

Counts are right-aligned in columns wide enough for seven figures and sizes in columns
wide enough for `1023 GiB`, so the line holds still as they grow.
Its bytes are measured as the answer’s are, allocated unless `--size apparent`: a sparse
disk image can be terabytes apparent and megabytes allocated.

It is drawn only for a person at an interactive terminal: stderr must be a terminal,
`TERM` must be set and not `dumb` (Windows consoles set no `TERM`, so there an unset or
empty one is accepted and the console’s escape-sequence support is the test), and `CI`
must be unset or empty.
Otherwise nothing is drawn, whatever the flag says, so pipes, files, logs, CI, and
agents never see it and need no flag.
`--progress=auto` (the default) also skips it for JSON, JSON Lines, and YAML;
`--progress=always` draws it for those too, and `--progress=never` turns it off.
`--color` and `NO_COLOR` decide only whether it is colored.
Ctrl-C while it is drawn erases the line, prints `fdu: interrupted`, and ends the
process by the interrupt signal, so the shell reports status 130 and a calling script
stops.

Check `complete` and `errors` before trusting totals, `freshness` and `source` before
presenting them as current, row or section bounds before assuming exhaustiveness, and
metric `coverage` before presenting analysis as complete.
`--cache=only` is the only mode that can return stale freshness.

Exit status 0 is complete success.
Status 1 is a fatal filesystem or cache failure.
Status 2 is invalid usage or a partial result; useful partial output remains on stdout.
Every request fdu refuses is invalid usage, whatever the reason: a value no grammar
accepts, a rule between two flags, and a scan scope this build cannot honour, such as
`--one-filesystem` where the platform has no device identity.
`--allow-partial` accepts an operationally partial result and returns 0.

`--watch` streams changes from a retained index.
`--interval` throttles rendering, not change detection; an idle tree performs no polling
scan. The duration uses the same age grammar as `--modified-since`: `2s`, `200ms`,
`1h30m`. Fractional ages such as `0.2s` are still rejected.
Content analysis is one-shot and cannot be combined with watch mode.

Run `fdu --docs` for the offline guide and `fdu --help` for every flag.

## Agent Skill

From the project that should use the skill, run:

```shell
uvx --no-build fdu@latest --install-skill
```

This needs [uv](https://docs.astral.sh/uv/) and a compatible prebuilt wheel, but no
persistent `fdu` command or Rust compilation.
It writes `.agents/skills/fdu/SKILL.md` and `.claude/skills/fdu/SKILL.md` under the git
root of the current directory, or under the current directory when it is not in a
repository; `--agent-base DIR` writes `DIR/skills/fdu/SKILL.md` instead, for one agent’s
user scope such as `~/.claude`. It reports each file as installed, updated, or
unchanged, replaces only files it generated, and refuses a `SKILL.md` written by hand
with exit 2. Deleting those directories uninstalls it.
The generated file is the complete agent-facing usage contract and does not depend on
the installing session’s prompt or memory.
The skill prefers an `fdu` on `PATH` and otherwise runs `uvx --no-build fdu@latest`;
installing the skill does not install the command.
The zero-install fallback follows uv’s `exclude-newer` policy; see the
[installation note](../README.md#install-the-command-line) if a just-published release
is filtered. To keep the command on `PATH`, run `uv tool install --no-build fdu` and
later `uv tool upgrade fdu`. `fdu --skill` prints the portable agent-facing contract.
The skill names the build that wrote it, so re-run the installer after upgrading `fdu`.

## Quiet Diagnostics

Use `--quiet` (or `-q`) to suppress notes, tips, performance lines, and progress while
keeping the same result output.
Warnings and errors remain visible on stderr, and exit status still reports incomplete
or failed results. Structured output retains its facts.

```shell
fdu . --quiet
fdu . --view tree --full --format json --quiet
```

## Find Files and Export Complete Inventories

`--full` lifts all display bounds.
Use it for a complete recursive hierarchy or a flat inventory with directory roll-ups:

```shell
# Every directory and regular file, with recursive directory totals.
fdu . --view tree --full --format json

# Every directory as one row with recursive bytes, allocation, and file counts.
fdu . --view list --kind dir --full --sort name --format json

# Every regular file as one row with its own size and modification time.
fdu . --view files --kind file --full --format json

# Paths only, like a recursive find/fd search.
fdu . --kind file --include '*.rs' --full --sort name --format paths
fdu . --kind dir --include node_modules --full --sort name --format paths

# The same directory search, with recursive usage and age.
fdu . --kind dir --include node_modules --full --long
```

For these inventory and search tasks, fdu can replace a `find` or `fd` invocation while
also providing usage totals:

| Search | `find` | `fd` | fdu |
| --- | --- | --- | --- |
| All regular files | `find . -type f` | `fd --unrestricted --type f . .` | `fdu . --kind file --full --format paths` |
| Rust filenames | `find . -type f -name '*.rs'` | `fd --unrestricted --case-sensitive --glob '*.rs' .` | `fdu . --kind file --include '*.rs' --full --format paths` |
| Dependency directories | `find . -type d -name node_modules` | `fd --unrestricted --type d --glob node_modules .` | `fdu . --kind dir --include node_modules --full --format paths` |

These examples compare matched paths, not ordering or byte-for-byte path spelling.
fdu includes hidden and ignored content by default; `fd --unrestricted` makes that
population explicit.
fdu’s `--ignored exclude` applies its per-directory `.gitignore` contract, not every
ignore source supported by fd.
Selection uses globs, not fd’s default regular expressions or find’s expression
language. fdu does not provide find’s `-exec` actions.
Paths output escapes control characters for display; use JSON/JSONL/YAML with native
path identity for robust programmatic consumption, not a newline pipeline for arbitrary
filenames.

Nested directory rows overlap: a parent’s recursive usage includes its descendants.
Do not sum those rows; add `--view list,summary` for a path-union total.
A file row has its own usage, not a recursive tally.
`--full` does not enable file-body analysis, change ignored population, or override
`--scan-depth`. Inspect completeness independently of display bounds.
Explicit flags override the shorthand, for example `--full --depth=3`.

## Complete Recursive Output

Use `fdu . --view tree --format json --full` for a full recursive roll-up.
`--full` is shorthand for `--depth=all --breadth=all --limit=all --min-share=0%`.
Explicit limits override it regardless of flag order: `--full --depth=3` expands all
rows through depth three.
The view selection is unchanged (`--view full` selects multiple views and has a
different purpose). Scan scope, ignored population, and analysis are also unchanged;
unreadable directories still make coverage incomplete.
With complete discovery and no exclusions, every directory and regular file appears,
`remainder` is null, and there are no omission notes or tips.

## Output and Diagnostics

Formatted results go to stdout.
Notes, warnings, and suggestions go to stderr as `note:`, `warn:`, and `tip:` lines; a
human text/tree report ends stderr with `perf:`. JSON, JSONL, YAML, paths, and long rows
remain cleanly consumable by other programs.
Warnings retain their cause and affected path, and partial results retain their exit
status. Flag suggestions appear once at the end rather than beside every omitted row.
Omitted sizes name the omitted contents and are already included in directory totals.
See the [output design](project/architecture/fdu-output-design.md) for category
ordering, colors, omission alignment, and diagnostic detail.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
