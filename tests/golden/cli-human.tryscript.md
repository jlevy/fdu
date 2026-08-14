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
patterns:
  SCAN_PATH: '[^\r\n]+'
  PERF_TIME: '[\d.]+ (ns|µs|ms|s)'
---
# Human CLI Output

## A Full Tree Has Stable Sizes, Ordering, Bars, and Indentation

```console
$ fdu --cache off --color never --size apparent --depth 2 --limit 10 project
     263 B  ██████████   100%  . (6 files)
     128 B  █████░░░░░    49%    dist (1 file)
      36 B  █░░░░░░░░░    14%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

## Depth and Number Limit Only the Rendered View

```console
$ fdu --cache off --color never --size apparent --depth 1 --limit 2 project
     263 B  ██████████   100%  . (6 files)
     128 B  █████░░░░░    49%    dist (1 file)
      36 B  █░░░░░░░░░    14%    src (2 files)
                                 …
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

## Type View Honors the Selected Size Metric

The old `--by-type` always reported apparent bytes, which made it the one view that
ignored the size metric.
Under the axis design `--size` applies to every view, so the type breakdown answers in
whichever metric was asked for — and apparent bytes are filesystem-independent, which is
what makes this block stable across platforms.

```console
$ fdu --cache off --color never --view types --limit 10 --size apparent project
     128 B   48.7%  archive            1 file
      71 B   27.0%  markdown           2 files, 2 documentation
      36 B   13.7%  rust               2 files
      28 B   10.6%  make               1 file
Performance: walked 6 files / 263 B; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
