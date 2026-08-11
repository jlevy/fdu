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
  BYTES: '\d+'
  CACHE_FILE: '[^\r\n]+\.fdu'
  CACHE_DIR: '[^\r\n]+'
  SCAN_PATH: '[^\r\n]+'
---

# Cache Lifecycle Flags

Inspecting and clearing the cache are explicit flags on the same grammar, never side
effects of a report. They run before scan validation, so they need no readable tree, and
they suppress the report entirely.

## Status Before Anything Is Cached

```console
$ fdu --cache-status project
No cached snapshots.
? 0
```

## A Report Leaves a Snapshot Behind

```console
$ fdu --view summary --size apparent project
     263 B  6 files, 3 directories
? 0
```

## Status Maps a Hash-Named File Back to Its Tree

Cache files are named by a hash of their root, which keeps two trees from colliding but
leaves a directory of opaque names. The header carries the answer.

```console
$ fdu --cache-status project
[CACHE_FILE]  10 entries, [BYTES] bytes  [SCAN_PATH]
? 0
```

## Status Renders Through the Format Axis

Agents get cache observability without a second schema style.

```console
$ fdu --cache-status --format json project
{
  "caches": [
    {"path": "[CACHE_FILE]", "bytes": [BYTES], "recognized": true, "root": "[SCAN_PATH]", "entries": 10}
  ]
}
? 0
```

## Clearing Echoes the Target Before Acting

```console
$ fdu --cache-clear project
Cache file: [CACHE_FILE]
Cache cleared.
? 0
```

## Clearing Is Idempotent

```console
$ fdu --cache-clear project
Cache file: [CACHE_FILE]
Cache already empty.
? 0
```

## Clear and Status Compose, With Clear First

```console
$ fdu --view summary --size apparent project
     263 B  6 files, 3 directories
? 0
```

```console
$ fdu --cache-clear --cache-status project
Cache file: [CACHE_FILE]
Cache cleared.
No cached snapshots.
? 0
```

## An Unknown Scope Names Both Accepted Values

```console
$ fdu --cache-status=sometimes project
fdu: invalid --cache-status "sometimes": expected root or all
? 2
```
