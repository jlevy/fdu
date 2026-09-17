---
type: is
id: is-01m2pye0422qnb6171mmxabt4x
title: "P1.1.2: Add the known-violation registry and --record"
kind: task
status: in_progress
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: claude-code@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye0e9mmthdmnttkmbfpgq
parent_id: is-01m2pmr9n3mq4nb2r1pc328qpz
hold: null
hold_until: null
created_at: 2026-09-17T05:46:32.449Z
updated_at: 2026-09-17T06:44:15.540Z
started_at: 2026-09-17T06:17:16.970Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 1: The Path-Independence Harness", commit 2. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `tests/path_independence/registry.py`: `load`, `verify`, `record`; the registry is TOML read with `tomllib` and reviewed like a golden:

```toml
[classes.content-containment]
clears_with = "Phase 1 item 2: content identity and equality serve"
bead = "fdu-gija"

[[violation]]
key = "warm/cli-report/auto/W_all/-/a_lines"
class = "content-containment"
paths = ["analysis.analyze[]", "reports[].metrics.total.metrics.physical_lines"]
```

- A run fails on: an unregistered difference; a registered key whose generalized paths changed; a registered key that now matches cold and the other routes; a class with no entries; an entry still marked `unclassified`; a run with zero cases or zero parseable cold answers.
- An optional `platforms` field records a genuinely platform-specific entry, which is itself a finding to explain.

**Tests**

- `test_harness.py`: each failure condition above, and `--record` round-tripping a registry.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

## Notes

Layer 1 of the core-models stack: branch claude/core-models-1-harness, PR https://github.com/jlevy/fdu/pull/79 (stack #80 on #78). Committed in 6c1c9f67 (harness, registry, targets) and 3106c945 (CI). Close when the layer merges.
