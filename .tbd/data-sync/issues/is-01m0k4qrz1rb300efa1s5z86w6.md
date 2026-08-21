---
type: is
id: is-01m0k4qrz1rb300efa1s5z86w6
title: Split the files view into files, largest, and recent; --view all becomes --view full
kind: epic
status: open
priority: 1
version: 9
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md
labels: []
dependencies: []
child_order_hints:
  - is-01m0k4rp0ad3axrpqa04n7qe1b
  - is-01m0k4rpbg8gdetv3qx5rfbtkk
  - is-01m0k4rpp91g28w9dwphgxj4mp
  - is-01m0k4rq0vhsvsk51jjb2qtk0e
  - is-01m0k4rqb831356m7sn0p66cdc
  - is-01m0k512k9a6dq2k51fbfe5xn4
  - is-01m0k5dv7ssrm0z1saak7ghcaq
created_at: 2026-08-21T21:48:22.881Z
updated_at: 2026-08-21T22:00:26.104Z
---
`files` was three views wearing one name, which is why it could not have a coherent
default: name-ascending order (right for an enumeration) plus a ten-row cap (right for a
summary) produced "the ten alphabetically-first entries" of a 192,871-entry tree.

Split it so each view answers one question and its defaults follow from that question:

  files     name asc, complete      "what is in here"       -- the fd/find replacement
  largest   size desc, 20, files    "what is eating my disk"
  recent    mtime desc, 20, files   "what changed"

`largest` and `recent` are named presets over the same machinery, not new machinery:

  largest ≡ files --sort size  --limit 20 --kind file
  recent  ≡ files --sort mtime --limit 20 --kind file

`--sort` and `--limit` still override, so the composition model the spec cares about is
intact. The spec removed `largest` and `recent` by asking "can it be expressed as a
composition?" -- a test that conflates capability with interface. A composition a caller
must know how to construct is not a default, and the failed `files` default is what that
conflation cost.

`--view all` becomes `--view full`: every *summary* view, which now includes both
`largest` and `recent`, and excludes `files` because an unbounded enumeration is not a
summary. The rename carries meaning -- `--analyze all` is literally every analyzer, while
`--view full` is a curated digest, and the different word marks the different semantics.

No backward compatibility is owed; this is pre-release.
