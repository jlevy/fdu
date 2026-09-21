---
type: is
id: is-01m2y7cf9fsawdtq8p5grnr3nq
title: Verify unchanged defaults, directory formats, surface parity, and stacked PR CI
kind: task
status: in_progress
priority: 1
version: 8
spec_path: docs/project/specs/active/plan-2026-09-20-directory-query-formats.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
hold: null
hold_until: null
created_at: 2026-09-20T01:37:40.650Z
updated_at: 2026-09-21T07:57:01.422Z
started_at: 2026-09-20T06:16:21.022Z
---
Validate the epic end to end with portable product goldens, engine tests, and Python
parity. Cover metadata default list/tree equivalence, explicit formats and aliases,
incompatible combinations, grouped/mixed views, legacy names, existing directory
hierarchy, ancestor context, sorting, folding, per-group limits, and unchanged bounded
full reports. Retain pre-change default-output goldens: no new file leaves, columns,
ordering, depth, limits, or omission markers under ordinary defaults.

Cover directory subtree size/age, nested overlap and union totals, descendant
exclusions, ignored policies, cold/warm/cache-only sessions, repeated names over one
index, no extra filesystem/content work, partial coverage/freshness, portable paths,
deep trees, and opened read budgets.
Compare matching paths and exact metrics across flat/machine formats and surfaces;
verify tree roll-ups agree with selected contents without requiring flat file leaves.
Validate JSON/JSONL/YAML schema changes and age reference consistency.

Review expected golden diffs and preserve named portability patterns.
Record parity artifacts on Linux per repository policy.
Run make docs-format and make check; run cross-lint if platform-gated code changes.
Review and commit the implementation separately, stack its PR above the reviewed plan
layer, and watch CI pass.
Close finished beads and tbd sync after successful delivery.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->

## Notes

Implementation committed and pushed 2026-09-21 as PR #103 (branch codex/directory-rollup-query, commit 31e271ec) stacked on the #96 plan. Local macOS status: 757 tests pass, 0 failed; all nine golden tryscripts pass with no diff, which satisfies this bead's retained-default-goldens condition; 18 suites green. NOT done: full make check (blocked by fdu-vjf2, a supply-chain scanner bug unrelated to the diff), make cross-lint, and any Linux/Windows verification beyond PR CI. Parity artifacts still to be recorded by CI on Linux per AGENTS.md. PR is draft until the full gate passes.
