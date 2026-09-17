---
type: is
id: is-01m2pye0e9mmthdmnttkmbfpgq
title: "P1.1.3: Seed the registry from a full Linux run; add Make targets and the subset in make check"
kind: task
status: closed
priority: 0
version: 9
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye0rg6x3k2bzbqgvv6nrz
  - type: blocks
    target: is-01m2pye12ype0stny4rb63rh5t
  - type: blocks
    target: is-01m2pye1dfapb52r07ztah5nap
  - type: blocks
    target: is-01m2pye2qa2z80jqz73hnp5y2h
  - type: blocks
    target: is-01m2pyeg0zegnzm857bvxrbpa6
parent_id: is-01m2pmr9n3mq4nb2r1pc328qpz
created_at: 2026-09-17T05:46:32.776Z
updated_at: 2026-09-17T23:46:47.262Z
closed_at: 2026-09-17T23:46:47.262Z
close_reason: "Shipped in PR #79 and landed on main via stack merge 98379c76."
delegate: claude-code@spud10
hold: null
hold_until: null
started_at: 2026-09-17T06:17:17.812Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 1: The Path-Independence Harness", commit 3. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `tests/path_independence/known-violations.toml`: seeded from a full Linux run (CI or a Linux container; not macOS). Seed classes, each naming the item that clears it: `content-containment` and `mixed-records` (P1.2.4), `projection-route` (P2.4.5), `unverified-subtree` (P1.4.5).
- `tests/path_independence/test_path_independence.py`: `PathIndependence.test_matrix`, with `FDU_PI_TIER=subset|full` and `FDU_PI_SURFACES=cli|cli,python`.
- `Makefile`: targets `path-independence`, `test-path-independence`, `path-independence-full`, `path-independence-record`; `check` (`:113`) runs the subset after `parity-check` with `FDU_PYTHON=$(SMOKE_PYTHON)`; add the targets to `UV_BACKED_TARGETS` (`:163`).
- The subset: 16 requests (`default`, `nogi`, `budget1k`, `scandepth1`, `exclign`, `onlyign`, `v_summary`, `v_summary_nogi`, `v_types`, `a_lines`, `a_code`, `a_words`, `a_all`, `a_lines_v_documents`, `a_code_langs_name_lim1`, `a_all_nogi`) across five warmers and three policies on `cli-report`, five mutations after two warmers, and the Python routes after two warmers: about 1,200 invocations, budgeted at 90 seconds on Linux. The full matrix is about 20,000 invocations, budgeted at 30 minutes per platform.

**Done when**

- The subset runs inside the 90-second Linux budget; the full Linux run passes against the seeded registry.
- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

## Notes

Layer 1 of the core-models stack: branch claude/core-models-1-harness, PR https://github.com/jlevy/fdu/pull/79 (stack #80 on #78). Committed in 6c1c9f67 (harness, registry, targets) and 3106c945 (CI). Close when the layer merges.
