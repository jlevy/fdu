---
type: is
id: is-01kzkzmrsbmqxgen0n8sh13hfz
title: Implement performance scenario runner and result schema
kind: feature
status: closed
priority: 1
version: 11
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzkzms089qshzydhxvjfkdt5
  - type: blocks
    target: is-01kzkzms7gmpjb0smwfc0c74wr
  - type: blocks
    target: is-01kzkzmsegmx4sfswka2084se6
  - type: blocks
    target: is-01kzg48zktc7ager8tcy3cst7r
  - type: blocks
    target: is-01kzg4c6h9v2dzand7t090p278
  - type: blocks
    target: is-01kzg48z8ykg6t1de81nbvdqpw
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
created_at: 2026-08-09T19:22:54.122Z
updated_at: 2026-08-10T04:15:49.872Z
closed_at: 2026-08-10T04:15:49.871Z
close_reason: Runner, schemas, immutable evidence, validation, statistics, report, documentation, adversarial tests, Python 3.9 coverage, and full repository handoff gate are complete.
---
Implement strict versioned JSON scenario/corpus/result schemas; direct-argv state-machine execution; a minimal recorded environment; unique marked run roots; exact validation outside the timer; per-invocation corpus, snapshot, and filesystem-cache preparation; timeout and process-group cleanup; randomized paired order; immutable raw trials; baseline compatibility; and deterministic report rendering. Invalid trials remain recorded and never enter statistics.

## Notes

Implemented strict scenario/result contracts and formal JSON Schemas; deterministic paired schedule reconstruction; fresh per-invocation corpus/snapshot/cache preparation; minimal tokenized environments; direct argv; pipe-drained output-digest and real output-file semantics; first-output/completion timing; process-group timeout cleanup; bounded JSON capture; immutable self-checking results with harness, source, host, executable, corpus, and raw-trial identity; compatible-baseline checks; complete summary statistics/review triggers; deterministic Markdown; and structured execute/validate/render/compare commands. Review findings PEV-10 through PEV-14 are resolved in the linked plan. Validation: 51 harness tests pass on Python 3.9 and current Python; full make check passes, including all Rust feature gates, golden CLI tests, docs, audit, wheel smoke, and uvx smoke. No numeric performance claim was made.
