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
    "views": ["tree"],
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
      "view": "tree",
      "tree": {
        "name": ".",
        "path": "",
        "kind": "dir",
        "bytes": 269,
        "allocated": [ALLOCATED],
        "files": 7,
        "dirs": 3,
        "ignored": {"files": 1, "dirs": 1, "bytes": 128, "allocated": [ALLOCATED]},
        "newest_mtime_ns": [MTIME_NS],
        "truncated": false,
        "children": [
          {
            "name": "dist",
            "path": "dist",
            "kind": "dir",
            "bytes": 128,
            "allocated": [ALLOCATED],
            "files": 1,
            "dirs": 0,
            "ignored": {"files": 1, "dirs": 0, "bytes": 128, "allocated": [ALLOCATED]},
            "newest_mtime_ns": [MTIME_NS],
            "truncated": false,
            "children": []
          },
          {
            "name": "src",
            "path": "src",
            "kind": "dir",
            "bytes": 36,
            "allocated": [ALLOCATED],
            "files": 2,
            "dirs": 0,
            "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0},
            "newest_mtime_ns": [MTIME_NS],
            "truncated": false,
            "children": []
          },
          {
            "name": "docs",
            "path": "docs",
            "kind": "dir",
            "bytes": 23,
            "allocated": [ALLOCATED],
            "files": 1,
            "dirs": 0,
            "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0},
            "newest_mtime_ns": [MTIME_NS],
            "truncated": false,
            "children": []
          }
        ]
      }
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
    "views": ["tree"],
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
      "view": "tree",
      "tree": {
        "name": ".",
        "path": "",
        "kind": "dir",
        "bytes": 269,
        "allocated": [ALLOCATED],
        "files": 7,
        "dirs": 3,
        "ignored": {"files": 1, "dirs": 1, "bytes": 128, "allocated": [ALLOCATED]},
        "newest_mtime_ns": [MTIME_NS],
        "truncated": true,
        "children": [
          {
            "name": "dist",
            "path": "dist",
            "kind": "dir",
            "bytes": 128,
            "allocated": [ALLOCATED],
            "files": 1,
            "dirs": 0,
            "ignored": {"files": 1, "dirs": 0, "bytes": 128, "allocated": [ALLOCATED]},
            "newest_mtime_ns": [MTIME_NS],
            "truncated": false,
            "children": []
          },
          {
            "name": "src",
            "path": "src",
            "kind": "dir",
            "bytes": 36,
            "allocated": [ALLOCATED],
            "files": 2,
            "dirs": 0,
            "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0},
            "newest_mtime_ns": [MTIME_NS],
            "truncated": false,
            "children": []
          }
        ]
      }
    }
  ]
}
? 0
```

## Scan Depth Is an Explicit Complete Scope

The report is complete for the scope it was asked for, and says so at the top.
The directories retained at the depth limit were never listed, so each of their rows
says `complete: false`: their sizes and counts are lower bounds and their age is null,
where a lower-bound maximum would have read as an old directory.

```console
$ fdu --cache off --format json --size apparent --scan-depth 1 --depth 2 --limit 10 project
{
  "schema": "fdu.report/7",
  "generator": "fdu 0.1.0",
  "root": "[SCAN_PATH]",
  "request": {
    "scope": {
      "max_depth": 1,
      "follow_symlinks": false,
      "one_filesystem": false,
      "exclude_special": false,
      "read_controls": true
    },
    "analyze": [],
    "size": "apparent",
    "views": ["tree"],
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
      "view": "tree",
      "tree": {
        "name": ".",
        "path": "",
        "kind": "dir",
        "bytes": 82,
        "allocated": [ALLOCATED],
        "files": 3,
        "dirs": 3,
        "ignored": {"files": 0, "dirs": 1, "bytes": 0, "allocated": 0},
        "newest_mtime_ns": [MTIME_NS],
        "truncated": false,
        "children": [
          {
            "name": "dist",
            "path": "dist",
            "kind": "dir",
            "bytes": 0,
            "allocated": 0,
            "files": 0,
            "dirs": 0,
            "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0},
            "newest_mtime_ns": null,
            "truncated": false,
            "children": []
          },
          {
            "name": "docs",
            "path": "docs",
            "kind": "dir",
            "bytes": 0,
            "allocated": 0,
            "files": 0,
            "dirs": 0,
            "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0},
            "newest_mtime_ns": null,
            "truncated": false,
            "children": []
          },
          {
            "name": "src",
            "path": "src",
            "kind": "dir",
            "bytes": 0,
            "allocated": 0,
            "files": 0,
            "dirs": 0,
            "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0},
            "newest_mtime_ns": null,
            "truncated": false,
            "children": []
          }
        ]
      }
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
    "views": ["types"],
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
      "view": "types",
      "metrics": {
        "group": "type",
        "share_metric": "apparent_bytes",
        "bound": null,
        "total": {
          "id": "total",
          "family": "unknown",
          "files": 7,
          "bytes": 269,
          "allocated": [ALLOCATED],
          "share": {"numerator": 269, "denominator": 269},
          "metrics": {},
          "coverage": {},
          "detection": {
            "sources": {"exact_filename": 1, "compound_extension": 1, "extension": 4, "unknown": 1},
            "confidence": {"certain": 6, "heuristic": 1},
            "flags": {"generated": 0, "vendored": 0, "documentation": 2}
          }
        },
        "rows": [
          {
            "id": "archive",
            "family": "binary",
            "files": 1,
            "bytes": 128,
            "allocated": [ALLOCATED],
            "share": {"numerator": 128, "denominator": 269},
            "metrics": {},
            "coverage": {},
            "detection": {
              "sources": {"compound_extension": 1},
              "confidence": {"certain": 1},
              "flags": {"generated": 0, "vendored": 0, "documentation": 0}
            }
          },
          {
            "id": "markdown",
            "family": "prose",
            "files": 2,
            "bytes": 71,
            "allocated": [ALLOCATED],
            "share": {"numerator": 71, "denominator": 269},
            "metrics": {},
            "coverage": {},
            "detection": {
              "sources": {"extension": 2},
              "confidence": {"certain": 2},
              "flags": {"generated": 0, "vendored": 0, "documentation": 2}
            }
          },
          {
            "id": "rust",
            "family": "code",
            "files": 2,
            "bytes": 36,
            "allocated": [ALLOCATED],
            "share": {"numerator": 36, "denominator": 269},
            "metrics": {},
            "coverage": {},
            "detection": {
              "sources": {"extension": 2},
              "confidence": {"certain": 2},
              "flags": {"generated": 0, "vendored": 0, "documentation": 0}
            }
          },
          {
            "id": "make",
            "family": "code",
            "files": 1,
            "bytes": 28,
            "allocated": [ALLOCATED],
            "share": {"numerator": 28, "denominator": 269},
            "metrics": {},
            "coverage": {},
            "detection": {
              "sources": {"exact_filename": 1},
              "confidence": {"certain": 1},
              "flags": {"generated": 0, "vendored": 0, "documentation": 0}
            }
          },
          {
            "id": "unknown",
            "family": "unknown",
            "files": 1,
            "bytes": 6,
            "allocated": [ALLOCATED],
            "share": {"numerator": 6, "denominator": 269},
            "metrics": {},
            "coverage": {},
            "detection": {
              "sources": {"unknown": 1},
              "confidence": {"heuristic": 1},
              "flags": {"generated": 0, "vendored": 0, "documentation": 0}
            }
          }
        ]
      }
    }
  ]
}
? 0
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
