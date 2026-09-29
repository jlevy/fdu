---
type: is
id: is-01m3q1fwrd3v6ry34e6abmzd7k
title: "Document the PR stack and experiment branches: merge-ready vs exploratory"
kind: task
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
hold: null
hold_until: null
created_at: 2026-09-29T16:55:42.092Z
updated_at: 2026-09-29T17:41:01.216Z
started_at: 2026-09-29T16:55:47.734Z
---
Open drafts: #157 (0.2.2 handoff, docs), #158 (evidence report revision, docs, stacked on #157), #161 (the overnight round, contains both via e5a71c8a). Branches without PRs pushed 2026-09-29: perf/h169-linux-reader (deferred public diagnostics fields, 0.3.0, fdu-q7hf) and perf/h181-h182 (rejected, exp-192). State for each: merge-ready (after independent review) vs exploratory/later release vs rejected-kept-for-record, and the merge order #157 -> #158 (retarget to main) -> #161. Status notes added to #157 and #158 descriptions; #161 gets a map section.

## Notes

Done: status notes on #157 and #158; #161 re-stacked on #158's branch (main merged into #157 55b59715, #157 into #158 aea39460, #158 into #161 e8225b59, base retargeted) and a PRs-and-branches section in #161. Pending: the README/docs layer's draft PR once its branch is pushed.
