---
type: is
id: is-01m2ngg0501h41kdrb0cr01c3d
title: "PR #64 review RN64-1: Memory limitation claims fdu peaks above dust; the release candidate measured the opposite"
kind: bug
status: in_progress
priority: 1
version: 2
delegate: claude-code@spud10
labels:
  - release
dependencies: []
parent_id: is-01m2ngfd0y2yzwg2v10p2j601z
hold: null
hold_until: null
created_at: 2026-09-16T16:23:43.519Z
updated_at: 2026-09-16T16:25:02.813Z
started_at: 2026-09-16T16:25:02.812Z
---
docs/project/release-notes/0.1.0.md:113-115@d303dc1 and CHANGELOG.md:269-271@d303dc1 say peak memory exceeds dust's. PR #68's report-2026-09-16-fdu-live-tool-comparison.md measured dust 640.7 MiB against fdu 285.4 MiB on the 1,000,001-entry balanced tree; the only support was fdu-syyl's ad-hoc, not-claim-grade runs. Fix: restate from measured numbers with provenance (fdu ~285 MiB vs dumac 29 MiB, dua 21 MiB, dust 641 MiB; --no-gitignore --view summary 4.876 s at 15.0 MiB vs default summary 4.942 s at 285.7 MiB; fdu-if7o as a range with its mechanism). No pdu figure.
