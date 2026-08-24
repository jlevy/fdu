---
type: is
id: is-01m0tdy6k1kfkywsy4f8kga870
title: Warm snapshots rebuild derived state under the wrong registry
kind: bug
status: open
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels:
  - pr47-review
  - metabrowser
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T17:43:51.648Z
updated_at: 2026-08-24T17:44:16.224Z
---
At PR 47 head e658915, snapshot::load materializes entries before the caller registry is attached. insert_loaded_child therefore recomputes canonical extension and group state with TypeRegistry::compiled, while with_types only swaps the registry pointer after derived rollups already exist. An unchanged warm reconciliation never rebuilds them, so a custom registry can return default-registry extension or group totals indefinitely. The same late-attachment path calls retag, whose traversal joins one PathBuf per entry even for Name-tier rules, reversing the loader optimization called out in the PR. Fix: make snapshot materialization accept the validated active TypeRegistry and TagRules after checking header fingerprints, install them before inserting entries, and derive every entry and rollup exactly once. Tests: cold, warm, and cache-only opens under a deliberately different compound-extension and group registry must agree; a Name-tier tag warm load must not reconstruct paths per entry. Review finding FDU47-R1.
