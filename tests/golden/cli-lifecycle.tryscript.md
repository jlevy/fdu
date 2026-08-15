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
  PERF_TIME: '[\d.]+ (ns|µs|ms|s)'
---
# Cache Lifecycle Flags

Inspecting and clearing the cache are explicit flags on the same grammar, never side
effects of a report.
They run before scan validation, so they need no readable tree, and they suppress the
report entirely.

## Status Before Anything Is Cached

```console
$ fdu --cache-status project
No cached snapshots.
? 0
```

## A Report Leaves a Snapshot Behind

```console
$ fdu --size apparent project
     263 B  ██████████   100%  . (6 files)
     128 B  █████░░░░░    49%    dist (1 file)
      36 B  █░░░░░░░░░    14%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

## The Compact Summary Retains Nothing, and Cache-Only Says So

An unfiltered `summary` is answered by the transient tier, which retains no index and so
has no snapshot to write: the cache cannot save the walk that request is already doing.
A tier that retained nothing has nothing for `--cache only` to read, and it says so
rather than quietly scanning.

```console
$ fdu --cache-clear project
Cache file: [CACHE_FILE]
Cache cleared.
? 0
```

```console
$ fdu --view summary --size apparent project
     263 B  6 files, 3 directories
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

```console
$ fdu --cache-status project
No cached snapshots.
? 0
```

```console
$ fdu --cache only --view summary project
fdu: snapshot is not usable: no usable snapshot for this root and scan scope
? 1
```

An ordinary report retains the index, so it does leave a snapshot that `--cache only`
can then answer from without touching the tree.

```console
$ fdu --size apparent project
     263 B  ██████████   100%  . (6 files)
     128 B  █████░░░░░    49%    dist (1 file)
      36 B  █░░░░░░░░░    14%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

```console
$ fdu --cache only --view summary --size apparent project
     263 B  6 files, 3 directories
Performance: walked 0 files / 0 B; content read 0 B; analysis 0 fresh, 0 cached; cache only; total [PERF_TIME]
? 0
```

## Status Maps a Hash-Named File Back to Its Tree

Cache files are named by a hash of their root, which keeps two trees from colliding but
leaves a directory of opaque names.
The header carries the answer.

```console
$ fdu --cache-status project
[CACHE_FILE]  10 entries, [BYTES] metadata bytes, 0 content bytes  [SCAN_PATH]
? 0
```

## Status Renders Through the Format Axis

Agents get cache observability without a second schema style.

```console
$ fdu --cache-status --format json project
{
  "caches": [
    {"path": "[CACHE_FILE]", "bytes": [BYTES], "content_bytes": null, "recognized": true, "root": "[SCAN_PATH]", "entries": 10}
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
$ fdu --size apparent project
     263 B  ██████████   100%  . (6 files)
     128 B  █████░░░░░    49%    dist (1 file)
      36 B  █░░░░░░░░░    14%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
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
