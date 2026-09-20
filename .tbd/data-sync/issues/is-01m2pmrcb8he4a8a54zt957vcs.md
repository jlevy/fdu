---
type: is
id: is-01m2pmrcb8he4a8a54zt957vcs
title: "Phase 2 item 4: .gitignore observation projection on every route"
kind: epic
status: open
priority: 0
version: 13
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - design
  - cache
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
  - type: blocks
    target: is-01m2yh8kc79nw7bn6k6xw8g3bp
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
child_order_hints:
  - is-01m2pyeeppbcbq3hfepwjcthyw
  - is-01m2pyef13ts4hgtt92bfeftfb
  - is-01m2pyefbdg5fgka94j1bvwpbd
  - is-01m2pyefnttqs5rzqcwbgf2qb5
  - is-01m2pyeg0zegnzm857bvxrbpa6
created_at: 2026-09-17T02:57:26.887Z
updated_at: 2026-09-20T05:30:29.038Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 4: The `.gitignore` Observation Projection on Every Route", moved here when the item became beads (PR #78 at `e52383d4`; locators verified at `5f2d36d`). The commits are this bead's children, P2.4.1 to P2.4.5; their blockers carry the ordering, so this bead only groups them and closes when they do.

A controls-on snapshot whose other identity fields equal the request’s is parsed into an
index of the requested scope, without installing its control table.
That index equals a controls-off scan, because entries do not depend on observation, so
the projection happens at load and serves `open`, Python `open`, the watch’s initial
report, and warm revalidation alike.

| File | Function or type | Change |
| --- | --- | --- |
| `stored_state.rs` | `serves_snapshot` | Extend item 2’s `Exact` and `Refuse` with `ProjectControlsOff`; it takes no policy and no consumer |
| `snapshot.rs` | `load_serving(path, types, wanted) -> LoadOutcome::{Served(Index, Serves), Refused(SnapshotIdentity), Absent}`; `parse_stream` (`:614-708`) | Add; a projection builds the index with the requested scope and skips installing the control section (`:699-700`) |
| `lib.rs` | `snapshot_scope_serves` (`:377-395`), `SnapshotUse` (`:368-374`), `RefusedSnapshot` (`:398-402`), `open_for_report` load filter (`:501-531`), `unusable_snapshot_message` (`:415-454`), `OpenReport` (`:282-291`) | Delete the first three; load through `load_serving`; the refusal message takes the refused identity; add `OpenReport.projected`, and a projected load does not overwrite the stronger snapshot |
| `execution.rs` | retag (`:324-334`) | Delete, with a debug assertion that the answer’s scope equals the request’s |
| `query/query_report.rs` | `forget_ignore_classification` (`:1014-1046`), export at `query.rs:21` | Delete |
| `crates/fdu/src/cli.rs` | `run_watch` (`:760-798`), `save_live` (`:931-948`) | The initial report warm-starts through `open_with_pending_save`; `save_live` skips writing while projected |
| Documentation | `lib.rs:174-183`, `:331-336`; `execution.rs:198-206`; `_api.py:367-374`; `cli-cache.tryscript.md:253-257`; cache design Known Gaps | Rewrite |

**Tests:** `snapshot_serving_is_equality_plus_observation_on_to_off`;
`a_projected_load_equals_a_controls_off_scan`; the `lib.rs` tests at `:966` and `:1019`
become projection tests, plus `a_projected_open_leaves_the_stronger_snapshot_in_place`;
extend `execution.rs:584` across policies and routes and flip `:630`;
`a_session_over_a_projected_index_refuses_an_ignored_selection`; a Python smoke check
that a controls-off open answers from a default snapshot; a `cli-cache` golden answering
`warm_revalidate`; the `cli-watch-initial` harness route, clearing `projection-route`.

**Commits:**
1. `ProjectControlsOff` in `serves_snapshot`, and `load_serving`, with tests.
2. Route `open_for_report` through them and delete `SnapshotUse` and
   `snapshot_scope_serves`.
3. Delete the retag and `forget_ignore_classification`.
4. The no-overwrite rule and the `save_live` guard.
5. The Python check, golden, harness route, and documentation.

**Risks:** a controls-off watch never persists over the stronger snapshot, so each run
revalidates from an older one, a cost rather than a different answer; confirm that a
projected index’s empty control table cannot later be saved claiming limits the request
never had.

## Original scope (before the implementation detail)

Implement the stored-state compatibility rule everywhere: the .gitignore observation projection (on to off) for
one-shot, open, Python, and watch alike; content analyzer-subset projection through fdu-vwkg once per-analyzer
records exist. Each projection is proven by the path-independence test.

## Notes

2026-09-17 (PR #78 review): Phase 2: the .gitignore observation projection on every route. Content subset projection is a deferral (fdu-vwkg).
