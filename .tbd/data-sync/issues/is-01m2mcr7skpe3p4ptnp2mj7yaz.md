---
type: is
id: is-01m2mcr7skpe3p4ptnp2mj7yaz
title: "PR #67 delta D67-3: cache_status calls a paired sidecar orphaned"
kind: bug
status: open
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2mckkzdt9pgwx782pcr49r5
created_at: 2026-09-16T05:59:04.754Z
updated_at: 2026-09-16T05:59:07.561Z
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
