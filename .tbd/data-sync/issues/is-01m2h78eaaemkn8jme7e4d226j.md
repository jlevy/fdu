---
type: is
id: is-01m2h78eaaemkn8jme7e4d226j
title: "PR #57 review PR57-AGB6-1: Index::apply installs control input into a non-observing index"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m2h77qemaay1jkhbhfzh35me
created_at: 2026-09-15T00:25:20.955Z
updated_at: 2026-09-15T00:28:45.953Z
closed_at: 2026-09-15T00:28:45.952Z
close_reason: "2eb8992: every reducer lane refuses ControlUpsert/ControlRemove on a non-observing index with ControlStateNotObserved, and install_controls refuses a non-empty table under such a scope (snapshot load); tests opted in by 7ef0d16. Holds at c141285: index.rs carries_unobserved_control_input, install_controls; an_index_that_does_not_observe_controls_refuses_control_input passes."
resolution: null
duplicate_of: null
---
PR #57 review https://github.com/jlevy/fdu/pull/57#pullrequestreview-5200240760, P2. At 7b804df, crates/fdu-core/src/index.rs:1819, :3529, :3549-3568: Index::apply accepted ControlUpsert/ControlRemove on an index whose scope observed no control state, so the table and ignored bits carried real state while is_ignored/controls refused, and a snapshot saved from it loaded as an exact scope match.
