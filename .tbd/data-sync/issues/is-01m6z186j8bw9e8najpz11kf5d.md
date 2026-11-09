---
type: is
id: is-01m6z186j8bw9e8najpz11kf5d
title: Say when a directory-admitting time window selected every file
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-09T02:12:30.920Z
updated_at: 2026-11-09T02:12:30.920Z
---
A time window without `--kind file` admits a directory whenever anything it counts falls
inside the window, and with it every file beneath. When that admits every file, the
selection coverage note (`note: selected N of M files, …`) is silent, because nothing
was narrowed, and the root reads 100% as "written since T". This is trap 1 of the
recent-growth plan at its most misleading (review D14 on jlevy/fdu#209).

Two candidate fixes:

- Keep the note whenever the selection has a time bound: `note: selected all 2 files,
  98 KiB`. No surface names a flag, so no parity class is needed, but every golden
  session with a time bound that admits everything gains a line.
- A tip naming the file kind when a time window admitted a directory:
  `tip: a directory matches a time window when anything in it does; add --kind file`.
  The flag is spelled per surface (`--kind file`, `kinds=[EntryKind.FILE]`), so this
  needs an `AxisNames` entry and a parity class for the surface-label deviation.

Decide which, implement it in `query_report.rs` beside `SelectionCoverage::note`, and
regenerate the goldens and parity.
