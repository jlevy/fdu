---
type: is
id: is-01m2ngg4ncjsp9ty9v6f5513ad
title: "PR #64 review RN64-6: clear_cache returns bool; only clear_all_caches returns ClearSummary"
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
created_at: 2026-09-16T16:23:48.139Z
updated_at: 2026-09-16T16:34:15.255Z
started_at: 2026-09-16T16:25:04.350Z
closed_at: 2026-09-16T16:34:15.254Z
close_reason: "258949d: clear_cache returns whether it removed a snapshot; only clear_all_caches returns ClearSummary (snapshots removed, leftovers reclaimed)"
resolution: null
duplicate_of: null
---
CHANGELOG.md:177-178@d303dc1. crates/fdu-core/src/cache.rs:526,558; crates/fdu-py/python/fdu/_api.py:560,565.
