---
type: is
id: is-01m2pmrn6ka9kt7f4kcjcmvn8c
title: "Close conformance: empty the violation registry and remove known-gaps sections"
kind: task
status: closed
priority: 0
version: 8
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m2phzegm4b3scda7d1xq3gnm
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
created_at: 2026-09-17T02:57:35.954Z
updated_at: 2026-09-24T09:10:06.754Z
closed_at: 2026-09-24T09:10:06.752Z
close_reason: "Done in PR #124 (merged to main as b06a0201, 2026-09-24): make check, cross-lint, and CI green on tree afe891a8."
resolution: null
duplicate_of: null
---
Acceptance for the plan: path-independence, metric-independence, and writer-equality tests pass on Linux, macOS,
and Windows with an empty registry; no request field parsed, defaulted, or validated outside the request model; no
route decides reads or writes outside the plan model; every tier has identity, fingerprint, serves/project;
sessions and Python Index compute provenance; design principles lose the conformance pointer and architecture
documents lose their known-gaps sections.

## Notes

2026-09-24: conformance pointer removed in PR #124; close on merge.
