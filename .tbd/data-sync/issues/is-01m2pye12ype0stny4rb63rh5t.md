---
type: is
id: is-01m2pye12ype0stny4rb63rh5t
title: "P1.1.5: Run the subset in CI and add the full-matrix workflow"
kind: task
status: closed
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies: []
parent_id: is-01m2pmr9n3mq4nb2r1pc328qpz
created_at: 2026-09-17T05:46:33.437Z
updated_at: 2026-09-17T23:46:47.269Z
closed_at: 2026-09-17T23:46:47.268Z
close_reason: "Shipped in PR #79 and landed on main via stack merge 98379c76."
delegate: claude-code@spud10
hold: null
hold_until: null
started_at: 2026-09-17T06:17:19.138Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 1: The Path-Independence Harness", commit 5. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `.github/workflows/ci.yml` `test` job (`:61-111`): add the pinned `setup-uv` step, which the job lacks, and run the command-line subset on three platforms through the Make target, so the harness runs on uv's Python 3.12 (`tomllib` needs 3.11). Windows budget: 4 minutes.
- `.github/workflows/ci.yml` `parity` job (`:228-274`): run the two-surface subset with `.venv-parity`.
- `.github/workflows/path-independence.yml` (new): schedule, `workflow_dispatch`, and the `path-independence-full` label; three platforms; failing diffs uploaded; toolchain and uv pins inventoried in `supply-chain-policy.json`.

**Done when**

- The new workflow passes `validateWorkflowSecurity` in `scripts/check-supply-chain.mjs`; the full matrix passes on Linux, macOS, and Windows against the registry, with any platform-specific entry explained.
- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

Risks: the Python routes use a release wheel while the command line uses a debug build, so a stale `.venv-parity` goes undetected, as parity already accepts; `samesize_keepmtime` may differ on Windows, where ctime is creation time.

## Notes

Layer 1 of the core-models stack: branch claude/core-models-1-harness, PR https://github.com/jlevy/fdu/pull/79 (stack #80 on #78). Committed in 6c1c9f67 (harness, registry, targets) and 3106c945 (CI). Close when the layer merges.
