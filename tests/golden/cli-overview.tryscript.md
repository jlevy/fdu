---
sandbox: true
fixtures:
  - fixtures/realistic-project
path:
  - $TRYSCRIPT_GIT_ROOT/target/debug
env:
  FORCE_COLOR: "0"
  LANG: C
  LC_ALL: C
  NO_COLOR: "1"
  TZ: UTC
patterns:
  PERF_TIME: '[\d.]+ (ns|µs|ms|s)'
---
# Realistic Default Overview

## An Explicit Path Gives a Useful Project-Shaped Report

This is the natural tree view with its default depth, row limit, ordering, and compact
ten-cell visualization.
Cache and color are disabled to isolate the report, and apparent size makes the same
committed files render identically on every filesystem.
The fixture has one directory below the displayed depth: its bytes and file count roll
up into `index`, while the report correctly avoids a misleading `…` row because no
ranked sibling was omitted.

```console
$ fdu --cache off --color never --size apparent realistic-project
   7.6 KiB  ██████████   100%  . (16 files)
   4.1 KiB  █████░░░░░    55%    src (7 files)
   2.6 KiB  ███░░░░░░░    34%      index (4 files)
   1.4 KiB  ██░░░░░░░░    18%      commands (2 files)
   1.4 KiB  ██░░░░░░░░    19%    docs (3 files)
   1.1 KiB  █░░░░░░░░░    14%      guides (2 files)
     343 B  ░░░░░░░░░░     4%      reference (1 file)
   1.1 KiB  ██░░░░░░░░    15%    tests (3 files)
     973 B  █░░░░░░░░░    12%      cli (2 files)
     232 B  ░░░░░░░░░░     3%      unit (1 file)
     285 B  ░░░░░░░░░░     4%    benches (1 file)
Performance: walked 16 files / 7.6 KiB; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
