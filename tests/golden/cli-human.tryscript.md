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
  PERF_RATE: '[0-9]{1,3}(?:,[0-9]{3})* files/s \(\d+\.\d{3} GiB/s\)'
  HUMAN_SIZE: '\s*[0-9]{1,3}(?:,[0-9]{3})*(?:\.[0-9]+)? (B|KiB|MiB|GiB|TiB|PiB)'
  AGE: '\s*-?[0-9]{1,3}(?:,[0-9]{3})*(s|m|h|d|w|mo|y)'
  SEP: '[/\\]'
---
# Human CLI Output

## A Full Tree Has Stable Sizes, Ordering, Bars, and Indentation

```console
$ fdu --cache off --color never --size apparent --depth 2 --limit 10 project
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
░░░░░░░░░░     2%         6 B    … and 1 more file
! note: totals include gitignored sizes and descendants
! note: display limits: row limit 10
! tip: show more: --limit=all
! perf: took [PERF_TIME] to walk 7 files (269 B) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cold scan
? 0
```

## The Full Shorthand Shows the Complete Tree

`--full` is the four explicit display bounds below.
Both reports show the same rows, including the smallest file, with no remainder or
display-limit diagnostic.

```console
$ fdu --cache off --color never --size apparent --view tree --full project
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
$ fdu --cache off --color never --size apparent --view tree --depth=all --breadth=all --limit=all --min-share=0% project
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

### Full Flat File and Directory Listings

The same shorthand reaches flat file paths and directory rows.
Directory sizes include regular files below each listed directory.

```console
$ fdu --cache off --color never --size apparent --kind file --full --format paths project
dist[SEP]acorn-0.1.0.tar.gz
README.md
Makefile
docs[SEP]FAQ.MD
src[SEP]alpha.rs
src[SEP]omega.rs
.gitignore
? 0
```

```console
$ fdu --cache off --color never --size apparent --kind dir --full --sort name --long project
     128 B  [AGE] dist
      23 B  [AGE] docs
      36 B  [AGE] src
? 0
```

### Zero Rows Still Account for Every Hidden File

```console
$ fdu --cache off --color never --size apparent --view tree --limit=0 project
██████████   100%       269 B  … and 7 more files
! note: totals include descendants
! note: display limits: row limit 0
! tip: show more: --limit=all
! perf: took [PERF_TIME] to walk 7 files (269 B) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cold scan
? 0
```

## Depth and Number Limit Only the Rendered View

```console
$ fdu --cache off --color never --size apparent --depth 1 --limit 2 project
██████████   100%       269 B  . 7 files (128 B gitignored)
█████░░░░░    48%       128 B    dist/ 1 file (128 B gitignored)
█████░░░░░    52%       141 B    … and 6 more files
! note: totals include gitignored sizes and descendants
! note: display limits: depth 1, row limit 2
! tip: show more: --depth=all --limit=all
! perf: took [PERF_TIME] to walk 7 files (269 B) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cold scan
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
      71 B   26.4%  markdown           2 files
                                         2 documentation
      36 B   13.4%  rust               2 files
      28 B   10.4%  make               1 file
       6 B    2.2%  unknown            1 file
! perf: took [PERF_TIME] to walk 7 files (269 B) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cold scan
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
██████████   100%       269 B  . 7 files (128 B gitignored)
█████░░░░░    48%       128 B    dist/ 1 file (128 B gitignored)
██░░░░░░░░    18%        48 B    README.md
█░░░░░░░░░    13%        36 B    src/ 2 files
█░░░░░░░░░    10%        28 B    Makefile
█░░░░░░░░░     9%        23 B    docs/ 1 file
░░░░░░░░░░     2%         6 B    .gitignore

TYPES
     128 B   47.6%  archive            1 file
      71 B   26.4%  markdown           2 files
                                         2 documentation
      36 B   13.4%  rust               2 files
      28 B   10.4%  make               1 file
       6 B    2.2%  unknown            1 file

FAMILIES
     128 B   47.6%  binary             1 file
      71 B   26.4%  prose              2 files
                                         2 documentation
      64 B   23.8%  code               3 files
       6 B    2.2%  unknown            1 file

SUMMARY
     269 B  7 files, 3 directories (128 B gitignored)
! note: totals include gitignored sizes
! note: display limits: depth 1
! tip: show more: --depth=all
! perf: took [PERF_TIME] to walk 7 files (269 B) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cold scan
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
! perf: took [PERF_TIME] to walk 7 files ([HUMAN_SIZE]) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cold scan
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
! perf: took [PERF_TIME] to walk 7 files ([HUMAN_SIZE]) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cold scan
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
! perf: took [PERF_TIME] to walk 7 files (269 B) at [PERF_RATE]; 1 gitignore rule (1 file); content read 0 B; analysis 0 fresh, 0 cached; cold scan
? 0
```

## Tree Bar Width

The bar width changes only the tree presentation.
Zero and negative widths omit the bar and its gutter; the percentages, sizes, names, and
remainder stay visible.

```console
$ fdu --cache off --quiet --color never --size apparent --view tree --depth 0 --bar-size 20 project
████████████████████   100%       269 B  . 7 files (128 B gitignored)
████████████████████   100%       269 B    … and 7 more files
? 0
```

```console
$ fdu --cache off --quiet --color never --size apparent --view tree --depth 0 --bar-size 0 project
 100%       269 B  . 7 files (128 B gitignored)
 100%       269 B    … and 7 more files
? 0
```

```console
$ fdu --cache off --quiet --color never --size apparent --view tree --depth 0 --bar-size -1 project
 100%       269 B  . 7 files (128 B gitignored)
 100%       269 B    … and 7 more files
? 0
```

## Oversized Human Tree Bars Fail Before Scanning

An oversized human tree bar is a usage error before the root is touched.

```console
$ fdu --bar-size 4097 --view tree missing-tree
fdu: invalid --bar-size "4097": expected at most 4096 cells
? 2
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
