---
type: is
id: is-01kzyev98rtws5r6q3gswp7jt0
title: "Retarget the stranded content-metrics work onto the PR #14 branch"
kind: task
status: open
priority: 1
version: 2
labels: []
dependencies: []
created_at: 2026-08-13T21:01:00.568Z
updated_at: 2026-08-13T21:58:42.712Z
---
Follow-up to fdu-dn4u. Re-land the content-metrics work (branch codex/file-content-metrics-plan at fbb36f8) on top of the PR #14 branch claude/pr-8-senior-review-egv3mq. GitHub cannot reopen or retarget closed PR #10 because its base branch is deleted, so this becomes a new PR with base claude/pr-8-senior-review-egv3mq.

INTEGRATION MAP (from a trial merge run 2026-08-13; the merge was aborted, nothing was pushed).
The merge is small mechanically — 3 conflicted files, 7 hunks — but carries one blocker that must be fixed first.

BLOCKER: colliding experiment and hypothesis ids. The content branch numbered its work while #8 was numbering its own, so both sides use exp-040 through exp-043, and the ids H62, H66, and H67 each mean two different things. The performance-loop registry rule is that no id ever means two things. Before merging, renumber the content branch's four experiment artifacts (exp-040 reject-inline-basic-content-analysis, exp-041 reject-prose-collector-gating-for-sloc, exp-042 reject-bounded-markdown-source-reserve, exp-043 decode-complete-utf-8-chunks-in-place) to the next free slots after main's exp-046, and reassign their hypotheses to the next free numbers after H78 (H79+). Verify H68 is genuinely unused on main before keeping it. Then regenerate the ledger with make perf-ledger rather than hand-merging it; the ledger conflict resolves itself once the artifacts are renumbered.

CONFLICT 1, crates/fdu/src/cli.rs (3 hunks): the content branch predates #8's execution planner and calls open_with_pending_save plus crate::query::report directly, while #8 routes one-shot reports through execution::prepare_report. Resolution is a design decision, not a textual pick: analysis needs the retained index, so plan_report must fall closed to RetainedState::FullIndex whenever an analysis profile is requested, exactly as it already does for cache participation, filters, and multiple views. Add that condition to execution::plan_report, keep prepare_report as the single entry point, and add a planner test asserting an analysis request never selects the summary tier.

CONFLICT 2, crates/fdu/src/lib.rs (2 hunks): module declarations union cleanly (pub mod content alongside the cfg-gated mod execution). PendingSave differs structurally — the content branch holds Vec<workers> so metadata and content-sidecar saves can both be outstanding, while #8 holds Option<worker> and made none() pub(crate). Keep the content branch's multi-worker shape and apply #8's visibility.

AFTER MERGING: run the complete gate on the merged head (the content branch adds content-selfcheck, a Python 3.12/3.14 wheel matrix, and roughly 92 tryscript scenarios), reconcile the content-metrics spec and beads with what actually lands, and confirm or reopen fdu-3n8c and fdu-eu80 to match main.
