---
type: is
id: is-01m3mcwy5q1j1nab2srxz38ebb
title: Streamline the release process end to end and document it
kind: task
status: in_progress
priority: 1
version: 8
delegate: claude-code
labels: []
dependencies: []
child_order_hints:
  - is-01m3mfpqhjhwv9redfnv8e87ya
  - is-01m3mfpqz3cd8w5f9289t6yejb
  - is-01m3mfprbtj5qnhxjj7vwvttav
  - is-01m3mfprr9bh9nrfg8ehwq5ez6
  - is-01m3mfps518xc4rmbrc5zsh8es
hold: null
hold_until: null
created_at: 2026-09-28T16:17:20.566Z
updated_at: 2026-09-28T17:06:24.544Z
started_at: 2026-09-28T16:26:30.261Z
---
User ask 2026-09-28: make the release process straightforward end to end and well documented going forward. The guide spells out 0.1.0 and asks the reader to substitute versions by hand; rewrite it version-parametric (VERSION, COMMIT variables), put a one-page checklist first, script the local maintainer steps where safe (rehearsal on a pinned commit, body check, tag verify, post-publish registry check), and fold in what cutting 0.2.0 taught. Keep irreversible writes behind the maintainer.

## Notes

2026-09-28 claude-code: branch claude/release-process-streamline. Added scripts/release/maintainer.py (steps: preflight, candidate, body, verify-tag, published, announced, cleanup; make release-<step>), 55 tests in tests/release/test_maintainer.py (scripted host refuses tag push, publish dispatch, release create, registry writes; one real-git SSH signing test). Smoked read-only against live data: preflight/body/candidate --run 36450043255 for 0.2.0 all pass; verify-tag/published/announced pass for 0.1.0. Next: rewrite release-process.md around a one-page checklist, AGENTS.md tbd --as note.
