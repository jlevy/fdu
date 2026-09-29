---
type: is
id: is-01m3mcx2yq517yvn7jbcxaw98m
title: Prepare the 0.2.1 release layer
kind: task
status: in_progress
priority: 1
version: 4
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
hold: null
hold_until: null
created_at: 2026-09-28T16:17:25.462Z
updated_at: 2026-09-29T01:52:04.244Z
started_at: 2026-09-29T00:30:47.649Z
---
Version bump, CHANGELOG [0.2.1], release notes, once the Linux verdicts and the .gitignore change have landed. Publishing stays with the maintainer.

## Notes

2026-09-29: release layer e889694c pushed (version 0.2.1, CHANGELOG, release notes, goldens/parity/test fixtures); make release-test 173 passed. Stability pass (make check, cross-lint, release-rehearse, semver-check, QA playbook, correctness runbook) running. Steps 3-5 need COMMIT on origin/main, SIGNING_KEY, gh.

2026-09-29: stack made conflict-free: origin/main merged into claude/perf-h159-recycle (f5a94da5; runbook id paragraph kept #150's H159 record), then into #155 (ee97bf33, tree identical to 672c2188). #150 marked ready, description updated to exp-190 accept. CI running on both heads.
