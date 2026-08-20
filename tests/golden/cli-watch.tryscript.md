---
sandbox: true
fixtures:
  - bin
path:
  - $TRYSCRIPT_GIT_ROOT/target/debug
env:
  FORCE_COLOR: "0"
  LANG: C
  LC_ALL: C
  NO_COLOR: "1"
  TZ: UTC
  XDG_CACHE_HOME: .cache
patterns:
  CLOCK: '\d+'
  DIR_BYTES: '\d+'
  ALLOCATED: '\d+'
  MTIME_NS: '\d+'
  STAMP: '[^ ]+'
---
# The Watch Change Stream

`--watch` streams one `fdu.stream/1` record per applied change.
A watch process never exits, so it cannot be goldened directly; the `watch-capture`
helper turns it into a command that does.
It starts `fdu --watch`, applies a scripted sequence of filesystem changes, waits for
each change’s own record before making the next, and prints the captured records.
The sequencing is causal, not timed: nothing here depends on how fast the machine or the
events backend is.

What this pins: the stream schema on every record, the op vocabulary, which fields are
present per op, and that removal records carry no metadata — a consumer distinguishes
“gone” from “unknown” by the fields being absent.

The clock is a named pattern rather than a literal because its starting value depends on
how the initial scan batched its observations, which is not part of the stream contract.
Ordering is pinned by the record sequence itself.
Directory sizes and allocated bytes are filesystem-dependent; file byte counts are
exact.

## Build a Tree and Capture a Watch Session

```console
$ node -e "require('node:fs').mkdirSync('tree'); require('node:fs').writeFileSync('tree/seed.txt', 'seed')"
? 0
```

```console
$ node bin/watch-capture.mjs tree
# create a file
{"schema": "fdu.stream/1", "record": "change", "op": "upsert", "path": "added.txt", "clock": [CLOCK], "kind": "file", "bytes": 5, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS]}
# change its size
{"schema": "fdu.stream/1", "record": "change", "op": "upsert", "path": "added.txt", "clock": [CLOCK], "kind": "file", "bytes": 12, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS]}
# remove it
{"schema": "fdu.stream/1", "record": "change", "op": "remove", "path": "added.txt", "clock": [CLOCK]}
# create a directory
{"schema": "fdu.stream/1", "record": "change", "op": "upsert", "path": "sub", "clock": [CLOCK], "kind": "dir", "bytes": [DIR_BYTES], "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS]}
? 0
```

## Text Repaints Are Separated From One Another

JSONL frames every repaint for free: each is a fresh envelope carrying its own
`generated_at`. Text has no such framing, and a watch run has no final answer and
therefore no performance footer, so nothing at all sat between one repaint and the next
— the last row of one and the first row of the following were adjacent lines.
A blank line alone would not do either, since that is already what separates two views
inside a single report.

The separator carries the instant it was rendered, which is what a watch reader wants to
know and the one thing that tells two repaints apart when their numbers happen to match.
It appears between repaints and never above the first, so the opening answer stays
byte-identical to the same query run without `--watch`.

The capture needs a tree of its own: the change stream above leaves its own behind with
a directory in it, and this section’s expectations are written against a tree holding
nothing but the seed file.

```console
$ node -e "require('node:fs').mkdirSync('repaint'); require('node:fs').writeFileSync('repaint/seed.txt', 'seed')"
? 0
```

```console
$ node bin/watch-repaint-capture.mjs repaint
TREE
       4 B  ██████████   100%  . (1 file)

SUMMARY
       4 B  1 file, 0 directories

──── [STAMP] ────
TREE
      16 B  ██████████   100%  . (2 files)

SUMMARY
      16 B  2 files, 0 directories
? 0
```
