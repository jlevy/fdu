---
type: is
id: is-01kztxsn4qpkq1y652qtwb6vta
title: Reuse macOS bulk metadata during full reconciliation (H53)
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md
labels: []
dependencies: []
parent_id: is-01kzpvt1vamkqp8fffnpwhd93v
created_at: 2026-08-12T12:05:18.102Z
updated_at: 2026-08-12T12:05:18.102Z
---
Profile current full warm revalidation after exp-025, then test reusing the exp-022 bounds-audited getattrlistbulk reader for per-directory child expectations. Preserve exact reconcile semantics, mutation/error handling, mount/firmlink fallback, non-macOS behavior, and FSEvents composability. Pre-register primary signal: warm-revalidate wall and component down at least 3%, with system CPU down and exact oracle parity; measure current code at 60k and confirm at 720k if scale-sensitive.
