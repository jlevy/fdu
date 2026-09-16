---
type: is
id: is-01m2ngg5cmp60rnnqtfy82qnpf
title: "PR #64 review RN64-8: portable-path round trip does not hold for Report projection rows"
kind: bug
status: closed
priority: 3
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m2ngfd0y2yzwg2v10p2j601z
hold: null
hold_until: null
created_at: 2026-09-16T16:23:48.883Z
updated_at: 2026-09-16T16:34:15.934Z
started_at: 2026-09-16T16:25:04.952Z
closed_at: 2026-09-16T16:34:15.933Z
close_reason: "258949d: narrowed; portable_path from Lookup, Tree or Flat rows passes back unchanged, Report projection rows carry the native path so a name with % or non-UTF-8 bytes does not (fdu-v9uf stays open for the API gap)"
resolution: null
duplicate_of: null
---
CHANGELOG.md:196-197@d303dc1 says a path from a page passes back as a filter. Report projection rows carry only the native path (open bead fdu-v9uf). Narrow the claim or list the limitation citing fdu-v9uf.
