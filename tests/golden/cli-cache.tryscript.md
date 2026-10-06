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
  AGE_NS: '-?\d+'
  RFC3339: '\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{9}Z'
  ALLOCATED: '\d+'
  MTIME_NS: '-?\d+'
  SCAN_PATH: '[^\r\n]+'
---
# CLI Cache Lifecycle

## No-Cache Is a Cold Scan Without a Side Effect

### Scan Without a Cache

```console
$ fdu --cache off --view tree --format json --size apparent --depth 0 --limit 0 project
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
      "limits": {"depth": 0, "min_share": "1%", "breadth": null, "rows": 0},
      "tree": null,
      "omissions": [
        {"reason": "rows", "entries": 1, "files": 7, "bytes": 269, "allocated": [ALLOCATED]}
      ],
      "remainder": {
        "files": 7,
        "bytes": 269,
        "allocated": [ALLOCATED],
        "reasons": ["rows"]
      }
    }
  ]
}
! note: totals include descendants
! note: display limits: row limit 0
! tip: show more: --limit=all
? 0
```

### Verify No Cache Was Created

```console
$ node -e "const fs=require('node:fs'); if (fs.existsSync('.cache')) process.exit(1); console.log('cache absent')"
cache absent
? 0
```

## The Default Report Leaves No Snapshot

Under `--cache auto` a one-shot metadata report neither reads nor writes the cache:
checking a snapshot costs as much as the scan it would save, and no later report reads
one.

### Report Under the Default Policy

```console
$ fdu --view tree --format json --size apparent --depth 0 --limit 0 project
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
      "limits": {"depth": 0, "min_share": "1%", "breadth": null, "rows": 0},
      "tree": null,
      "omissions": [
        {"reason": "rows", "entries": 1, "files": 7, "bytes": 269, "allocated": [ALLOCATED]}
      ],
      "remainder": {
        "files": 7,
        "bytes": 269,
        "allocated": [ALLOCATED],
        "reasons": ["rows"]
      }
    }
  ]
}
! note: totals include descendants
! note: display limits: row limit 0
! tip: show more: --limit=all
? 0
```

### Verify the Default Created No Cache

```console
$ node -e "const fs=require('node:fs'); if (fs.existsSync('.cache')) process.exit(1); console.log('cache absent')"
cache absent
? 0
```

### A Stale Answer Has Nothing to Read

`--stale-ok` never scans, so with no snapshot it fails and names both ways out.

```console
$ fdu --stale-ok --view tree --format json --size apparent --depth 0 --limit 0 project
fdu: snapshot is not usable: no usable snapshot for this root and scan scope; a stale answer never scans, so run the request once with the `on` cache policy to leave one, or ask for a verified answer, which scans when none serves
? 1
```

## `--cache on` Is Cold and Writes One Snapshot

### Create the Snapshot

```console
$ fdu --cache on --view tree --format json --size apparent --depth 0 --limit 0 project
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
      "limits": {"depth": 0, "min_share": "1%", "breadth": null, "rows": 0},
      "tree": null,
      "omissions": [
        {"reason": "rows", "entries": 1, "files": 7, "bytes": 269, "allocated": [ALLOCATED]}
      ],
      "remainder": {
        "files": 7,
        "bytes": 269,
        "allocated": [ALLOCATED],
        "reasons": ["rows"]
      }
    }
  ]
}
! note: totals include descendants
! note: display limits: row limit 0
! tip: show more: --limit=all
? 0
```

### Verify Exactly One Snapshot Exists

```console
$ node -e "const fs=require('node:fs'); const files=fs.readdirSync('.cache/fdu').filter((name) => name.endsWith('.metadata.bin')); if (files.length !== 1) process.exit(1); console.log('snapshot present')"
snapshot present
? 0
```

## A Second One-Shot Report Scans Cold Again

Loading the snapshot cannot save a one-shot metadata report any work: revalidation stats
every entry regardless, so the read would only ever add to the walk.
Under `auto` the report scans fresh and leaves the snapshot as it found it.

```console
$ fdu --view tree --format json --size apparent --depth 0 --limit 0 project
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
      "limits": {"depth": 0, "min_share": "1%", "breadth": null, "rows": 0},
      "tree": null,
      "omissions": [
        {"reason": "rows", "entries": 1, "files": 7, "bytes": 269, "allocated": [ALLOCATED]}
      ],
      "remainder": {
        "files": 7,
        "bytes": 269,
        "allocated": [ALLOCATED],
        "reasons": ["rows"]
      }
    }
  ]
}
! note: totals include descendants
! note: display limits: row limit 0
! tip: show more: --limit=all
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

`--cache on` writes the snapshot again after the scan.

```console
$ fdu --cache on --view tree --format json --size apparent --depth 0 --limit 0 project
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
      "limits": {"depth": 0, "min_share": "1%", "breadth": null, "rows": 0},
      "tree": null,
      "omissions": [
        {"reason": "rows", "entries": 1, "files": 7, "bytes": 294, "allocated": [ALLOCATED]}
      ],
      "remainder": {
        "files": 7,
        "bytes": 294,
        "allocated": [ALLOCATED],
        "reasons": ["rows"]
      }
    }
  ]
}
! note: totals include descendants
! note: display limits: row limit 0
! tip: show more: --limit=all
? 0
```

### The Rewrite Kept the Snapshot Current for a Stale Answer

A `--cache on` report rewrites the snapshot it skipped reading, so the no-scan tier
answers with the changed total rather than the one the first run recorded.

```console
$ fdu --stale-ok --view tree --format json --size apparent --depth 0 --limit 0 project
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
    "rules": 1,
    "refused": 0,
    "refusals": []
  },
  "analysis": null,
  "reports": [
    {
      "view": "tree",
      "limits": {"depth": 0, "min_share": "1%", "breadth": null, "rows": 0},
      "tree": null,
      "omissions": [
        {"reason": "rows", "entries": 1, "files": 7, "bytes": 294, "allocated": [ALLOCATED]}
      ],
      "remainder": {
        "files": 7,
        "bytes": 294,
        "allocated": [ALLOCATED],
        "reasons": ["rows"]
      }
    }
  ]
}
! note: totals include descendants
! note: display limits: row limit 0
! warn: stale answer: served from the snapshot without filesystem verification; drop --stale-ok for a fresh answer
! tip: show more: --limit=all
? 0
```

## Watching Rejects a Snapshot Nothing Verified

The snapshot above is usable, and that is not enough: nothing observes the window
between when it was written and when a watch starts, so the first answer could describe
a tree that has already moved and every later one would build on it.

```console
$ fdu --watch --stale-ok project
! fdu: --watch cannot start from a --stale-ok answer: nothing verifies what changed between the snapshot and the start of the watch; drop --stale-ok
? 2
```

## A Different Semantic Scan Scope Misses the Snapshot

```console
$ fdu --view tree --format json --size apparent --scan-depth 1 --depth 0 --limit 0 project
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
      "limits": {"depth": 0, "min_share": "1%", "breadth": null, "rows": 0},
      "tree": null,
      "omissions": [
        {"reason": "rows", "entries": 1, "files": 3, "bytes": 82, "allocated": [ALLOCATED]}
      ],
      "remainder": {
        "files": 3,
        "bytes": 82,
        "allocated": [ALLOCATED],
        "reasons": ["rows"]
      }
    }
  ]
}
! note: totals include descendants
! note: display limits: row limit 0
! note: incomplete subtrees remain visible below the size threshold
! tip: show more: --limit=all
? 0
```

## A Corrupt Snapshot Fails Closed and Is Replaced

### Corrupt the Snapshot

```console
$ node -e "const fs=require('node:fs'); const path=require('node:path'); const dir='.cache/fdu'; const file=fs.readdirSync(dir).find((name) => name.endsWith('.metadata.bin')); if (!file) process.exit(1); fs.writeFileSync(path.join(dir, file), 'corrupt'); console.log('snapshot corrupted')"
snapshot corrupted
? 0
```

### Recover with a Cold Scan

A run that writes replaces the corrupt file.

```console
$ fdu --cache on --view tree --format json --size apparent --scan-depth 1 --depth 0 --limit 0 project
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
      "limits": {"depth": 0, "min_share": "1%", "breadth": null, "rows": 0},
      "tree": null,
      "omissions": [
        {"reason": "rows", "entries": 1, "files": 3, "bytes": 82, "allocated": [ALLOCATED]}
      ],
      "remainder": {
        "files": 3,
        "bytes": 82,
        "allocated": [ALLOCATED],
        "reasons": ["rows"]
      }
    }
  ]
}
! note: totals include descendants
! note: display limits: row limit 0
! note: incomplete subtrees remain visible below the size threshold
! tip: show more: --limit=all
? 0
```

### Verify the Corrupt File Was Replaced

```console
$ node -e "const fs=require('node:fs'); const path=require('node:path'); const dir='.cache/fdu'; const file=fs.readdirSync(dir).find((name) => name.endsWith('.metadata.bin')); if (!file || fs.readFileSync(path.join(dir, file), 'utf8') === 'corrupt') process.exit(1); console.log('snapshot replaced')"
snapshot replaced
? 0
```

## Reading No .gitignore Is a Separate Scope

A report that turns `.gitignore` off records a scope without classification.
A `--stale-ok` report that also turns it off may answer from a default snapshot, because
it reads only the sizes a default scan also recorded; it says it read no rules.

```console
$ fdu --cache on --view tree --format json --size apparent --depth 0 --limit 0 project
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
      "limits": {"depth": 0, "min_share": "1%", "breadth": null, "rows": 0},
      "tree": null,
      "omissions": [
        {"reason": "rows", "entries": 1, "files": 7, "bytes": 294, "allocated": [ALLOCATED]}
      ],
      "remainder": {
        "files": 7,
        "bytes": 294,
        "allocated": [ALLOCATED],
        "reasons": ["rows"]
      }
    }
  ]
}
! note: totals include descendants
! note: display limits: row limit 0
! tip: show more: --limit=all
? 0
```

```console
$ fdu --no-gitignore --stale-ok --view tree --format json --size apparent --depth 0 --limit 0 project
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
      "read_controls": false,
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
      "limits": {"depth": 0, "min_share": "1%", "breadth": null, "rows": 0},
      "tree": null,
      "omissions": [
        {"reason": "rows", "entries": 1, "files": 7, "bytes": 294, "allocated": [ALLOCATED]}
      ],
      "remainder": {
        "files": 7,
        "bytes": 294,
        "allocated": [ALLOCATED],
        "reasons": ["rows"]
      }
    }
  ]
}
! note: totals include descendants
! note: display limits: row limit 0
! warn: stale answer: served from the snapshot without filesystem verification; drop --stale-ok for a fresh answer
! tip: show more: --limit=all
? 0
```

A default one-shot metadata report scans cold and writes nothing, so it leaves the
stronger snapshot usable by a subsequent default `--stale-ok` request.

```console
$ fdu --no-gitignore --format json --size apparent --limit 0 project
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
      "read_controls": false,
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
  "ignore_rules": null,
  "analysis": null,
  "reports": [
    {
      "view": "list",
      "bound": {"shown": 0, "total": 10},
      "files": []
    }
  ]
}
! note: display limits: 0 of 10 rows shown
! tip: show more: --limit=all
? 0
```

```console
$ fdu --stale-ok --format json --size apparent --limit 0 project
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
    "rules": 1,
    "refused": 0,
    "refusals": []
  },
  "analysis": null,
  "reports": [
    {
      "view": "list",
      "bound": {"shown": 0, "total": 10},
      "files": []
    }
  ]
}
! note: display limits: 0 of 10 rows shown
! warn: stale answer: served from the snapshot without filesystem verification; drop --stale-ok for a fresh answer
! tip: show more: --limit=all
? 0
```

The reverse cannot work: a snapshot written without the rules has no classification for
a default request to report, so a default `--stale-ok` request refuses it and names the
way out.

```console
$ fdu --no-gitignore --cache on --view tree --format json --size apparent --depth 0 --limit 0 project
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
      "read_controls": false,
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
  "ignore_rules": null,
  "analysis": null,
  "reports": [
    {
      "view": "tree",
      "limits": {"depth": 0, "min_share": "1%", "breadth": null, "rows": 0},
      "tree": null,
      "omissions": [
        {"reason": "rows", "entries": 1, "files": 7, "bytes": 294, "allocated": [ALLOCATED]}
      ],
      "remainder": {
        "files": 7,
        "bytes": 294,
        "allocated": [ALLOCATED],
        "reasons": ["rows"]
      }
    }
  ]
}
! note: totals include descendants
! note: display limits: row limit 0
! tip: show more: --limit=all
? 0
```

```console
$ fdu --stale-ok --view tree --format json --size apparent --depth 0 --limit 0 project
fdu: snapshot is not usable: no usable snapshot for this root and scan scope: the cached snapshot has no .gitignore state, because the request that wrote it did not observe it, and this request does; a stale answer never scans, so run the request once with the `on` cache policy to leave one for this scope, ask for a verified answer, which scans when none serves, or turn .gitignore observation off as that request did
? 1
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
