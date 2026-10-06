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
$ fdu --cache off --format json --size apparent --limit 10 project
{
  "schema": "fdu.report/10",
  "generator": "fdu 0.3.0",
  "root": "[SCAN_PATH]",
  "age_reference_ns": [AGE_NS],
  "request": {
    "scope": {
      "max_depth": null,
      "follow_symlinks": false,
      "one_filesystem": false,
      "exclude_special": false,
      "read_controls": true,
      "population": "include"
    },
    "analyze": [],
    "size": "apparent",
    "sort_metric": null,
    "views": ["list"],
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
    "rules": 1,
    "refused": 0,
    "refusals": []
  },
  "analysis": null,
  "reports": [
    {
      "view": "list",
      "bound": null,
      "files": [
        {
          "path": "dist",
          "kind": "dir",
          "bytes": 128,
          "allocated": [ALLOCATED],
          "mtime_ns": [MTIME_NS],
          "files": 1,
          "dirs": 0,
          "complete": true,
          "age_ns": [AGE_NS],
          "ignored": true,
          "sort_value": null,
          "classification": null
        },
        {
          "path": "dist[JSON_SEP]acorn-0.1.0.tar.gz",
          "kind": "file",
          "bytes": 128,
          "allocated": [ALLOCATED],
          "mtime_ns": [MTIME_NS],
          "files": null,
          "dirs": null,
          "complete": null,
          "age_ns": [AGE_NS],
          "ignored": true,
          "sort_value": null,
          "classification": {
            "file_type": "archive",
            "family": "binary",
            "source": "compound_extension",
            "confidence": "certain",
            "flags": {"generated": false, "vendored": false, "documentation": false}
          }
        },
        {
          "path": "README.md",
          "kind": "file",
          "bytes": 48,
          "allocated": [ALLOCATED],
          "mtime_ns": [MTIME_NS],
          "files": null,
          "dirs": null,
          "complete": null,
          "age_ns": [AGE_NS],
          "ignored": false,
          "sort_value": null,
          "classification": {
            "file_type": "markdown",
            "family": "prose",
            "source": "extension",
            "confidence": "certain",
            "flags": {"generated": false, "vendored": false, "documentation": true}
          }
        },
        {
          "path": "src",
          "kind": "dir",
          "bytes": 36,
          "allocated": [ALLOCATED],
          "mtime_ns": [MTIME_NS],
          "files": 2,
          "dirs": 0,
          "complete": true,
          "age_ns": [AGE_NS],
          "ignored": false,
          "sort_value": null,
          "classification": null
        },
        {
          "path": "Makefile",
          "kind": "file",
          "bytes": 28,
          "allocated": [ALLOCATED],
          "mtime_ns": [MTIME_NS],
          "files": null,
          "dirs": null,
          "complete": null,
          "age_ns": [AGE_NS],
          "ignored": false,
          "sort_value": null,
          "classification": {
            "file_type": "make",
            "family": "code",
            "source": "exact_filename",
            "confidence": "certain",
            "flags": {"generated": false, "vendored": false, "documentation": false}
          }
        },
        {
          "path": "docs",
          "kind": "dir",
          "bytes": 23,
          "allocated": [ALLOCATED],
          "mtime_ns": [MTIME_NS],
          "files": 1,
          "dirs": 0,
          "complete": true,
          "age_ns": [AGE_NS],
          "ignored": false,
          "sort_value": null,
          "classification": null
        },
        {
          "path": "docs[JSON_SEP]FAQ.MD",
          "kind": "file",
          "bytes": 23,
          "allocated": [ALLOCATED],
          "mtime_ns": [MTIME_NS],
          "files": null,
          "dirs": null,
          "complete": null,
          "age_ns": [AGE_NS],
          "ignored": false,
          "sort_value": null,
          "classification": {
            "file_type": "markdown",
            "family": "prose",
            "source": "extension",
            "confidence": "certain",
            "flags": {"generated": false, "vendored": false, "documentation": true}
          }
        },
        {
          "path": "src[JSON_SEP]alpha.rs",
          "kind": "file",
          "bytes": 18,
          "allocated": [ALLOCATED],
          "mtime_ns": [MTIME_NS],
          "files": null,
          "dirs": null,
          "complete": null,
          "age_ns": [AGE_NS],
          "ignored": false,
          "sort_value": null,
          "classification": {
            "file_type": "rust",
            "family": "code",
            "source": "extension",
            "confidence": "certain",
            "flags": {"generated": false, "vendored": false, "documentation": false}
          }
        },
        {
          "path": "src[JSON_SEP]omega.rs",
          "kind": "file",
          "bytes": 18,
          "allocated": [ALLOCATED],
          "mtime_ns": [MTIME_NS],
          "files": null,
          "dirs": null,
          "complete": null,
          "age_ns": [AGE_NS],
          "ignored": false,
          "sort_value": null,
          "classification": {
            "file_type": "rust",
            "family": "code",
            "source": "extension",
            "confidence": "certain",
            "flags": {"generated": false, "vendored": false, "documentation": false}
          }
        },
        {
          "path": ".gitignore",
          "kind": "file",
          "bytes": 6,
          "allocated": [ALLOCATED],
          "mtime_ns": [MTIME_NS],
          "files": null,
          "dirs": null,
          "complete": null,
          "age_ns": [AGE_NS],
          "ignored": false,
          "sort_value": null,
          "classification": {
            "file_type": "unknown",
            "family": "unknown",
            "source": "unknown",
            "confidence": "heuristic",
            "flags": {"generated": false, "vendored": false, "documentation": false}
          }
        }
      ]
    }
  ]
}
? 0
```

## Render Limits Mark the Projection as Truncated

```console
$ fdu --cache off --format json --view tree --size apparent --depth 1 --limit 2 project
{
  "schema": "fdu.report/10",
  "generator": "fdu 0.3.0",
  "root": "[SCAN_PATH]",
  "age_reference_ns": [AGE_NS],
  "request": {
    "scope": {
      "max_depth": null,
      "follow_symlinks": false,
      "one_filesystem": false,
      "exclude_special": false,
      "read_controls": true,
      "population": "include"
    },
    "analyze": [],
    "size": "apparent",
    "sort_metric": null,
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
    "rules": 1,
    "refused": 0,
    "refusals": []
  },
  "analysis": null,
  "reports": [
    {
      "view": "tree",
      "limits": {"depth": 1, "min_share": "1%", "breadth": null, "rows": 2},
      "tree": {
        "name": ".",
        "path": "",
        "kind": "dir",
        "entry_ignored": false,
        "bytes": 269,
        "allocated": [ALLOCATED],
        "files": 7,
        "dirs": 3,
        "ignored": {"files": 1, "dirs": 1, "bytes": 128, "allocated": [ALLOCATED]},
        "newest_mtime_ns": [MTIME_NS],
        "truncated": true,
        "omissions": [
          {"reason": "rows", "entries": 5, "files": 6, "bytes": 141, "allocated": [ALLOCATED]}
        ],
        "children": [
          {
            "name": "dist",
            "path": "dist",
            "kind": "dir",
            "entry_ignored": true,
            "bytes": 128,
            "allocated": [ALLOCATED],
            "files": 1,
            "dirs": 0,
            "ignored": {"files": 1, "dirs": 0, "bytes": 128, "allocated": [ALLOCATED]},
            "newest_mtime_ns": [MTIME_NS],
            "truncated": true,
            "omissions": [
              {"reason": "depth", "entries": 1, "files": 1, "bytes": 128, "allocated": [ALLOCATED]}
            ],
            "children": []
          }
        ]
      },
      "omissions": [],
      "remainder": {
        "files": 6,
        "bytes": 141,
        "allocated": [ALLOCATED],
        "reasons": ["rows"]
      }
    }
  ]
}
! note: totals include gitignored sizes and descendants
! note: display limits: depth 1, row limit 2
! tip: show more: --depth=all --limit=all
? 0
```

## Scan Depth Is an Explicit Complete Scope

The report is complete for the scope it was asked for, and says so at the top.
The directories retained at the depth limit were never listed.
Keep these unknown branches despite their zero observed sizes; their unseen contents
could exceed the share threshold.
Their newest modification time is null because a lower-bound maximum cannot establish an
age.

```console
$ fdu --cache off --format json --view tree --size apparent --scan-depth 1 --depth 2 --limit 10 project
{
  "schema": "fdu.report/10",
  "generator": "fdu 0.3.0",
  "root": "[SCAN_PATH]",
  "age_reference_ns": [AGE_NS],
  "request": {
    "scope": {
      "max_depth": 1,
      "follow_symlinks": false,
      "one_filesystem": false,
      "exclude_special": false,
      "read_controls": true,
      "population": "include"
    },
    "analyze": [],
    "size": "apparent",
    "sort_metric": null,
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
    "rules": 1,
    "refused": 0,
    "refusals": []
  },
  "analysis": null,
  "reports": [
    {
      "view": "tree",
      "limits": {"depth": 2, "min_share": "1%", "breadth": null, "rows": 10},
      "tree": {
        "name": ".",
        "path": "",
        "kind": "dir",
        "entry_ignored": false,
        "bytes": 82,
        "allocated": [ALLOCATED],
        "files": 3,
        "dirs": 3,
        "ignored": {"files": 0, "dirs": 1, "bytes": 0, "allocated": 0},
        "newest_mtime_ns": [MTIME_NS],
        "truncated": false,
        "omissions": [],
        "children": [
          {
            "name": "README.md",
            "path": "README.md",
            "kind": "file",
            "entry_ignored": false,
            "bytes": 48,
            "allocated": [ALLOCATED],
            "files": 1,
            "dirs": 0,
            "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0},
            "newest_mtime_ns": [MTIME_NS],
            "truncated": false,
            "omissions": [],
            "children": []
          },
          {
            "name": "Makefile",
            "path": "Makefile",
            "kind": "file",
            "entry_ignored": false,
            "bytes": 28,
            "allocated": [ALLOCATED],
            "files": 1,
            "dirs": 0,
            "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0},
            "newest_mtime_ns": [MTIME_NS],
            "truncated": false,
            "omissions": [],
            "children": []
          },
          {
            "name": ".gitignore",
            "path": ".gitignore",
            "kind": "file",
            "entry_ignored": false,
            "bytes": 6,
            "allocated": [ALLOCATED],
            "files": 1,
            "dirs": 0,
            "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0},
            "newest_mtime_ns": [MTIME_NS],
            "truncated": false,
            "omissions": [],
            "children": []
          },
          {
            "name": "dist",
            "path": "dist",
            "kind": "dir",
            "entry_ignored": true,
            "bytes": 0,
            "allocated": 0,
            "files": 0,
            "dirs": 0,
            "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0},
            "newest_mtime_ns": null,
            "truncated": false,
            "omissions": [],
            "children": []
          },
          {
            "name": "docs",
            "path": "docs",
            "kind": "dir",
            "entry_ignored": false,
            "bytes": 0,
            "allocated": 0,
            "files": 0,
            "dirs": 0,
            "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0},
            "newest_mtime_ns": null,
            "truncated": false,
            "omissions": [],
            "children": []
          },
          {
            "name": "src",
            "path": "src",
            "kind": "dir",
            "entry_ignored": false,
            "bytes": 0,
            "allocated": 0,
            "files": 0,
            "dirs": 0,
            "ignored": {"files": 0, "dirs": 0, "bytes": 0, "allocated": 0},
            "newest_mtime_ns": null,
            "truncated": false,
            "omissions": [],
            "children": []
          }
        ]
      },
      "omissions": [],
      "remainder": null
    }
  ]
}
! note: incomplete subtrees remain visible below the size threshold
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
  "schema": "fdu.report/10",
  "generator": "fdu 0.3.0",
  "root": "[SCAN_PATH]",
  "age_reference_ns": [AGE_NS],
  "request": {
    "scope": {
      "max_depth": null,
      "follow_symlinks": false,
      "one_filesystem": false,
      "exclude_special": false,
      "read_controls": true,
      "population": "include"
    },
    "analyze": [],
    "size": "apparent",
    "sort_metric": null,
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
    "rules": 1,
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
        "share_omitted": 0,
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
