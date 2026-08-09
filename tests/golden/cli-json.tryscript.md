---
sandbox: true
fixtures:
  - fixtures/tree
path:
  - $TRYSCRIPT_GIT_ROOT/target/debug
env:
  FORCE_COLOR: "0"
  LANG: C
  LC_ALL: C
  NO_COLOR: "1"
  TZ: UTC
patterns:
  ALLOCATED: '\d+'
  BLANK_LINE: '\n'
  MTIME_NS: '-?\d+'
  SCAN_PATH: '[^\r\n]+'
---
# JSON CLI Output

## Full Output Exposes Scan and Projection Completeness Separately

```console
$ fdu --no-cache --json --apparent-size --depth 2 --number 10 tree
{
  "schema": "fdu.tree/2",
  "generator": "fdu 0.0.1",
  "root": "[SCAN_PATH]",
  "source": "cold_scan",
  "complete": true,
  "display_depth": 2,
  "entries_per_directory": 10,
  "scan_max_depth": null,
  "tree_truncated": false,
  "freshness": "fresh",
  "errors": [
  ],
  "by_extension": {
    ".tar.gz": {"files": 1, "bytes": 13},
    ".md": {"files": 1, "bytes": 9},
    ".txt": {"files": 2, "bytes": 8},
    ".bin": {"files": 1, "bytes": 5}
  },
  "tree": {
    "name": ".",
    "kind": "dir",
    "bytes": 37,
    "allocated": [ALLOCATED],
    "files": 6,
    "dirs": 2,
    "newest_mtime_ns": [MTIME_NS],
    "children": [
      {
        "name": "assets",
        "kind": "dir",
        "bytes": 13,
        "allocated": [ALLOCATED],
        "files": 1,
        "dirs": 0,
        "newest_mtime_ns": [MTIME_NS],
        "children": [
          {
            "name": "archive.tar.gz",
            "kind": "file",
            "bytes": 13,
            "allocated": [ALLOCATED],
            "newest_mtime_ns": [MTIME_NS]
          }
        ]
      },
      {
        "name": "docs",
        "kind": "dir",
        "bytes": 12,
        "allocated": [ALLOCATED],
        "files": 2,
        "dirs": 0,
        "newest_mtime_ns": [MTIME_NS],
        "children": [
          {
            "name": "guide.md",
            "kind": "file",
            "bytes": 9,
            "allocated": [ALLOCATED],
            "newest_mtime_ns": [MTIME_NS]
          },
          {
            "name": "note.txt",
            "kind": "file",
            "bytes": 3,
            "allocated": [ALLOCATED],
            "newest_mtime_ns": [MTIME_NS]
          }
        ]
      },
      {
        "name": "ALPHA.TXT",
        "kind": "file",
        "bytes": 5,
        "allocated": [ALLOCATED],
        "newest_mtime_ns": [MTIME_NS]
      },
      {
        "name": "beta.bin",
        "kind": "file",
        "bytes": 5,
        "allocated": [ALLOCATED],
        "newest_mtime_ns": [MTIME_NS]
      },
      {
        "name": "README",
        "kind": "file",
        "bytes": 2,
        "allocated": [ALLOCATED],
        "newest_mtime_ns": [MTIME_NS]
      }
    ]
  }
}
? 0
```

## Render Limits Mark the Projection as Truncated

```console
$ fdu --no-cache --json --apparent-size --depth 1 --number 2 tree
{
  "schema": "fdu.tree/2",
  "generator": "fdu 0.0.1",
  "root": "[SCAN_PATH]",
  "source": "cold_scan",
  "complete": true,
  "display_depth": 1,
  "entries_per_directory": 2,
  "scan_max_depth": null,
  "tree_truncated": true,
  "freshness": "fresh",
  "errors": [
  ],
  "by_extension": {
    ".tar.gz": {"files": 1, "bytes": 13},
    ".md": {"files": 1, "bytes": 9},
    ".txt": {"files": 2, "bytes": 8},
    ".bin": {"files": 1, "bytes": 5}
  },
  "tree": {
    "name": ".",
    "kind": "dir",
    "bytes": 37,
    "allocated": [ALLOCATED],
    "files": 6,
    "dirs": 2,
    "newest_mtime_ns": [MTIME_NS],
    "children": [
      {
        "name": "assets",
        "kind": "dir",
        "bytes": 13,
        "allocated": [ALLOCATED],
        "files": 1,
        "dirs": 0,
        "newest_mtime_ns": [MTIME_NS]
      },
      {
        "name": "docs",
        "kind": "dir",
        "bytes": 12,
        "allocated": [ALLOCATED],
        "files": 2,
        "dirs": 0,
        "newest_mtime_ns": [MTIME_NS]
      }
    ]
  }
}
? 0
```

## Scan Depth Is an Explicit Complete Scope

```console
$ fdu --no-cache --json --apparent-size --max-depth 1 --depth 2 --number 10 tree
{
  "schema": "fdu.tree/2",
  "generator": "fdu 0.0.1",
  "root": "[SCAN_PATH]",
  "source": "cold_scan",
  "complete": true,
  "display_depth": 2,
  "entries_per_directory": 10,
  "scan_max_depth": 1,
  "tree_truncated": false,
  "freshness": "fresh",
  "errors": [
  ],
  "by_extension": {
    ".bin": {"files": 1, "bytes": 5},
    ".txt": {"files": 1, "bytes": 5}
  },
  "tree": {
    "name": ".",
    "kind": "dir",
    "bytes": 12,
    "allocated": [ALLOCATED],
    "files": 3,
    "dirs": 2,
    "newest_mtime_ns": [MTIME_NS],
    "children": [
      {
        "name": "ALPHA.TXT",
        "kind": "file",
        "bytes": 5,
        "allocated": [ALLOCATED],
        "newest_mtime_ns": [MTIME_NS]
      },
      {
        "name": "beta.bin",
        "kind": "file",
        "bytes": 5,
        "allocated": [ALLOCATED],
        "newest_mtime_ns": [MTIME_NS]
      },
      {
        "name": "README",
        "kind": "file",
        "bytes": 2,
        "allocated": [ALLOCATED],
        "newest_mtime_ns": [MTIME_NS]
      },
      {
        "name": "assets",
        "kind": "dir",
        "bytes": 0,
        "allocated": 0,
        "files": 0,
        "dirs": 0,
        "newest_mtime_ns": 0
      },
      {
        "name": "docs",
        "kind": "dir",
        "bytes": 0,
        "allocated": 0,
        "files": 0,
        "dirs": 0,
        "newest_mtime_ns": 0
      }
    ]
  }
}
? 0
```

## Human-Only Type View Cannot Be Combined with JSON

```console
$ fdu --no-cache --by-type --json tree
! error: the argument '--by-type' cannot be used with '--json'[BLANK_LINE]
! Usage: fdu --no-cache --by-type <PATH>[BLANK_LINE]
! For more information, try '--help'.
? 2
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
