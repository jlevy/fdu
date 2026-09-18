---
type: is
id: is-01m2nh73s7pvk4n5k2szdzff9s
title: Move the release-body derivation and checks from release-process.md into scripts/release with a test
kind: task
status: closed
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-16T16:36:20.899Z
updated_at: 2026-09-18T03:07:32.185Z
closed_at: 2026-09-18T03:07:32.185Z
close_reason: "Implemented on PR #88: runbook and plan-spec accuracy rebased onto channel setup; GitHub release-body derivation is a checked-in script."
---
PR #64 fe45170 (review RN64-3): docs/project/guides/release-process.md step 1 of 'Tag the Release Commit' derives $RELEASE/notes.md from docs/project/release-notes/<version>.md (strip HTML comments with a Python one-liner, join lines with the pinned flowmark --width 0) and checks it: exactly one comment in the notes (the footer), a whitespace-only difference, and zero <br> from 'gh api markdown -f mode=gfm'. release-engineering-rules asks that release logic live in checked-in programs the test suite exercises; this is inline shell in a manual guide. Proposed: scripts/release/release_body.py (derive + the first two checks, version-parameterized) with a test in tests/release covering an unfilled placeholder, a comment inside a code span, and a table; the render check stays a documented gh call since it needs the network. Not a 0.1.0 blocker: the guide's commands were verified verbatim on PR #64's head.
