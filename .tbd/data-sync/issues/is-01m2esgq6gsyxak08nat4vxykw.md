---
type: is
id: is-01m2esgq6gsyxak08nat4vxykw
title: "Re-measure PR #51's whole-scan figures after COMMIT-4 replaced the canonical-path fast path"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2eafpfpe8k5c9z9dhrqvy2y
created_at: 2026-09-14T01:46:43.279Z
updated_at: 2026-09-14T01:46:43.279Z
---
Follow-up on PR #51, recorded by its fixer. Linked to fdu-pro1, which carries the historical regression and says "the current numbers live in #51's description".

**What is stale.** PR #51's description reports release builds, interleaved, five-run medians: `~/.rustup/toolchains` (no gitignores) 1.58 s before → 0.70 s, against main's 0.37 s; a 304-gitignore tree 11.49 s → 3.92 s, against main's 1.32 s. It also reports allocation counters of 2.23M / 422k on the control-free tree. All of these were measured before the review fixes.

**Why they no longer describe the code.** COMMIT-4's fix, ported from #52 3bdfbb2 as 50e6ca5, replaced the measured canonical-path copy lane (`canonical_relative_path` fast path) with one canonicalizing pass into a pre-sized buffer. The PR itself says that change "has not been re-measured". The control-observation gate also moved from `cli.rs` into the shared planner (a69b95e).

**To do.**
- Re-measure #51's head, or the merged stack head if #51 no longer stands alone by then, on the same two subjects under the same protocol, plus the allocation counters.
- Update #51's description, or fdu-pro1's notes if the PR has merged.
- Record the regime and binary identity.

If fdu-lj4h's quiet-host parity run on the merged engine happens first, and covers control-free and control-rich real trees, it supersedes this: record that on fdu-pro1 and close this bead against it.

Review: https://github.com/jlevy/fdu/pull/51#pullrequestreview-5192254822
