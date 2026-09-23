---
type: is
id: is-01m37nx80t02jdgc65nqdt1tqa
title: Real-terminal smoke test and manual terminal QA for progress
kind: task
status: in_progress
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
labels:
  - cli
  - ux
dependencies: []
parent_id: is-01m2nsj4zgw7r9j3d1rphnmwf2
created_at: 2026-09-23T17:44:40.215Z
updated_at: 2026-09-23T23:46:17.445Z
---
Python pty smoke test (Unix only, skipped on Windows): run the built binary with TERM set and CI removed, under a timeout; assert a drawn frame, a clean final line, and death by SIGINT after an interrupt. Add a terminal phase to tests/qa/cli-installed-e2e.qa.md: large tree shows the indicator, small tree none, redirected stderr none, Ctrl-C leaves a clean prompt, resize shrinks the frame; a leftover line fails the check.
