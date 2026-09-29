---
type: is
id: is-01m3pf4gb9t71bs1cp3m382rxn
title: "H181: wake parked walkers only when one is waiting (conditional condvar notify in DirectoryQueue)"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
created_at: 2026-09-29T11:34:54.569Z
updated_at: 2026-09-29T11:34:54.569Z
---
From the Fable mid-night sweep. DirectoryQueue::extend calls notify_all unconditionally (scan.rs ~4506-4517), and std's futex Condvar issues futex(WAKE) whether or not anyone waits. Keep a waiter count in DirectoryQueueState (inc before wait, dec after) and notify only when nonzero; same for claim's finish path. Not H166 (no deque, no scheduling change) and not H158 (consumer wakes). Predicted -0.5 to -1 ms on the real trees, -10 to -15 ms on balanced-1m (31k extends). Measured as a bundle with H182 against the H169 head: default-tree on both real trees, 20 pairs; placebo aggregate-summary --no-controls; secondary strace -c --threads 4 futex count.
