---
type: is
id: is-01m3kapbb3yh970n1vw0h973gw
title: Share the permission-bits opt-out between integration tests
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-28T06:19:33.091Z
updated_at: 2026-09-28T06:19:33.091Z
---
The FDU_TEST_ALLOW_NO_PERMISSION_BITS rule is now spelled in three places: crates/fdu-core/tests/state_transition_regressions.rs, crates/fdu-core/tests/partial_directory_queries.rs, and crates/fdu-core/src/test_support.rs (pub(crate), so integration tests cannot reach it). Move it to a tests/common/mod.rs helper. Found in the #138 senior review.
