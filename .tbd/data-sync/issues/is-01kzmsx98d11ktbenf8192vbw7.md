---
type: is
id: is-01kzmsx98d11ktbenf8192vbw7
title: Upgrade golden harness to tryscript 0.2.0
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/done/plan-2026-08-09-fdu-cli-golden-tests.md
labels:
  - golden-tests
dependencies: []
created_at: 2026-08-10T03:01:56.108Z
updated_at: 2026-08-10T03:10:27.271Z
closed_at: 2026-08-10T03:10:27.270Z
close_reason: Upgraded the exact lock to tryscript 0.2.0, adopted its released stderr and updater fixes, pinned the audited esbuild graph, documented the maintenance state, and passed clean-install, ordinary, update-mode, supply-chain, audit, and full make check validation.
---
Adopt the exact-pinned first-party tryscript 0.2.0 release under the maintainer-approved supply-chain exception; review the new reference and changelog, simplify fdu's sessions where the new behavior removes workarounds, and prove the complete CLI golden contract plus update mode end to end before returning to the performance workstream.
