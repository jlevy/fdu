---
type: is
id: is-01m2pj0q04xsqtjqj2v62vhhq5
title: GitHub Release job with scripted notes
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
delegate: claude-code@spud10
labels:
  - release
dependencies: []
parent_id: is-01m2pj0jfrvm78rpmv6rwc2ktv
hold: null
hold_until: null
created_at: 2026-09-17T02:09:34.211Z
updated_at: 2026-09-29T06:30:21.924Z
started_at: 2026-09-29T06:23:04.513Z
closed_at: 2026-09-29T06:30:21.924Z
close_reason: "Implemented in https://github.com/jlevy/fdu/pull/160 (7970ad6c): Actions derives committed notes and publishes a fully verified GitHub release after registry publication; safe draft retries, documented recovery, no-download rehearsal, and next-release evidence checklist. Independent agent review posted with no outstanding findings. Full make check, 187 release tests, and all 19 PR CI checks passed. Existing v0.2.1 release verified as unchanged no-op; first automatic publication will be recorded with the next release after merge."
resolution: null
duplicate_of: null
---
The only job with contents: write; derive the body from the release notes as the runbook does by hand.
