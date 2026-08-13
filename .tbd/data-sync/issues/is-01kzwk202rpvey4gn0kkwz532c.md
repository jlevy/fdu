---
type: is
id: is-01kzwk202rpvey4gn0kkwz532c
title: "H57: Revisit live 1M-tree worker depth after BFS and CLI integration"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01kzw3te81j66eehy48rx2djv5
created_at: 2026-08-13T03:36:05.975Z
updated_at: 2026-08-13T04:02:31.723Z
closed_at: 2026-08-13T04:02:31.722Z
close_reason: Rejected by exp-036; automatic worker policy remains best measured resource/complexity tradeoff.
---
Record the completed post-composable-CLI 1M-entry automatic/8/12/16-worker sweep. Decide against the paired 3% rule, preserving the result as exp-036 so diskus/gdu-style over-subscription is not repeatedly retried.

## Notes

Resolved by exp-036 on the 1,007,659-entry live workspace: automatic/six workers were neutral; eight improved wall only 1.30% while total CPU rose 33.5%; 12 and 16 regressed wall 2.46% and 10.65%. No code retained. Source review of dust/gdu/diskus supplies no new APFS concurrency mechanism.
