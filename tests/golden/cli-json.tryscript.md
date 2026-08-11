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
$ fdu --no-cache --format json --size apparent --depth 2 --limit 10 project
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
      "view": "tree",
      "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 263, "allocated": [ALLOCATED], "files": 6, "dirs": 3, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": [{"name": "dist", "path": "dist", "kind": "dir", "bytes": 128, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []},{"name": "src", "path": "src", "kind": "dir", "bytes": 36, "allocated": [ALLOCATED], "files": 2, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []},{"name": "docs", "path": "docs", "kind": "dir", "bytes": 23, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": []}]}
    }
  ]
}
? 0
```

## Render Limits Mark the Projection as Truncated

```console
$ fdu --no-cache --format json --size apparent --depth 1 --limit 2 project
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
      "view": "tree",
      "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 263, "allocated": [ALLOCATED], "files": 6, "dirs": 3, "newest_mtime_ns": [MTIME_NS], "truncated": true, "children": [{"name": "dist", "path": "dist", "kind": "dir", "bytes": 128, "allocated": [ALLOCATED], "files": 1, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": true, "children": []},{"name": "src", "path": "src", "kind": "dir", "bytes": 36, "allocated": [ALLOCATED], "files": 2, "dirs": 0, "newest_mtime_ns": [MTIME_NS], "truncated": true, "children": []}]}
    }
  ]
}
? 0
```

## Scan Depth Is an Explicit Complete Scope

```console
$ fdu --no-cache --format json --size apparent --scan-depth 1 --depth 2 --limit 10 project
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
      "view": "tree",
      "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 76, "allocated": [ALLOCATED], "files": 2, "dirs": 3, "newest_mtime_ns": [MTIME_NS], "truncated": false, "children": [{"name": "dist", "path": "dist", "kind": "dir", "bytes": 0, "allocated": [ALLOCATED], "files": 0, "dirs": 0, "newest_mtime_ns": null, "truncated": false, "children": []},{"name": "docs", "path": "docs", "kind": "dir", "bytes": 0, "allocated": [ALLOCATED], "files": 0, "dirs": 0, "newest_mtime_ns": null, "truncated": false, "children": []},{"name": "src", "path": "src", "kind": "dir", "bytes": 0, "allocated": [ALLOCATED], "files": 0, "dirs": 0, "newest_mtime_ns": null, "truncated": false, "children": []}]}
    }
  ]
}
? 0
```

## Every View Serializes in Every Format

This block previously asserted the opposite: `--by-type` conflicted with `--json`,
because the type breakdown was a human-only feature. Under the axis design a view and a
format are independent choices, so the combination is not just legal but required to
work — formats are serializations, not features.

```console
$ fdu --no-cache --view types --format json --size apparent project
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
      "view": "types",
      "types": [
        {"extension": ".tar.gz", "files": 1, "bytes": 128, "allocated": [ALLOCATED]},
        {"extension": ".md", "files": 2, "bytes": 71, "allocated": [ALLOCATED]},
        {"extension": ".rs", "files": 2, "bytes": 36, "allocated": [ALLOCATED]}
      ]
    }
  ]
}
? 0
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
