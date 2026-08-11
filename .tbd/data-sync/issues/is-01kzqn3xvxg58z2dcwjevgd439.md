---
type: is
id: is-01kzqn3xvxg58z2dcwjevgd439
title: "P2: document two-layer cache and tiered verification"
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzqn66p0pmck4yg6pexhww2z
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:35:54.237Z
updated_at: 2026-08-11T05:37:08.799Z
---
Documentation-only, but load-bearing for the format's future. Record in help, SKILL.md, and the schema docs: (1) Two cache layers - the core snapshot stays small and fast to open, while content-derived metrics live in a separate per-analyzer derived-data layer keyed by (fingerprint, analyzer id, analyzer version), loaded lazily, invalidated per analyzer without touching tree truth, size-bounded and purgeable via --cache-clear. No analyzer ships here; the shape is fixed so the content tier arrives without a format break. (2) Verification cost follows the query, per the frontier research's tier rule: name-tier questions (counts, tree shape) verify with one stat per directory; any stat-tier metric (sizes, mtimes - every currently shipped view) requires one stat per entry because in-place edits are invisible to directory fingerprints; content-tier adds re-reads of changed files only. This is exact and needs no staleness label. Reducers declare their tier when the registry (fdu-a6dz) lands; until then all shipped views are stat-tier and verification is the N-stat sweep. A labeled stale-sizes mode (research H44) is possible but never a default and out of scope here.
