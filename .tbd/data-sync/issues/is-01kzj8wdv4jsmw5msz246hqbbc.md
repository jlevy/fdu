---
type: is
id: is-01kzj8wdv4jsmw5msz246hqbbc
title: "PR #1 review R7: Stream and bound untrusted snapshot parsing"
kind: bug
status: closed
priority: 1
version: 3
labels:
  - pr-review
dependencies: []
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:53.379Z
updated_at: 2026-08-09T03:42:35.975Z
closed_at: 2026-08-09T03:42:35.974Z
close_reason: Fixed with metadata-gated streaming snapshot load, explicit total/path/record limits, out-of-band trailer validation, bounded incremental parsing, one-record baseline rebuilds, and sparse-file and oversized-path regressions.
---
PR #1 review R7. File: crates/fdu/src/snapshot.rs. Remove whole-file fs::read from load. Validate metadata, header, record counts, component lengths, and total size before allocations; parse through a bounded reader and test oversized or truncated input.
