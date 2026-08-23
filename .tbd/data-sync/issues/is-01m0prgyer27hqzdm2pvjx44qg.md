---
type: is
id: is-01m0prgyer27hqzdm2pvjx44qg
title: "Partitioned tallies: tag rules and per-plane roll-ups in the engine"
kind: feature
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prgyv1eq0g0mzgntn1p4n6
  - type: blocks
    target: is-01m0ptezmtmkn04mh1f1rwgdxb
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:31:53.944Z
updated_at: 2026-08-23T08:19:18.564Z
---
Opt-in tag configuration on ScanOptions: compiled gitignore (ignore-crate matcher, correct negation) and hidden-with-allowlist rules; entry tag bits; per-plane roll-up state (files, dirs, bytes, allocated, newest mtime, per-extension) through merge_upward, refresh, and watch re-tagging; .gitignore-edit escalation to InvalidateSubtree; enabled rule set versions the snapshot fingerprint. Builds on the closed spike fdu-p35d (0.39-1.76 us/entry measured). Partition-sum property tests and fingerprint-invalidation tests land with it.
