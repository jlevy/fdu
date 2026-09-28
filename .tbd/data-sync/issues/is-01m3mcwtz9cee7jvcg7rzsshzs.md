---
type: is
id: is-01m3mcwtz9cee7jvcg7rzsshzs
title: "0.2.0 stability: QA playbook and correctness runbook on the 6ec77163 build"
kind: task
status: in_progress
priority: 0
version: 3
delegate: claude-code
labels: []
dependencies:
  - type: blocks
    target: is-01m3mcwvbyhdpbhtakmadmqq0w
parent_id: is-01m3mcwtkbvr9kj5j2qwpyyd3j
hold: null
hold_until: null
created_at: 2026-09-28T16:17:17.288Z
updated_at: 2026-09-28T16:18:33.259Z
started_at: 2026-09-28T16:18:33.258Z
---
Before tagging: run tests/qa/cli-installed-e2e.qa.md against the installed 0.2.0 wheel built from 6ec77163, and docs/project/guides/correctness-runbook.md (required before tagging; the stack changed cache policy and watch reconciliation). CI and the full path-independence matrix already passed on 6ec77163.
