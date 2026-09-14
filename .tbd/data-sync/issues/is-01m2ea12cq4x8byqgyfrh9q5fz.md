---
type: is
id: is-01m2ea12cq4x8byqgyfrh9q5fz
title: Implement durable disk-usage checkpoints and repeatable net comparisons
kind: feature
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
created_at: 2026-09-13T21:16:01.814Z
updated_at: 2026-09-14T23:26:30.297Z
---
Implement delivery slice 2 in fdu-core with CLI and Python parity: immutable durable checkpoint identity distinct from live clocks, explicit baseline capture and advancement, signed allocated/apparent/count deltas computed before ranking, and repeated A-to-B reads without refresh. Begin with complete scanned roots; preserve pinned baselines across refresh, failure, and cache eviction. Define compatible scope, unknown coverage, rename attribution, own-store exclusion, and concurrency/publication semantics before persistence. Integrate with existing block-format and durable-journal work for later bounded access.

## Notes

2026-09-14 (PR #55 review, 4727de0): slice 2 scope grew; the checkpoint plan now specifies it.
- Checkpoint store separate from the snapshot cache, under the user data directory, with its own format version. Each checkpoint records ScopeIdentity, SemanticIdentity, classification version, and accounting version. Reason: engine_fingerprint mixes the crate version (crates/fdu-core/src/snapshot.rs:173-186 at dda7e6a), so a cache-keyed checkpoint would be discarded on every release.
- Immutable checkpoint ids with movable labels and pins. Comparisons record the resolved ids and refuse a removed checkpoint rather than substituting one.
- Three measures: apparent bytes, per-path allocated bytes, and unique allocated bytes (once per (dev, inode) within one checkpoint). Unique needs a retained link count in fdu-core; Attrs has none (crates/fdu-core/src/engine_contract.rs:81-95 at dda7e6a). Durable hard-link attribution depends on fdu-579b.
- Denied subtrees become typed gaps; a capture with gaps publishes a gap-marked partial checkpoint.
- Open questions for the user are listed at the end of the plan: default measure, label reuse, released-format policy, partial-checkpoint default.
