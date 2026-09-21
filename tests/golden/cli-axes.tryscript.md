---
sandbox: true
path:
  - $FDU_BIN
fixtures:
  - bin
  - fixtures/project
  - fixtures/extension-levels
env:
  FORCE_COLOR: "0"
  LANG: C
  LC_ALL: C
  NO_COLOR: "1"
  TZ: UTC
  XDG_CACHE_HOME: .cache
patterns:
  AGE_DAYS: '\s*\d+'
  AGE_NS: '-?\d+'
  ALLOCATED: '\d+'
  # Paths are reported with the platform's own separator, so the separator is matched
  # rather than asserted. Every other character of the path still has to be exact.
  SEP: '[/\\]'
  # The same separator inside a JSON string, where Windows' backslash is escaped.
  JSON_SEP: '(?:/|\\\\)'
  MTIME_NS: '-?\d+'
  SCAN_PATH: '[^\r\n]+'
  RFC3339: '\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{9}Z'
  SOURCE: 'cold_scan|warm_revalidate'
  HUMAN_SIZE: '\s*[\d.]+ (B|KiB|MiB|GiB)'
  PERF_TIME: '[\d.]+ (ns|µs|ms|s)'
---
# The Five Axes Compose

Each axis is exercised on its own, then in combination.
Sizes and timestamps vary by filesystem and clock, so they are matched by named patterns
rather than elided — the field stays visible in a diff, which is the point of a golden.

## View: One Scan, Four Shapes

### Summary Is One Aggregate Row

```console
$ fdu --cache off --view summary --size apparent project
     269 B  7 files, 3 directories (128 B ignored)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Types Use Stable Classification IDs

Exact filenames, compound extensions, and ordinary extensions resolve through the
compiled type rules.
The extensionless `Makefile` therefore remains visible as `make`.

```console
$ fdu --cache off --view types --size apparent project
     128 B   47.6%  archive            1 file
      71 B   26.4%  markdown           2 files, 2 documentation
      36 B   13.4%  rust               2 files
      28 B   10.4%  make               1 file
       6 B    2.2%  unknown            1 file
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Extensions Preserve the Original Raw Grouping

Compound extensions fold to their full tail, so `archive.tar.gz` is `.tar.gz` rather
than `.gz`.

The rows are a partition of the tree rather than a selection from it, so they sum to
what the summary reports.
A name with no extension — `Makefile` here, and `.gitignore` or `README` elsewhere —
goes under `(none)` rather than dropping out of the roll-up, which is what the view used
to do: three rows totalling 235 bytes reported on a tree of 263, with nothing in the
output to say which 28 bytes were unaccounted for.
The label is parenthesised and dot-free, and a derived extension always carries its dot,
so the two can never collide.

```console
$ fdu --cache off --view extensions,summary --size apparent project
EXTENSIONS
     128 B  .tar.gz      1 file (128 B ignored)
      71 B  .md          2 files
      36 B  .rs          2 files
      34 B  (none)       2 files

SUMMARY
     269 B  7 files, 3 directories (128 B ignored)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Extensions Are Raw, Not File Rollup’s Logical Extensions

A file name has three extension levels, and this view uses the raw one.
Three of these names would land on a different pile at File Rollup Format’s logical
level, which admits only ASCII alphanumeric components and keeps up to two: `file.c++`
and `notes.md~` would fall under `(none)`, and `release.v2.zip` would be `.v2.zip`.
`archive.tar.gz` is `.tar.gz` at every level.
The `classify` module documentation tabulates all three levels.

```console
$ fdu --cache off --view extensions --size apparent extension-levels
      40 B  .tar.gz      1 file
      32 B  .zip         1 file
      25 B  .c++         1 file
      10 B  .md~         1 file
Performance: walked 4 files / 107 B; ignore rules 0 files; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Files Keep One Path per Row

The report section keeps one path per row; the one-shot text footer follows it.
Programmatic consumers use a machine format, which omits transient performance data.

```console
$ fdu --cache off --view files --kind file --size apparent project
.gitignore
Makefile
README.md
dist[SEP]acorn-0.1.0.tar.gz
docs[SEP]FAQ.MD
src[SEP]alpha.rs
src[SEP]omega.rs
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Tree Reports Every Directory’s Roll-Up

```console
$ fdu --cache off --view tree --size apparent --depth all project
     269 B  ██████████   100%  . (7 files) (128 B ignored)
     128 B  █████░░░░░    48%    dist (1 file) (128 B ignored)
      36 B  █░░░░░░░░░    13%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Several Views Come Back in Request Order, From One Scan

More than one view in a text report means more than one block of similar-looking rows,
so each is introduced by an all-caps header naming the view that produced it.
The header is what makes request order legible instead of something the reader has to
remember; a single-view report has nothing to disambiguate and stays bare.

```console
$ fdu --cache off --view summary,types --size apparent --limit 1 project
SUMMARY
     269 B  7 files, 3 directories (128 B ignored)

TYPES  (1 of 5; --limit all for every one)
     128 B   47.6%  archive            1 file
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

## Selection: Filters Are Query-Time, Not Scan-Time

### Include Narrows by Glob

```console
$ fdu --cache off --view files --include "*.rs" project
src[SEP]alpha.rs
src[SEP]omega.rs
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Brace Globs Survive Because Pattern Flags Are Repeatable

A comma-split would shred `*.{md,rs}`, which is why only closed vocabularies are lists.

```console
$ fdu --cache off --view files --include "*.{md,rs}" project
README.md
src[SEP]alpha.rs
src[SEP]omega.rs
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Exclude Beats Include

```console
$ fdu --cache off --view files --include "*.{md,rs}" --exclude "src/**" project
README.md
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Kind Selects What an Entry Is

```console
$ fdu --cache off --view files --kind dir project
dist
docs
src
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Every Tally Counts Only What the Selection Admits

A directory is an entry like any other, so a selection that rejects it must leave it out
of the count as well as out of the listing.
`--kind file` used to answer “6 files, 3 directories”, which disagreed with the files
view over the very same query — the walk counted every directory it descended into
rather than every directory the selection kept.
Descending is still unconditional: rejecting a directory hides it from the tally, never
what is underneath it.

```console
$ fdu --cache off --view summary --kind file --size apparent project
     269 B  7 files, 0 directories (128 B ignored)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

```console
$ fdu --cache off --view summary --kind dir --size apparent project
     187 B  4 files, 3 directories (128 B ignored)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Min-Size Follows the Selected Metric

```console
$ fdu --cache off --view files --kind file --min-size 100 --size apparent project
dist[SEP]acorn-0.1.0.tar.gz
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Sort and Limit Compose Into a Top-N, With No Dedicated View

```console
$ fdu --cache off --view files --kind file --sort size --limit 2 --size apparent project
(2 of 7; --limit all for every one)
dist[SEP]acorn-0.1.0.tar.gz
README.md
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Reverse Flips Whatever Order Is in Effect

```console
$ fdu --cache off --view files --kind file --sort size --reverse --limit 2 --size apparent project
(2 of 7; --limit all for every one)
.gitignore
src[SEP]omega.rs
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Depth Bounds the Rendered Tree, Not the Scan

`--depth 0` keeps du’s meaning: totals for the root and nothing beneath it.

```console
$ fdu --cache off --view tree --depth 0 --size apparent project
     269 B  ██████████   100%  . (7 files) (128 B ignored)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Every Row Says How Much of It .gitignore Ignores

`project/.gitignore` ignores `dist/`, so the summary, every tree row, and every
extension row end with the part of their size those rules ignore.
The share is left off a row with nothing ignored, and the performance line counts the
rule files read, which is what separates a row with nothing ignored from a report that
read no rules.

```console
$ fdu --cache off --view summary,tree,extensions --size apparent project
SUMMARY
     269 B  7 files, 3 directories (128 B ignored)

TREE
     269 B  ██████████   100%  . (7 files) (128 B ignored)
     128 B  █████░░░░░    48%    dist (1 file) (128 B ignored)
      36 B  █░░░░░░░░░    13%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)

EXTENSIONS
     128 B  .tar.gz      1 file (128 B ignored)
      71 B  .md          2 files
      36 B  .rs          2 files
      34 B  (none)       2 files
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Ignored Entries Are a Selection, and Sizes Follow It

`--exclude-ignored` reports only what no rule ignores, so `dist` keeps its row with
nothing in it, and rows rank by what is left.
`--only-ignored` reports the other side, and a row that is all ignored does not repeat
its size as a share.

```console
$ fdu --cache off --view summary,tree --exclude-ignored --size apparent project
SUMMARY
     141 B  6 files, 2 directories

TREE
     141 B  ██████████   100%  . (6 files)
      36 B  ███░░░░░░░    26%    src (2 files)
      23 B  ██░░░░░░░░    16%    docs (1 file)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

```console
$ fdu --cache off --view summary,tree --only-ignored --size apparent project
SUMMARY
     128 B  1 file, 1 directory

TREE
     128 B  ██████████   100%  . (1 file)
     128 B  ██████████   100%    dist (1 file)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Machine Rows Mark Each Entry, and Null Means No Rule Was Read

A file row carries `ignored: true` or `false`, so the question “which files do the rules
cover?” is one listing.
Under `--no-gitignore` no rule is read, and every share is `null` rather than a zero.

```console
$ fdu --cache off --view files --only-ignored --kind file --format jsonl --size apparent project
{"schema": "fdu.report/7", "generator": "fdu 0.1.0", "root": "[SCAN_PATH]", "request": {"scope": {"max_depth": null, "follow_symlinks": false, "one_filesystem": false, "exclude_special": false, "read_controls": true}, "analyze": [], "size": "apparent", "views": ["files"], "omitted_views": []}, "status": {"complete": true, "coverage": {"kind": "complete"}, "errors": [], "errors_omitted": 0}, "provenance": {"source": "cold_scan", "freshness": "fresh", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "tiers": {"entries": {"source": "scanned", "freshness": "fresh", "observed_at_ns": [MTIME_NS]}, "content": null}}, "ignore_rules": {"limits": {"budget": 4194304, "line_limit": 16384}, "applied": 1, "refused": 0, "refusals": []}, "analysis": null}
{"view": "files", "bound": null, "files": [{"path": "dist/acorn-0.1.0.tar.gz", "kind": "file", "bytes": 128, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "ignored": true}]}
? 0
```

```console
$ fdu --cache off --view summary --no-gitignore --format json --size apparent project
{
  "schema": "fdu.report/7",
  "generator": "fdu 0.1.0",
  "root": "[SCAN_PATH]",
  "request": {
    "scope": {
      "max_depth": null,
      "follow_symlinks": false,
      "one_filesystem": false,
      "exclude_special": false,
      "read_controls": false
    },
    "analyze": [],
    "size": "apparent",
    "views": ["summary"],
    "omitted_views": []
  },
  "status": {
    "complete": true,
    "coverage": {"kind": "complete"},
    "errors": [],
    "errors_omitted": 0
  },
  "provenance": {
    "source": "cold_scan",
    "freshness": "fresh",
    "scan_started_at": "[RFC3339]",
    "generated_at": "[RFC3339]",
    "tiers": {
      "entries": {"source": "scanned", "freshness": "fresh", "observed_at_ns": [MTIME_NS]},
      "content": null
    }
  },
  "ignore_rules": null,
  "analysis": null,
  "reports": [
    {
      "view": "summary",
      "summary": {
        "files": 7,
        "dirs": 3,
        "bytes": 269,
        "allocated": [ALLOCATED],
        "ignored": null,
        "newest_mtime_ns": [MTIME_NS]
      }
    }
  ]
}
? 0
```

### A Selection by Ignored State Needs the Rules

```console
$ fdu --cache off --no-gitignore --only-ignored project
fdu: --only-ignored needs .gitignore classification, and --no-gitignore turned it off; drop one of them
? 2
```

### A Rule File the Budget Refuses Is Named, and Sizes Stay Exact

A `.gitignore` with a line over the 16 KiB line limit is refused rather than ending the
scan.
The note names where the split is not exact and the flag that raises the limit that
fired, not the other one.

```console
$ node -e "const fs=require('node:fs'); fs.mkdirSync('long-rule'); fs.writeFileSync('long-rule/.gitignore', 'x'.repeat(16385) + '\n'); fs.writeFileSync('long-rule/kept.txt', 'kept\n')"
? 0
```

```console
$ fdu --cache off --view summary --size apparent long-rule
    16 KiB  2 files, 0 directories
note: 1 .gitignore file not applied (1 with a line over the 16 KiB line limit), so ignored shares under . are not exact; sizes are. To apply them, raise --gitignore-line-limit above 16 KiB, or set it to all
Performance: walked 2 files / 16 KiB; ignore rules 0 files, 1 refused; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

## Format: Every View in Every Serialization

### JSON Carries the Versioned Envelope

```console
$ fdu --cache off --view summary --format json --size apparent project
{
  "schema": "fdu.report/7",
  "generator": "fdu 0.1.0",
  "root": "[SCAN_PATH]",
  "request": {
    "scope": {
      "max_depth": null,
      "follow_symlinks": false,
      "one_filesystem": false,
      "exclude_special": false,
      "read_controls": true
    },
    "analyze": [],
    "size": "apparent",
    "views": ["summary"],
    "omitted_views": []
  },
  "status": {
    "complete": true,
    "coverage": {"kind": "complete"},
    "errors": [],
    "errors_omitted": 0
  },
  "provenance": {
    "source": "cold_scan",
    "freshness": "fresh",
    "scan_started_at": "[RFC3339]",
    "generated_at": "[RFC3339]",
    "tiers": {
      "entries": {"source": "scanned", "freshness": "fresh", "observed_at_ns": [MTIME_NS]},
      "content": null
    }
  },
  "ignore_rules": {
    "limits": {"budget": 4194304, "line_limit": 16384},
    "applied": 1,
    "refused": 0,
    "refusals": []
  },
  "analysis": null,
  "reports": [
    {
      "view": "summary",
      "summary": {
        "files": 7,
        "dirs": 3,
        "bytes": 269,
        "allocated": [ALLOCATED],
        "ignored": {"files": 1, "dirs": 1, "bytes": 128, "allocated": [ALLOCATED]},
        "newest_mtime_ns": [MTIME_NS]
      }
    }
  ]
}
? 0
```

### JSONL Is One Document per Line

```console
$ fdu --cache off --view types --format jsonl --size apparent --limit 1 project
{"schema": "fdu.report/7", "generator": "fdu 0.1.0", "root": "[SCAN_PATH]", "request": {"scope": {"max_depth": null, "follow_symlinks": false, "one_filesystem": false, "exclude_special": false, "read_controls": true}, "analyze": [], "size": "apparent", "views": ["types"], "omitted_views": []}, "status": {"complete": true, "coverage": {"kind": "complete"}, "errors": [], "errors_omitted": 0}, "provenance": {"source": "cold_scan", "freshness": "fresh", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "tiers": {"entries": {"source": "scanned", "freshness": "fresh", "observed_at_ns": [MTIME_NS]}, "content": null}}, "ignore_rules": {"limits": {"budget": 4194304, "line_limit": 16384}, "applied": 1, "refused": 0, "refusals": []}, "analysis": null}
{"view": "types", "metrics": {"group": "type", "share_metric": "apparent_bytes", "bound": {"shown": 1, "total": 5}, "total": {"id": "total", "family": "unknown", "files": 7, "bytes": 269, "allocated": [ALLOCATED], "share": {"numerator": 269, "denominator": 269}, "metrics": {}, "coverage": {}, "detection": {"sources": {"exact_filename": 1, "compound_extension": 1, "extension": 4, "unknown": 1}, "confidence": {"certain": 6, "heuristic": 1}, "flags": {"generated": 0, "vendored": 0, "documentation": 2}}}, "rows": [{"id": "archive", "family": "binary", "files": 1, "bytes": 128, "allocated": [ALLOCATED], "share": {"numerator": 128, "denominator": 269}, "metrics": {}, "coverage": {}, "detection": {"sources": {"compound_extension": 1}, "confidence": {"certain": 1}, "flags": {"generated": 0, "vendored": 0, "documentation": 0}}}]}}
? 0
```

### YAML Quotes Only What Would Be Ambiguous

```console
$ fdu --cache off --view summary --format yaml --size apparent project
schema: fdu.report/7
generator: "fdu 0.1.0"
root: [SCAN_PATH]
request:
  scope:
    max_depth: null
    follow_symlinks: false
    one_filesystem: false
    exclude_special: false
    read_controls: true
  analyze: []
  size: apparent
  views: [summary]
  omitted_views: []
status:
  complete: true
  coverage: {kind: complete}
  errors: []
  errors_omitted: 0
provenance:
  source: cold_scan
  freshness: fresh
  scan_started_at: "[RFC3339]"
  generated_at: "[RFC3339]"
  tiers:
    entries: {source: scanned, freshness: fresh, observed_at_ns: [MTIME_NS]}
    content: null
ignore_rules:
  limits: {budget: 4194304, line_limit: 16384}
  applied: 1
  refused: 0
  refusals: []
analysis: null
reports:
  -
    view: summary
    summary:
      files: 7
      dirs: 3
      bytes: 269
      allocated: [ALLOCATED]
      ignored: {files: 1, dirs: 1, bytes: 128, allocated: [ALLOCATED]}
      newest_mtime_ns: [MTIME_NS]
? 0
```

## Errors Name the Value and the Fix

An agent should be able to correct a command from its rejection alone.

### An Unknown View Lists Every Valid One

```console
$ fdu --cache off --view bogus project
fdu: invalid --view "bogus": expected one of list, summary, tree, families, types, extensions, languages, documents, largest, recent, files, full
? 2
```

### The Unreleased View Alias Is Rejected

```console
$ fdu --cache off --view docs project
fdu: invalid --view "docs": expected one of list, summary, tree, families, types, extensions, languages, documents, largest, recent, files, full
? 2
```

### A Repeated View Is a Typo, Not a No-Op

```console
$ fdu --cache off --view tree,tree project
fdu: invalid --view "tree,tree": "tree" appears more than once
? 2
```

### An Empty List Entry Is Rejected

```console
$ fdu --cache off --view tree,,types project
fdu: invalid --view "tree,,types": empty entry in the list
? 2
```

### An Unknown Format Lists Every Valid One

```console
$ fdu --cache off --format xml project
fdu: invalid --format "xml": expected one of text, tree, paths, long, json, jsonl, yaml
? 2
```

### Fractional Ages Point at the Compound Spelling

```console
$ fdu --cache off --modified-since 1.5h project
fdu: invalid time "1.5h": fractional ages are not supported; write them as compounds, as in `1h30m` rather than `1.5h`
? 2
```

### Calendar Units Point at Days

```console
$ fdu --cache off --modified-before 3months project
fdu: invalid time "3months": calendar units are not supported because they are not a fixed length; use days, as in `30d` or `365d`
? 2
```

### Local Timestamps Ask for an Offset Rather Than Guessing

Guessing UTC would answer a prompt in another timezone hours off, in silence.

```console
$ fdu --cache off --modified-since 2026-08-10 project
fdu: invalid time "2026-08-10": local date and time are not supported yet because resolving one needs a time-zone database; write an RFC 3339 timestamp with an offset, as in `2026-08-10T12:30:00Z` or `2026-08-10T12:30:00-08:00`, or use `@` epoch seconds
? 2
```

### An Unknown Size Unit Lists the Accepted Ones

```console
$ fdu --cache off --min-size 10X project
fdu: invalid size "10X": unknown size unit "X"; use B, K/KB, M/MB, G/GB, T/TB, P/PB, or the binary forms KiB, MiB, GiB, TiB, PiB
? 2
```

### A Malformed Glob Says Which Delimiter Is Unmatched

```console
$ fdu --cache off --include "{a,b" project
fdu: invalid pattern "{a,b": unmatched `{` in pattern
? 2
```

### A Bad Bound Names Both Accepted Forms

```console
$ fdu --cache off --depth two project
fdu: invalid --depth "two": expected a whole number or `all`
? 2
```

## Mode: Cache Policy Is Explicit, and Never Silently Stale

Every report says which tier answered it, so no policy can quietly serve old data.

### A First Run Scans Cold and Leaves a Snapshot

```console
$ fdu --view tree --format jsonl --size apparent project
{"schema": "fdu.report/7", "generator": "fdu 0.1.0", "root": "[SCAN_PATH]", "request": {"scope": {"max_depth": null, "follow_symlinks": false, "one_filesystem": false, "exclude_special": false, "read_controls": true}, "analyze": [], "size": "apparent", "views": ["tree"], "omitted_views": []}, "status": {"complete": true, "coverage": {"kind": "complete"}, "errors": [], "errors_omitted": 0}, "provenance": {"source": "cold_scan", "freshness": "fresh", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "tiers": {"entries": {"source": "scanned", "freshness": "fresh", "observed_at_ns": [MTIME_NS]}, "content": null}}, "ignore_rules": {"limits": {"budget": 4194304, "line_limit": 16384}, "applied": 1, "refused": 0, "refusals": []}, "analysis": null}
{"view": "tree", "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 269, "allocated": [ALLOCATED], "files": 7, "dirs": 3, "ignored": {"files": 1, "dirs": 1, "bytes": 128, "allocated": [ALLOCATED]}, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": [{"name": "dist", "path": "dist", "kind": "dir", "bytes": 128, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "ignored": {"files": 1, "dirs": 0, "bytes": 128, "allocated": [ALLOCATED]}, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}, {"name": "src", "path": "src", "kind": "dir", "bytes": 36, "allocated": [ALLOCATED], "files": 2, "dirs": 0, "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0}, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}, {"name": "docs", "path": "docs", "kind": "dir", "bytes": 23, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0}, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}]}}
? 0
```

### The Next Run Scans Cold Again, Because Reading Cannot Pay

A one-shot report never loads the snapshot for a metadata query: revalidating one stats
every entry regardless, so the load would be added to the walk, never instead of it.
The run rewrites the snapshot instead, which is what keeps the cache-only tier below
current. Sessions opened through the library hold their index and do amortise the load;
this is the one-shot contract only.

```console
$ fdu --view tree --format jsonl --size apparent project
{"schema": "fdu.report/7", "generator": "fdu 0.1.0", "root": "[SCAN_PATH]", "request": {"scope": {"max_depth": null, "follow_symlinks": false, "one_filesystem": false, "exclude_special": false, "read_controls": true}, "analyze": [], "size": "apparent", "views": ["tree"], "omitted_views": []}, "status": {"complete": true, "coverage": {"kind": "complete"}, "errors": [], "errors_omitted": 0}, "provenance": {"source": "cold_scan", "freshness": "fresh", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "tiers": {"entries": {"source": "scanned", "freshness": "fresh", "observed_at_ns": [MTIME_NS]}, "content": null}}, "ignore_rules": {"limits": {"budget": 4194304, "line_limit": 16384}, "applied": 1, "refused": 0, "refusals": []}, "analysis": null}
{"view": "tree", "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 269, "allocated": [ALLOCATED], "files": 7, "dirs": 3, "ignored": {"files": 1, "dirs": 1, "bytes": 128, "allocated": [ALLOCATED]}, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": [{"name": "dist", "path": "dist", "kind": "dir", "bytes": 128, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "ignored": {"files": 1, "dirs": 0, "bytes": 128, "allocated": [ALLOCATED]}, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}, {"name": "src", "path": "src", "kind": "dir", "bytes": 36, "allocated": [ALLOCATED], "files": 2, "dirs": 0, "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0}, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}, {"name": "docs", "path": "docs", "kind": "dir", "bytes": 23, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0}, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}]}}
? 0
```

### Cache-Only Answers Without Touching the Tree, and Says It Is Stale

```console
$ fdu --cache only --view tree --format jsonl --size apparent project
{"schema": "fdu.report/7", "generator": "fdu 0.1.0", "root": "[SCAN_PATH]", "request": {"scope": {"max_depth": null, "follow_symlinks": false, "one_filesystem": false, "exclude_special": false, "read_controls": true}, "analyze": [], "size": "apparent", "views": ["tree"], "omitted_views": []}, "status": {"complete": true, "coverage": {"kind": "complete"}, "errors": [], "errors_omitted": 0}, "provenance": {"source": "cache_only", "freshness": "stale", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "tiers": {"entries": {"source": "cached", "freshness": "stale", "observed_at_ns": [MTIME_NS]}, "content": null}}, "ignore_rules": {"limits": {"budget": 4194304, "line_limit": 16384}, "applied": 1, "refused": 0, "refusals": []}, "analysis": null}
{"view": "tree", "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 269, "allocated": [ALLOCATED], "files": 7, "dirs": 3, "ignored": {"files": 1, "dirs": 1, "bytes": 128, "allocated": [ALLOCATED]}, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": [{"name": "dist", "path": "dist", "kind": "dir", "bytes": 128, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "ignored": {"files": 1, "dirs": 0, "bytes": 128, "allocated": [ALLOCATED]}, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}, {"name": "src", "path": "src", "kind": "dir", "bytes": 36, "allocated": [ALLOCATED], "files": 2, "dirs": 0, "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0}, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}, {"name": "docs", "path": "docs", "kind": "dir", "bytes": 23, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0}, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}]}}
? 0
```

### Refresh Ignores the Snapshot and Scans Cold Again

```console
$ fdu --cache refresh --view tree --format jsonl --size apparent project
{"schema": "fdu.report/7", "generator": "fdu 0.1.0", "root": "[SCAN_PATH]", "request": {"scope": {"max_depth": null, "follow_symlinks": false, "one_filesystem": false, "exclude_special": false, "read_controls": true}, "analyze": [], "size": "apparent", "views": ["tree"], "omitted_views": []}, "status": {"complete": true, "coverage": {"kind": "complete"}, "errors": [], "errors_omitted": 0}, "provenance": {"source": "cold_scan", "freshness": "fresh", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "tiers": {"entries": {"source": "scanned", "freshness": "fresh", "observed_at_ns": [MTIME_NS]}, "content": null}}, "ignore_rules": {"limits": {"budget": 4194304, "line_limit": 16384}, "applied": 1, "refused": 0, "refusals": []}, "analysis": null}
{"view": "tree", "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 269, "allocated": [ALLOCATED], "files": 7, "dirs": 3, "ignored": {"files": 1, "dirs": 1, "bytes": 128, "allocated": [ALLOCATED]}, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": [{"name": "dist", "path": "dist", "kind": "dir", "bytes": 128, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "ignored": {"files": 1, "dirs": 0, "bytes": 128, "allocated": [ALLOCATED]}, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}, {"name": "src", "path": "src", "kind": "dir", "bytes": 36, "allocated": [ALLOCATED], "files": 2, "dirs": 0, "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0}, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}, {"name": "docs", "path": "docs", "kind": "dir", "bytes": 23, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0}, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}]}}
? 0
```

### An Unknown Policy Lists Every Valid One

```console
$ fdu --cache sometimes project
fdu: invalid --cache "sometimes": expected one of auto, refresh, read-only, only, off
? 2
```

### The Unreleased Cache Alias Is Rejected

```console
$ fdu --cache readonly project
fdu: invalid --cache "readonly": expected one of auto, refresh, read-only, only, off
? 2
```

## Old Build Directories Use Subtree Size and Latest Activity

An old directory containing a recently modified file is excluded.
Empty old directories still match.
The paths are complete and size-ranked; long adds the same size and age.

```console
$ node bin/directory-builds.cjs
? 0
```

```console
$ fdu --cache off --size apparent --kind dir --include .venv --include node_modules --include target --modified-before 30d --format paths builds
c[SEP]target
b[SEP]node_modules
a[SEP].venv
empty[SEP].venv
? 0
```

```console
$ fdu --cache off --size apparent --kind dir --include .venv --include node_modules --include target --modified-before 30d --long builds
      70 B [AGE_DAYS]d c[SEP]target
      50 B [AGE_DAYS]d b[SEP]node_modules
      30 B [AGE_DAYS]d a[SEP].venv
       0 B [AGE_DAYS]d empty[SEP].venv
? 0
```
