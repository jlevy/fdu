---
sandbox: true
fixtures:
  - fixtures/content-project
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
---
# Content Metrics CLI Contract

The fixture combines source, prose, data, known binary data, Unicode, and every common
line-ending convention.
Generated files use explicit byte buffers so the checkout’s line-ending policy cannot
alter the corpus.

## Prepare Mixed Line Endings and Observed Binary Data

```console
$ node -e "const fs=require('node:fs'); fs.writeFileSync('content-project/src/tool.py', Buffer.from('#!/usr/bin/env python3\r\nprint(\"hi\")\r\n\r\n')); fs.writeFileSync('content-project/docs/notes.txt', Buffer.from('one two\r\r三 四\r')); fs.writeFileSync('content-project/assets/late.bin.txt', Buffer.concat([Buffer.from('valid text\n'), Buffer.from([0]), Buffer.from('binary')])); console.log('content fixture prepared')"
content fixture prepared
? 0
```

## Human Summaries Separate Code and Documents

```console
$ fdu --cache off --analyze basic --view languages,documents --size apparent content-project
      39 B   50.6%  python             1 file, 3 lines (2 nonblank, 1 blank)
      38 B   49.4%  rust               1 file, 4 lines (3 nonblank, 1 blank)

      42 B   54.5%  markdown           1 file, 5 lines (3 nonblank, 2 blank), 7 words (0.0 pages)
      35 B   45.5%  text               2 files, 3 lines (2 nonblank, 1 blank), 4 words (0.0 pages)
? 0
```

## JSON Exposes Provenance, Exact Shares, Metrics, and Coverage

```console
$ fdu --cache off --analyze basic --view types,families --format json --size apparent content-project
{
  "schema": "fdu.report/2",
  "generator": "fdu 0.0.1",
  "root": "[SCAN_PATH]",
  "scan_started_at": "[RFC3339]",
  "generated_at": "[RFC3339]",
  "source": "cold_scan",
  "freshness": "fresh",
  "complete": true,
  "errors": [],
  "analysis": {"profile": "basic", "type_rules_fingerprint": 17146903830125060145, "options_fingerprint": 6866036729376448015, "analyzers": [{"id": "content-basic-v1", "version": 1}]},
  "reports": [
    {
      "view": "types",
      "metrics": {"group": "type", "words_per_page": 300, "total": {"id": "total", "family": "unknown", "files": 7, "bytes": 256, "allocated": [ALLOCATED], "analyzed_files": 5, "share": {"numerator": 256, "denominator": 256}, "metrics": {"physical_lines": 18, "blank_lines": 5, "nonblank_lines": 13, "code_lines": 0, "comment_lines": 0, "raw_words": 11, "logical_words": 10, "paragraphs": 5, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 5, "binary": 2}, "pages": {"words": 11, "words_per_page": 300}}, "rows": [
      {"id": "image", "family": "binary", "files": 1, "bytes": 80, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 80, "denominator": 256}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"binary": 1}, "pages": {"words": 0, "words_per_page": 300}},
      {"id": "markdown", "family": "prose", "files": 1, "bytes": 42, "allocated": [ALLOCATED], "analyzed_files": 1, "share": {"numerator": 42, "denominator": 256}, "metrics": {"physical_lines": 5, "blank_lines": 2, "nonblank_lines": 3, "code_lines": 0, "comment_lines": 0, "raw_words": 7, "logical_words": 7, "paragraphs": 3, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 1}, "pages": {"words": 7, "words_per_page": 300}},
      {"id": "python", "family": "code", "files": 1, "bytes": 39, "allocated": [ALLOCATED], "analyzed_files": 1, "share": {"numerator": 39, "denominator": 256}, "metrics": {"physical_lines": 3, "blank_lines": 1, "nonblank_lines": 2, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 1}, "pages": {"words": 0, "words_per_page": 300}},
      {"id": "rust", "family": "code", "files": 1, "bytes": 38, "allocated": [ALLOCATED], "analyzed_files": 1, "share": {"numerator": 38, "denominator": 256}, "metrics": {"physical_lines": 4, "blank_lines": 1, "nonblank_lines": 3, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 1}, "pages": {"words": 0, "words_per_page": 300}},
      {"id": "text", "family": "prose", "files": 2, "bytes": 35, "allocated": [ALLOCATED], "analyzed_files": 1, "share": {"numerator": 35, "denominator": 256}, "metrics": {"physical_lines": 3, "blank_lines": 1, "nonblank_lines": 2, "code_lines": 0, "comment_lines": 0, "raw_words": 4, "logical_words": 3, "paragraphs": 2, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 1, "binary": 1}, "pages": {"words": 4, "words_per_page": 300}},
      {"id": "json", "family": "data", "files": 1, "bytes": 22, "allocated": [ALLOCATED], "analyzed_files": 1, "share": {"numerator": 22, "denominator": 256}, "metrics": {"physical_lines": 3, "blank_lines": 0, "nonblank_lines": 3, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 1}, "pages": {"words": 0, "words_per_page": 300}}
    ]}
    },
    {
      "view": "families",
      "metrics": {"group": "family", "words_per_page": 300, "total": {"id": "total", "family": "unknown", "files": 7, "bytes": 256, "allocated": [ALLOCATED], "analyzed_files": 5, "share": {"numerator": 256, "denominator": 256}, "metrics": {"physical_lines": 18, "blank_lines": 5, "nonblank_lines": 13, "code_lines": 0, "comment_lines": 0, "raw_words": 11, "logical_words": 10, "paragraphs": 5, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 5, "binary": 2}, "pages": {"words": 11, "words_per_page": 300}}, "rows": [
      {"id": "binary", "family": "binary", "files": 1, "bytes": 80, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 80, "denominator": 256}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"binary": 1}, "pages": {"words": 0, "words_per_page": 300}},
      {"id": "code", "family": "code", "files": 2, "bytes": 77, "allocated": [ALLOCATED], "analyzed_files": 2, "share": {"numerator": 77, "denominator": 256}, "metrics": {"physical_lines": 7, "blank_lines": 2, "nonblank_lines": 5, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 2}, "pages": {"words": 0, "words_per_page": 300}},
      {"id": "prose", "family": "prose", "files": 3, "bytes": 77, "allocated": [ALLOCATED], "analyzed_files": 2, "share": {"numerator": 77, "denominator": 256}, "metrics": {"physical_lines": 8, "blank_lines": 3, "nonblank_lines": 5, "code_lines": 0, "comment_lines": 0, "raw_words": 11, "logical_words": 10, "paragraphs": 5, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 2, "binary": 1}, "pages": {"words": 11, "words_per_page": 300}},
      {"id": "data", "family": "data", "files": 1, "bytes": 22, "allocated": [ALLOCATED], "analyzed_files": 1, "share": {"numerator": 22, "denominator": 256}, "metrics": {"physical_lines": 3, "blank_lines": 0, "nonblank_lines": 3, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 1}, "pages": {"words": 0, "words_per_page": 300}}
    ]}
    }
  ]
}
? 0
```

## JSONL Keeps the Envelope and Section Independently Parseable

```console
$ fdu --cache off --analyze basic --view documents --format jsonl --size apparent --limit 1 content-project
{"schema": "fdu.report/2", "generator": "fdu 0.0.1", "root": "[SCAN_PATH]", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "source": "cold_scan", "freshness": "fresh", "complete": true, "errors": [], "analysis": {"profile": "basic", "type_rules_fingerprint": 17146903830125060145, "options_fingerprint": 6866036729376448015, "analyzers": [{"id": "content-basic-v1", "version": 1}]}}
{"view": "documents", "metrics": {"group": "type", "words_per_page": 300, "total": {"id": "total", "family": "unknown", "files": 3, "bytes": 77, "allocated": [ALLOCATED], "analyzed_files": 2, "share": {"numerator": 77, "denominator": 77}, "metrics": {"physical_lines": 8, "blank_lines": 3, "nonblank_lines": 5, "code_lines": 0, "comment_lines": 0, "raw_words": 11, "logical_words": 10, "paragraphs": 5, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 2, "binary": 1}, "pages": {"words": 11, "words_per_page": 300}}, "rows": [{"id": "markdown", "family": "prose", "files": 1, "bytes": 42, "allocated": [ALLOCATED], "analyzed_files": 1, "share": {"numerator": 42, "denominator": 77}, "metrics": {"physical_lines": 5, "blank_lines": 2, "nonblank_lines": 3, "code_lines": 0, "comment_lines": 0, "raw_words": 7, "logical_words": 7, "paragraphs": 3, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 1}, "pages": {"words": 7, "words_per_page": 300}}]}}
? 0
```

## YAML Derives Pages Only After Document Aggregation

```console
$ fdu --cache off --analyze basic --view documents --format yaml --size apparent --words-per-page 5 content-project
schema: fdu.report/2
generator: "fdu 0.0.1"
root: [SCAN_PATH]
scan_started_at: "[RFC3339]"
generated_at: "[RFC3339]"
source: cold_scan
freshness: fresh
complete: true
errors: []
analysis:
  profile: basic
  type_rules_fingerprint: 17146903830125060145
  options_fingerprint: 6866036729376448015
  analyzers:
    - id: content-basic-v1
      version: 1
reports:
  - view: documents
    metrics:
      group: type
      words_per_page: 5
      total:
        id: total
        family: unknown
        files: 3
        bytes: 77
        allocated: [ALLOCATED]
        analyzed_files: 2
        share_numerator: 77
        share_denominator: 77
        physical_lines: 8
        blank_lines: 3
        nonblank_lines: 5
        code_lines: 0
        comment_lines: 0
        raw_words: 11
        logical_words: 10
        paragraphs: 5
        visible_words: 0
        page_words: 11
        words_per_page: 5
        coverage:
          analyzed: 2
          binary: 1
      rows:
        - id: markdown
          family: prose
          files: 1
          bytes: 42
          allocated: [ALLOCATED]
          analyzed_files: 1
          share_numerator: 42
          share_denominator: 77
          physical_lines: 5
          blank_lines: 2
          nonblank_lines: 3
          code_lines: 0
          comment_lines: 0
          raw_words: 7
          logical_words: 7
          paragraphs: 3
          visible_words: 0
          page_words: 7
          words_per_page: 5
          coverage:
            analyzed: 1
        - id: text
          family: prose
          files: 2
          bytes: 35
          allocated: [ALLOCATED]
          analyzed_files: 1
          share_numerator: 35
          share_denominator: 77
          physical_lines: 3
          blank_lines: 1
          nonblank_lines: 2
          code_lines: 0
          comment_lines: 0
          raw_words: 4
          logical_words: 3
          paragraphs: 2
          visible_words: 0
          page_words: 4
          words_per_page: 5
          coverage:
            analyzed: 1
            binary: 1
? 0
```

## A Byte Bound Is Explicitly Partial

Without `--allow-partial`, bounded-out text files produce their rows and coverage but a
partial-result exit status.

```console
$ fdu --cache off --analyze basic --max-file-size 20 --view types --format jsonl --size apparent content-project
{"schema": "fdu.report/2", "generator": "fdu 0.0.1", "root": "[SCAN_PATH]", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "source": "cold_scan", "freshness": "fresh", "complete": false, "errors": ["content analysis incomplete: 0 invalid UTF-8, 4 too large, 0 I/O errors, 0 changed during read, 0 unsupported, 0 stale"], "analysis": {"profile": "basic", "type_rules_fingerprint": 17146903830125060145, "options_fingerprint": 14293159993244691768, "analyzers": [{"id": "content-basic-v1", "version": 1}]}}
{"view": "types", "metrics": {"group": "type", "words_per_page": 300, "total": {"id": "total", "family": "unknown", "files": 7, "bytes": 256, "allocated": [ALLOCATED], "analyzed_files": 1, "share": {"numerator": 256, "denominator": 256}, "metrics": {"physical_lines": 3, "blank_lines": 1, "nonblank_lines": 2, "code_lines": 0, "comment_lines": 0, "raw_words": 4, "logical_words": 3, "paragraphs": 2, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 1, "binary": 2, "too_large": 4}, "pages": {"words": 4, "words_per_page": 300}}, "rows": [{"id": "image", "family": "binary", "files": 1, "bytes": 80, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 80, "denominator": 256}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"binary": 1}, "pages": {"words": 0, "words_per_page": 300}}, {"id": "markdown", "family": "prose", "files": 1, "bytes": 42, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 42, "denominator": 256}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"too_large": 1}, "pages": {"words": 0, "words_per_page": 300}}, {"id": "python", "family": "code", "files": 1, "bytes": 39, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 39, "denominator": 256}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"too_large": 1}, "pages": {"words": 0, "words_per_page": 300}}, {"id": "rust", "family": "code", "files": 1, "bytes": 38, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 38, "denominator": 256}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"too_large": 1}, "pages": {"words": 0, "words_per_page": 300}}, {"id": "text", "family": "prose", "files": 2, "bytes": 35, "allocated": [ALLOCATED], "analyzed_files": 1, "share": {"numerator": 35, "denominator": 256}, "metrics": {"physical_lines": 3, "blank_lines": 1, "nonblank_lines": 2, "code_lines": 0, "comment_lines": 0, "raw_words": 4, "logical_words": 3, "paragraphs": 2, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 1, "binary": 1}, "pages": {"words": 4, "words_per_page": 300}}, {"id": "json", "family": "data", "files": 1, "bytes": 22, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 22, "denominator": 256}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"too_large": 1}, "pages": {"words": 0, "words_per_page": 300}}]}}
? 2
```

The opt-in changes only the exit status, not the evidence in the report.

```console
$ fdu --cache off --allow-partial --analyze basic --max-file-size 20 --view types --format jsonl --size apparent content-project
{"schema": "fdu.report/2", "generator": "fdu 0.0.1", "root": "[SCAN_PATH]", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "source": "cold_scan", "freshness": "fresh", "complete": false, "errors": ["content analysis incomplete: 0 invalid UTF-8, 4 too large, 0 I/O errors, 0 changed during read, 0 unsupported, 0 stale"], "analysis": {"profile": "basic", "type_rules_fingerprint": 17146903830125060145, "options_fingerprint": 14293159993244691768, "analyzers": [{"id": "content-basic-v1", "version": 1}]}}
{"view": "types", "metrics": {"group": "type", "words_per_page": 300, "total": {"id": "total", "family": "unknown", "files": 7, "bytes": 256, "allocated": [ALLOCATED], "analyzed_files": 1, "share": {"numerator": 256, "denominator": 256}, "metrics": {"physical_lines": 3, "blank_lines": 1, "nonblank_lines": 2, "code_lines": 0, "comment_lines": 0, "raw_words": 4, "logical_words": 3, "paragraphs": 2, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 1, "binary": 2, "too_large": 4}, "pages": {"words": 4, "words_per_page": 300}}, "rows": [{"id": "image", "family": "binary", "files": 1, "bytes": 80, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 80, "denominator": 256}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"binary": 1}, "pages": {"words": 0, "words_per_page": 300}}, {"id": "markdown", "family": "prose", "files": 1, "bytes": 42, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 42, "denominator": 256}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"too_large": 1}, "pages": {"words": 0, "words_per_page": 300}}, {"id": "python", "family": "code", "files": 1, "bytes": 39, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 39, "denominator": 256}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"too_large": 1}, "pages": {"words": 0, "words_per_page": 300}}, {"id": "rust", "family": "code", "files": 1, "bytes": 38, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 38, "denominator": 256}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"too_large": 1}, "pages": {"words": 0, "words_per_page": 300}}, {"id": "text", "family": "prose", "files": 2, "bytes": 35, "allocated": [ALLOCATED], "analyzed_files": 1, "share": {"numerator": 35, "denominator": 256}, "metrics": {"physical_lines": 3, "blank_lines": 1, "nonblank_lines": 2, "code_lines": 0, "comment_lines": 0, "raw_words": 4, "logical_words": 3, "paragraphs": 2, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 1, "binary": 1}, "pages": {"words": 4, "words_per_page": 300}}, {"id": "json", "family": "data", "files": 1, "bytes": 22, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 22, "denominator": 256}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"too_large": 1}, "pages": {"words": 0, "words_per_page": 300}}]}}
? 0
```

## Content Cache Hits Preserve the Same Tallies

```console
$ fdu --analyze basic --view documents --format jsonl --size apparent content-project
{"schema": "fdu.report/2", "generator": "fdu 0.0.1", "root": "[SCAN_PATH]", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "source": "cold_scan", "freshness": "fresh", "complete": true, "errors": [], "analysis": {"profile": "basic", "type_rules_fingerprint": 17146903830125060145, "options_fingerprint": 6866036729376448015, "analyzers": [{"id": "content-basic-v1", "version": 1}]}}
{"view": "documents", "metrics": {"group": "type", "words_per_page": 300, "total": {"id": "total", "family": "unknown", "files": 3, "bytes": 77, "allocated": [ALLOCATED], "analyzed_files": 2, "share": {"numerator": 77, "denominator": 77}, "metrics": {"physical_lines": 8, "blank_lines": 3, "nonblank_lines": 5, "code_lines": 0, "comment_lines": 0, "raw_words": 11, "logical_words": 10, "paragraphs": 5, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 2, "binary": 1}, "pages": {"words": 11, "words_per_page": 300}}, "rows": [{"id": "markdown", "family": "prose", "files": 1, "bytes": 42, "allocated": [ALLOCATED], "analyzed_files": 1, "share": {"numerator": 42, "denominator": 77}, "metrics": {"physical_lines": 5, "blank_lines": 2, "nonblank_lines": 3, "code_lines": 0, "comment_lines": 0, "raw_words": 7, "logical_words": 7, "paragraphs": 3, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 1}, "pages": {"words": 7, "words_per_page": 300}}, {"id": "text", "family": "prose", "files": 2, "bytes": 35, "allocated": [ALLOCATED], "analyzed_files": 1, "share": {"numerator": 35, "denominator": 77}, "metrics": {"physical_lines": 3, "blank_lines": 1, "nonblank_lines": 2, "code_lines": 0, "comment_lines": 0, "raw_words": 4, "logical_words": 3, "paragraphs": 2, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 1, "binary": 1}, "pages": {"words": 4, "words_per_page": 300}}]}}
? 0
```

```console
$ fdu --cache only --analyze basic --view documents --format jsonl --size apparent content-project
{"schema": "fdu.report/2", "generator": "fdu 0.0.1", "root": "[SCAN_PATH]", "scan_started_at": "[RFC3339]", "generated_at": "[RFC3339]", "source": "cache_only", "freshness": "stale", "complete": true, "errors": [], "analysis": {"profile": "basic", "type_rules_fingerprint": 17146903830125060145, "options_fingerprint": 6866036729376448015, "analyzers": [{"id": "content-basic-v1", "version": 1}]}}
{"view": "documents", "metrics": {"group": "type", "words_per_page": 300, "total": {"id": "total", "family": "unknown", "files": 3, "bytes": 77, "allocated": [ALLOCATED], "analyzed_files": 2, "share": {"numerator": 77, "denominator": 77}, "metrics": {"physical_lines": 8, "blank_lines": 3, "nonblank_lines": 5, "code_lines": 0, "comment_lines": 0, "raw_words": 11, "logical_words": 10, "paragraphs": 5, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 2, "binary": 1}, "pages": {"words": 11, "words_per_page": 300}}, "rows": [{"id": "markdown", "family": "prose", "files": 1, "bytes": 42, "allocated": [ALLOCATED], "analyzed_files": 1, "share": {"numerator": 42, "denominator": 77}, "metrics": {"physical_lines": 5, "blank_lines": 2, "nonblank_lines": 3, "code_lines": 0, "comment_lines": 0, "raw_words": 7, "logical_words": 7, "paragraphs": 3, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 1}, "pages": {"words": 7, "words_per_page": 300}}, {"id": "text", "family": "prose", "files": 2, "bytes": 35, "allocated": [ALLOCATED], "analyzed_files": 1, "share": {"numerator": 35, "denominator": 77}, "metrics": {"physical_lines": 3, "blank_lines": 1, "nonblank_lines": 2, "code_lines": 0, "comment_lines": 0, "raw_words": 4, "logical_words": 3, "paragraphs": 2, "visible_words": 0, "visible_logical_words": 0}, "coverage": {"analyzed": 1, "binary": 1}, "pages": {"words": 4, "words_per_page": 300}}]}}
? 0
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
