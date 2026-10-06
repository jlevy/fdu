---
type: is
id: is-01m4950mfg8vm0n2r9s85hjnh0
title: Restructure README per senior review R (R1-R30)
kind: task
status: in_progress
priority: 2
version: 4
delegate: claude-code@spud10.local
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-06T17:43:36.174Z
updated_at: 2026-10-06T18:22:05.135Z
started_at: 2026-10-06T17:43:39.096Z
---
Apply the senior README review taken at origin/main 55d66863: 30 findings R1-R30, the proposed ~430-line outline, and cross-surface consistency (tagline, speed sentence) in crate READMEs, lib.rs, docs index, --help/--docs, SKILL.md, goldens. User decisions: R1 'Cut 0.4.0 soon' (no version note), scope 'Full restructure'. Branch claude/readme-senior-review, draft PR.

## Notes

Draft PR https://github.com/jlevy/fdu/pull/183, head 8440dfa4; CI 20/20 green. Local make check blocked by a full /Volumes/spud-ext1; targeted checks passed on internal scratch. Awaiting review and merge.
