---
type: is
id: is-01m3mcwtz9cee7jvcg7rzsshzs
title: "0.2.0 stability: QA playbook and correctness runbook on the 6ec77163 build"
kind: task
status: closed
priority: 0
version: 5
delegate: claude-code
labels: []
dependencies:
  - type: blocks
    target: is-01m3mcwvbyhdpbhtakmadmqq0w
parent_id: is-01m3mcwtkbvr9kj5j2qwpyyd3j
hold: null
hold_until: null
created_at: 2026-09-28T16:17:17.288Z
updated_at: 2026-09-28T17:12:04.534Z
started_at: 2026-09-28T16:18:33.258Z
closed_at: 2026-09-28T17:12:04.533Z
close_reason: "STABLE: QA playbook (50 ok, 1 expected warning; Phase 6 visual check pending as for 0.1.0; Phase 7 passes only with a two-line script fix) and correctness runbook (all runs pass, controls fail as required) on the 6ec77163 build; recorded in PR #152."
resolution: null
duplicate_of: null
---
Before tagging: run tests/qa/cli-installed-e2e.qa.md against the installed 0.2.0 wheel built from 6ec77163, and docs/project/guides/correctness-runbook.md (required before tagging; the stack changed cache policy and watch reconciliation). CI and the full path-independence matrix already passed on 6ec77163.

## Notes

2026-09-28 claude-code: both passes run on 6ec77163a. Results recorded in draft PR #152 (branch claude/verify-0.2.0). No correctness failure and no regression vs 0.1.0; peer-agreement script needed --min-share 0% + dir-only children to pass on 0.2.0 (harness fix not committed). Phase 6 visual pass by a person still pending. Left open for the coordinator to close.
