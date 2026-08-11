---
type: is
id: is-01kzqk48vnyz79nvkv2keacw45
title: "PR #3 review R3: validate actual producer and complete roll-up semantics"
kind: bug
status: closed
priority: 1
version: 4
labels:
  - pr-review
dependencies:
  - type: blocks
    target: is-01kzqk493tkcy6nwws6vf9md7f
parent_id: is-01kzqk2ct4s2qjv9e2z17fvywr
created_at: 2026-08-11T05:01:08.340Z
updated_at: 2026-08-11T05:25:01.337Z
closed_at: 2026-08-11T05:25:01.336Z
close_reason: "Implemented v2 producer/index evidence: actual emitted-record digesting inside the measured component, independent per-directory roll-up digest, diagnostic raw mode, strict v2 corpus/probe schemas, and negative oracle tests. Targeted Rust and 68 Python tests pass."
---
FDU-PR3-R3. crates/fdu/examples/perf_probe.rs and benchmarks/realtree/tree.py. The index digest omits per-directory reducers and named extension tallies; scan-producer copies a digest from a second scan and folds that scan into process metrics. Define an independent normalized semantic contract over the measured output, detect duplicates, separate raw and validated work if needed, and add corruption tests. Review: https://github.com/jlevy/fdu/pull/3#issuecomment-5249058288.
