---
type: is
id: is-01kzpvt1me7pzvhsjnpk3yag8s
title: Add a mutation-detecting real-tree benchmark baseline
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzpvt1vamkqp8fffnpwhd93v
  - type: blocks
    target: is-01kzpvt22bex8ed6d155y014py
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-10T22:13:36.013Z
updated_at: 2026-08-23T02:11:33.033Z
closed_at: 2026-08-11T00:24:28.827Z
close_reason: "Real-tree harness landed in benchmarks/realtree: path-redacted fingerprint (root_id is the SHA-256 of the path, never the path), fdu-index-record-v1 content digest, before/after mutation check that exits nonzero, and a per-trial oracle comparison that marks disagreeing samples invalid rather than dropping them. Baselines recorded for scan-producer, scan-index, snapshot save/load, and revalidation on a 59,654-entry checkout, with snapshot state and page-cache state reported independently. The mutation check earned itself immediately: the first run against a live checkout invalidated 48 of 60 samples, so reference trees are now APFS clones held still. Evidence: exp-000 in the ledger."
---
Extend the evidence workflow for a read-only operator-supplied tree. Record a normalized path-free inventory/oracle, file and directory counts, apparent bytes, root identity, source revision where available, and a before/after mutation check; reject any trial set if the subject changes. Persist tokenized command shapes rather than personal absolute paths. Establish repeated release-build baselines for scan producer, scan plus index, CLI human/JSON, snapshot save/load, and revalidation on a checkout with a large dependency tree, with snapshot state and filesystem-cache state reported independently.
