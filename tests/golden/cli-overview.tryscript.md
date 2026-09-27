---
sandbox: true
path:
  - $FDU_BIN
fixtures:
  - fixtures/realistic-project
env:
  FORCE_COLOR: "0"
  LANG: C
  LC_ALL: C
  NO_COLOR: "1"
  TZ: UTC
patterns:
  PERF_TIME: '[\d.]+ (ns|µs|ms|s) \(\d+ files/s, \d+\.\d{3} GB/s represented\)'
---
# Realistic Default Overview

## An Explicit Path Gives a Useful Project-Shaped Report

This is the natural tree view with its default depth, 1% share threshold, size ordering,
and compact ten-cell visualization.
Cache and color are disabled to isolate the report, and apparent size makes the same
committed files render identically on every filesystem.
The default includes the significant source files and the nested `index/format`
directory.

```console
$ fdu --cache off --color never --size apparent realistic-project
   7.6 KiB  ██████████   100%  . 16 files
   4.1 KiB  █████░░░░░    55%    src 7 files
   2.6 KiB  ███░░░░░░░    34%      index 4 files
     886 B  █░░░░░░░░░    11%        snapshot.rs
     685 B  █░░░░░░░░░     9%        tree.rs
     641 B  █░░░░░░░░░     8%        format 1 file
     641 B  █░░░░░░░░░     8%          binary.rs
     462 B  █░░░░░░░░░     6%        mod.rs
   1.4 KiB  ██░░░░░░░░    18%      commands 2 files
     726 B  █░░░░░░░░░     9%        scan.rs
     710 B  █░░░░░░░░░     9%        report.rs
     184 B  ░░░░░░░░░░     2%      main.rs
   1.4 KiB  ██░░░░░░░░    19%    docs 3 files
   1.1 KiB  █░░░░░░░░░    14%      guides 2 files
     653 B  █░░░░░░░░░     8%        cache.md
     475 B  █░░░░░░░░░     6%        performance.md
     343 B  ░░░░░░░░░░     4%      reference 1 file
     343 B  ░░░░░░░░░░     4%        cli.md
   1.1 KiB  ██░░░░░░░░    15%    tests 3 files
     973 B  █░░░░░░░░░    12%      cli 2 files
     520 B  █░░░░░░░░░     7%        overview.rs
     453 B  █░░░░░░░░░     6%        cache.rs
     232 B  ░░░░░░░░░░     3%      unit 1 file
     232 B  ░░░░░░░░░░     3%        index.rs
     381 B  ░░░░░░░░░░     5%    README.md
     285 B  ░░░░░░░░░░     4%    benches 1 file
     285 B  ░░░░░░░░░░     4%      reconcile.rs
     172 B  ░░░░░░░░░░     2%    Cargo.toml
! perf: walked 16 files / 7.6 KiB; ignore 0 files / 0 rules; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

## Unknown Branches Do Not Disable Filtering for Known Sizes

A scan-depth boundary gives a portable unknown subtree without depending on permission
bits.
Its unseen contents might be large, so the branch stays visible, while the verified
one-byte sibling is hidden.
The partial-error counterpart uses the complete `partial-tree.txt` engine golden through
the real CLI permission test; both guard the same default threshold rather than testing
status and limits separately.

```console
$ node -e "const fs=require('node:fs'); fs.mkdirSync('shallow-project/pending',{recursive:true}); fs.writeFileSync('shallow-project/large','x'.repeat(10000)); fs.writeFileSync('shallow-project/tiny','x'); fs.writeFileSync('shallow-project/pending/hidden','x'.repeat(20000));"
? 0
```

```console
$ fdu --cache off --color never --size apparent --scan-depth 1 shallow-project
   9.7 KiB  ██████████   100%  . 2 files
   9.7 KiB  ██████████   100%    large
       0 B  ░░░░░░░░░░     0%    pending 0 files
                                 … and 1 B (1 file) more
! note: more includes hidden subtrees already counted in directory totals; files are counted recursively
! note: display limits: below 1% of selected root
! note: incomplete subtrees remain visible below the size threshold
! tip: show smaller entries: --min-share=0%
! perf: walked 2 files / 9.7 KiB; ignore 0 files / 0 rules; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
