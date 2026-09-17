---
type: is
id: is-01m2pmr9n3mq4nb2r1pc328qpz
title: Path-independence, metric-independence, and writer-equality tests with a known-violation registry
kind: task
status: open
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - testing
dependencies:
  - type: blocks
    target: is-01m2pmram44dgp78vm6xq4w7k7
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
created_at: 2026-09-17T02:57:24.130Z
updated_at: 2026-09-17T02:57:35.954Z
---
Bring the 2026-09-17 empirical matrix into the repository (local copy: attic/path-independence-matrix-2026-09-17):
requests across every scope flag, selection filter, view, analyzer set, and size metric; warming histories;
auto, read-only, only; file mutations; command line, Python one-shot, Python Index; parsed comparison with cold
answers apart from provenance. Add a metric-independence test (a metric's cold value is identical under every
analyzer set containing its analyzer) and a writer-equality test (JSON, JSONL, YAML strict 1.1 and 1.2, Python
models; adversarial names and non-UTF-8 paths). A registry of known violations fails on any new difference and on
any stale entry, like the parity artifact. Fast subset in make check; full matrix in CI on three platforms.
