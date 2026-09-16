---
sandbox: true
path:
  - $FDU_BIN
fixtures:
  - fixtures/project
  - bin
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
fdu: snapshot is not usable: no usable snapshot for this root and scan scope; the `only` cache policy never scans, so use `auto`, which scans when none serves
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
  "schema": "fdu.cache/1",
  "caches": [
    {"path": "[CACHE_FILE]", "bytes": [BYTES], "content_bytes": null, "state": "current", "root": "[SCAN_PATH]", "entries": 10}
  ]
}
? 0
```

Cache status is its own document, not a report, so it carries its own schema identity in
every machine format.

```console
$ fdu --cache-status --format yaml project
schema: fdu.cache/1
caches:
  - path: [CACHE_FILE]
    bytes: [BYTES]
    content_bytes: null
    state: current
    root: [SCAN_PATH]
    entries: 10
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

## Snapshots From Another Build Are Stale, Not Foreign

Every release changes the engine fingerprint and some change the snapshot format, so the
snapshots an earlier build wrote can serve no later one.
They are still fdu’s files: status names each with the reason this build cannot use it,
sizes it, and says which command reclaims it.
A file that is not an fdu snapshot is listed and left alone.

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
$ node bin/cache-plant.mjs stale
planted: format 1, another engine, truncated, notes.txt
? 0
```

```console
$ fdu --cache-status=all project
[CACHE_FILE]  stale (older snapshot format 1), 12 metadata bytes, 0 content bytes
[CACHE_FILE]  stale (written by another fdu version), [BYTES] metadata bytes, 0 content bytes
[CACHE_FILE]  stale (unreadable by this build), [BYTES] metadata bytes, 0 content bytes
[CACHE_FILE]  10 entries, [BYTES] metadata bytes, 0 content bytes  [SCAN_PATH]
[CACHE_DIR]notes.txt  unrecognized, 15 bytes
3 stale snapshots ([BYTES] bytes) cannot be served by this build; fdu --cache-clear=all removes them, along with every current snapshot.
1 unrecognized file (15 bytes) is not an fdu snapshot, so fdu leaves it in place.
? 0
```

```console
$ fdu --cache-status=all --format json project
{
  "schema": "fdu.cache/1",
  "caches": [
    {"path": "[CACHE_FILE]", "bytes": 12, "content_bytes": null, "state": "stale", "stale_reason": "older_format", "format_version": 1},
    {"path": "[CACHE_FILE]", "bytes": [BYTES], "content_bytes": null, "state": "stale", "stale_reason": "other_engine", "format_version": null},
    {"path": "[CACHE_FILE]", "bytes": [BYTES], "content_bytes": null, "state": "stale", "stale_reason": "unreadable", "format_version": null},
    {"path": "[CACHE_FILE]", "bytes": [BYTES], "content_bytes": null, "state": "current", "root": "[SCAN_PATH]", "entries": 10},
    {"path": "[CACHE_DIR]notes.txt", "bytes": 15, "content_bytes": null, "state": "unrecognized"}
  ]
}
? 0
```

## Clearing Everything Takes Stale Snapshots Too, and Nothing Else

```console
$ fdu --cache-clear=all project
Cache directory: [CACHE_DIR]
Cache cleared: 4 snapshots.
Left in place: 1 file that is not an fdu snapshot; fdu --cache-status=all lists it.
? 0
```

What is left is still reported, rather than hidden behind “No cached snapshots.”

```console
$ fdu --cache-status=all project
[CACHE_DIR]notes.txt  unrecognized, 15 bytes
1 unrecognized file (15 bytes) is not an fdu snapshot, so fdu leaves it in place.
? 0
```

## One Root’s Stale Snapshot Is Cleared by Its Path

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
$ node bin/cache-plant.mjs other-engine
planted: another engine
? 0
```

```console
$ fdu --cache-status project
[CACHE_FILE]  stale (written by another fdu version), [BYTES] metadata bytes, 0 content bytes
1 stale snapshot ([BYTES] bytes) cannot be served by this build; fdu --cache-clear PATH removes it.
? 0
```

```console
$ fdu --cache-clear project
Cache file: [CACHE_FILE]
Cache cleared.
? 0
```

## A Root’s Cache Path Holding Another File Is Left Alone

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
$ node bin/cache-plant.mjs foreign
planted: not a snapshot
? 0
```

```console
$ fdu --cache-clear --cache-status project
Cache file: [CACHE_FILE]
Cache already empty.
Left in place: the file is not an fdu snapshot.
[CACHE_FILE]  unrecognized, 14 bytes
1 unrecognized file (14 bytes) is not an fdu snapshot, so fdu leaves it in place.
? 0
```

## fdu’s Own Leftovers Are Named as fdu’s, and Reclaimed on Its Own Terms

A killed writer leaves a staging file it never renamed, and a removed snapshot can leave
its content sidecar behind.
Both begin with an fdu magic, so calling them foreign would tell the user to leave fdu’s
own debris alone. Clearing the directory takes them, but only a staging file too old to
belong to a running writer, and only a sidecar no snapshot still wants.

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
$ node bin/cache-plant.mjs leftovers
planted: abandoned staging file, in-flight staging file, orphaned sidecar
? 0
```

```console
$ fdu --cache-status=all project
[CACHE_FILE].tmp.1.0011223344556677.0  leftover (staging temporary), [BYTES] bytes
[CACHE_FILE].tmp.1.0011223344556677.0  leftover (staging temporary), [BYTES] bytes
[CACHE_FILE].content  leftover (orphaned content sidecar), 15 bytes
[CACHE_FILE]  10 entries, [BYTES] metadata bytes, 0 content bytes  [SCAN_PATH]
[CACHE_DIR]notes.txt  unrecognized, 15 bytes
3 leftover files ([BYTES] bytes) are fdu's own, left by an interrupted write; fdu --cache-clear=all reclaims them.
1 unrecognized file (15 bytes) is not an fdu snapshot, so fdu leaves it in place.
? 0
```

The staging file written a moment ago stays: nothing here can prove no writer still
holds it, and the age threshold is what stands in for that proof.

```console
$ fdu --cache-clear=all project
Cache directory: [CACHE_DIR]
Cache cleared: 1 snapshot.
Also reclaimed: 2 files fdu left behind.
Left in place: 1 file that is not an fdu snapshot; fdu --cache-status=all lists it.
Left in place: 1 staging file another fdu may still be writing.
? 0
```

## An Unknown Scope Names Both Accepted Values

```console
$ fdu --cache-status=sometimes project
fdu: invalid --cache-status "sometimes": expected root or all
? 2
```
