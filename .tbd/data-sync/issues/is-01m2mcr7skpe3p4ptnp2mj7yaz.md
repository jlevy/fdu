---
type: is
id: is-01m2mcr7skpe3p4ptnp2mj7yaz
title: "PR #67 delta D67-3: cache_status calls a paired sidecar orphaned"
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m2mckkzdt9pgwx782pcr49r5
created_at: 2026-09-16T05:59:04.754Z
updated_at: 2026-09-16T06:31:35.708Z
closed_at: 2026-09-16T06:31:35.707Z
close_reason: "02b44ee: took the documentation option; cache_status says orphaned_content on a sidecar path means the file is a sidecar, not that no snapshot claims it, since CacheState has no state for a claimed sidecar and inventing one is a schema change no surface could reach"
resolution: null
duplicate_of: null
---
Delta review 5218953084 of PR #67, P3 (optional), REPRODUCED by the reviewer.

`crates/fdu-core/src/cache.rs:332-339,356-373@a5e2124`.

Public `fdu_core::cache_status(&sidecar)` on the sidecar of a *present* snapshot reports
`Leftover(OrphanedContent)`: `status_at` decides by name and magic and never asks whether
the snapshot is there; the pairing lives only in `list_caches`. Unreachable from the CLI
and from Python, which derive the path from a root.

Fix, either: check `is_snapshot_image(sidecar_snapshot_path(path))` in `status_at` for
`NameShape::Sidecar`, or say on `cache_status` that the sidecar state is a listing-only
meaning.
