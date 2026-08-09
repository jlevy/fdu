---
type: is
id: is-01kzj8wfq2ft9scx1gw0sf347p
title: "PR #1 review C6: Canonicalize direct scan roots consistently"
kind: bug
status: closed
priority: 3
version: 2
labels:
  - pr-review
dependencies: []
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:55.298Z
updated_at: 2026-08-09T03:54:45.886Z
closed_at: 2026-08-09T03:54:45.885Z
close_reason: scan_into_index now canonicalizes roots exactly like open; direct scan identity and existing Python realpath smoke coverage verify the contract.
---
PR #1 Cursor thread C6: https://github.com/jlevy/fdu/pull/1#discussion_r3742319336. Files: crates/fdu/src/lib.rs and crates/fdu-py/tests/smoke.py. scan_into_index and open must use the same canonical root identity so macOS realpath aliases do not break cache keys or wheel smoke tests.
