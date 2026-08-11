---
type: is
id: is-01kzrv2fca4h3rb4mtf263b5gc
title: Streaming scan session with breadth-first order
kind: feature
status: closed
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
created_at: 2026-08-11T16:39:12.521Z
updated_at: 2026-08-11T19:34:03.651Z
---
Blocks the interactive-browser use case (research-2026-08-11-interactive-browser-use-case.md). A browser opening a multi-million-entry tree needs results while the walk runs, not after. fdu already streams internally - the parallel producer emits Observation batches and merge_upward keeps every ancestor roll-up current, so index.rollup(path) is already a valid monotonic lower bound mid-scan. Two things are missing. (1) TRAVERSAL ORDER: the walker's DirectoryQueue claims LIFO, so partial results are depth-first - mid-scan one subtree reads complete while siblings read zero, and a user sorting by size sees a confident wrong ranking. Breadth-first (FIFO, or depth-ordered priority so shallow work is always preferred) makes every top-level number a lower bound that only grows, which is why metabrowser's Python walker already queues BFS to a first-render depth. Keep depth-first available - better locality and memory, right default for a one-shot CLI - as a scan policy chosen by the caller's contract. Measure the memory cost of a wide BFS frontier. (2) SESSION API: start the scan, return immediately, read roll-ups and per-path completeness while it runs, in Rust and Python. IndexHandle already gives safe shared reads during writes and Freshness already distinguishes partial from fresh, so this is mostly surface. Acceptance: time-to-useful-top-level-ranking on a home-folder-scale tree, BFS vs DFS.
