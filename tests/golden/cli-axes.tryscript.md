---
sandbox: true
fixtures:
  - fixtures/project
path:
  - $TRYSCRIPT_GIT_ROOT/target/debug
env:
  FORCE_COLOR: "0"
  LANG: C
  LC_ALL: C
  NO_COLOR: "1"
  TZ: UTC
  XDG_CACHE_HOME: .cache
patterns:
  ALLOCATED: '\d+'
  MTIME_NS: '-?\d+'
  SCAN_PATH: '[^\r\n]+'
  RFC3339: '\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{9}Z'
  SOURCE: 'cold_scan|warm_revalidate'
  HUMAN_SIZE: '\s*[\d.]+ (B|KiB|MiB|GiB)'
---

# The Five Axes Compose

Each axis is exercised on its own, then in combination. Sizes and timestamps vary by
filesystem and clock, so they are matched by named patterns rather than elided — the
field stays visible in a diff, which is the point of a golden.

## View: One Scan, Four Shapes

### Summary Is One Aggregate Row

```console
$ fdu --no-cache --view summary --size apparent project
     263 B  6 files, 3 directories
? 0
```

### Types Groups by Derived Extension

Compound extensions fold to their full tail, so `archive.tar.gz` is `.tar.gz` rather
than `.gz`.

```console
$ fdu --no-cache --view types --size apparent project
     128 B  .tar.gz      1 file
      71 B  .md          2 files
      36 B  .rs          2 files
? 0
```

### Files Is a Bare Path List

Nothing but paths, so the output pipes straight into another command.

```console
$ fdu --no-cache --view files --kind file --size apparent project
Makefile
README.md
dist/acorn-0.1.0.tar.gz
docs/FAQ.MD
src/alpha.rs
src/omega.rs
? 0
```

### Tree Reports Every Directory's Roll-Up

```console
$ fdu --no-cache --view tree --size apparent --depth all project
     263 B  . (6 files)
       128 B  dist (1 file)
        36 B  src (2 files)
        23 B  docs (1 file)
? 0
```

### Several Views Come Back in Request Order, From One Scan

```console
$ fdu --no-cache --view summary,types --size apparent --limit 1 project
     263 B  6 files, 3 directories

     128 B  .tar.gz      1 file
? 0
```

## Selection: Filters Are Query-Time, Not Scan-Time

### Include Narrows by Glob

```console
$ fdu --no-cache --view files --include '*.rs' project
src/alpha.rs
src/omega.rs
? 0
```

### Brace Globs Survive Because Pattern Flags Are Repeatable

A comma-split would shred `*.{md,rs}`, which is why only closed vocabularies are lists.

```console
$ fdu --no-cache --view files --include '*.{md,rs}' project
README.md
src/alpha.rs
src/omega.rs
? 0
```

### Exclude Beats Include

```console
$ fdu --no-cache --view files --include '*.{md,rs}' --exclude 'src/**' project
README.md
? 0
```

### Kind Selects What an Entry Is

```console
$ fdu --no-cache --view files --kind dir project
dist
docs
src
? 0
```

### Min-Size Follows the Selected Metric

```console
$ fdu --no-cache --view files --kind file --min-size 100 --size apparent project
dist/acorn-0.1.0.tar.gz
? 0
```

### Sort and Limit Compose Into a Top-N, With No Dedicated View

```console
$ fdu --no-cache --view files --kind file --sort size --limit 2 --size apparent project
dist/acorn-0.1.0.tar.gz
README.md
? 0
```

### Reverse Flips Whatever Order Is in Effect

```console
$ fdu --no-cache --view files --kind file --sort size --reverse --limit 2 --size apparent project
src/omega.rs
src/alpha.rs
? 0
```

### Depth Bounds the Rendered Tree, Not the Scan

`--depth 0` keeps du's meaning: totals for the root and nothing beneath it.

```console
$ fdu --no-cache --view tree --depth 0 --size apparent project
     263 B  . (6 files)
  …
? 0
```

## Format: Every View in Every Serialization

### JSON Carries the Versioned Envelope

```console
$ fdu --no-cache --view summary --format json --size apparent project
{
  "schema": "fdu.report/1",
  "generator": "fdu 0.0.1",
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
$ fdu --no-cache --view types --format jsonl --size apparent --limit 1 project
{"schema": "fdu.report/1", "generator": "fdu 0.0.1", "root": "[SCAN_PATH]", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "source": "cold_scan", "freshness": "fresh", "complete": true, "errors": []}
{"view": "types", "types": [{"extension": ".tar.gz", "files": 1, "bytes": 128, "allocated": [ALLOCATED]}]}
? 0
```

### YAML Quotes Only What Would Be Ambiguous

```console
$ fdu --no-cache --view summary --format yaml --size apparent project
schema: fdu.report/1
generator: "fdu 0.0.1"
root: [SCAN_PATH]/project
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
$ fdu --no-cache --view bogus project
fdu: invalid --view "bogus": expected one of tree, types, files, summary
? 2
```

### A Repeated View Is a Typo, Not a No-Op

```console
$ fdu --no-cache --view tree,tree project
fdu: invalid --view "tree,tree": "tree" appears more than once
? 2
```

### An Empty List Entry Is Rejected

```console
$ fdu --no-cache --view tree,,types project
fdu: invalid --view "tree,,types": empty entry in the list
? 2
```

### An Unknown Format Lists Every Valid One

```console
$ fdu --no-cache --format xml project
fdu: invalid --format "xml": expected one of text, json, jsonl, yaml
? 2
```

### Fractional Ages Point at the Compound Spelling

```console
$ fdu --no-cache --modified-since 1.5h project
fdu: invalid time "1.5h": fractional ages are not supported; write them as compounds, as in `1h30m` rather than `1.5h`
? 2
```

### Calendar Units Point at Days

```console
$ fdu --no-cache --modified-before 3months project
fdu: invalid time "3months": calendar units are not supported because they are not a fixed length; use days, as in `30d` or `365d`
? 2
```

### Local Timestamps Ask for an Offset Rather Than Guessing

Guessing UTC would answer a prompt in another timezone hours off, in silence.

```console
$ fdu --no-cache --modified-since 2026-08-10 project
fdu: invalid time "2026-08-10": local date and time are not supported yet because resolving one needs a time-zone database; write an RFC 3339 timestamp with an offset, as in `2026-08-10T12:30:00Z` or `2026-08-10T12:30:00-08:00`, or use `@` epoch seconds
? 2
```

### An Unknown Size Unit Lists the Accepted Ones

```console
$ fdu --no-cache --min-size 10X project
fdu: invalid size "10X": unknown size unit "X"; use B, K/KB, M/MB, G/GB, T/TB, P/PB, or the binary forms KiB, MiB, GiB, TiB, PiB
? 2
```

### A Malformed Glob Says Which Delimiter Is Unmatched

```console
$ fdu --no-cache --include '{a,b' project
fdu: invalid pattern "{a,b": unmatched `{` in pattern
? 2
```

### A Bad Bound Names Both Accepted Forms

```console
$ fdu --no-cache --depth two project
fdu: invalid --depth "two": expected a whole number or `all`
? 2
```
