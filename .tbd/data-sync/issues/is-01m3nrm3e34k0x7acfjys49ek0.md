---
type: is
id: is-01m3nrm3e34k0x7acfjys49ek0
title: "Publish 0.2.1 from c1644575: tag, publish, announce, check, clean up (checklist steps 6-11)"
kind: task
status: in_progress
priority: 1
version: 2
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
hold: null
hold_until: null
created_at: 2026-09-29T05:01:28.387Z
updated_at: 2026-09-29T05:01:30.120Z
started_at: 2026-09-29T05:01:30.119Z
---
Release commit c16445757ce8 (merge of #156; tree = 1568ab6c). Done: preflight 12/12 ok, rehearsal run 36522734549 green (local artifact download blocked by env network policy on productionresultssa1.blob.core.windows.net), release-body ok. Remaining: step 6 tag via gh api (unsigned annotated, allowed since #156), release-verify-tag; step 7 gh workflow run release.yml --ref v0.2.1 -f publish=true and approve the release environment; step 8 release-published (needs the blob host or a registry-based verification); step 9 gh release create with assets; step 10 release-announced; step 11 release-cleanup (delete release/v0.2.1; the git proxy no-ops deletions, use gh api DELETE). gh runs on the direct channel with NO_PROXY for api.github.com only (git must stay on the proxy).
