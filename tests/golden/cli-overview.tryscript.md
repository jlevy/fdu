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
  PERF_TIME: '[\d.]+ (ns|µs|ms|s)'
  PERF_RATE: '[0-9]{1,3}(?:,[0-9]{3})* files/s \(\d+\.\d{3} GB/s\)'
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
██████████   100%     7.6 KiB  . 16 files
█████░░░░░    55%     4.1 KiB    src/ 7 files
███░░░░░░░    34%     2.6 KiB      index/ 4 files
█░░░░░░░░░    11%       886 B        snapshot.rs
█░░░░░░░░░     9%       685 B        tree.rs
█░░░░░░░░░     8%       641 B        format/ 1 file
█░░░░░░░░░     8%       641 B          binary.rs
█░░░░░░░░░     6%       462 B        mod.rs
██░░░░░░░░    18%     1.4 KiB      commands/ 2 files
█░░░░░░░░░     9%       726 B        scan.rs
█░░░░░░░░░     9%       710 B        report.rs
░░░░░░░░░░     2%       184 B      main.rs
██░░░░░░░░    19%     1.4 KiB    docs/ 3 files
█░░░░░░░░░    14%     1.1 KiB      guides/ 2 files
█░░░░░░░░░     8%       653 B        cache.md
█░░░░░░░░░     6%       475 B        performance.md
░░░░░░░░░░     4%       343 B      reference/ 1 file
░░░░░░░░░░     4%       343 B        cli.md
██░░░░░░░░    15%     1.1 KiB    tests/ 3 files
█░░░░░░░░░    12%       973 B      cli/ 2 files
█░░░░░░░░░     7%       520 B        overview.rs
█░░░░░░░░░     6%       453 B        cache.rs
░░░░░░░░░░     3%       232 B      unit/ 1 file
░░░░░░░░░░     3%       232 B        index.rs
░░░░░░░░░░     5%       381 B    README.md
░░░░░░░░░░     4%       285 B    benches/ 1 file
░░░░░░░░░░     4%       285 B      reconcile.rs
░░░░░░░░░░     2%       172 B    Cargo.toml
! perf: took [PERF_TIME] to walk 16 files (7.6 KiB) at [PERF_RATE]; 0 gitignore rules (0 files); content read 0 B; analysis 0 fresh, 0 cached; cold scan
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
██████████   100%     9.7 KiB  . 2 files
██████████   100%     9.7 KiB    large
░░░░░░░░░░     0%         0 B    pending/ 0 files
░░░░░░░░░░    <1%         1 B    … and 1 more file
! note: more covers unlisted root branches; listed directory totals already include their descendants
! note: display limits: below 1% of selected root
! note: incomplete subtrees remain visible below the size threshold
! tip: show smaller entries: --min-share=0%
! perf: took [PERF_TIME] to walk 2 files (9.7 KiB) at [PERF_RATE]; 0 gitignore rules (0 files); content read 0 B; analysis 0 fresh, 0 cached; cold scan
? 0
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
