---
type: is
id: is-01m4kwnfb5r464bewb2nc5bgm3
title: Run the correctness runbook on the 0.5.0 stack and record it
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-10T21:49:20.613Z
updated_at: 2026-10-12T01:29:05.476Z
started_at: 2026-10-11T21:55:24.735Z
closed_at: 2026-10-12T01:29:05.475Z
close_reason: null
resolution: null
duplicate_of: null
---
Review E4 on PR #192 (jlevy/fdu#192). AGENTS.md and docs/project/guides/correctness-runbook.md require the runbook after any change to cache identity, serving, or reconciliation. The 0.5.0 stack changes reconciliation (push_directory_attributes in reconcile_target_inner; the directory upserts verify_with_directory_times adds to every watch intent) and how each route names its snapshot (Delivery::for_root, for_canonical_root), and extends the runbook's own age checks. Last recorded run is 0.4.0's. Run it at the top-of-stack head before tagging 0.5.0 (macOS, and Linux where a host is free) and record it under Last Recorded Run. This is the release-checklist run; it gates tagging, not merging #191/#192.

## Notes

Completed the installed-candidate correctness pass on d0862487939ea653351808d6143b4fe8e1f92351, tree identical to release preparation merge 33a3a890. Refusal 26/26 partial and withheld; served 26/26 cache_only/stale with nine analysis revalidated; cross-warm 72/72 exact with zero violations; negative controls caught all intended missing/partial-snapshot cases. macOS APFS, unprivileged; unsupported device/non-UTF8 fixture kinds explicitly recorded, no Linux/Windows runtime claim. Full tables and updated Last Recorded Run merged in PR #198 at 3d296c15, whose tree equals reviewed and make-check-tested abcb4e29. This closes the correctness-runbook record only: live Library peer QA fdu-wo7c, Linux performance fdu-4arl, and human terminal QA remain open before tagging.
