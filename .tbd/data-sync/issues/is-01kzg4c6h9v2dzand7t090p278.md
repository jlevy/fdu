---
type: is
id: is-01kzg4c6h9v2dzand7t090p278
title: "Benchmark harness: cold/warm x raw-walk/with-stats matrix vs dut and gdu"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:28:38.441Z
updated_at: 2026-08-08T07:28:38.441Z
---
'Fastest with full stats' must be benchmarked like-for-like or the headline claim compares different jobs: the fastest walkers in the survey (bfs, dut) discard most metadata, while fdu retains a full inventory.

The harness must report FOUR quadrants separately: cold and warm page cache, crossed with raw-walk and with-stats. Every benchmark in the research doc that omitted the cold/warm distinction was misleading, including the two headline tables — which are also from different corpora and so are not comparable to each other.

Build the corpus generator first, mirroring flowmark's benchmarks/generate_corpus.sh, and share it with the revalidation spike.

Benchmark against dut and gdu, NOT dust. Dust is mid-pack by its competitors' published numbers and was only ever the informal model for this work. Targets: cold scan within ~1.5x of dut on the same corpus; warm re-run well under 1s for 500k entries, against flowmark's 23ms bar at ~1k files.

Nothing in the README may claim performance until this exists and cites it.
