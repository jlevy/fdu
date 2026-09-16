---
type: is
id: is-01m2ngg3qhg18q7q6s3mavqps3
title: "PR #64 review RN64-4: 'Renamed and removed interfaces' is a development log of contracts no release published"
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
created_at: 2026-09-16T16:23:47.181Z
updated_at: 2026-09-16T16:34:19.787Z
started_at: 2026-09-16T16:25:03.758Z
closed_at: 2026-09-16T16:34:19.785Z
close_reason: "5eb497b: removed the 33-line rename list from CHANGELOG and its pointer from the notes; kept the cold-scan, cache-clear and gitignore-feature bullets plus one instruction to take released names; no single entry kept (each fails loudly naming the old spelling or is shipped behavior under Added; the silent glob case is stated there by 258949d)"
resolution: null
duplicate_of: null
---
CHANGELOG.md:233-265@d303dc1 and docs/project/release-notes/0.1.0.md:164-166@d303dc1. release-notes-guidelines: users see one behavior under its final name. Keep the two cache bullets and the gitignore build-feature bullet; remove the rename list, keeping any single entry a pre-release user would actually trip over, with the reason.
