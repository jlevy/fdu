---
type: is
id: is-01kzg4d32s8s6g47686dpk8ddk
title: "Metabrowser integration: replace the Python walker and inventory hot path"
kind: feature
status: open
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels:
  - future
dependencies: []
parent_id: is-01kzm3v6nndedpwk414enwysv3
created_at: 2026-08-08T07:29:07.672Z
updated_at: 2026-08-23T07:33:24.380Z
---
The seam is already clean: metabrowser's walker yields a well-defined record stream, the inventory consumes it, and plugins consume classification and projections through a documented API. fdu slots in at the walker/inventory seam without disturbing the plugin boundary.

Replaces: the cold boot walk (~7,000 files/s, so 500k files takes ~70s), the aggregate maintenance, the recent/tree queries, and the gitignore evaluation (~1.5s parse on large roots). Leaves untouched: the SSE bus, projections, the plugin API, and classification-dependent views.

Also lifts the INVENTORY_MAX_FILES = 500_000 cap and the no-persistence problem — today every server start re-walks everything.

Open question, deliberately unresolved: does fdu::watch replace watch_backends.py outright (the clean end state — watchfiles wraps the same notify crate anyway, so events then never cross into Python), or does metabrowser keep its watcher and push paths through ingest_events() as unverified hints (the low-risk step that preserves its tuned NFS/FUSE fallbacks)? The delta contract supports both. What is open is sequencing: which ships first, and what the acceptance test for dropping the Python watcher looks like.

Coordinate with the metabrowser repository; this bead tracks the fdu side.

## Notes

2026-08-23: the fdu-side design now lives in plan-2026-08-23-fdu-interactive-client-integration.md, which resolves this bead's open watcher question (fdu watch where native backends serve the filesystem; foreign-watcher hints through scoped refresh, fdu-fh0k, elsewhere) and defines the acceptance test (cross-engine agreement fixture, fdu-vfyw). This bead remains the tracker for the metabrowser-coordination side.
