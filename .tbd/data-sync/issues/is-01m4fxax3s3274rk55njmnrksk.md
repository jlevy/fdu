---
type: is
id: is-01m4fxax3s3274rk55njmnrksk
title: "Roots: command line PATH..., Python report(paths), parity shim"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies:
  - type: blocks
    target: is-01m4fxay0x5wxrjarhc892gfhp
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T08:44:05.109Z
updated_at: 2026-10-09T09:06:03.856Z
---
CLI: PATH... (usage fdu [OPTIONS] <PATH>...); --watch, --cache-status, --cache-clear refuse a second PATH (usage error naming the limit); perf: line sums and names tiers when roots differ; text prints label/path. Python: fdu.report takes a path or a sequence (test str and os.PathLike first; root= keyword still works); Report.roots tuple or None; Report.root None for several; rows gain root; stubs. Parity shim accepts several positionals.
