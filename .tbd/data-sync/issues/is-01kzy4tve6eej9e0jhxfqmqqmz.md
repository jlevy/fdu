---
type: is
id: is-01kzy4tve6eej9e0jhxfqmqqmz
title: "Address review: PR #8 — macOS stability and general correctness"
kind: task
status: closed
priority: 1
version: 14
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - review
  - correctness
  - macos
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
child_order_hints:
  - is-01kzy3ea5bks8mfpj1fabv5tm7
  - is-01kzy3eakncq1hs75wd35em3y8
  - is-01kzy3eb1181ps223nwn9rj5ws
  - is-01kzy3ebejg5cwxnm06fwwert5
  - is-01kzy4vn3zzvjv1xfjh382z2pe
  - is-01kzy4vncqfxrw2msxezgp0nsb
  - is-01kzy4vnnjsa5z7a53tx7fssve
  - is-01kzy5tnajrwx4070n0m2dym82
  - is-01kzy5tnkdtdvnxv2zcv4kthv8
  - is-01kzy5tp48xn18jsjqttbj5ac8
created_at: 2026-08-13T18:06:00.645Z
updated_at: 2026-08-23T02:11:33.033Z
closed_at: 2026-08-13T18:40:32.900Z
close_reason: "Complete: every PR #8 review finding has an explicit fixed, rebutted, or deferred disposition; APFS resource-fork parity is proven; portable differential coverage is integrated; make check and all 14 CI checks pass; PR description and formal review channel are current; branch is zero behind main with no conflicts."
---
Address every unresolved general correctness finding in the 2026-08-13 senior engineering review of PR #8. Fix and verify the macOS resource-fork metadata mismatch; disposition reconciliation attribution, privileged test portability, scheduler and reconciliation invariants, differential producer coverage, argument-count maintainability, and benchmark work-class labeling. Linux performance measurement and Linux-only optimization stay outside this stabilization bead and move to the Linux handoff epic. Publish a per-finding disposition map, update the PR description, run the full handoff gate, and merge only after CI and conflict state are clean.
