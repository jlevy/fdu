---
sandbox: true
fixtures:
  - fixtures/tree
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
$ fdu --no-cache --no-color --apparent-size --depth 2 --number 10 tree
[SCAN_PATH]  6 files, 2 dirs, 37 B
      13 B  ████░░░░░░    35%  assets/
      13 B  ████░░░░░░    35%    archive.tar.gz
      12 B  ███░░░░░░░    32%  docs/
       9 B  ██░░░░░░░░    24%    guide.md
       3 B  █░░░░░░░░░     8%    note.txt
       5 B  █░░░░░░░░░    14%  ALPHA.TXT
       5 B  █░░░░░░░░░    14%  beta.bin
       2 B  █░░░░░░░░░     5%  README
? 0
```

## Depth and Number Limit Only the Rendered View

```console
$ fdu --no-cache --no-color --apparent-size --depth 1 --number 2 tree
[SCAN_PATH]  6 files, 2 dirs, 37 B
      13 B  ████░░░░░░    35%  assets/
      12 B  ███░░░░░░░    32%  docs/
? 0
```

## Type View Uses Apparent Bytes Consistently

```console
$ fdu --no-cache --no-color --by-type --number 10 tree
[SCAN_PATH]  6 files, 2 dirs, 37 B
      13 B  ████░░░░░░    35%  .tar.gz  1 files
       9 B  ██░░░░░░░░    24%  .md  1 files
       8 B  ██░░░░░░░░    22%  .txt  2 files
       5 B  █░░░░░░░░░    14%  .bin  1 files
? 0
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
