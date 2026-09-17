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
  # A YAML scalar is quoted only when it would otherwise be ambiguous, which a Windows
  # path with backslashes is and a POSIX path is not. The quoting is the platform's, so
  # it is matched rather than asserted; every other character still has to be exact.
  CACHE_FILE_SCALAR: '"?[^\r\n]+\.fdu"?'
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
     269 B  ██████████   100%  . (7 files) (128 B ignored)
     128 B  █████░░░░░    48%    dist (1 file) (128 B ignored)
      36 B  █░░░░░░░░░    13%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

## The Compact Summary Retains Nothing, and Cache-Only Says So

An unfiltered `summary` that reads no `.gitignore` is answered by the transient tier,
which retains no index and so has no snapshot to write: the cache cannot save the walk
that request is already doing.
A tier that retained nothing has nothing for `--cache only` to read, and it says so
rather than quietly scanning.
A default summary reads `.gitignore` to report its ignored share, which needs the index,
so it saves a snapshot like any other report.

```console
$ fdu --cache-clear project
Cache file: [CACHE_FILE]
Cache cleared.
? 0
```

```console
$ fdu --no-gitignore --view summary --size apparent project
     269 B  7 files, 3 directories
Performance: walked 7 files / 269 B; no ignore rules; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

```console
$ fdu --cache-status project
No cached snapshots.
? 0
```

```console
$ fdu --no-gitignore --cache only --view summary project
fdu: snapshot is not usable: no usable snapshot for this root and scan scope; the `only` cache policy never scans, so use `auto`, which scans when none serves
? 1
```

An ordinary report retains the index, so it does leave a snapshot that `--cache only`
can then answer from without touching the tree.

```console
$ fdu --size apparent project
     269 B  ██████████   100%  . (7 files) (128 B ignored)
     128 B  █████░░░░░    48%    dist (1 file) (128 B ignored)
      36 B  █░░░░░░░░░    13%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

```console
$ fdu --cache only --view summary --size apparent project
     269 B  7 files, 3 directories (128 B ignored)
Performance: walked 0 files / 0 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cache only; total [PERF_TIME]
? 0
```

## Status Maps a Hash-Named File Back to Its Tree

Cache files are named by a hash of their root, which keeps two trees from colliding but
leaves a directory of opaque names.
The header carries the answer.

```console
$ fdu --cache-status project
[CACHE_FILE]  11 entries, [BYTES] metadata bytes, 0 content bytes  [SCAN_PATH]
? 0
```

## Status Renders Through the Format Axis

Agents get cache observability without a second schema style.

```console
$ fdu --cache-status --format json project
{
  "schema": "fdu.cache/2",
  "caches": [
    {"path": "[CACHE_FILE]", "bytes": [BYTES], "state": "current", "root": "[SCAN_PATH]", "entries": 11, "identity": {"entries": {"engine": 2764270474160771455, "max_depth": null, "follow_symlinks": false, "one_filesystem": false, "hidden_fingerprint": 0, "exclude_special": false, "type_rules_fingerprint": 12438660251313372799, "reducers_fingerprint": 1}, "ignore_rules": {"limits": {"budget": 4194304, "line_limit": 16384}}}, "content": null}
  ]
}
? 0
```

Cache status is its own document, not a report, so it carries its own schema identity in
every machine format.

```console
$ fdu --cache-status --format yaml project
schema: fdu.cache/2
caches:
  - path: [CACHE_FILE_SCALAR]
    bytes: [BYTES]
    state: current
    root: [SCAN_PATH]
    entries: 11
    identity:
      entries:
        engine: 2764270474160771455
        max_depth: null
        follow_symlinks: false
        one_filesystem: false
        hidden_fingerprint: 0
        exclude_special: false
        type_rules_fingerprint: 12438660251313372799
        reducers_fingerprint: 1
      ignore_rules:
        limits:
          budget: 4194304
          line_limit: 16384
    content: null
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
     269 B  ██████████   100%  . (7 files) (128 B ignored)
     128 B  █████░░░░░    48%    dist (1 file) (128 B ignored)
      36 B  █░░░░░░░░░    13%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
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
A file that is not an fdu snapshot is listed and left alone, and so is a directory,
which is sized at nothing: what a filesystem calls a directory’s size is its own
accounting, it differs per platform, and it is not bytes a clear could reclaim.

```console
$ fdu --size apparent project
     269 B  ██████████   100%  . (7 files) (128 B ignored)
     128 B  █████░░░░░    48%    dist (1 file) (128 B ignored)
      36 B  █░░░░░░░░░    13%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

```console
$ node bin/cache-plant.mjs stale
planted: format 1, another engine, truncated, notes.txt
? 0
```

```console
$ node bin/cache-plant.mjs directory
planted: a directory under a snapshot's name
? 0
```

```console
$ fdu --cache-status=all project
[CACHE_FILE]  stale (older snapshot format 1), 12 metadata bytes, 0 content bytes
[CACHE_FILE]  stale (written by another fdu version), [BYTES] metadata bytes, 0 content bytes
[CACHE_FILE]  stale (unreadable by this build), [BYTES] metadata bytes, 0 content bytes
[CACHE_FILE]  unrecognized, 0 bytes
[CACHE_FILE]  11 entries, [BYTES] metadata bytes, 0 content bytes  [SCAN_PATH]
[CACHE_DIR]notes.txt  unrecognized, 15 bytes
3 stale snapshots ([BYTES] bytes) cannot be served by this build; fdu --cache-clear=all removes them, along with every current snapshot.
2 unrecognized files (15 bytes) are not fdu snapshots, so fdu leaves them in place.
? 0
```

The two unrecognized entries total the one file’s fifteen bytes: the directory adds
nothing to a figure that is meant to say how much a clear would leave behind.

```console
$ fdu --cache-status=all --format json project
{
  "schema": "fdu.cache/2",
  "caches": [
    {"path": "[CACHE_FILE]", "bytes": 12, "state": "stale", "stale_reason": "older_format", "format_version": 1, "content": null},
    {"path": "[CACHE_FILE]", "bytes": [BYTES], "state": "stale", "stale_reason": "other_engine", "format_version": null, "content": null},
    {"path": "[CACHE_FILE]", "bytes": [BYTES], "state": "stale", "stale_reason": "unreadable", "format_version": null, "content": null},
    {"path": "[CACHE_FILE]", "bytes": 0, "state": "unrecognized", "content": null},
    {"path": "[CACHE_FILE]", "bytes": [BYTES], "state": "current", "root": "[SCAN_PATH]", "entries": 11, "identity": {"entries": {"engine": 2764270474160771455, "max_depth": null, "follow_symlinks": false, "one_filesystem": false, "hidden_fingerprint": 0, "exclude_special": false, "type_rules_fingerprint": 12438660251313372799, "reducers_fingerprint": 1}, "ignore_rules": {"limits": {"budget": 4194304, "line_limit": 16384}}}, "content": null},
    {"path": "[CACHE_DIR]notes.txt", "bytes": 15, "state": "unrecognized", "content": null}
  ]
}
? 0
```

## Clearing Everything Takes Stale Snapshots Too, and Nothing Else

```console
$ fdu --cache-clear=all project
Cache directory: [CACHE_DIR]
Cache cleared: 4 snapshots.
Left in place: 2 files that are not fdu snapshots; fdu --cache-status=all lists them.
? 0
```

What is left is still reported, rather than hidden behind “No cached snapshots.”

```console
$ fdu --cache-status=all project
[CACHE_FILE]  unrecognized, 0 bytes
[CACHE_DIR]notes.txt  unrecognized, 15 bytes
2 unrecognized files (15 bytes) are not fdu snapshots, so fdu leaves them in place.
? 0
```

## One Root’s Stale Snapshot Is Cleared by Its Path

```console
$ fdu --size apparent project
     269 B  ██████████   100%  . (7 files) (128 B ignored)
     128 B  █████░░░░░    48%    dist (1 file) (128 B ignored)
      36 B  █░░░░░░░░░    13%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
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
     269 B  ██████████   100%  . (7 files) (128 B ignored)
     128 B  █████░░░░░    48%    dist (1 file) (128 B ignored)
      36 B  █░░░░░░░░░    13%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
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
     269 B  ██████████   100%  . (7 files) (128 B ignored)
     128 B  █████░░░░░    48%    dist (1 file) (128 B ignored)
      36 B  █░░░░░░░░░    13%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
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
[CACHE_FILE]  unrecognized, 0 bytes
[CACHE_FILE]  11 entries, [BYTES] metadata bytes, 0 content bytes  [SCAN_PATH]
[CACHE_DIR]notes.txt  unrecognized, 15 bytes
3 leftover files ([BYTES] bytes) are fdu's own, left by an interrupted write; fdu --cache-clear=all reclaims them, though a staging file waits until it is too old to be a running writer's.
2 unrecognized files (15 bytes) are not fdu snapshots, so fdu leaves them in place.
? 0
```

The staging file written a moment ago stays: nothing here can prove no writer still
holds it, and the age threshold is what stands in for that proof.
Status says so while counting it, rather than promising three and reclaiming two.

```console
$ fdu --cache-clear=all project
Cache directory: [CACHE_DIR]
Cache cleared: 1 snapshot.
Also reclaimed: 2 files fdu left behind.
Left in place: 2 files that are not fdu snapshots; fdu --cache-status=all lists them.
Left in place: 1 staging file another fdu may still be writing.
? 0
```

## An Unknown Scope Names Both Accepted Values

```console
$ fdu --cache-status=sometimes project
fdu: invalid --cache-status "sometimes": expected root or all
? 2
```
