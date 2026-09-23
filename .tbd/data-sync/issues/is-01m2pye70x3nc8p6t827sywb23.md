---
type: is
id: is-01m2pye70x3nc8p6t827sywb23
title: "P1.4.5: Reconciliation drops facts under unverified directories; clear unverified-subtree"
kind: task
status: closed
priority: 0
version: 6
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - core-models
dependencies: []
parent_id: is-01m2pmra8yqrcxg27kc6ezg9vd
hold: null
hold_until: null
created_at: 2026-09-17T05:46:39.516Z
updated_at: 2026-09-23T08:14:06.544Z
started_at: 2026-09-20T04:40:17.361Z
closed_at: 2026-09-23T08:14:06.544Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 4: Provenance and Tree Status", commit 5. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `scan.rs`: a pass that cannot list a directory removes its retained descendants through `remove_known_children` (`:5178`).
- Registry: clear `unverified-subtree` after a full Linux run.

**Tests**

- Cold and warm answers are equal with an unreadable subtree.
- Update `scan.rs:8218` and `:8580` for dropped descendants.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

Risk: dropping descendants matches a cold walk and keeps roll-ups exact, but an opened root loses last-known children it could have shown as unknown, and a transient permission error forces a re-walk. Retaining them marked stale is a scope deferral.

## Notes

2026-09-20 independent integration review at9e96e850: existing reconciling_an_unreadable_control_file_root_drops_its_rules_and_stays_partial test FAILS outside sandbox with permissions enforced. Serial read_control_op error branches keep old rules; parallel path removes via control_read_failed. Must fix serial subtree and listing paths before closure; reviewer preparing exact public repro.

2026-09-20 implementation on codex/release-state-transitions: serial subtree-root and directory-listing control read failures now conditionally remove retained control state, matching cold and parallel scans. The public serial/parallel regression and the existing unreadable-control regression pass outside the sandbox with PermissionDenied enforced.
