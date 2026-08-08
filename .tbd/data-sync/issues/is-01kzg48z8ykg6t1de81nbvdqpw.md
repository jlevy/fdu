---
type: is
id: is-01kzg48z8ykg6t1de81nbvdqpw
title: "Spike: revalidation cost curve at 500k entries"
kind: task
status: open
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg49rw1p40pjc18feb9ghpv
  - type: blocks
    target: is-01kzg4ak7v8z2a7s41rsms8jcb
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:26:52.701Z
updated_at: 2026-08-08T07:27:45.915Z
---
THE load-bearing assumption of the whole cache design: 'a parallel stat sweep of 500k unchanged files is fast enough to feel instant.' Nobody has measured it. If it is false, the cache tiering changes shape and downstream format work is wasted.

Build the corpus generator first (mirror flowmark's benchmarks/generate_corpus.sh). Measure a parallel stat sweep at 100k/500k/1M entries: with and without the directory-mtime shortcut (git's untracked-cache trick), cold and warm page cache. Report the curve, not a single number.

Must land before the snapshot format is frozen.
