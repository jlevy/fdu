---
sandbox: true
path:
  - $TRYSCRIPT_GIT_ROOT/target/debug
env:
  FORCE_COLOR: "0"
  LANG: C
  LC_ALL: C
  NO_COLOR: "1"
  TZ: UTC
patterns:
  BLANK_LINE: '\n'
  OS_ERROR: '[^\r\n]+'
  SCAN_PATH: '[^\r\n]+'
---
# CLI Surface

## Help Is the Complete Invocation Contract

```console
$ fdu --help
A fast, incremental file roll-up engine: hierarchical tallies over large directory trees

Usage: fdu [OPTIONS] [PATH]

Arguments:
  [PATH]  Directory to summarize [default: .]

Options:
  -d, --depth <N>      Directory levels below the root to render; does not limit scanning [default: 2]
  -n, --number <N>     Entries to render per directory, largest first; does not limit scanning [default: 10]
  -a, --apparent-size  Report apparent size rather than the space actually allocated on disk
      --by-type        Break the tree down by file extension instead of by directory
      --json           Emit machine-readable JSON on stdout
      --no-cache       Ignore any cached snapshot and do not write one
      --max-depth <N>  Maximum entry depth to scan and retain; zero keeps only the root
      --no-color       Never colorize output
      --allow-partial  Exit successfully even when unreadable paths make the result partial
  -h, --help           Print help
  -V, --version        Print version

Result scope:
  --depth and --number limit only the rendered view.
  --max-depth limits the scan scope and retained index.

Exit status:
  0  Complete result, or a partial result accepted with --allow-partial
  1  Fatal filesystem or cache error
  2  Partial result, or command-line usage error
? 0
```

## Version Is Exact

```console
$ fdu --version
fdu 0.0.1
? 0
```

## The Default Root Works for an Empty Sandbox

```console
$ fdu --no-cache --no-color --apparent-size --depth 0
[SCAN_PATH]  0 files, 0 dirs, 0 B
? 0
```

## Unknown Options Are Usage Errors on Stderr

```console
$ fdu --definitely-not-an-option
! error: unexpected argument '--definitely-not-an-option' found[BLANK_LINE]
!   tip: to pass '--definitely-not-an-option' as a value, use '-- --definitely-not-an-option'[BLANK_LINE]
! Usage: fdu [OPTIONS] [PATH][BLANK_LINE]
! For more information, try '--help'.
? 2
```

## A Missing Root Is a Fatal Filesystem Error

```console
$ fdu --no-cache missing
! fdu: I/O error at missing: [OS_ERROR]
!   caused by: [OS_ERROR]
? 1
```

## A File Cannot Be Used as the Scan Root

### Create a Regular File

```console
$ node -e "require('node:fs').writeFileSync('plain-file', 'x')"
? 0
```

### Reject It as the Root

```console
$ fdu --no-cache plain-file
! fdu: I/O error at [SCAN_PATH]: scan root is not a directory
!   caused by: scan root is not a directory
? 1
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
