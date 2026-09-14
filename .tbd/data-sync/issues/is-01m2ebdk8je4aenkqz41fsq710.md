---
type: is
id: is-01m2ebdk8je4aenkqz41fsq710
title: "PR #48 review LIFE-8: journal capacity counts items, not bytes"
kind: bug
status: closed
priority: 3
version: 5
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:20.881Z
updated_at: 2026-09-14T15:52:07.890Z
closed_at: 2026-09-14T15:33:25.187Z
close_reason: "5899366: Commit::retained_cost estimates bytes (inline sizes plus path bytes); DEFAULT_JOURNAL_CAPACITY is 8 MiB; journal_capacity keeps its name and now means bytes (public behaviour change, called out in the PR body). Reference model restates the rule; goldens regenerated per scenario and read (header 65536->8388608; observation golden scripted capacity 32 items -> 8192 bytes, Reset commits_visited 7->17). Python field doc left to the Python owner."
resolution: null
duplicate_of: null
---
Low. engine_contract.rs:1560-1565; index.rs:1696-1713. Retention counts changes, transitions, and dirty paths as items, so a 64 KiB item budget can hold tens of MiB of paths, and since() clones all of it. Fix: weight retention by bytes. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).

## Notes

2026-09-14 (fix wave, w-life): fix written, then landed as 5899366 on claude/post-merge-engine-fixes (PR #56). Commit::retained_cost estimates bytes; DEFAULT_JOURNAL_CAPACITY is 8 MiB; OpenOptions::journal_capacity keeps its name and means bytes; the reference model restates the rule; the five opened-root goldens were regenerated per scenario and read (header 65536 -> 8388608; journal-and-observation-recovery scripted capacity 32 items -> 8192 bytes). Spec plan-2026-08-25 line 1479 ('journal capacity must be defined in provider-batch terms') is a MetaBrowser observation about the host's view, not a constraint on the core unit. The Python OpenedOptions.journal_capacity field doc (should say bytes) is left to the Python owner.

2026-09-14 follow-up cfd1335: the first estimate used size_of for the per-item and per-commit allowances, which differ by target; Windows CI held 16 commits in the golden's Reset where macOS and Linux held 17. Replaced with fixed allowances (256 per commit, 128 per item, plus path bytes), restated as literals in the reference model; the golden's Reset line now reads commits_visited: 10 on every target.
