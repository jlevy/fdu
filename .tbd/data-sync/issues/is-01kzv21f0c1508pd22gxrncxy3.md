---
type: is
id: is-01kzv21f0c1508pd22gxrncxy3
title: Amortize H12 reconciliation worker startup with larger waves (H56)
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzpvt22bex8ed6d155y014py
created_at: 2026-08-12T13:19:28.267Z
updated_at: 2026-08-12T13:21:30.919Z
closed_at: 2026-08-12T13:21:30.919Z
close_reason: Rejected and reverted in exp-031. At 60k, 4096-directory waves changed warm wall +1.64% (CI crossed zero) and component +13.24%; CPU/context-switch evidence did not support startup amortization. The 1024-directory wave remains.
---
Post-exp-030 profile attributes roughly 13% of 60k warm samples to scoped thread startup/waiting. Compare 4,096-directory waves with the accepted 1,024-directory H12 control, holding the four-worker cap and deferred-op bound constant. Pre-registered 60k gate: warm wall or reconciliation component at least 3% lower with CI below zero, no more than 5% RSS growth, exact oracle parity. The larger wave delays changed-tree delta publication by at most 4x in directory count but remains bounded. Confirm at 720k only if the 60k gate passes; otherwise revert.
