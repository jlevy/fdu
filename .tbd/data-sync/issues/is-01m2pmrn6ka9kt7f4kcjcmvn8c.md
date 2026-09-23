---
type: is
id: is-01m2pmrn6ka9kt7f4kcjcmvn8c
title: "Close conformance: empty the violation registry and remove known-gaps sections"
kind: task
status: in_progress
priority: 0
version: 6
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m2phzegm4b3scda7d1xq3gnm
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
created_at: 2026-09-17T02:57:35.954Z
updated_at: 2026-09-23T08:58:59.461Z
---
Acceptance for the plan: path-independence, metric-independence, and writer-equality tests pass on Linux, macOS,
and Windows with an empty registry; no request field parsed, defaulted, or validated outside the request model; no
route decides reads or writes outside the plan model; every tier has identity, fingerprint, serves/project;
sessions and Python Index compute provenance; design principles lose the conformance pointer and architecture
documents lose their known-gaps sections.

## Notes

2026-09-23 on merged main 7e06e5a4: known-violations registry empty; full path-independence matrix run 35837236472 passed on ubuntu, macOS and Windows; uninterrupted make check (metric independence, writer equality, parity, subset) and cross-lint passed on the identical tree; Plan::admit is the single admission decision (fdu-ftsh); architecture docs have no known-gaps sections. Remaining for this bead: fdu-design-principles.md still carries the 'Conformance is tracked in' pointer (line ~76) that acceptance says to remove, and the analyzer-set containment deferral stays on fdu-7dj6.
