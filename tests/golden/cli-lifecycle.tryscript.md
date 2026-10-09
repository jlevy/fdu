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
  HUMAN_SIZE: '[0-9]{1,3}(?:,[0-9]{3})*(?:\.[0-9])? (?:B|KiB|MiB|GiB|TiB|PiB)'
  # Fingerprints change with the engine version and the type rules, not with the tree.
  FINGERPRINT: '\d+'
  CACHE_FILE: '[^\r\n]+\.metadata\.bin'
  CACHE_ANALYSIS: '[^\r\n]+\.analysis\.bin'
  # A YAML scalar is quoted only when it would otherwise be ambiguous, which a Windows
  # path with backslashes is and a POSIX path is not. The quoting is the platform's, so
  # it is matched rather than asserted; every other character still has to be exact.
  CACHE_FILE_SCALAR: '"?[^\r\n]+\.metadata\.bin"?'
  CACHE_DIR: '[^\r\n]+'
  SCAN_PATH: '[^\r\n]+'
  PERF_TIME: '[\d.]+ (ns|µs|ms|s)'
  PERF_RATE: '[0-9]{1,3}(?:,[0-9]{3})* files/s \(\d+\.\d{3} GiB/s\)'
  FILE_RATE: '[0-9]{1,3}(?:,[0-9]{3})* files/s'
  BYTE_RATE: '[0-9]{1,3}(?:,[0-9]{3})*(?:\.[0-9]+)? (B|KiB|MiB|GiB)/s'
---
# Cache Lifecycle Flags

Inspecting and clearing the cache are explicit flags on the same grammar, never side
effects of a report.
They run before scan validation, so they need no readable tree, and they suppress the
report entirely.

## All-Cache Actions Need No Scan Root

An explicit cache directory and `all` scope work even when the supplied report root does
not exist. This also covers the same operations through the Python CLI shim.

```console
$ fdu --cache-dir root-free-cache --cache-status=all missing-root
No cached snapshots.
? 0
```

```console
$ fdu --cache-dir root-free-cache --cache-clear=all missing-root
Cache directory: [CACHE_DIR]
Cache already empty.
? 0
```

## Status Before Anything Is Cached

```console
$ fdu --cache-status project
No cached snapshots.
? 0
```

## The Default Report Leaves Nothing Behind

Under `--cache auto` a one-shot metadata report writes no snapshot: no later report
reads one.

```console
$ fdu --view summary --size apparent project
     269 B  7 files, 3 directories (128 B gitignored)
! note: totals include gitignored sizes
! perf: took [PERF_TIME] to walk 7 files (269 B) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cold scan
? 0
```

```console
$ fdu --cache-status project
No cached snapshots.
? 0
```

## `--cache on` Leaves a Snapshot Behind

```console
$ fdu --cache on --size apparent project
██████████   100%       269 B  . 7 files (128 B gitignored)
█████░░░░░    48%       128 B    dist/ 1 file (128 B gitignored)
█████░░░░░    48%       128 B      acorn-0.1.0.tar.gz (128 B gitignored)
██░░░░░░░░    18%        48 B    README.md
█░░░░░░░░░    13%        36 B    src/ 2 files
█░░░░░░░░░     7%        18 B      alpha.rs
█░░░░░░░░░     7%        18 B      omega.rs
█░░░░░░░░░    10%        28 B    Makefile
█░░░░░░░░░     9%        23 B    docs/ 1 file
█░░░░░░░░░     9%        23 B      FAQ.MD
░░░░░░░░░░     2%         6 B    .gitignore
! note: totals include gitignored sizes
! perf: took [PERF_TIME] to walk 7 files (269 B) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cold scan
? 0
```

## A Stale Answer Needs a Snapshot Something Left

An unfiltered `summary` that reads no `.gitignore` is answered by the transient tier,
which retains no index, and under `auto` no one-shot metadata report writes a snapshot:
the cache cannot save the walk that request is already doing.
With nothing stored, `--stale-ok` has nothing to read, and it says so rather than
quietly scanning.

```console
$ fdu --cache-clear project
Cache file: [CACHE_FILE]
Cache cleared.
? 0
```

```console
$ fdu --no-gitignore --view summary --size apparent project
     269 B  7 files, 3 directories
! perf: took [PERF_TIME] to walk 7 files (269 B) at [PERF_RATE]; gitignore not read; content read 0 B; analysis 0 fresh, 0 cached; cold scan
? 0
```

```console
$ fdu --cache-status project
No cached snapshots.
? 0
```

```console
$ fdu --no-gitignore --stale-ok --view summary project
fdu: snapshot is not usable: no usable snapshot for this root and scan scope; a stale answer never scans, so run the request once with the `on` cache policy to leave one, or ask for a verified answer, which scans when none serves
? 1
```

A report under `--cache on` retains the index and leaves a snapshot, which `--stale-ok`
can then answer from without touching the tree.
Stdout carries the answer alone; stderr says it is stale and how to get a fresh one.

```console
$ fdu --cache on --size apparent project
██████████   100%       269 B  . 7 files (128 B gitignored)
█████░░░░░    48%       128 B    dist/ 1 file (128 B gitignored)
█████░░░░░    48%       128 B      acorn-0.1.0.tar.gz (128 B gitignored)
██░░░░░░░░    18%        48 B    README.md
█░░░░░░░░░    13%        36 B    src/ 2 files
█░░░░░░░░░     7%        18 B      alpha.rs
█░░░░░░░░░     7%        18 B      omega.rs
█░░░░░░░░░    10%        28 B    Makefile
█░░░░░░░░░     9%        23 B    docs/ 1 file
█░░░░░░░░░     9%        23 B      FAQ.MD
░░░░░░░░░░     2%         6 B    .gitignore
! note: totals include gitignored sizes
! perf: took [PERF_TIME] to walk 7 files (269 B) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cold scan
? 0
```

```console
$ fdu --stale-ok --view summary --size apparent project
     269 B  7 files, 3 directories (128 B gitignored)
! note: totals include gitignored sizes
! warn: stale answer: served from the snapshot without filesystem verification; drop --stale-ok for a fresh answer
! perf: took [PERF_TIME] to walk 0 files (0 B) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cache only
? 0
```

`--quiet` drops the note and the performance footer but keeps the warning, so a quiet
stale answer still cannot pass for a current one.

```console
$ fdu --quiet --stale-ok --view summary --size apparent project
     269 B  7 files, 3 directories (128 B gitignored)
! warn: stale answer: served from the snapshot without filesystem verification; drop --stale-ok for a fresh answer
? 0
```

A verified answer carries no such warning, quiet or not.

```console
$ fdu --quiet --view summary --size apparent project
     269 B  7 files, 3 directories (128 B gitignored)
? 0
```

## Status Maps a Hash-Named File Back to Its Tree

Cache files are named by a hash of their root, which keeps two trees from colliding but
leaves a directory of opaque names.
The header carries the answer.

```console
$ fdu --cache-status project
[CACHE_FILE]  11 entries, [HUMAN_SIZE] metadata, 0 B content  [SCAN_PATH]
? 0
```

## Status Renders Through the Format Axis

Agents get cache observability without a second schema style.

```console
$ fdu --cache-status --format json project
{
  "schema": "fdu.cache/3",
  "caches": [
    {
      "path": "[CACHE_FILE]",
      "bytes": [BYTES],
      "state": "current",
      "root": "[SCAN_PATH]",
      "entries": 11,
      "identity": {
        "entries": {
          "engine": [FINGERPRINT],
          "max_depth": null,
          "follow_symlinks": false,
          "one_filesystem": false,
          "hidden_fingerprint": 0,
          "exclude_special": false,
          "population": "include",
          "control_fingerprint": 0,
          "type_rules_fingerprint": [FINGERPRINT],
          "reducers_fingerprint": 1
        },
        "ignore_rules": {
          "limits": {
            "budget": 4194304,
            "line_limit": 16384
          }
        }
      },
      "content": null
    }
  ]
}

? 0
```

Cache status is its own document, not a report, so it carries its own schema identity in
every machine format.

```console
$ fdu --cache-status --format yaml project
schema: fdu.cache/3
caches:
  -
    path: [CACHE_FILE_SCALAR]
    bytes: [BYTES]
    state: current
    root: [SCAN_PATH]
    entries: 11
    identity:
      entries:
        engine: [FINGERPRINT]
        max_depth: null
        follow_symlinks: false
        one_filesystem: false
        hidden_fingerprint: 0
        exclude_special: false
        population: include
        control_fingerprint: 0
        type_rules_fingerprint: [FINGERPRINT]
        reducers_fingerprint: 1
      ignore_rules:
        limits:
          budget: 4194304
          line_limit: 16384
    content: null

? 0
```

## A Content Sidecar Is Reported Beside Its Snapshot

Analysis is stored in a sidecar of its own, invalidated separately from the snapshot, so
status reports it as the snapshot’s `content` rather than as another file.
It carries how many file records the sidecar holds and the identity that decides which
requests they may serve: the entry tier the records were analyzed over, which alone
carries the type rules, then the analyzer set, the options fingerprint, and each
analyzer with its version.

```console
$ fdu --analyze lines --view families --size apparent project
     128 B   47.6%  binary             1 file
                                         1 binary
      71 B   26.4%  prose              2 files
                                         6 lines (4 nonblank, 2 blank)
                                         2 documentation
      64 B   23.8%  code               3 files
                                         4 lines (4 nonblank, 0 blank)
       6 B    2.2%  unknown            1 file
                                         1 lines (1 nonblank, 0 blank)
! perf: took [PERF_TIME] to walk 7 files (269 B) at [PERF_RATE]; 1 gitignore rule (1 file); content read 141 B at [BYTE_RATE]; analysis 7 fresh at [FILE_RATE], 0 cached; warm revalidation
? 0
```

```console
$ fdu --cache-status project
[CACHE_FILE]  11 entries, [HUMAN_SIZE] metadata, [HUMAN_SIZE] content  [SCAN_PATH]
? 0
```

```console
$ fdu --cache-status --format json project
{
  "schema": "fdu.cache/3",
  "caches": [
    {
      "path": "[CACHE_FILE]",
      "bytes": [BYTES],
      "state": "current",
      "root": "[SCAN_PATH]",
      "entries": 11,
      "identity": {
        "entries": {
          "engine": [FINGERPRINT],
          "max_depth": null,
          "follow_symlinks": false,
          "one_filesystem": false,
          "hidden_fingerprint": 0,
          "exclude_special": false,
          "population": "include",
          "control_fingerprint": 0,
          "type_rules_fingerprint": [FINGERPRINT],
          "reducers_fingerprint": 1
        },
        "ignore_rules": {
          "limits": {
            "budget": 4194304,
            "line_limit": 16384
          }
        }
      },
      "content": {
        "bytes": [BYTES],
        "state": "current",
        "records": 7,
        "identity": {
          "entries": {
            "engine": [FINGERPRINT],
            "max_depth": null,
            "follow_symlinks": false,
            "one_filesystem": false,
            "hidden_fingerprint": 0,
            "exclude_special": false,
            "population": "include",
            "control_fingerprint": 0,
            "type_rules_fingerprint": [FINGERPRINT],
            "reducers_fingerprint": 1
          },
          "analyze": [
            "lines"
          ],
          "options_fingerprint": 12638152016183539244,
          "analyzers": [
            {
              "id": "content-basic-v1",
              "version": 1
            }
          ]
        }
      }
    }
  ]
}

? 0
```

```console
$ fdu --cache-status --format yaml project
schema: fdu.cache/3
caches:
  -
    path: [CACHE_FILE_SCALAR]
    bytes: [BYTES]
    state: current
    root: [SCAN_PATH]
    entries: 11
    identity:
      entries:
        engine: [FINGERPRINT]
        max_depth: null
        follow_symlinks: false
        one_filesystem: false
        hidden_fingerprint: 0
        exclude_special: false
        population: include
        control_fingerprint: 0
        type_rules_fingerprint: [FINGERPRINT]
        reducers_fingerprint: 1
      ignore_rules:
        limits:
          budget: 4194304
          line_limit: 16384
    content:
      bytes: [BYTES]
      state: current
      records: 7
      identity:
        entries:
          engine: [FINGERPRINT]
          max_depth: null
          follow_symlinks: false
          one_filesystem: false
          hidden_fingerprint: 0
          exclude_special: false
          population: include
          control_fingerprint: 0
          type_rules_fingerprint: [FINGERPRINT]
          reducers_fingerprint: 1
        analyze:
          - lines
        options_fingerprint: 12638152016183539244
        analyzers:
          -
            id: content-basic-v1
            version: 1

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
$ fdu --cache on --size apparent project
██████████   100%       269 B  . 7 files (128 B gitignored)
█████░░░░░    48%       128 B    dist/ 1 file (128 B gitignored)
█████░░░░░    48%       128 B      acorn-0.1.0.tar.gz (128 B gitignored)
██░░░░░░░░    18%        48 B    README.md
█░░░░░░░░░    13%        36 B    src/ 2 files
█░░░░░░░░░     7%        18 B      alpha.rs
█░░░░░░░░░     7%        18 B      omega.rs
█░░░░░░░░░    10%        28 B    Makefile
█░░░░░░░░░     9%        23 B    docs/ 1 file
█░░░░░░░░░     9%        23 B      FAQ.MD
░░░░░░░░░░     2%         6 B    .gitignore
! note: totals include gitignored sizes
! perf: took [PERF_TIME] to walk 7 files (269 B) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cold scan
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
$ fdu --cache on --size apparent project
██████████   100%       269 B  . 7 files (128 B gitignored)
█████░░░░░    48%       128 B    dist/ 1 file (128 B gitignored)
█████░░░░░    48%       128 B      acorn-0.1.0.tar.gz (128 B gitignored)
██░░░░░░░░    18%        48 B    README.md
█░░░░░░░░░    13%        36 B    src/ 2 files
█░░░░░░░░░     7%        18 B      alpha.rs
█░░░░░░░░░     7%        18 B      omega.rs
█░░░░░░░░░    10%        28 B    Makefile
█░░░░░░░░░     9%        23 B    docs/ 1 file
█░░░░░░░░░     9%        23 B      FAQ.MD
░░░░░░░░░░     2%         6 B    .gitignore
! note: totals include gitignored sizes
! perf: took [PERF_TIME] to walk 7 files (269 B) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cold scan
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
[CACHE_FILE]  stale (older snapshot format 1), 12 B metadata, 0 B content
[CACHE_FILE]  stale (written by another fdu version), [HUMAN_SIZE] metadata, 0 B content
[CACHE_FILE]  stale (unreadable by this build), [HUMAN_SIZE] metadata, 0 B content
[CACHE_FILE]  unrecognized, 0 B
[CACHE_FILE]  11 entries, [HUMAN_SIZE] metadata, 0 B content  [SCAN_PATH]
[CACHE_DIR]notes.txt  unrecognized, 15 B
3 stale snapshots ([HUMAN_SIZE]) cannot be served by this build; fdu --cache-clear=all removes them, along with every current snapshot.
2 unrecognized files (15 B) are not fdu snapshots, so fdu leaves them in place.
? 0
```

The two unrecognized entries total the one file’s fifteen bytes: the directory adds
nothing to a figure that is meant to say how much a clear would leave behind.

```console
$ fdu --cache-status=all --format json project
{
  "schema": "fdu.cache/3",
  "caches": [
    {
      "path": "[CACHE_FILE]",
      "bytes": 12,
      "state": "stale",
      "stale_reason": "older_format",
      "format_version": 1,
      "content": null
    },
    {
      "path": "[CACHE_FILE]",
      "bytes": [BYTES],
      "state": "stale",
      "stale_reason": "other_engine",
      "format_version": null,
      "content": null
    },
    {
      "path": "[CACHE_FILE]",
      "bytes": [BYTES],
      "state": "stale",
      "stale_reason": "unreadable",
      "format_version": null,
      "content": null
    },
    {
      "path": "[CACHE_FILE]",
      "bytes": 0,
      "state": "unrecognized",
      "content": null
    },
    {
      "path": "[CACHE_FILE]",
      "bytes": [BYTES],
      "state": "current",
      "root": "[SCAN_PATH]",
      "entries": 11,
      "identity": {
        "entries": {
          "engine": [FINGERPRINT],
          "max_depth": null,
          "follow_symlinks": false,
          "one_filesystem": false,
          "hidden_fingerprint": 0,
          "exclude_special": false,
          "population": "include",
          "control_fingerprint": 0,
          "type_rules_fingerprint": [FINGERPRINT],
          "reducers_fingerprint": 1
        },
        "ignore_rules": {
          "limits": {
            "budget": 4194304,
            "line_limit": 16384
          }
        }
      },
      "content": null
    },
    {
      "path": "[CACHE_DIR]notes.txt",
      "bytes": 15,
      "state": "unrecognized",
      "content": null
    }
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
[CACHE_FILE]  unrecognized, 0 B
[CACHE_DIR]notes.txt  unrecognized, 15 B
2 unrecognized files (15 B) are not fdu snapshots, so fdu leaves them in place.
? 0
```

## One Root’s Stale Snapshot Is Cleared by Its Path

```console
$ fdu --cache on --size apparent project
██████████   100%       269 B  . 7 files (128 B gitignored)
█████░░░░░    48%       128 B    dist/ 1 file (128 B gitignored)
█████░░░░░    48%       128 B      acorn-0.1.0.tar.gz (128 B gitignored)
██░░░░░░░░    18%        48 B    README.md
█░░░░░░░░░    13%        36 B    src/ 2 files
█░░░░░░░░░     7%        18 B      alpha.rs
█░░░░░░░░░     7%        18 B      omega.rs
█░░░░░░░░░    10%        28 B    Makefile
█░░░░░░░░░     9%        23 B    docs/ 1 file
█░░░░░░░░░     9%        23 B      FAQ.MD
░░░░░░░░░░     2%         6 B    .gitignore
! note: totals include gitignored sizes
! perf: took [PERF_TIME] to walk 7 files (269 B) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cold scan
? 0
```

```console
$ node bin/cache-plant.mjs other-engine
planted: another engine
? 0
```

```console
$ fdu --cache-status project
[CACHE_FILE]  stale (written by another fdu version), [HUMAN_SIZE] metadata, 0 B content
1 stale snapshot ([HUMAN_SIZE]) cannot be served by this build; fdu --cache-clear PATH removes it.
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
$ fdu --cache on --size apparent project
██████████   100%       269 B  . 7 files (128 B gitignored)
█████░░░░░    48%       128 B    dist/ 1 file (128 B gitignored)
█████░░░░░    48%       128 B      acorn-0.1.0.tar.gz (128 B gitignored)
██░░░░░░░░    18%        48 B    README.md
█░░░░░░░░░    13%        36 B    src/ 2 files
█░░░░░░░░░     7%        18 B      alpha.rs
█░░░░░░░░░     7%        18 B      omega.rs
█░░░░░░░░░    10%        28 B    Makefile
█░░░░░░░░░     9%        23 B    docs/ 1 file
█░░░░░░░░░     9%        23 B      FAQ.MD
░░░░░░░░░░     2%         6 B    .gitignore
! note: totals include gitignored sizes
! perf: took [PERF_TIME] to walk 7 files (269 B) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cold scan
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
[CACHE_FILE]  unrecognized, 14 B
1 unrecognized file (14 B) is not an fdu snapshot, so fdu leaves it in place.
? 0
```

## fdu’s Own Leftovers Are Named as fdu’s, and Reclaimed on Its Own Terms

A killed writer leaves a staging file it never renamed, and a removed snapshot can leave
its content sidecar behind.
Both begin with an fdu magic, so calling them foreign would tell the user to leave fdu’s
own debris alone. Clearing the directory takes them, but only a staging file too old to
belong to a running writer, and only a sidecar no snapshot still wants.

```console
$ fdu --cache on --size apparent project
██████████   100%       269 B  . 7 files (128 B gitignored)
█████░░░░░    48%       128 B    dist/ 1 file (128 B gitignored)
█████░░░░░    48%       128 B      acorn-0.1.0.tar.gz (128 B gitignored)
██░░░░░░░░    18%        48 B    README.md
█░░░░░░░░░    13%        36 B    src/ 2 files
█░░░░░░░░░     7%        18 B      alpha.rs
█░░░░░░░░░     7%        18 B      omega.rs
█░░░░░░░░░    10%        28 B    Makefile
█░░░░░░░░░     9%        23 B    docs/ 1 file
█░░░░░░░░░     9%        23 B      FAQ.MD
░░░░░░░░░░     2%         6 B    .gitignore
! note: totals include gitignored sizes
! perf: took [PERF_TIME] to walk 7 files (269 B) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cold scan
? 0
```

```console
$ node bin/cache-plant.mjs leftovers
planted: abandoned staging file, in-flight staging file, orphaned sidecar
? 0
```

```console
$ fdu --cache-status=all project
[CACHE_FILE].tmp.1.0011223344556677.0  leftover (staging temporary), [HUMAN_SIZE]
[CACHE_FILE].tmp.1.0011223344556677.0  leftover (staging temporary), [HUMAN_SIZE]
[CACHE_ANALYSIS]  leftover (orphaned content sidecar), 15 B
[CACHE_FILE]  unrecognized, 0 B
[CACHE_FILE]  11 entries, [HUMAN_SIZE] metadata, 0 B content  [SCAN_PATH]
[CACHE_DIR]notes.txt  unrecognized, 15 B
3 leftover files ([HUMAN_SIZE]) are fdu's own, left by an interrupted write; fdu --cache-clear=all reclaims them, though a staging file waits until it is too old to be a running writer's.
2 unrecognized files (15 B) are not fdu snapshots, so fdu leaves them in place.
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
