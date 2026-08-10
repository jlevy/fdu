---
type: is
id: is-01kzpvt1me7pzvhsjnpk3yag8s
title: Add a mutation-detecting real-tree benchmark baseline
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzpvt1vamkqp8fffnpwhd93v
  - type: blocks
    target: is-01kzpvt22bex8ed6d155y014py
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-10T22:13:36.013Z
updated_at: 2026-08-10T22:13:43.171Z
---
Extend the evidence workflow for a read-only operator-supplied tree. Record a normalized path-free inventory/oracle, file and directory counts, apparent bytes, root identity, source revision where available, and a before/after mutation check; reject any trial set if the subject changes. Persist tokenized command shapes rather than personal absolute paths. Establish repeated release-build baselines for scan producer, scan plus index, CLI human/JSON, snapshot save/load, and revalidation on a checkout with a large dependency tree, with snapshot state and filesystem-cache state reported independently.
