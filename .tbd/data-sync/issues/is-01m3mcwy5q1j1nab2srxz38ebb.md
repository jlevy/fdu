---
type: is
id: is-01m3mcwy5q1j1nab2srxz38ebb
title: Streamline the release process end to end and document it
kind: task
status: in_progress
priority: 1
version: 10
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
updated_at: 2026-09-28T18:03:19.161Z
started_at: 2026-09-28T16:26:30.261Z
---
User ask 2026-09-28: make the release process straightforward end to end and well documented going forward. The guide spells out 0.1.0 and asks the reader to substitute versions by hand; rewrite it version-parametric (VERSION, COMMIT variables), put a one-page checklist first, script the local maintainer steps where safe (rehearsal on a pinned commit, body check, tag verify, post-publish registry check), and fold in what cutting 0.2.0 taught. Keep irreversible writes behind the maintainer.

## Notes

2026-09-28 claude-code: PR #153 head 8783a58a. Folded in the live 0.2.0 run (part 2): agent go-ahead rule for tag/publish/announce, API approval route, manual tag verification reading exit status and Good line, stability results recorded beside procedures (#152 form; dated report optional), drift beads fdu-djz0/46eu/wxrq/mdop linked. Fix 4b0df53c: release-announced reports an unbuilt docs.rs page as wait/exit 3 (FAIL only on a failed build); verify-tag requires git's Good line. 60 maintainer tests; make release-test 133 OK; docs-format-check OK. Lead dogfooded release-body/published/announced/cleanup for 0.2.0. Children: fdu-vkbq, fdu-808x, fdu-02dw, fdu-brkf, fdu-bxra. ./target trashed.
