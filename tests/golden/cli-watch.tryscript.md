---
sandbox: true
path:
  - $FDU_BIN
fixtures:
  - bin
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
Every record an observing run emits states the entry's `.gitignore` classification, the
same fact the initial rows carry; a run that read no rules omits the field rather than
calling every entry unignored.

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
{"schema": "fdu.stream/1", "record": "change", "op": "upsert", "path": "added.txt", "clock": [CLOCK], "kind": "file", "bytes": 5, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "ignored": false}
# change its size
{"schema": "fdu.stream/1", "record": "change", "op": "upsert", "path": "added.txt", "clock": [CLOCK], "kind": "file", "bytes": 12, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "ignored": false}
# remove it
{"schema": "fdu.stream/1", "record": "change", "op": "remove", "path": "added.txt", "clock": [CLOCK]}
# create a directory
{"schema": "fdu.stream/1", "record": "change", "op": "upsert", "path": "sub", "clock": [CLOCK], "kind": "dir", "bytes": [DIR_BYTES], "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "ignored": false}
? 0
```

## Selection Filters the Stream, Not What Is Watched

Scope flags are refused under `--watch`, because a watcher cannot filter backend events
against a narrowed boundary.
Selection flags are accepted: they filter the retained index, so they decide what the
stream reports and leave what is observed untouched.
This is the accepted half of that rule; the refusals are in the command-line surface
session.

Given `--min-size`, the helper adds `--min-size 100 --size apparent` to the stream
above. The bound is on apparent bytes because the default metric, allocated bytes, gives
a four-byte file a whole filesystem block.
It creates a file under the bound, then one over it, then removes the first.
The file under the bound is printed with whatever the stream said about it before the
next record arrived, which is nothing: a record that leaked through the selection would
appear under its label.
Its removal is reported all the same, because a removal carries no size to filter on,
and hiding it would hide the disappearance of something the caller was watching.

```console
$ node -e "require('node:fs').mkdirSync('sized'); require('node:fs').writeFileSync('sized/seed.txt', 'seed')"
? 0
```

```console
$ node bin/watch-capture.mjs --min-size sized
# create a file under the bound
# create a file over the bound
{"schema": "fdu.stream/1", "record": "change", "op": "upsert", "path": "b-large.txt", "clock": [CLOCK], "kind": "file", "bytes": 200, "allocated": [ALLOCATED], "mtime_ns": [MTIME_NS], "ignored": false}
# remove the file under the bound
{"schema": "fdu.stream/1", "record": "change", "op": "remove", "path": "a-small.txt", "clock": [CLOCK]}
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

One record is excluded from the capture rather than pinned here.
The engine reconciles the root when its optimistic apply loop loses three times — it
reads the clock, verifies a sample with filesystem I/O outside the index lock, then
commits only if the clock has not moved — and that reconciliation renders as an empty
path followed by `invalidate`. Doing the I/O under the lock or letting a stale sample
win would both be worse, so this is correct behavior rather than a defect, and the
product is not changed to hide it.
Whether it happens at all depends on how commits interleave, which is not something a
golden can pin.
What this section pins is where the separator falls, so the capture drops
those lines and leaves the change vocabulary to the stream section above, whose helper
matches each record by exact path and operation and is therefore already immune.

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
