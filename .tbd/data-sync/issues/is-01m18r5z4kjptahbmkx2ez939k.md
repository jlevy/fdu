---
type: is
id: is-01m18r5z4kjptahbmkx2ez939k
title: Control state is built for every scan, including roll-ups that never use it
kind: bug
status: open
priority: 0
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - scale
  - control-state
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-30T07:12:13.970Z
updated_at: 2026-08-30T07:12:13.970Z
---
Root cause of the control-table aborts.

crates/fdu/Cargo.toml:47 sets default = ["watch", "gitignore"], and crates/fdu-core/src/scan.rs calls read_control_op() at every walk site with no runtime gate - only the compile-time gitignore feature. So a plain 'fdu ~ -d 1 --sort size' opens, reads, parses, and retains every .gitignore in the tree, then dies on a budget for state the roll-up never consumes.

The control table exists to serve the opened-root inventory (MetaBrowser partitioning). A size roll-up needs none of it.

Fix direction: gate control observation on a runtime capability - build it when a consumer asks for ignore classification, not unconditionally. This alone makes 'fdu ~' work irrespective of the cap, and removes thousands of small-file reads from every scan.

Acceptance: a default CLI roll-up performs no control-file I/O and retains no control state; opened-root/inventory consumers still get exact control state; no-default-features build unaffected.
