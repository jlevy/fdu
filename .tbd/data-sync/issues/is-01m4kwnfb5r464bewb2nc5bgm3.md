---
type: is
id: is-01m4kwnfb5r464bewb2nc5bgm3
title: Run the correctness runbook on the 0.5.0 stack and record it
kind: task
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-10T21:49:20.613Z
updated_at: 2026-10-11T22:05:27.741Z
started_at: 2026-10-11T21:55:24.735Z
---
Review E4 on PR #192 (jlevy/fdu#192). AGENTS.md and docs/project/guides/correctness-runbook.md require the runbook after any change to cache identity, serving, or reconciliation. The 0.5.0 stack changes reconciliation (push_directory_attributes in reconcile_target_inner; the directory upserts verify_with_directory_times adds to every watch intent) and how each route names its snapshot (Delivery::for_root, for_canonical_root), and extends the runbook's own age checks. Last recorded run is 0.4.0's. Run it at the top-of-stack head before tagging 0.5.0 (macOS, and Linux where a host is free) and record it under Last Recorded Run. This is the release-checklist run; it gates tagging, not merging #191/#192.

## Notes

2026-10-11 preliminary stack verification at ce8efce24ae6d3184bfc568566a0a31ba978bd32 using the preserved make-check-tested binary: macOS APFS external scratch, non-root. Refusal26/26 partial and withheld, served26/26 cache_only/stale with9 analysis revalidated, cross-warm72/72 with17 held to serving; zero mismatch/mechanism/reference failures. Mutation controls caught26 missing snapshots,17 same-analyzer pairs,26 partial-stored cases; all exited1 as expected. Kinds14 refusal/13 served; platform refused chardev/blockdev/non-UTF8. Durable logs retained with the stabilization artifacts. Final installed-candidate stability pass and committed record still pending; this preliminary success does not close the pre-tag gate.
