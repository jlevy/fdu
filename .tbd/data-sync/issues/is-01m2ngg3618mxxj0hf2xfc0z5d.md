---
type: is
id: is-01m2ngg3618mxxj0hf2xfc0z5d
title: "PR #64 review RN64-3: release-process.md both freezes the notes before the tag and unwraps them after a draft release"
kind: bug
status: closed
priority: 2
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m2ngfd0y2yzwg2v10p2j601z
hold: null
hold_until: null
created_at: 2026-09-16T16:23:46.623Z
updated_at: 2026-09-16T16:34:20.432Z
started_at: 2026-09-16T16:25:03.439Z
closed_at: 2026-09-16T16:34:20.431Z
close_reason: "fe45170: one procedure in Tag the Release Commit step 1 before tagging: derive $RELEASE/notes.md (strip comments, flowmark --width 0), require exactly the footer comment, whitespace-only difference, and 0 <br> from the read-only gfm render; Announce uses the derived file. Verified: 97 <br> before, 0 after; no release created"
resolution: null
duplicate_of: null
---
docs/project/guides/release-process.md:342-347@d303dc1. The notes are 'as tagged' and final before the tag, yet the guide says to unwrap paragraphs after gh release create --verify-tag --draft. GitHub renders a single newline as <br> (POST /markdown mode=gfm), so the flowmark-wrapped file renders ragged. Fix: one executable procedure, verified with the read-only render API, no GitHub release created.
