---
type: is
id: is-01m2ebdk8je4aenkqz41fsq710
title: "PR #48 review LIFE-8: journal capacity counts items, not bytes"
kind: bug
status: closed
priority: 3
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:20.881Z
updated_at: 2026-09-14T15:33:25.188Z
closed_at: 2026-09-14T15:33:25.187Z
close_reason: "5899366: Commit::retained_cost estimates bytes (inline sizes plus path bytes); DEFAULT_JOURNAL_CAPACITY is 8 MiB; journal_capacity keeps its name and now means bytes (public behaviour change, called out in the PR body). Reference model restates the rule; goldens regenerated per scenario and read (header 65536->8388608; observation golden scripted capacity 32 items -> 8192 bytes, Reset commits_visited 7->17). Python field doc left to the Python owner."
resolution: null
duplicate_of: null
---
Low. engine_contract.rs:1560-1565; index.rs:1696-1713. Retention counts changes, transitions, and dirty paths as items, so a 64 KiB item budget can hold tens of MiB of paths, and since() clones all of it. Fix: weight retention by bytes. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).

## Notes

2026-09-14 (fix wave, w-life): fix written, NOT yet compiled or tested -- host below the 6 GiB disk floor before any build. Local branch w-life in worktree agent-a365bdbfabb1cd605, off baf6c00. Change: Commit::retained_cost (engine_contract.rs) now estimates bytes: size_of::<Commit>() + per change/transition/dirty path (that item's inline size + its path's bytes) + domains. DEFAULT_JOURNAL_CAPACITY becomes 8 MiB (about what 64 Ki items retained for short paths; strictly bounded for long ones), doc says why bytes and why there is no unbounded setting (truncation is always announced). OpenOptions::journal_capacity keeps its name; its doc and the Python OpenedOptions.journal_capacity field doc say bytes. The reference model (tests/reference_model.rs) restates the byte rule and the 8 MiB constant independently, and bounded_journal_reports_loss_at_the_same_clock_as_the_model now overflows with kibibyte paths in hundreds of commits instead of iterating the capacity. Tests in index.rs: oversized_single_batch_is_not_retained and journal_eviction_charges_the_complete_retained_payload compute their capacities from retained_cost; new journal_eviction_is_charged_in_path_bytes (two one-item commits, the second with a 4 KiB name; an item budget kept both). Goldens: all five opened-root goldens record journal_capacity in their action.open line (65536 -> 8388608), and journal-and-observation-recovery's scripted capacity changes from 32 items to 8192 bytes (golden_tests.rs) so the twelve-file refresh still evicts the handoff history for changes.reset; expect commits_visited in that Reset line to change. Each must be regenerated per scenario and the diff read. Spec plan-2026-08-25 line 1479 ('journal capacity must be defined in provider-batch terms') is a MetaBrowser observation about the host's view, not a constraint on the core unit.
