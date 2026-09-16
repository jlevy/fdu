---
type: is
id: is-01m2ks0eh255wymkmwdwxgc6dz
title: "PR #67 review PR67-4: corrupt-equals-absent text and the unreadable label are stale"
kind: bug
status: closed
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2krzt6endqrw5gq2phas5g6
created_at: 2026-09-16T00:14:02.273Z
updated_at: 2026-09-16T03:49:57.172Z
closed_at: 2026-09-16T03:49:57.171Z
close_reason: "f39b701: the corrupt-equals-absent bullet in cache-design.md is scoped to loading and says what status and clearing do instead; the human label is now 'unreadable by this build', which covers a snapshot whose paths another OS encoded as well as a truncated one."
resolution: null
duplicate_of: null
---
docs/project/guides/cache-design.md:40-43@816fcf7; crates/fdu-core/src/cache.rs:107-109; report_format.rs:1536. The bullet still says corrupt equals absent at any entry point (loading, status, or clearing), which is now true only of loading. The human label 'truncated or unreadable' also covers a snapshot whose paths another OS encoded. DECISION (user): scope the bullet to loading, and reword the label so it covers the encoding case too.
