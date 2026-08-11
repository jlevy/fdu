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
$ fdu --no-cache --color never --size apparent --depth 2 --limit 10 project
     263 B  . (6 files)
       128 B  dist (1 file)
        36 B  src (2 files)
        23 B  docs (1 file)
? 0
```

## Depth and Number Limit Only the Rendered View

```console
$ fdu --no-cache --color never --size apparent --depth 1 --limit 2 project
     263 B  . (6 files)
  …
       128 B  dist (1 file)
    …
        36 B  src (2 files)
    …
? 0
```

## Type View Uses Apparent Bytes Consistently

```console
$ fdu --no-cache --color never --view types --limit 10 project
   8.0 KiB  .md          2 files
   8.0 KiB  .rs          2 files
   4.0 KiB  .tar.gz      1 file
? 0
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
