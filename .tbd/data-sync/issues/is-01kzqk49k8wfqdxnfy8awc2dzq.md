---
type: is
id: is-01kzqk49k8wfqdxnfy8awc2dzq
title: "PR #3 review R6: stop presenting mtime filters as exact sync"
kind: bug
status: open
priority: 1
version: 1
labels:
  - pr-review
dependencies: []
parent_id: is-01kzqk2ct4s2qjv9e2z17fvywr
created_at: 2026-08-11T05:01:09.095Z
updated_at: 2026-08-11T05:01:09.095Z
---
FDU-PR3-R6. docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md. An mtime watermark misses deletions, unchanged-mtime renames, and backdated edits. Specify it only as a modified-file candidate query; exact synchronization requires manifest diff or a complete durable op log, with deletion/rename/backdating/interruption tests. Review: https://github.com/jlevy/fdu/pull/3#issuecomment-5249058288.
