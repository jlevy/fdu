---
type: is
id: is-01m4ghh5h0wmyx09t533pfm7zx
title: Size the partial-index activity pass's table by directories, not every arena slot
kind: task
status: open
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T14:37:01.855Z
updated_at: 2026-10-09T15:02:42.934Z
---
Review C9 on #191 (https://github.com/jlevy/fdu/pull/191#issuecomment-6082950211), deferred from the PR.

The per-report activity pass, which still serves an index that may hold an unlisted subtree (an opened root mid-discovery, an --allow-partial cold walk, a scan depth), sizes its table by every arena slot: crates/fdu-core/src/query/query_subtrees.rs `activity()` allocates `vec![None; index.slots()]`, 24 bytes a slot, file slots included (query_report.rs:1697-1712 chooses the route).

Numbers from review C:
- An opened million-entry root mid-discovery allocates and touches about 24 MB on every report or repaint.
- For unfiltered partial trees with a share floor this pass replaced `measure`, which builds a PathBuf and an ignore lookup per directory, so it is probably a speed-up there; for `--min-share 0` partial trees it is new work where no pass ran before.
- Neither route change has been measured.

Fix: index the table by directory ordinal (directories are a few percent of entries: 3,427 of 77,355 on rustup, 125,001 of ~1M on linux-balanced-1m), or let fdu-s88s (A7, a count of unlisted directories) remove the pass. Then record a mid-discovery report on a large opened root (opened-second-report variant mid-discovery) against the current head.
