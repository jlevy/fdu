---
type: is
id: is-01kzwxej9axvrpknyke9hz9m2k
title: Plan phased fast file content metrics
kind: task
status: closed
priority: 2
version: 7
spec_path: docs/project/specs/active/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies: []
parent_id: is-01kzg4d256qmchmtyvttnpvn4y
created_at: 2026-08-13T06:37:43.593Z
updated_at: 2026-08-13T07:09:44.189Z
closed_at: 2026-08-13T07:09:44.184Z
close_reason: "Plan spec drafted, reviewed, committed, and published in draft PR #10; make check and all 13 GitHub checks pass."
---
Turn the completed fast file-content metrics research into an active implementation spec with independently useful phases, explicit metric semantics, API and cache boundaries, performance gates, and rollout criteria.

## Notes

Drafted a five-phase implementation ladder: zero-I/O type rollups; basic streaming lines, binary admission, and raw prose; common-language standard SLOC; logical and markup-aware prose; bounded deep detection. The spec defines compatibility, analyzer slots, coverage, conditional derived deltas, sidecar cache identity, Rust/CLI/Python surfaces, semantic and performance gates, and phase exit criteria. Precommit review resolved type-view compatibility and late-NUL handling gaps. Follow-up review added a generic grouped-metric reducer with languages and documents presets, exact named percentage denominators, independent blank-line tallies and whitespace-inclusive or excluding metrics, deterministic ordering, and a shared LF, CRLF, lone-CR, mixed-ending, and chunk-boundary contract. The complete make check gate passes.
