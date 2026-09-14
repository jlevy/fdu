---
type: is
id: is-01m2ebdjw3vkxk0dppemsjcskw
title: "PR #48 review LIFE-7: the observation handoff fails permanently after three convergent races"
kind: bug
status: closed
priority: 3
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:20.482Z
updated_at: 2026-09-14T15:33:16.127Z
closed_at: 2026-09-14T15:33:16.126Z
close_reason: "7f3152f: expectation_matches accepts an op whose target the index already holds (Upsert equal to the current state, Remove of an absent path) on any baseline; it applies as unchanged, not stale, so a refresh converging with the observation handoff no longer costs a full-root walk or fails the root after three. Reference model states the same rule. Tests: two index-level, plus a gate-driven handoff race run 8x."
resolution: null
duplicate_of: null
---
Low. opened.rs:1236-1272; index.rs:2464-2492. expectation_matches rejects an operation whose baseline moved even when the current state already equals its target, so a refresh committing identical attrs makes the handoff pass stale; each retry is a full-root walk and three in a row end in ObservationHandoffIncomplete -> Failed; retained issues duplicate across retries. Fix: count target-equal mismatches as unchanged; retry only conflicting subtrees. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).

## Notes

2026-09-14 (fix wave, w-life): fix written, NOT yet compiled or tested -- the host fell below the 6 GiB disk floor before any build (4.0 GiB free and falling; no cargo process of ours running). Local branch w-life in worktree agent-a365bdbfabb1cd605, off baf6c00. Change: Index::expectation_matches (index.rs) accepts an operation whose target the index already holds -- Upsert{kind,attrs} == current path_state, or Remove with the path Absent -- on any baseline; it then applies as unchanged (apply_upsert/apply_remove already count that). New fn target_state(op) states which ops have a target; control and invalidation ops keep baseline-only arbitration. The independent reference model (tests/reference_model.rs Model::expectation_matches) gets the same rule, stated separately. Tests: index.rs convergent_conditional_upsert_applies_as_unchanged_not_stale and convergent_conditional_remove_applies_as_unchanged_not_stale; opened.rs handoff_settles_through_a_convergent_refresh_without_a_second_walk drives the race with the BeforeObservationHandoff and AfterObservationVerification gates (no sleeps) and asserts no Freshness->Partial transition and exactly one Verified{""} in the journal. Existing ABA tests all have targets that differ from the current state, so they stay stale. To finish: build+run fdu-core lib, reference_model, watch_session tests through the wrapper; run the handoff test 8x; then split the local commit per bead and push.
