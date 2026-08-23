---
type: is
id: is-01m0nzxxqj44qqhsv0qajgzt8v
title: "PR #42 review R11: active fsevents spec still names crates/fdu/src for engine modules"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0nzwr6pcg3fnj1445zpc81z
created_at: 2026-08-23T00:22:04.786Z
updated_at: 2026-08-23T00:39:56.524Z
closed_at: 2026-08-23T00:39:56.524Z
close_reason: Fixed. The four engine paths in the fsevents spec point at crates/fdu-core/src/; cli.rs was already correct.
---
docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md:351,354,361,363 point implementers at journal/, scan.rs and snapshot.rs under crates/fdu. Line 365 (cli.rs) is still correct.
