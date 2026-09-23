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
  RFC3339: '\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{9}Z'
  ALLOCATED: '\d+'
  MTIME_NS: '-?\d+'
  SCAN_PATH: '[^\r\n]+'
---
# CLI Cache Lifecycle

## No-Cache Is a Cold Scan Without a Side Effect

### Scan Without a Cache

```console
$ fdu --cache off --format json --size apparent --depth 0 --limit 0 project
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
        "children": []
      }
    }
  ]
}
? 0
```

### Verify No Cache Was Created

```console
$ node -e "const fs=require('node:fs'); if (fs.existsSync('.cache')) process.exit(1); console.log('cache absent')"
cache absent
? 0
```

## The First Cached Open Is Cold and Writes One Snapshot

### Create the Snapshot

```console
$ fdu --format json --size apparent --depth 0 --limit 0 project
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
        "children": []
      }
    }
  ]
}
? 0
```

### Verify Exactly One Snapshot Exists

```console
$ node -e "const fs=require('node:fs'); const files=fs.readdirSync('.cache/fdu').filter((name) => name.endsWith('.fdu')); if (files.length !== 1) process.exit(1); console.log('snapshot present')"
snapshot present
? 0
```

## A Second One-Shot Report Scans Cold Again

Loading the snapshot cannot save a one-shot metadata report any work: revalidation stats
every entry regardless, so the read would only ever add to the walk.
The report scans fresh and rewrites the snapshot, and the tier below shows what the
rewrite is for.

```console
$ fdu --format json --size apparent --depth 0 --limit 0 project
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
        "children": []
      }
    }
  ]
}
? 0
```

## A Change Shows Up Without the Snapshot’s Help

### Expand the Fixture

```console
$ node -e "require('node:fs').writeFileSync('project/docs/FAQ.MD', '# FAQ\n\nRun the full check before every release.\n'); console.log('fixture expanded')"
fixture expanded
? 0
```

### Report the Changed Tree

```console
$ fdu --format json --size apparent --depth 0 --limit 0 project
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
        "bytes": 294,
        "allocated": [ALLOCATED],
        "files": 7,
        "dirs": 3,
        "ignored": {"files": 1, "dirs": 1, "bytes": 128, "allocated": [ALLOCATED]},
        "newest_mtime_ns": [MTIME_NS],
        "truncated": true,
        "children": []
      }
    }
  ]
}
? 0
```

### The Rewrite Kept the Snapshot Current for Cache-Only

Every one-shot report rewrites the snapshot it skipped reading, so the no-scan tier
answers with the changed total rather than the one the first run recorded.

```console
$ fdu --cache only --format json --size apparent --depth 0 --limit 0 project
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
    "source": "cache_only",
    "freshness": "stale",
    "scan_started_at": "[RFC3339]",
    "generated_at": "[RFC3339]",
    "tiers": {
      "entries": {"source": "cached", "freshness": "stale", "observed_at_ns": [MTIME_NS]},
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
        "bytes": 294,
        "allocated": [ALLOCATED],
        "files": 7,
        "dirs": 3,
        "ignored": {"files": 1, "dirs": 1, "bytes": 128, "allocated": [ALLOCATED]},
        "newest_mtime_ns": [MTIME_NS],
        "truncated": true,
        "children": []
      }
    }
  ]
}
? 0
```

## Watching Rejects a Snapshot Nothing Verified

The snapshot above is usable, and that is not enough: nothing observes the window
between when it was written and when a watch starts, so the first answer could describe
a tree that has already moved and every later one would build on it.

```console
$ fdu --watch --cache only project
! fdu: --watch cannot start from --cache only: nothing verifies what changed between the snapshot and the start of the watch; use --cache auto or read-only
? 2
```

## A Different Semantic Scan Scope Misses the Snapshot

```console
$ fdu --format json --size apparent --scan-depth 1 --depth 0 --limit 0 project
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
        "truncated": true,
        "children": []
      }
    }
  ]
}
? 0
```

## A Corrupt Snapshot Fails Closed and Is Replaced

### Corrupt the Snapshot

```console
$ node -e "const fs=require('node:fs'); const path=require('node:path'); const dir='.cache/fdu'; const file=fs.readdirSync(dir).find((name) => name.endsWith('.fdu')); if (!file) process.exit(1); fs.writeFileSync(path.join(dir, file), 'corrupt'); console.log('snapshot corrupted')"
snapshot corrupted
? 0
```

### Recover with a Cold Scan

```console
$ fdu --format json --size apparent --scan-depth 1 --depth 0 --limit 0 project
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
        "truncated": true,
        "children": []
      }
    }
  ]
}
? 0
```

### Verify the Corrupt File Was Replaced

```console
$ node -e "const fs=require('node:fs'); const path=require('node:path'); const dir='.cache/fdu'; const file=fs.readdirSync(dir).find((name) => name.endsWith('.fdu')); if (!file || fs.readFileSync(path.join(dir, file), 'utf8') === 'corrupt') process.exit(1); console.log('snapshot replaced')"
snapshot replaced
? 0
```

## Reading No .gitignore Is a Separate Scope

A report that turns `.gitignore` off records a scope without classification.
A cache-only report that also turns it off may answer from a default snapshot, because
it reads only the sizes a default scan also recorded; it says it read no rules.

```console
$ fdu --format json --size apparent --depth 0 --limit 0 project
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
        "bytes": 294,
        "allocated": [ALLOCATED],
        "files": 7,
        "dirs": 3,
        "ignored": {"files": 1, "dirs": 1, "bytes": 128, "allocated": [ALLOCATED]},
        "newest_mtime_ns": [MTIME_NS],
        "truncated": true,
        "children": []
      }
    }
  ]
}
? 0
```

```console
$ fdu --no-gitignore --cache only --format json --size apparent --depth 0 --limit 0 project
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
    "source": "cache_only",
    "freshness": "stale",
    "scan_started_at": "[RFC3339]",
    "generated_at": "[RFC3339]",
    "tiers": {
      "entries": {"source": "cached", "freshness": "stale", "observed_at_ns": [MTIME_NS]},
      "content": null
    }
  },
  "ignore_rules": null,
  "analysis": null,
  "reports": [
    {
      "view": "tree",
      "tree": {
        "name": ".",
        "path": "",
        "kind": "dir",
        "bytes": 294,
        "allocated": [ALLOCATED],
        "files": 7,
        "dirs": 3,
        "ignored": null,
        "newest_mtime_ns": [MTIME_NS],
        "truncated": true,
        "children": []
      }
    }
  ]
}
? 0
```

A read-only one-shot metadata report scans cold without installing control state.
It leaves the stronger snapshot usable by a subsequent default cache-only request.

```console
$ fdu --no-gitignore --cache read-only --format json --size apparent --depth 0 --limit 0 project
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
  "ignore_rules": null,
  "analysis": null,
  "reports": [
    {
      "view": "tree",
      "tree": {
        "name": ".",
        "path": "",
        "kind": "dir",
        "bytes": 294,
        "allocated": [ALLOCATED],
        "files": 7,
        "dirs": 3,
        "ignored": null,
        "newest_mtime_ns": [MTIME_NS],
        "truncated": true,
        "children": []
      }
    }
  ]
}
? 0
```

```console
$ fdu --cache only --format json --size apparent --depth 0 --limit 0 project
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
    "source": "cache_only",
    "freshness": "stale",
    "scan_started_at": "[RFC3339]",
    "generated_at": "[RFC3339]",
    "tiers": {
      "entries": {"source": "cached", "freshness": "stale", "observed_at_ns": [MTIME_NS]},
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
        "bytes": 294,
        "allocated": [ALLOCATED],
        "files": 7,
        "dirs": 3,
        "ignored": {"files": 1, "dirs": 1, "bytes": 128, "allocated": [ALLOCATED]},
        "newest_mtime_ns": [MTIME_NS],
        "truncated": true,
        "children": []
      }
    }
  ]
}
? 0
```

The reverse cannot work: a snapshot written without the rules has no classification for
a default request to report, so a cache-only default request refuses it and names the
way out.

```console
$ fdu --no-gitignore --format json --size apparent --depth 0 --limit 0 project
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
  "ignore_rules": null,
  "analysis": null,
  "reports": [
    {
      "view": "tree",
      "tree": {
        "name": ".",
        "path": "",
        "kind": "dir",
        "bytes": 294,
        "allocated": [ALLOCATED],
        "files": 7,
        "dirs": 3,
        "ignored": null,
        "newest_mtime_ns": [MTIME_NS],
        "truncated": true,
        "children": []
      }
    }
  ]
}
? 0
```

```console
$ fdu --cache only --format json --size apparent --depth 0 --limit 0 project
fdu: snapshot is not usable: no usable snapshot for this root and scan scope: the cached snapshot has no .gitignore state, because the request that wrote it did not observe it, and this request does; the `only` cache policy never scans, so use `auto`, or turn .gitignore observation off as that request did
? 1
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
