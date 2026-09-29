---
type: is
id: is-01m3mcx2yq517yvn7jbcxaw98m
title: Prepare the 0.2.1 release layer
kind: task
status: closed
priority: 1
version: 6
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
hold: null
hold_until: null
created_at: 2026-09-28T16:17:25.462Z
updated_at: 2026-09-29T02:01:12.225Z
started_at: 2026-09-29T00:30:47.649Z
closed_at: 2026-09-29T02:01:12.224Z
close_reason: "Release layer merged in #155 (b10fe7b3). Checklist steps 3-7 are the maintainer's (gh, signing key, tag, publish)."
resolution: null
duplicate_of: null
---
Version bump, CHANGELOG [0.2.1], release notes, once the Linux verdicts and the .gitignore change have landed. Publishing stays with the maintainer.

## Notes

2026-09-29: release layer e889694c pushed (version 0.2.1, CHANGELOG, release notes, goldens/parity/test fixtures); make release-test 173 passed. Stability pass (make check, cross-lint, release-rehearse, semver-check, QA playbook, correctness runbook) running. Steps 3-5 need COMMIT on origin/main, SIGNING_KEY, gh.

2026-09-29: stack made conflict-free: origin/main merged into claude/perf-h159-recycle (f5a94da5; runbook id paragraph kept #150's H159 record), then into #155 (ee97bf33, tree identical to 672c2188). #150 marked ready, description updated to exp-190 accept. CI running on both heads.

2026-09-29 02:00: #150 and #155 merged; release COMMIT = b10fe7b3 (tree identical to 672c2188/ee97bf33; #155 CI run 1209 green on ee97bf33; main CI running). Offline release_body.py derivation from b10fe7b3's notes passes; all 7 v0.2.1 links resolve. Steps 3-5 (release-preflight, release-candidate, release-body) need gh + SIGNING_KEY: maintainer's machine.
