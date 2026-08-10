---
type: is
id: is-01kzkzms089qshzydhxvjfkdt5
title: Add fdu performance probe and resource collectors
kind: task
status: in_progress
priority: 1
version: 11
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzkzmsegmx4sfswka2084se6
  - type: blocks
    target: is-01kzg49sfhtxshw3senkhjmc24
  - type: blocks
    target: is-01kzg4akvjfp8s9h0a1vs7h1c4
  - type: blocks
    target: is-01kzg4c6h9v2dzand7t090p278
  - type: blocks
    target: is-01kzg48z8ykg6t1de81nbvdqpw
  - type: blocks
    target: is-01kzg48zktc7ager8tcy3cst7r
  - type: blocks
    target: is-01kzg49rw1p40pjc18feb9ghpv
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
created_at: 2026-08-09T19:22:54.343Z
updated_at: 2026-08-10T04:36:01.207Z
---
Add a non-production probe in the existing fdu crate for scan-producer, scan-index, snapshot save/load, revalidation, delta apply, and supported query phases. Timed component modes emit compact summaries; untimed validation emits the full stable semantic digest. Add external wall/first-output timing and capability-negotiated CPU, RSS, fault, I/O, syscall, layout/arena, and profile collectors. External end-to-end time stays authoritative, absent counters are null with reasons, record layout is not inferred from RSS, and no benchmark-only stable API or third crate is introduced.

## Notes

Implemented the excluded std-only Rust probe for scan-producer, scan-index, snapshot save/load, unchanged/churn revalidation, deterministic delta apply, exact validation, and steady queries. Added the per-run engine digest oracle, eight committed scenarios, separate external/component evidence, POSIX per-child wait4 rusage with exact unavailable reasons, report surfaces, and six negative state/oracle cases. make test-performance now builds the probe and runs 56 tests on the CI OS matrix. Remaining in this bead: dedicated-host opt-in byte-I/O/syscall/profile collection, build/host provenance hardening, then measured scale scenarios.
