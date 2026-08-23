---
type: is
id: is-01m0p06r0jdaqfdzy7jdjazx83
title: "PR #42 R1: moved stack-safety test filters on the old path and runs zero tests"
kind: bug
status: closed
priority: 0
version: 2
labels: []
dependencies: []
parent_id: is-01m0p06qgww21s4fpdkj2bb6bf
created_at: 2026-08-23T00:26:53.842Z
updated_at: 2026-08-23T00:57:54.108Z
closed_at: 2026-08-23T00:57:54.107Z
close_reason: "Fixed in 4e34ce3; addressed the review on PR #42, verified through make check and make release-rehearse."
---
crates/fdu-core/src/report_format.rs:2184 filters --exact cli::tests::deep_rendering_is_stack_safe after the test moved to report_format::tests. libtest runs 0 tests, exits 0, parent assert passes vacuously.
