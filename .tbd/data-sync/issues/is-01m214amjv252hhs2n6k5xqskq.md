---
type: is
id: is-01m214amjv252hhs2n6k5xqskq
title: Audit disk usage and stage unused local worktree artifacts
kind: chore
status: closed
priority: 1
version: 3
labels: []
dependencies: []
created_at: 2026-09-08T18:26:16.259Z
updated_at: 2026-09-08T18:42:28.276Z
closed_at: 2026-09-08T18:42:28.274Z
close_reason: Audited local worktrees and caches; staged 5 inactive Cargo build directories, 42 Python virtual environments, and 24 node_modules directories using trash only. All 71 original paths are absent and match the native Trash listing. Reported staged disk usage is 28.49 GiB; physical free space is tracked separately because Trash remains unemptied. Preserved all source checkouts, unique changes, research data, active processes, runtimes, and agent history. Exact local path manifest retained outside source control.
resolution: null
duplicate_of: null
---
Use the reclaim-macos-disk-space skill to audit local worktrees and caches, preserve unique changes and active processes, and stage only reproducible data in Trash. Record exact staged paths outside source control and verify filesystem free space. No product source changes.
