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
---
# CLI Cache Lifecycle

## No-Cache Is a Cold Scan Without a Side Effect

### Scan Without a Cache

```console
$ fdu --no-cache --json --apparent-size --depth 0 --number 0 project
{
  "schema": "fdu.tree/2",
  "generator": "fdu 0.0.1",
  "root": "[SCAN_PATH]",
  "source": "cold_scan",
  "complete": true,
  "display_depth": 0,
  "entries_per_directory": 0,
  "scan_max_depth": null,
  "tree_truncated": true,
  "freshness": "fresh",
  "errors": [
  ],
  "by_extension": {
    ".tar.gz": {"files": 1, "bytes": 128},
    ".md": {"files": 2, "bytes": 71},
    ".rs": {"files": 2, "bytes": 36}
  },
  "tree": {
    "name": ".",
    "kind": "dir",
    "bytes": 263,
    "allocated": [ALLOCATED],
    "files": 6,
    "dirs": 3,
    "newest_mtime_ns": [MTIME_NS]
  }
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
$ fdu --json --apparent-size --depth 0 --number 0 project
{
  "schema": "fdu.tree/2",
  "generator": "fdu 0.0.1",
  "root": "[SCAN_PATH]",
  "source": "cold_scan",
  "complete": true,
  "display_depth": 0,
  "entries_per_directory": 0,
  "scan_max_depth": null,
  "tree_truncated": true,
  "freshness": "fresh",
  "errors": [
  ],
  "by_extension": {
    ".tar.gz": {"files": 1, "bytes": 128},
    ".md": {"files": 2, "bytes": 71},
    ".rs": {"files": 2, "bytes": 36}
  },
  "tree": {
    "name": ".",
    "kind": "dir",
    "bytes": 263,
    "allocated": [ALLOCATED],
    "files": 6,
    "dirs": 3,
    "newest_mtime_ns": [MTIME_NS]
  }
}
? 0
```

### Verify Exactly One Snapshot Exists

```console
$ node -e "const fs=require('node:fs'); const files=fs.readdirSync('.cache/fdu').filter((name) => name.endsWith('.fdu')); if (files.length !== 1) process.exit(1); console.log('snapshot present')"
snapshot present
? 0
```

## An Unchanged Second Open Revalidates the Snapshot

```console
$ fdu --json --apparent-size --depth 0 --number 0 project
{
  "schema": "fdu.tree/2",
  "generator": "fdu 0.0.1",
  "root": "[SCAN_PATH]",
  "source": "warm_revalidate",
  "complete": true,
  "display_depth": 0,
  "entries_per_directory": 0,
  "scan_max_depth": null,
  "tree_truncated": true,
  "freshness": "fresh",
  "errors": [
  ],
  "by_extension": {
    ".tar.gz": {"files": 1, "bytes": 128},
    ".md": {"files": 2, "bytes": 71},
    ".rs": {"files": 2, "bytes": 36}
  },
  "tree": {
    "name": ".",
    "kind": "dir",
    "bytes": 263,
    "allocated": [ALLOCATED],
    "files": 6,
    "dirs": 3,
    "newest_mtime_ns": [MTIME_NS]
  }
}
? 0
```

## Revalidation Detects a File Size Change

### Expand the Fixture

```console
$ node -e "require('node:fs').writeFileSync('project/docs/FAQ.MD', '# FAQ\n\nRun the full check before every release.\n'); console.log('fixture expanded')"
fixture expanded
? 0
```

### Revalidate the Changed Tree

```console
$ fdu --json --apparent-size --depth 0 --number 0 project
{
  "schema": "fdu.tree/2",
  "generator": "fdu 0.0.1",
  "root": "[SCAN_PATH]",
  "source": "warm_revalidate",
  "complete": true,
  "display_depth": 0,
  "entries_per_directory": 0,
  "scan_max_depth": null,
  "tree_truncated": true,
  "freshness": "fresh",
  "errors": [
  ],
  "by_extension": {
    ".tar.gz": {"files": 1, "bytes": 128},
    ".md": {"files": 2, "bytes": 96},
    ".rs": {"files": 2, "bytes": 36}
  },
  "tree": {
    "name": ".",
    "kind": "dir",
    "bytes": 288,
    "allocated": [ALLOCATED],
    "files": 6,
    "dirs": 3,
    "newest_mtime_ns": [MTIME_NS]
  }
}
? 0
```

## A Different Semantic Scan Scope Misses the Snapshot

```console
$ fdu --json --apparent-size --max-depth 1 --depth 0 --number 0 project
{
  "schema": "fdu.tree/2",
  "generator": "fdu 0.0.1",
  "root": "[SCAN_PATH]",
  "source": "cold_scan",
  "complete": true,
  "display_depth": 0,
  "entries_per_directory": 0,
  "scan_max_depth": 1,
  "tree_truncated": true,
  "freshness": "fresh",
  "errors": [
  ],
  "by_extension": {
    ".md": {"files": 1, "bytes": 48}
  },
  "tree": {
    "name": ".",
    "kind": "dir",
    "bytes": 76,
    "allocated": [ALLOCATED],
    "files": 2,
    "dirs": 3,
    "newest_mtime_ns": [MTIME_NS]
  }
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
$ fdu --json --apparent-size --max-depth 1 --depth 0 --number 0 project
{
  "schema": "fdu.tree/2",
  "generator": "fdu 0.0.1",
  "root": "[SCAN_PATH]",
  "source": "cold_scan",
  "complete": true,
  "display_depth": 0,
  "entries_per_directory": 0,
  "scan_max_depth": 1,
  "tree_truncated": true,
  "freshness": "fresh",
  "errors": [
  ],
  "by_extension": {
    ".md": {"files": 1, "bytes": 48}
  },
  "tree": {
    "name": ".",
    "kind": "dir",
    "bytes": 76,
    "allocated": [ALLOCATED],
    "files": 2,
    "dirs": 3,
    "newest_mtime_ns": [MTIME_NS]
  }
}
? 0
```

### Verify the Corrupt File Was Replaced

```console
$ node -e "const fs=require('node:fs'); const path=require('node:path'); const dir='.cache/fdu'; const file=fs.readdirSync(dir).find((name) => name.endsWith('.fdu')); if (!file || fs.readFileSync(path.join(dir, file), 'utf8') === 'corrupt') process.exit(1); console.log('snapshot replaced')"
snapshot replaced
? 0
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
