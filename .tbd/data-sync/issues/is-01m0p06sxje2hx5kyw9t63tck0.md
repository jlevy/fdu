---
type: is
id: is-01m0p06sxje2hx5kyw9t63tck0
title: "PR #42 R5: two live docs link to crates/fdu/src/counters.rs"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m0p06qgww21s4fpdkj2bb6bf
created_at: 2026-08-23T00:26:55.794Z
updated_at: 2026-08-23T00:57:54.134Z
closed_at: 2026-08-23T00:57:54.134Z
close_reason: "Fixed in 4e34ce3; addressed the review on PR #42, verified through make check and make release-rehearse."
---
AGENTS.md:160 and docs/project/guides/performance-instrumentation-playbook.md:21. Module is now crates/fdu-core/src/counters.rs and the path is fdu_core::counters.
