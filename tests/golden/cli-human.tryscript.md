---
sandbox: true
path:
  - $FDU_BIN
fixtures:
  - fixtures/project
env:
  FORCE_COLOR: "0"
  LANG: C
  LC_ALL: C
  NO_COLOR: "1"
  TZ: UTC
patterns:
  SCAN_PATH: '[^\r\n]+'
  PERF_TIME: '[\d.]+ (ns|µs|ms|s)'
  HUMAN_SIZE: '\s*[\d.]+ (B|KiB|MiB|GiB)'
  SEP: '[/\\]'
---
# Human CLI Output

## A Full Tree Has Stable Sizes, Ordering, Bars, and Indentation

```console
$ fdu --cache off --color never --size apparent --depth 2 --limit 10 project
     269 B  ██████████   100%  . (7 files) (128 B ignored)
     128 B  █████░░░░░    48%    dist (1 file) (128 B ignored)
      36 B  █░░░░░░░░░    13%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

## Depth and Number Limit Only the Rendered View

```console
$ fdu --cache off --color never --size apparent --depth 1 --limit 2 project
     269 B  ██████████   100%  . (7 files) (128 B ignored)
     128 B  █████░░░░░    48%    dist (1 file) (128 B ignored)
      36 B  █░░░░░░░░░    13%    src (2 files)
                                 …
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
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
     128 B   47.6%  archive            1 file
      71 B   26.4%  markdown           2 files, 2 documentation
      36 B   13.4%  rust               2 files
      28 B   10.4%  make               1 file
       6 B    2.2%  unknown            1 file
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

## Several Views Are Labelled; One View Is Left Bare

Text was the only format that lost track of which view produced which rows.
JSON, JSONL, and YAML name the view in a field on every report, but text simply
concatenated the blocks — and `types` and `families` render as tables of the same shape,
so which one was which came down to remembering the order they had been requested in.
An all-caps header above each block, one blank line between blocks, is enough to fix it.

```console
$ fdu --cache off --color never --view tree,types,families,summary --size apparent --depth 1 --limit 10 project
TREE
     269 B  ██████████   100%  . (7 files) (128 B ignored)
     128 B  █████░░░░░    48%    dist (1 file) (128 B ignored)
      36 B  █░░░░░░░░░    13%    src (2 files)
      23 B  █░░░░░░░░░     9%    docs (1 file)

TYPES
     128 B   47.6%  archive            1 file
      71 B   26.4%  markdown           2 files, 2 documentation
      36 B   13.4%  rust               2 files
      28 B   10.4%  make               1 file
       6 B    2.2%  unknown            1 file

FAMILIES
     128 B   47.6%  binary             1 file
      71 B   26.4%  prose              2 files, 2 documentation
      64 B   23.8%  code               3 files
       6 B    2.2%  unknown            1 file

SUMMARY
     269 B  7 files, 3 directories (128 B ignored)
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### A Lone View Keeps the Bare Layout

One block has nothing to disambiguate, so it gets no header and every single-view report
renders exactly as it did before.
That is also what keeps `fdu --view files` a listing of paths and nothing else, which is
the property behind piping it into `xargs`.

```console
$ fdu --cache off --color never --view files --include "*.rs" project
src[SEP]alpha.rs
src[SEP]omega.rs
Performance: walked 7 files / [HUMAN_SIZE]; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### A View That Matched Nothing Still Says So

Before the header existed, a view whose selection admitted nothing rendered as no output
at all, so a run asking for three views and getting one table gave no sign the other two
had even been asked for.
The header is what makes an empty result distinguishable from a view that was never
requested.

```console
$ fdu --cache off --color never --view files,types --include "*.nomatch" project
FILES

TYPES
Performance: walked 7 files / [HUMAN_SIZE]; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

### Asking for a Second View Labels the Paths Too

Once a run returns more than one block, the listing is one block among several and is
labelled like the rest.
A caller that wants the bare listing back asks for the single view it actually wanted.

```console
$ fdu --cache off --color never --view files,summary --include "*.rs" --size apparent project
FILES
src[SEP]alpha.rs
src[SEP]omega.rs

SUMMARY
      36 B  2 files, 0 directories
Performance: walked 7 files / 269 B; ignore rules 1 file; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total [PERF_TIME]
? 0
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
