---
type: is
id: is-01kzsa5f4bfby8rvnz5w7yd57t
title: "PR#6 D1: journal-scoped revalidation is not provably sound after the spike's silent-omission finding"
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:02:59.210Z
updated_at: 2026-08-11T21:02:59.210Z
---
plan-2026-08-10-fdu-fsevents-scoped-revalidation.md. Pick exact/verified vs risk-bounded/approximate contract and name it consistently; JournalConfirmed overclaims. Rerun Phase 0 with a genuine pre-scan cursor incl UUID transition, retention, remount, clock change, non-root permissions. High.
