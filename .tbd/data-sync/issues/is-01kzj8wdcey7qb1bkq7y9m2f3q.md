---
type: is
id: is-01kzj8wdcey7qb1bkq7y9m2f3q
title: "PR #1 review R5: Make partial freshness explicit and non-durable by default"
kind: bug
status: closed
priority: 1
version: 5
labels:
  - pr-review
dependencies:
  - type: blocks
    target: is-01kzj8wf8380p0rf53pzemj4w7
  - type: blocks
    target: is-01kzg4bf862ajh8g2tmv5bznng
  - type: blocks
    target: is-01kzj8wgcxppt8qpvkzw907j0s
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:52.909Z
updated_at: 2026-08-09T04:06:08.971Z
closed_at: 2026-08-09T04:06:08.970Z
close_reason: Freshness/completeness and per-path errors are exposed in Rust, schema-v2 JSON, and Python; CLI partial results exit 2 unless allowed; incomplete indexes cannot overwrite snapshots. Rust, CLI integration, and installed-wheel tests pass.
---
PR #1 review R5. Files: crates/fdu/src/lib.rs, crates/fdu-py/src/lib.rs, crates/fdu/src/cli.rs, crates/fdu/src/main.rs. Expose completeness and error details on Rust, Python, and JSON surfaces; make CLI partial results nonzero unless allow-partial; never replace a complete snapshot with an incomplete result.
