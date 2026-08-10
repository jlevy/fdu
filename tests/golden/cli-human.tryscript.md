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
$ fdu --no-cache --no-color --apparent-size --depth 2 --number 10 project
[SCAN_PATH]  6 files, 3 dirs, 263 B
     128 B  █████░░░░░    49%  dist/
     128 B  █████░░░░░    49%    acorn-0.1.0.tar.gz
      48 B  ██░░░░░░░░    18%  README.md
      36 B  █░░░░░░░░░    14%  src/
      18 B  █░░░░░░░░░     7%    alpha.rs
      18 B  █░░░░░░░░░     7%    omega.rs
      28 B  █░░░░░░░░░    11%  Makefile
      23 B  █░░░░░░░░░     9%  docs/
      23 B  █░░░░░░░░░     9%    FAQ.MD
? 0
```

## Depth and Number Limit Only the Rendered View

```console
$ fdu --no-cache --no-color --apparent-size --depth 1 --number 2 project
[SCAN_PATH]  6 files, 3 dirs, 263 B
     128 B  █████░░░░░    49%  dist/
      48 B  ██░░░░░░░░    18%  README.md
? 0
```

## Type View Uses Apparent Bytes Consistently

```console
$ fdu --no-cache --no-color --by-type --number 10 project
[SCAN_PATH]  6 files, 3 dirs, 263 B
     128 B  █████░░░░░    49%  .tar.gz  1 files
      71 B  ███░░░░░░░    27%  .md  2 files
      36 B  █░░░░░░░░░    14%  .rs  2 files
? 0
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
