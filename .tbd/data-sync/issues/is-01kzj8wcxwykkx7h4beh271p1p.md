---
type: is
id: is-01kzj8wcxwykkx7h4beh271p1p
title: "PR #1 review R3: Use lossless path identity and reversible snapshot names"
kind: bug
status: closed
priority: 1
version: 6
labels:
  - pr-review
dependencies:
  - type: blocks
    target: is-01kzj8wd52dh1pxg4834fb2q0v
  - type: blocks
    target: is-01kzj8wdv4jsmw5msz246hqbbc
  - type: blocks
    target: is-01kzj8wfq2ft9scx1gw0sf347p
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:52.443Z
updated_at: 2026-08-09T03:37:29.098Z
closed_at: 2026-08-09T03:37:29.097Z
close_reason: Fixed by storing OsString names and keys, lossless path normalization, text-only classification, platform-tagged reversible Unix-byte/Windows-wide snapshot encoding, and non-UTF-8 identity and round-trip tests.
---
PR #1 review R3. Files: crates/fdu/src/index.rs, crates/fdu/src/scan.rs, crates/fdu/src/snapshot.rs. Replace String identity and lossy conversion with platform-lossless names. Keep display and classification text separate. Round-trip distinct non-UTF-8 Unix names through index and snapshot.
