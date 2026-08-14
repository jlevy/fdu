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
patterns:
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
  "schema": "fdu.report/1",
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
      "view": "tree",
      "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 263, "allocated": [ALLOCATED], "files": 6, "dirs": 3, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": [{"name": "dist", "path": "dist", "kind": "dir", "bytes": 128, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []},{"name": "src", "path": "src", "kind": "dir", "bytes": 36, "allocated": [ALLOCATED], "files": 2, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []},{"name": "docs", "path": "docs", "kind": "dir", "bytes": 23, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}]}
    }
  ]
}
? 0
```

## Render Limits Mark the Projection as Truncated

```console
$ fdu --cache off --format json --size apparent --depth 1 --limit 2 project
{
  "schema": "fdu.report/1",
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
      "view": "tree",
      "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 263, "allocated": [ALLOCATED], "files": 6, "dirs": 3, "newest_mtime_ns": [MTIME_NS], "truncated": true, "children": [{"name": "dist", "path": "dist", "kind": "dir", "bytes": 128, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []},{"name": "src", "path": "src", "kind": "dir", "bytes": 36, "allocated": [ALLOCATED], "files": 2, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}]}
    }
  ]
}
? 0
```

## Scan Depth Is an Explicit Complete Scope

```console
$ fdu --cache off --format json --size apparent --scan-depth 1 --depth 2 --limit 10 project
{
  "schema": "fdu.report/1",
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
      "view": "tree",
      "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 76, "allocated": [ALLOCATED], "files": 2, "dirs": 3, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": [{"name": "dist", "path": "dist", "kind": "dir", "bytes": 0, "allocated": [ALLOCATED], "files": 0, "dirs": 0, "newest_mtime_ns": null, "truncated": false, "children": []},{"name": "docs", "path": "docs", "kind": "dir", "bytes": 0, "allocated": [ALLOCATED], "files": 0, "dirs": 0, "newest_mtime_ns": null, "truncated": false, "children": []},{"name": "src", "path": "src", "kind": "dir", "bytes": 0, "allocated": [ALLOCATED], "files": 0, "dirs": 0, "newest_mtime_ns": null, "truncated": false, "children": []}]}
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
  "schema": "fdu.report/2",
  "generator": "fdu 0.1.0",
  "root": "[SCAN_PATH]",
  "scan_started_at": "[RFC3339]",
  "generated_at": "[RFC3339]",
  "source": "cold_scan",
  "freshness": "fresh",
  "complete": true,
  "errors": [],
  "analysis": null,
  "reports": [
    {
      "view": "types",
      "metrics": {"group": "type", "share_metric": "apparent_bytes", "words_per_page": 250, "total": {"id": "total", "family": "unknown", "files": 6, "bytes": 263, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 263, "denominator": 263}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "code_blank_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0, "document_words": 0}, "coverage": {}, "detection": {"sources": {"exact_filename": 1, "compound_extension": 1, "extension": 4}, "confidence": {"certain": 6}, "flags": {"generated": 0, "vendored": 0, "documentation": 2}}, "pages": {"words": 0, "words_per_page": 250}}, "rows": [
      {"id": "archive", "family": "binary", "files": 1, "bytes": 128, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 128, "denominator": 263}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "code_blank_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0, "document_words": 0}, "coverage": {}, "detection": {"sources": {"compound_extension": 1}, "confidence": {"certain": 1}, "flags": {"generated": 0, "vendored": 0, "documentation": 0}}, "pages": {"words": 0, "words_per_page": 250}},
      {"id": "markdown", "family": "prose", "files": 2, "bytes": 71, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 71, "denominator": 263}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "code_blank_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0, "document_words": 0}, "coverage": {}, "detection": {"sources": {"extension": 2}, "confidence": {"certain": 2}, "flags": {"generated": 0, "vendored": 0, "documentation": 2}}, "pages": {"words": 0, "words_per_page": 250}},
      {"id": "rust", "family": "code", "files": 2, "bytes": 36, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 36, "denominator": 263}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "code_blank_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0, "document_words": 0}, "coverage": {}, "detection": {"sources": {"extension": 2}, "confidence": {"certain": 2}, "flags": {"generated": 0, "vendored": 0, "documentation": 0}}, "pages": {"words": 0, "words_per_page": 250}},
      {"id": "make", "family": "code", "files": 1, "bytes": 28, "allocated": [ALLOCATED], "analyzed_files": 0, "share": {"numerator": 28, "denominator": 263}, "metrics": {"physical_lines": 0, "blank_lines": 0, "nonblank_lines": 0, "code_lines": 0, "comment_lines": 0, "code_blank_lines": 0, "raw_words": 0, "logical_words": 0, "paragraphs": 0, "visible_words": 0, "visible_logical_words": 0, "document_words": 0}, "coverage": {}, "detection": {"sources": {"exact_filename": 1}, "confidence": {"certain": 1}, "flags": {"generated": 0, "vendored": 0, "documentation": 0}}, "pages": {"words": 0, "words_per_page": 250}}
    ]}
    }
  ]
}
? 0
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
