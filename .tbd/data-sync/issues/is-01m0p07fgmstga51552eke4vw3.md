---
type: is
id: is-01m0p07fgmstga51552eke4vw3
title: "PR #42 R13: architecture doc says the golden corpus has 126 sessions"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m0p06qgww21s4fpdkj2bb6bf
created_at: 2026-08-23T00:27:17.908Z
updated_at: 2026-08-23T00:57:54.800Z
closed_at: 2026-08-23T00:57:54.800Z
close_reason: "Half fixed, half rebutted: the architecture doc did say 126 where the corpus has 129, and is corrected. The parity spec's 126 is correct -- parity compares 126 of 129 because run-parity declines three by name -- so changing it would have introduced an error. The doc now states both numbers and why they differ."
---
docs/project/architecture/fdu-surface-architecture.md:65. The corpus is 129; parity compares 126 because run-parity DECLINES 3. The parity spec 126s are correct and must NOT change.
