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
  -d, --depth <N>      Directory levels to show; does not limit scanning [default: 2]
  -n, --number <N>     Entries to show per directory, largest first [default: 10]
  -a, --apparent-size  Use apparent bytes instead of allocated disk space
      --by-type        Group totals by file extension instead of directory
      --json           Write schema-versioned JSON to stdout
      --no-cache       Do not read or write the snapshot cache
      --max-depth <N>  Limit scanning and retention to N entry levels
      --color <WHEN>   Colorize human output: auto, always, or never [default: auto]
      --allow-partial  Accept incomplete totals when paths cannot be read
      --skill          Print a portable agent skill to stdout
  -h, --help           Print help
  -V, --version        Print version

Examples:
  fdu
  fdu --depth 3 --number 20 ~/src
  fdu --by-type ~/Downloads
  fdu --json --depth 1 --number 50 .

Output and automation:
  Human output reports allocated disk space unless --apparent-size is set.
  Results go to stdout; warnings and errors go to stderr.
  JSON is schema-versioned, never colorized, and includes completeness and truncation.
  For automation, check the exit status, complete, errors, tree_truncated, and scan_max_depth.
  The command never prompts, pages, or animates progress.

Result scope:
  --depth and --number limit only the rendered view.
  --max-depth limits the scan scope and retained index.

Cache:
  Unless --no-cache is set, fdu reads and writes a snapshot in the user cache directory.

Color:
  --color overrides NO_COLOR and FORCE_COLOR. In auto mode, NO_COLOR disables color,
  FORCE_COLOR enables it, and otherwise the destination must be a terminal.

Exit status:
  0  Complete result, or a partial result accepted with --allow-partial
  1  Fatal filesystem or cache error
  2  Partial result, or command-line usage error
? 0
```

## The Portable Skill Is Complete and Version-Pinned

````console
$ fdu --skill
---
name: fdu
description: >-
  Inspect directory trees with hierarchical file counts, apparent and allocated sizes,
  recency, and extension tallies. Use when investigating disk usage, finding large
  directories, summarizing file types, or collecting stable JSON filesystem roll-ups
  for scripts and coding agents.
---
# fdu Directory Roll-Ups

Use `fdu` to summarize a directory tree without modifying files in that tree.

## Run fdu

Use the local command when it is available:

```bash
fdu --json --depth 2 --number 20 PATH
```

If no local command exists and this release is published on PyPI, use the exact reviewed
version. Never use an unversioned `uvx` runner or `latest` in agent instructions:

```bash
uvx --from fdu==0.0.1 fdu --json --depth 2 --number 20 PATH
```

## Choose the View Deliberately

- Use the default human tree for terminal investigation.
- Use `--by-type` for an extension summary.
- Use `--json` for scripts and agents; it never contains ANSI color.
- Use `--apparent-size` for logical bytes instead of allocated disk space.
- Use `--no-cache` when the run must not read or write the user cache.

`--depth` and `--number` limit only the returned view.
`--max-depth` limits what is scanned and retained, so do not use it merely to reduce
output.

## Validate Every Automated Result

Check the process exit status and these JSON fields:

- `schema` before parsing fields
- `complete` and `errors` before trusting totals
- `tree_truncated` before treating the rendered tree as exhaustive
- `scan_max_depth` before treating the scan scope as exhaustive
- `freshness` before presenting cached or partial data as current

Exit 0 is accepted success, exit 1 is a fatal failure, and exit 2 is incomplete data or
invalid usage. Do not discard useful stdout from exit 2; inspect the completeness fields
and use `--allow-partial` only when incomplete totals are acceptable.

Run `fdu --help` for the complete flag, stream, cache, color, scope, and exit contract.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
? 0
````

## Version Is Exact

```console
$ fdu --version
fdu 0.0.1
? 0
```

## The Default Root Works for an Empty Sandbox

```console
$ fdu --no-cache --color never --apparent-size --depth 0
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
