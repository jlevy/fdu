---
type: is
id: is-01m3mcwy5q1j1nab2srxz38ebb
title: Streamline the release process end to end and document it
kind: task
status: closed
priority: 1
version: 12
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
updated_at: 2026-09-28T20:25:06.799Z
started_at: 2026-09-28T16:26:30.261Z
closed_at: 2026-09-28T20:25:06.797Z
close_reason: "Merged: #153 (45c7c577) rewrote the release guide as a version-parametric checklist with make release-* helpers (144 release tests), reviewed and fixed; dogfooded on 0.2.0. Machinery gaps remain as children: fdu-vkbq, fdu-808x, fdu-02dw, fdu-brkf, fdu-bxra."
resolution: null
duplicate_of: null
---
User ask 2026-09-28: make the release process straightforward end to end and well documented going forward. The guide spells out 0.1.0 and asks the reader to substitute versions by hand; rewrite it version-parametric (VERSION, COMMIT variables), put a one-page checklist first, script the local maintainer steps where safe (rehearsal on a pinned commit, body check, tag verify, post-publish registry check), and fold in what cutting 0.2.0 taught. Keep irreversible writes behind the maintainer.

## Notes

2026-09-28 claude-code: PR #153 head 0a2c0b57. Review fixes e2280c16 (by-hand published mode, verify-tag checks Cargo versions, resolve() binds state for every step, --previous compare base, workflow/run-name/job-name contract test + verify_run workflowName/title, attach policed, lease on fallback push, cleanup --abandon <commit>, unreadable secrets FAIL, announced-before-published FAIL line, candidate on-main before pin, notes refuse any non-tag ref, PVR enable command, trims). 0a2c0b57 drops the AGENTS.md claimer-name note at the user's direction. make release-test 144 OK (71 maintainer tests; 11 fail on the pre-review helper). Gate rerun in progress after the lead killed a hung deep_rendering_is_stack_safe child in the first attempt.
