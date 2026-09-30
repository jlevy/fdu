---
type: is
id: is-01m3qfdxw5sxhvpthzh1z40dmw
title: "0.3.0: mark counters::Counts #[non_exhaustive] so new counters stop being breaking changes"
kind: task
status: closed
priority: 3
version: 4
labels: []
dependencies: []
parent_id: is-01m3pc4384h1zyw0hdjyzp3p7h
created_at: 2026-09-29T20:59:17.765Z
updated_at: 2026-09-30T05:13:17.048Z
closed_at: 2026-09-30T05:13:17.047Z
close_reason: "Done in 1eb3d570: counters::Counts is #[non_exhaustive] with a doc line saying to build it with Counts::default() and read or assign fields. Audit: the only out-of-crate struct literals were perf_probe's component_counters test (rewritten to assign fields); crates/fdu/tests/detached_performance_invariants.rs only reads fields; fdu-py never names Counts. cargo build --workspace --all-targets --all-features, the example's test, the CLI invariants test, clippy -D warnings, fmt, and docs-format pass. CHANGELOG's Breaking bullet now says Counts is non-exhaustive so future counters are additive."
resolution: null
duplicate_of: null
---
Review finding R161-1 (#161): H171 added three public fields to counters::Counts, which has only public fields and no #[non_exhaustive], so every new counter is a semver-breaking change. The maintainer accepted the break (the release becomes 0.3.0 under the 0.2.2 plan's gate). Since 0.3.0 breaks Counts anyway, marking it #[non_exhaustive] in the same release costs nothing further and makes later counter additions compatible; external code keeps Counts::default() and field reads. Check that nothing outside fdu-core (the fdu CLI tests, perf_probe example, fdu-py) builds Counts with a struct literal. Maintainer decision.

## Notes

2026-09-30: maintainer approved the recommendations: fdu-8f6k (Counts #[non_exhaustive]) and fdu-q7hf (reader diagnostics fields, port of bda4ade1) ship in 0.3.0 on claude/stability-fixes; fdu-4nue's output is by design (surface architecture: machine List materializes every row) and the bead is now the performance follow-up for that machine path.
