---
type: is
id: is-01m3mcwy5q1j1nab2srxz38ebb
title: Streamline the release process end to end and document it
kind: task
status: in_progress
priority: 1
version: 9
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
updated_at: 2026-09-28T17:26:48.199Z
started_at: 2026-09-28T16:26:30.261Z
---
User ask 2026-09-28: make the release process straightforward end to end and well documented going forward. The guide spells out 0.1.0 and asks the reader to substitute versions by hand; rewrite it version-parametric (VERSION, COMMIT variables), put a one-page checklist first, script the local maintainer steps where safe (rehearsal on a pinned commit, body check, tag verify, post-publish registry check), and fold in what cutting 0.2.0 taught. Keep irreversible writes behind the maintainer.

## Notes

2026-09-28 claude-code: draft PR https://github.com/jlevy/fdu/pull/153 (branch claude/release-process-streamline, commits 449b7d3a, 75ea4971, 009e4bdf). maintainer.py + make release-{preflight,candidate,body,verify-tag,published,announced,cleanup,audit}; 58 tests; guide rewritten around one checklist; AGENTS.md tbd --as note. Gates: make check exit 0, make cross-lint exit 0, docs-format-check exit 0 on macOS arm64. Live read-only smoke: 0.2.0 preflight/body/candidate --run 36450043255 pass; 0.1.0 verify-tag/published/audit/announced pass (docs.rs once timed out, reported FAIL correctly). Gaps filed as children: fdu-vkbq, fdu-808x, fdu-02dw, fdu-brkf, fdu-bxra. Waiting for the lead's 0.2.0 part-2 notes before marking ready.
