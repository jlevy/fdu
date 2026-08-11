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
      "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 263, "allocated": [ALLOCATED], "files": 6, "dirs": 3, "newest_mtime_ns": [MTIME_NS], "truncated": true, "children": []}
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
      "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 263, "allocated": [ALLOCATED], "files": 6, "dirs": 3, "newest_mtime_ns": [MTIME_NS], "truncated": true, "children": []}
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

## An Unchanged Second Open Revalidates the Snapshot

```console
$ fdu --format json --size apparent --depth 0 --limit 0 project
{
  "schema": "fdu.report/1",
  "generator": "fdu 0.0.1",
  "root": "[SCAN_PATH]",
  "scan_started_at": "[RFC3339]",
  "generated_at": "[RFC3339]",
  "source": "warm_revalidate",
  "freshness": "fresh",
  "complete": true,
  "errors": [],
  "reports": [
    {
      "view": "tree",
      "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 263, "allocated": [ALLOCATED], "files": 6, "dirs": 3, "newest_mtime_ns": [MTIME_NS], "truncated": true, "children": []}
    }
  ]
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
$ fdu --format json --size apparent --depth 0 --limit 0 project
{
  "schema": "fdu.report/1",
  "generator": "fdu 0.0.1",
  "root": "[SCAN_PATH]",
  "scan_started_at": "[RFC3339]",
  "generated_at": "[RFC3339]",
  "source": "warm_revalidate",
  "freshness": "fresh",
  "complete": true,
  "errors": [],
  "reports": [
    {
      "view": "tree",
      "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 288, "allocated": [ALLOCATED], "files": 6, "dirs": 3, "newest_mtime_ns": [MTIME_NS], "truncated": true, "children": []}
    }
  ]
}
? 0
```

## A Different Semantic Scan Scope Misses the Snapshot

```console
$ fdu --format json --size apparent --scan-depth 1 --depth 0 --limit 0 project
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
      "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 76, "allocated": [ALLOCATED], "files": 2, "dirs": 3, "newest_mtime_ns": [MTIME_NS], "truncated": true, "children": []}
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
      "tree": {"name": ".", "path": "", "kind": "dir", "bytes": 76, "allocated": [ALLOCATED], "files": 2, "dirs": 3, "newest_mtime_ns": [MTIME_NS], "truncated": true, "children": []}
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

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
