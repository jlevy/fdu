---
type: is
id: is-01m2mcq7mmy20q1vpn2654jac7
title: "PR #67 delta D67-1: a snapshot image under a sidecar-shaped name is listed as a snapshot and cleared"
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m2mckkzdt9pgwx782pcr49r5
created_at: 2026-09-16T05:58:31.827Z
updated_at: 2026-09-16T06:31:27.503Z
closed_at: 2026-09-16T06:31:27.502Z
close_reason: "02b44ee: status_at answers a leftover shape at the leftover gate and never falls through to snapshot::identify, so a snapshot image under a sidecar-shaped name is unrecognized and survives a clear; both cases pinned in the leftover test, red first"
resolution: null
duplicate_of: null
---
Delta review 5218953084 of PR #67, P2, REPRODUCED by the reviewer.

`crates/fdu-core/src/cache.rs:356-379,453@a5e2124`; `docs/project/guides/cache-design.md:69,91-92`.

`status_at`'s leftover match returns `None` when a leftover shape's magic check fails, and
control falls through to `snapshot::identify`, which classifies by the snapshot magic
alone. For the two sidecar shapes (`{16hex}.fdu.content` and
`.{16hex}.fdu.content.tmp.*`) a file holding the snapshot magic is then listed `current`
or `stale`, and `clear_all_caches` removes it in the snapshot phase through `clear_cache`,
which applies no age gate.

Breaks the stated invariant: a listing counts a file as a snapshot only when it has both
the snapshot's name pattern and the magic. Either alone proves nothing.

Fix: when a leftover shape's magic check fails, return `unrecognized` rather than falling
through, so only `NameShape::Snapshot` and `NameShape::Other` reach `identify`. Pin both
reproduced cases in the leftover test.
