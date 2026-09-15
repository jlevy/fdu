---
type: is
id: is-01m2jzaxp5zhxav64j7wmvcjxc
title: "PR #61 review PR61-SMOKE-1: a changed fdu-core entry is misreported as 'the patch did not apply'"
kind: bug
status: closed
priority: 3
version: 4
labels:
  - release
dependencies: []
parent_id: is-01m2jzadk7w8m1xcsewzwg5wj1
created_at: 2026-09-15T16:45:22.497Z
updated_at: 2026-09-15T17:10:16.123Z
closed_at: 2026-09-15T17:10:16.122Z
close_reason: "c28e9e2: verify_relock distinguishes missing entry, unapplied patch, and changed fields; tests for each"
resolution: null
duplicate_of: null
---
PR #61 at eb89150, scripts/release/smoke_crate.py:78-81. verify_relock reports 'fdu-core is still resolved from crates.io: the patch did not apply' for any difference in the relocked fdu-core entry, including a changed dependency list with the source correctly dropped. The check fails correctly; the diagnosis names the wrong cause. Distinguish the two causes and add a test.
