---
type: is
id: is-01m2pye91dbqqaf1qy70cem3mt
title: "P2.1.5: The metric-independence test"
kind: task
status: in_progress
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - core-models
dependencies: []
parent_id: is-01m2phzn814exmf4ty5vw6zha0
hold: null
hold_until: null
created_at: 2026-09-17T05:46:41.580Z
updated_at: 2026-09-20T04:39:46.892Z
started_at: 2026-09-20T04:39:46.892Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 1: Measured Values", commit 5. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `crates/fdu-core/tests/metric_independence.rs` (new) over Rust, Python, Haskell, Markdown, `.txt`, a C++ `.h`, an extensionless shebang script, an extensionless `%PDF` file, invalid UTF-8, a NUL file, and a generated marker.

**Tests**

- For every `METRICS` entry on every row and total, the cold value is identical under every analyzer set including its owner and absent under every set that does not; grouping is identical under every set.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
