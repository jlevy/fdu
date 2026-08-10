---
type: is
id: is-01kzkzms089qshzydhxvjfkdt5
title: Add fdu performance probe and resource collectors
kind: task
status: closed
priority: 1
version: 13
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
updated_at: 2026-08-10T04:50:58.269Z
closed_at: 2026-08-10T04:50:58.268Z
close_reason: "The portable probe and collector foundation is complete: eight real component jobs, exact fingerprint-sensitive oracle checks, compact untimed validation, per-invocation wall/component/first-output timing, per-child wait4 CPU/RSS/fault/block/context-switch evidence, null-with-reason capability handling, adversarial validation, 56 harness tests, and full make-check coverage. Claim-grade build/host manifests moved to fdu-849g and intrusive dedicated-Linux collectors moved to fdu-bmhr; both now block stable release evidence rather than the local scale spikes."
---
Add a non-production probe in the existing fdu crate for scan-producer, scan-index, snapshot save/load, revalidation, delta apply, and supported query phases. Timed component modes emit compact summaries; untimed validation emits the full stable semantic digest. Add external wall/first-output timing and capability-negotiated CPU, RSS, fault, I/O, syscall, layout/arena, and profile collectors. External end-to-end time stays authoritative, absent counters are null with reasons, record layout is not inferred from RSS, and no benchmark-only stable API or third crate is introduced.

## Notes

Portable probe/collector slice passes the full make check: all Rust feature matrices and MSRV, 26 tryscript blocks, 56 performance tests including eight real probe jobs plus six adversarial cases, docs, audits, Python concurrency, wheel install, and uvx smoke. fdu-6x07 is closed after the oracle exposed and verified the symlink/special-node roll-up correction. Remaining: dedicated-host profile/byte-I/O collectors and stronger build/host provenance before scale evidence.
