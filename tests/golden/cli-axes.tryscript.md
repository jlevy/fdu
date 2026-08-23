---
sandbox: true
path:
  - $FDU_BIN
fixtures:
  - fixtures/project
env:
  FORCE_COLOR: "0"
  LANG: C
  LC_ALL: C
  NO_COLOR: "1"
  TZ: UTC
  XDG_CACHE_HOME: .cache
patterns:
  ALLOCATED: '\d+'
  # Paths are reported with the platform's own separator, so the separator is matched
  # rather than asserted. Every other character of the path still has to be exact.
  SEP: '[/\\]'
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
     263 B  6 files, 3 directories
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Types Use Stable Classification IDs

Exact filenames, compound extensions, and ordinary extensions resolve through the
compiled type rules.
The extensionless `Makefile` therefore remains visible as `make`.

```console
$ fdu --cache off --view types --size apparent project
     128 B   48.7%  archive            1 file
      71 B   27.0%  markdown           2 files, 2 documentation
      36 B   13.7%  rust               2 files
      28 B   10.6%  make               1 file
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
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
     128 B  .tar.gz      1 file
      71 B  .md          2 files
      36 B  .rs          2 files
      28 B  (none)       1 file

SUMMARY
     263 B  6 files, 3 directories
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Files Keep One Path per Row

The report section keeps one path per row; the one-shot text footer follows it.
Programmatic consumers use a machine format, which omits transient performance data.

```console
$ fdu --cache off --view files --kind file --size apparent project
Makefile
README.md
dist[SEP]acorn-0.1.0.tar.gz
docs[SEP]FAQ.MD
src[SEP]alpha.rs
src[SEP]omega.rs
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Tree Reports Every Directory’s Roll-Up

```console
$ fdu --cache off --view tree --size apparent --depth all project
     263 B  ██████████   100%  . (6 files)
     128 B  █████░░░░░    49%    dist (1 file)
      36 B  █░░░░░░░░░    14%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
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
     263 B  6 files, 3 directories

TYPES  (1 of 4; --limit all for every one)
     128 B   48.7%  archive            1 file
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

## Selection: Filters Are Query-Time, Not Scan-Time

### Include Narrows by Glob

```console
$ fdu --cache off --view files --include "*.rs" project
src[SEP]alpha.rs
src[SEP]omega.rs
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Brace Globs Survive Because Pattern Flags Are Repeatable

A comma-split would shred `*.{md,rs}`, which is why only closed vocabularies are lists.

```console
$ fdu --cache off --view files --include "*.{md,rs}" project
README.md
src[SEP]alpha.rs
src[SEP]omega.rs
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Exclude Beats Include

```console
$ fdu --cache off --view files --include "*.{md,rs}" --exclude "src/**" project
README.md
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Kind Selects What an Entry Is

```console
$ fdu --cache off --view files --kind dir project
dist
docs
src
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
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
     263 B  6 files, 0 directories
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

```console
$ fdu --cache off --view summary --kind dir --size apparent project
       0 B  0 files, 3 directories
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Min-Size Follows the Selected Metric

```console
$ fdu --cache off --view files --kind file --min-size 100 --size apparent project
dist[SEP]acorn-0.1.0.tar.gz
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Sort and Limit Compose Into a Top-N, With No Dedicated View

```console
$ fdu --cache off --view files --kind file --sort size --limit 2 --size apparent project
(2 of 6; --limit all for every one)
dist[SEP]acorn-0.1.0.tar.gz
README.md
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Reverse Flips Whatever Order Is in Effect

```console
$ fdu --cache off --view files --kind file --sort size --reverse --limit 2 --size apparent project
(2 of 6; --limit all for every one)
src[SEP]omega.rs
src[SEP]alpha.rs
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Depth Bounds the Rendered Tree, Not the Scan

`--depth 0` keeps du’s meaning: totals for the root and nothing beneath it.

```console
$ fdu --cache off --view tree --depth 0 --size apparent project
     263 B  ██████████   100%  . (6 files)
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

## Format: Every View in Every Serialization

### JSON Carries the Versioned Envelope

```console
$ fdu --cache off --view summary --format json --size apparent project
{
  "schema": "fdu.report/4",
  "generator": "fdu 0.1.0",
  "root": "[SCAN_PATH]",
  "scan_started_at": "[RFC3339]",
  "generated_at": "[RFC3339]",
  "source": "cold_scan",
  "freshness": "fresh",
  "complete": true,
  "errors": [],
  "reports": [
    {
      "view": "summary",
      "summary": {"files": 6, "dirs": 3, "bytes": 263, "allocated": [ALLOCATED], "newest_mtime_ns": [MTIME_NS]}
    }
  ]
}
? 0
```

### JSONL Is One Document per Line

```console
$ fdu --cache off --view types --format jsonl --size apparent --limit 1 project
{"schema": "fdu.report/5", "generator": "fdu 0.1.0", "root": "[SCAN_PATH]", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "source": "cold_scan", "freshness": "fresh", "complete": true, "errors": [], "analysis": null}
{"view": "types", "metrics": {"group": "type", "share_metric": "apparent_bytes", "words_per_page": 250, "bound": {"shown": 1, "total": 4}, "total": {"id": "total", "family": "unknown", "files": 6, "bytes": 263, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 263, "denominator": 263}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "code_blank_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0, "document_words": 0}, "coverage": {}, "detection": {"sources": {"exact_filename": 1, "compound_extension": 1, "extension": 4}, "confidence": {"certain": 6}, "flags": {"generated": 0, "vendored": 0, "documentation": 2}}, "pages": {"words": 0, "words_per_page": 250}}, "rows": [{"id": "archive", "family": "binary", "files": 1, "bytes": 128, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 128, "denominator": 263}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "code_blank_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0, "document_words": 0}, "coverage": {}, "detection": {"sources": {"compound_extension": 1}, "confidence": {"certain": 1}, "flags": {"generated": 0, "vendored": 0, "documentation": 0}}, "pages": {"words": 0, "words_per_page": 250}}]}}
? 0
```

### YAML Quotes Only What Would Be Ambiguous

```console
$ fdu --cache off --view summary --format yaml --size apparent project
schema: fdu.report/4
generator: "fdu 0.1.0"
root: [SCAN_PATH]
scan_started_at: "[RFC3339]"
generated_at: "[RFC3339]"
source: cold_scan
freshness: fresh
complete: true
errors: []
reports:
  - view: summary
    summary:
      files: 6
      dirs: 3
      bytes: 263
      allocated: [ALLOCATED]
      newest_mtime_ns: [MTIME_NS]
? 0
```

## Errors Name the Value and the Fix

An agent should be able to correct a command from its rejection alone.

### An Unknown View Lists Every Valid One

```console
$ fdu --cache off --view bogus project
fdu: invalid --view "bogus": expected one of summary, tree, families, types, extensions, languages, documents, largest, recent, files, full
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
fdu: invalid --format "xml": expected one of text, json, jsonl, yaml
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
{"schema": "fdu.report/4", "generator": "fdu 0.1.0", "root": "[SCAN_PATH]", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "source": "cold_scan", "freshness": "fresh", "complete": true, "errors": []}
{"view": "tree", "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 263, "allocated": [ALLOCATED], "files": 6, "dirs": 3, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": [{"name": "dist", "path": "dist", "kind": "dir", "bytes": 128, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []},{"name": "src", "path": "src", "kind": "dir", "bytes": 36, "allocated": [ALLOCATED], "files": 2, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []},{"name": "docs", "path": "docs", "kind": "dir", "bytes": 23, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}]}}
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
{"schema": "fdu.report/4", "generator": "fdu 0.1.0", "root": "[SCAN_PATH]", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "source": "cold_scan", "freshness": "fresh", "complete": true, "errors": []}
{"view": "tree", "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 263, "allocated": [ALLOCATED], "files": 6, "dirs": 3, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": [{"name": "dist", "path": "dist", "kind": "dir", "bytes": 128, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []},{"name": "src", "path": "src", "kind": "dir", "bytes": 36, "allocated": [ALLOCATED], "files": 2, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []},{"name": "docs", "path": "docs", "kind": "dir", "bytes": 23, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}]}}
? 0
```

### Cache-Only Answers Without Touching the Tree, and Says It Is Stale

```console
$ fdu --cache only --view tree --format jsonl --size apparent project
{"schema": "fdu.report/4", "generator": "fdu 0.1.0", "root": "[SCAN_PATH]", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "source": "cache_only", "freshness": "stale", "complete": true, "errors": []}
{"view": "tree", "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 263, "allocated": [ALLOCATED], "files": 6, "dirs": 3, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": [{"name": "dist", "path": "dist", "kind": "dir", "bytes": 128, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []},{"name": "src", "path": "src", "kind": "dir", "bytes": 36, "allocated": [ALLOCATED], "files": 2, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []},{"name": "docs", "path": "docs", "kind": "dir", "bytes": 23, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}]}}
? 0
```

### Refresh Ignores the Snapshot and Scans Cold Again

```console
$ fdu --cache refresh --view tree --format jsonl --size apparent project
{"schema": "fdu.report/4", "generator": "fdu 0.1.0", "root": "[SCAN_PATH]", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "source": "cold_scan", "freshness": "fresh", "complete": true, "errors": []}
{"view": "tree", "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 263, "allocated": [ALLOCATED], "files": 6, "dirs": 3, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": [{"name": "dist", "path": "dist", "kind": "dir", "bytes": 128, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []},{"name": "src", "path": "src", "kind": "dir", "bytes": 36, "allocated": [ALLOCATED], "files": 2, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []},{"name": "docs", "path": "docs", "kind": "dir", "bytes": 23, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}]}}
? 0
```

### An Unknown Policy Lists Every Valid One

```console
$ fdu --cache sometimes project
fdu: invalid --cache "sometimes": expected one of auto, refresh, read-only, only, off
? 2
```

## Scope: Traversal Order and Worker Count

Both orders visit every entry exactly once and leave an identical index behind, so a
completed report is the same either way.
What they change is *when* observations are produced, which matters only to a consumer
reading while the walk runs — and which is what makes a recorded progressive session
reproducible.

### Breadth-First Is the Default, and Naming It Changes Nothing

```console
$ fdu --order breadth-first --view summary --format jsonl --size apparent project
{"schema": "fdu.report/4", "generator": "fdu 0.1.0", "root": "[SCAN_PATH]", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "source": "cold_scan", "freshness": "fresh", "complete": true, "errors": []}
{"view": "summary", "summary": {"files": 6, "dirs": 3, "bytes": 263, "allocated": [ALLOCATED], "newest_mtime_ns": [MTIME_NS]}}
? 0
```

### Depth-First Answers Identically

```console
$ fdu --order depth-first --view summary --format jsonl --size apparent project
{"schema": "fdu.report/4", "generator": "fdu 0.1.0", "root": "[SCAN_PATH]", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "source": "cold_scan", "freshness": "fresh", "complete": true, "errors": []}
{"view": "summary", "summary": {"files": 6, "dirs": 3, "bytes": 263, "allocated": [ALLOCATED], "newest_mtime_ns": [MTIME_NS]}}
? 0
```

### One Worker Answers Identically Too

```console
$ fdu --threads 1 --view summary --format jsonl --size apparent project
{"schema": "fdu.report/4", "generator": "fdu 0.1.0", "root": "[SCAN_PATH]", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "source": "cold_scan", "freshness": "fresh", "complete": true, "errors": []}
{"view": "summary", "summary": {"files": 6, "dirs": 3, "bytes": 263, "allocated": [ALLOCATED], "newest_mtime_ns": [MTIME_NS]}}
? 0
```

### An Unknown Order Names the Two That Exist

```console
$ fdu --order sideways project
fdu: unknown --order sideways; expected breadth-first or depth-first
? 2
```
