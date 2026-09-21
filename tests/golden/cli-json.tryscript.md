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
patterns:
  JSON_SEP: '(?:/|\\\\)'
  AGE_NS: '-?\d+'
  RFC3339: '\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{9}Z'
  ALLOCATED: '\d+'
  MTIME_NS: '-?\d+'
  SCAN_PATH: '[^\r\n]+'
---
# JSON CLI Output

## Full Output Exposes Scan and Projection Completeness Separately

```console
$ fdu --cache off --format json --size apparent --depth 2 --limit 10 project
{
  "schema": "fdu.report/7",
  "generator": "fdu 0.1.0",
  "root": "[SCAN_PATH]",
  "scan_started_at": "[RFC3339]",
  "generated_at": "[RFC3339]",
  "age_reference_ns": [AGE_NS],
  "source": "cold_scan",
  "freshness": "fresh",
  "complete": true,
  "errors": [],
  "ignore_rules": {"limits": {"budget": 4194304, "line_limit": 16384}, "applied": 1, "refused": 0, "refusals": []},
  "reports": [
    {
      "view": "list",
      "bound": null, "files": [
        {"path": "dist", "kind": "dir", "bytes": 128, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "files": 1, "dirs": 0, "age_ns": [AGE_NS], "ignored": true},
        {"path": "dist[JSON_SEP]acorn-0.1.0.tar.gz", "kind": "file", "bytes": 128, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "files": null, "dirs": null, "age_ns": [AGE_NS], "ignored": true},
        {"path": "README.md", "kind": "file", "bytes": 48, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "files": null, "dirs": null, "age_ns": [AGE_NS], "ignored": false},
        {"path": "src", "kind": "dir", "bytes": 36, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "files": 2, "dirs": 0, "age_ns": [AGE_NS], "ignored": false},
        {"path": "Makefile", "kind": "file", "bytes": 28, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "files": null, "dirs": null, "age_ns": [AGE_NS], "ignored": false},
        {"path": "docs", "kind": "dir", "bytes": 23, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "files": 1, "dirs": 0, "age_ns": [AGE_NS], "ignored": false},
        {"path": "docs[JSON_SEP]FAQ.MD", "kind": "file", "bytes": 23, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "files": null, "dirs": null, "age_ns": [AGE_NS], "ignored": false},
        {"path": "src[JSON_SEP]alpha.rs", "kind": "file", "bytes": 18, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "files": null, "dirs": null, "age_ns": [AGE_NS], "ignored": false},
        {"path": "src[JSON_SEP]omega.rs", "kind": "file", "bytes": 18, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "files": null, "dirs": null, "age_ns": [AGE_NS], "ignored": false},
        {"path": ".gitignore", "kind": "file", "bytes": 6, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "files": null, "dirs": null, "age_ns": [AGE_NS], "ignored": false}
      ]
    }
  ]
}
? 0
```

## Render Limits Mark the Projection as Truncated

```console
$ fdu --cache off --format json --size apparent --depth 1 --limit 2 project
{
  "schema": "fdu.report/7",
  "generator": "fdu 0.1.0",
  "root": "[SCAN_PATH]",
  "scan_started_at": "[RFC3339]",
  "generated_at": "[RFC3339]",
  "age_reference_ns": [AGE_NS],
  "source": "cold_scan",
  "freshness": "fresh",
  "complete": true,
  "errors": [],
  "ignore_rules": {"limits": {"budget": 4194304, "line_limit": 16384}, "applied": 1, "refused": 0, "refusals": []},
  "reports": [
    {
      "view": "list",
      "bound": {"shown": 2, "total": 10}, "files": [
        {"path": "dist", "kind": "dir", "bytes": 128, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "files": 1, "dirs": 0, "age_ns": [AGE_NS], "ignored": true},
        {"path": "dist[JSON_SEP]acorn-0.1.0.tar.gz", "kind": "file", "bytes": 128, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "files": null, "dirs": null, "age_ns": [AGE_NS], "ignored": true}
      ]
    }
  ]
}
? 0
```

## Scan Depth Is an Explicit Complete Scope

```console
$ fdu --cache off --format json --size apparent --scan-depth 1 --depth 2 --limit 10 project
{
  "schema": "fdu.report/7",
  "generator": "fdu 0.1.0",
  "root": "[SCAN_PATH]",
  "scan_started_at": "[RFC3339]",
  "generated_at": "[RFC3339]",
  "age_reference_ns": [AGE_NS],
  "source": "cold_scan",
  "freshness": "fresh",
  "complete": true,
  "errors": [],
  "ignore_rules": {"limits": {"budget": 4194304, "line_limit": 16384}, "applied": 1, "refused": 0, "refusals": []},
  "reports": [
    {
      "view": "list",
      "bound": null, "files": [
        {"path": "README.md", "kind": "file", "bytes": 48, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "files": null, "dirs": null, "age_ns": [AGE_NS], "ignored": false},
        {"path": "Makefile", "kind": "file", "bytes": 28, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "files": null, "dirs": null, "age_ns": [AGE_NS], "ignored": false},
        {"path": ".gitignore", "kind": "file", "bytes": 6, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "files": null, "dirs": null, "age_ns": [AGE_NS], "ignored": false},
        {"path": "dist", "kind": "dir", "bytes": 0, "allocated": 0, "mtime_ns": [MTIME_NS], "files": 0, "dirs": 0, "age_ns": [AGE_NS], "ignored": true},
        {"path": "docs", "kind": "dir", "bytes": 0, "allocated": 0, "mtime_ns": [MTIME_NS], "files": 0, "dirs": 0, "age_ns": [AGE_NS], "ignored": false},
        {"path": "src", "kind": "dir", "bytes": 0, "allocated": 0, "mtime_ns": [MTIME_NS], "files": 0, "dirs": 0, "age_ns": [AGE_NS], "ignored": false}
      ]
    }
  ]
}
? 0
```

## Every View Serializes in Every Format

This block previously asserted the opposite: `--by-type` conflicted with `--json`,
because the type breakdown was a human-only feature.
Under the axis design a view and a format are independent choices, so the combination is
not just legal but required to work — formats are serializations, not features.

```console
$ fdu --cache off --view types --format json --size apparent project
{
  "schema": "fdu.report/8",
  "generator": "fdu 0.1.0",
  "root": "[SCAN_PATH]",
  "scan_started_at": "[RFC3339]",
  "generated_at": "[RFC3339]",
  "age_reference_ns": [AGE_NS],
  "source": "cold_scan",
  "freshness": "fresh",
  "complete": true,
  "errors": [],
  "ignore_rules": {"limits": {"budget": 4194304, "line_limit": 16384}, "applied": 1, "refused": 0, "refusals": []},
  "analysis": null,
  "reports": [
    {
      "view": "types",
      "metrics": {"group": "type", "share_metric": "apparent_bytes", "words_per_page": 250, "bound": null, "total": {"id": "total", "family": "unknown", "files": 7, "bytes": 269, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 269, "denominator": 269}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "code_blank_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0, "document_words": 0}, "coverage": {}, "detection": {"sources": {"exact_filename": 1, "compound_extension": 1, "extension": 4, "unknown": 1}, "confidence": {"certain": 6, "heuristic": 1}, "flags": {"generated": 0, "vendored": 0, "documentation": 2}}, "pages": {"words": 0, "words_per_page": 250}}, "rows": [
      {"id": "archive", "family": "binary", "files": 1, "bytes": 128, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 128, "denominator": 269}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "code_blank_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0, "document_words": 0}, "coverage": {}, "detection": {"sources": {"compound_extension": 1}, "confidence": {"certain": 1}, "flags": {"generated": 0, "vendored": 0, "documentation": 0}}, "pages": {"words": 0, "words_per_page": 250}},
      {"id": "markdown", "family": "prose", "files": 2, "bytes": 71, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 71, "denominator": 269}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "code_blank_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0, "document_words": 0}, "coverage": {}, "detection": {"sources": {"extension": 2}, "confidence": {"certain": 2}, "flags": {"generated": 0, "vendored": 0, "documentation": 2}}, "pages": {"words": 0, "words_per_page": 250}},
      {"id": "rust", "family": "code", "files": 2, "bytes": 36, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 36, "denominator": 269}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "code_blank_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0, "document_words": 0}, "coverage": {}, "detection": {"sources": {"extension": 2}, "confidence": {"certain": 2}, "flags": {"generated": 0, "vendored": 0, "documentation": 0}}, "pages": {"words": 0, "words_per_page": 250}},
      {"id": "make", "family": "code", "files": 1, "bytes": 28, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 28, "denominator": 269}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "code_blank_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0, "document_words": 0}, "coverage": {}, "detection": {"sources": {"exact_filename": 1}, "confidence": {"certain": 1}, "flags": {"generated": 0, "vendored": 0, "documentation": 0}}, "pages": {"words": 0, "words_per_page": 250}},
      {"id": "unknown", "family": "unknown", "files": 1, "bytes": 6, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 6, "denominator": 269}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "code_blank_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0, "document_words": 0}, "coverage": {}, "detection": {"sources": {"unknown": 1}, "confidence": {"heuristic": 1}, "flags": {"generated": 0, "vendored": 0, "documentation": 0}}, "pages": {"words": 0, "words_per_page": 250}}
    ]}
    }
  ]
}
? 0
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
