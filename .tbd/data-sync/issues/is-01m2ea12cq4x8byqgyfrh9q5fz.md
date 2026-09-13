---
type: is
id: is-01m2ea12cq4x8byqgyfrh9q5fz
title: Implement durable disk-usage checkpoints and repeatable net comparisons
kind: feature
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
created_at: 2026-09-13T21:16:01.814Z
updated_at: 2026-09-13T21:16:01.814Z
---
Implement delivery slice 2 in fdu-core with CLI and Python parity: immutable durable checkpoint identity distinct from live clocks, explicit baseline capture and advancement, signed allocated/apparent/count deltas computed before ranking, and repeated A-to-B reads without refresh. Begin with complete scanned roots; preserve pinned baselines across refresh, failure, and cache eviction. Define compatible scope, unknown coverage, rename attribution, own-store exclusion, and concurrency/publication semantics before persistence. Integrate with existing block-format and durable-journal work for later bounded access.
