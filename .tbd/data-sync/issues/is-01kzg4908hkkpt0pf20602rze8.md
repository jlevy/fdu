---
type: is
id: is-01kzg4908hkkpt0pf20602rze8
title: "Spike: gitignore tag-don't-prune via the ignore crate matcher"
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg4bfj2cqzcksgpmfce89w6
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:26:53.712Z
updated_at: 2026-08-13T08:16:22.495Z
closed_at: 2026-08-13T08:16:22.494Z
close_reason: "Measured ignore 0.4.25 GitignoreBuilder plus matched_path_or_any_parents on an Apple M1 Pro/Rust 1.97.1: 100 patterns cost about 0.39–0.51 microseconds per entry and 1,000 patterns about 1.50–1.76 microseconds per entry. Recorded reproducible conditions and the design decision: optional compiled tagging mode, never hidden in the default metadata walk. Removed the one-off dependency after the spike."
---
Confirm GitignoreBuilder/Gitignore can be used standalone — build the matcher from .gitignore files and call matched_path_or_any_parents() during a normal walk — at acceptable per-entry cost, tagging every entry rather than pruning.

This is the fix for the ~1.5s gitignore parse that dominates metabrowser's walker today, and for the special-casing it needed (children of ignored dirs inherit the flag) because per-entry pathspec matching dominated walker time at 500k files. ignore compiles patterns into a single RegexSet/globset automaton, so the cost should be O(1) per entry.
