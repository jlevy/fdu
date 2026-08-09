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
  ALLOCATED: '\d+'
  BLANK_LINE: '\n'
  MTIME_NS: '-?\d+'
  SCAN_PATH: '[^\r\n]+'
---
# JSON CLI Output

## Human-Only Type View Cannot Be Combined with JSON

```console
$ fdu --no-cache --by-type --json tree
! error: the argument '--by-type' cannot be used with '--json'[BLANK_LINE]
! Usage: fdu --no-cache --by-type <PATH>[BLANK_LINE]
! For more information, try '--help'.
? 2
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
