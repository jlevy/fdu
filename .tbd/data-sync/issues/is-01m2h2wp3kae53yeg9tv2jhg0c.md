---
type: is
id: is-01m2h2wp3kae53yeg9tv2jhg0c
title: "PR #58 review PR58-REC-4: exp-104 transfer claims outrun the data (scale mechanism, regime)"
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m2h2t7k65srszyhv02tag8ba
created_at: 2026-09-14T23:09:01.424Z
updated_at: 2026-09-14T23:27:40.659Z
closed_at: 2026-09-14T23:27:40.658Z
close_reason: "bd8ed0b: not-instruction-bound carries its regime (virtualized 4-core Linux, warm-steady) in the artifact, H103 row, verdict reason and PR body; high-IPC mechanism marked plausible; halves-with-scale replaced by per-entry arithmetic (about 2,000 instructions saved per entry on both subjects, rest about 2x)."
resolution: null
duplicate_of: null
---
PR #58, P3. `performance-loop.md:798` (H103 row), exp-104 "Why it fails" paragraphs 1 and 3, and the PR body, at 3a67552.

The saving per entry is about constant (1,950 vs 2,141 instructions) while the rest of the per-entry cost is about 2x on the kernel checkout, and the two subjects differ in shape as well as size. Only the registry subject was profiled, so "the snapshot parse grows faster" is plausible, not shown. "Not instruction-bound" was measured on one virtualized 4-core Linux host and needs that regime stated.

Fix: state the arithmetic, mark the mechanism as plausible, give the conclusion its regime.
