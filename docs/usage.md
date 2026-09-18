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

- `tree` view
- allocated filesystem bytes
- largest entries first
- two directory levels
- at most ten children per directory
- metadata only; regular file bodies are not opened
- hidden and ignored entries included

fdu reads per-directory `.gitignore` files by default so it can annotate ignored byte
shares. Reading the rules does not exclude the entries they match.

These commands cover the most common questions:

```shell
fdu . --exclude-ignored
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
| `tree` | Directory hierarchy and recursive sizes |
| `summary` | One total for the selected tree |
| `families` | Broad code, prose, markup, data, binary, and unknown groups |
| `types` | Detected file types |
| `extensions` | Raw filename extensions; extensionless names use `(none)` |
| `languages` | Programming languages, metadata-only unless code analysis is enabled |
| `documents` | Prose metrics; requires an enabled analyzer |
| `largest` | Twenty largest regular files by default |
| `recent` | Twenty most recently modified regular files by default |
| `files` | Every selected entry in name order |
| `full` | Every supported view except the unbounded `files` listing |

`largest` and `recent` are presets over `files`. `--sort` and `--limit` override their
defaults. Use `--limit=all` where a bounded view should print every row.
`--depth` limits reported tree levels, not scan depth; `--scan-depth` limits what can be
scanned and therefore changes the cache scope.

Sizes use allocated bytes by default.
Add `--size=apparent` for logical file lengths.

The percentage column in grouped views normally shows byte share.
When code analysis is shown by language, it shows code-line share instead.
In `documents`, it shows document-word share.
Text output labels those two non-byte denominators; machine output always carries the
exact `share_metric`, numerator, and denominator.

## Select Entries

Selection changes which retained entries contribute to a report.
It does not narrow the scan or cache scope.

Useful selections include:

```shell
fdu . --include='*.{rs,toml}'
fdu . --exclude='target/**'
fdu . --min-size=10MiB
fdu . --modified-since=1h --view=files --sort=mtime
fdu . --kind=file --view=largest
```

Quote glob patterns so the shell does not expand them before fdu sees them.

### Select by `.gitignore`

```shell
fdu . --exclude-ignored
fdu . --view=files --only-ignored --format=jsonl
```

These flags select one side of the `.gitignore` classification after the scan.
They change totals, ordering, and `--min-size`, but do not prune metadata work or
content analysis. `--exclude-ignored` means “what the observed rules leave,” not “what
Git would commit”: fdu does not read trackedness, `.git/info/exclude`,
`core.excludesFile`, or a global ignore file.
`.git` itself is unignored unless a rule names it.

For the most recent working files without ignored entries or repository internals:

```shell
fdu . --view=recent --limit=10 --exclude-ignored --exclude='.git/**'
```

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

Naming analysis without a view chooses one that displays it: `code` selects `languages`,
`words` selects `documents`, and `lines`, `code,words`, or `all` select `families`. An
explicit `--view` always wins and never enables an analyzer.
If that view displays no content metric, fdu still performs the requested analysis and
prints a note explaining the mismatch.

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

The performance footer reports metadata source, content bytes read, and fresh versus
cached analysis counts.
On an unchanged tree the second run can show zero content bytes read and every analysis
record cached, while metadata verification still occurs.

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
Read them as strings, or use a parser that preserves integers, if exact identity matters.
Results go to stdout; diagnostics go to stderr.
The command never prompts or pages.

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

Run `fdu --docs` for the offline guide, `fdu --help` for every flag, and `fdu --skill`
for the portable agent-facing contract.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
