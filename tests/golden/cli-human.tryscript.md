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
---
# Human CLI Output

## A Full Tree Has Stable Sizes, Ordering, Bars, and Indentation

```console
$ fdu --cache off --color never --size apparent --depth 2 --limit 10 project
     263 B  ██████████   100%  . (6 files)
     128 B  █████░░░░░    49%    dist (1 file)
      36 B  █░░░░░░░░░    14%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)
? 0
```

## Depth and Number Limit Only the Rendered View

```console
$ fdu --cache off --color never --size apparent --depth 1 --limit 2 project
     263 B  ██████████   100%  . (6 files)
     128 B  █████░░░░░    49%    dist (1 file)
      36 B  █░░░░░░░░░    14%    src (2 files)
                                 …
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
     128 B  .tar.gz      1 file
      71 B  .md          2 files
      36 B  .rs          2 files
? 0
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
