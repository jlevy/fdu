---
type: is
id: is-01m2h6q5ce9b0c0ccwehw4dxkb
title: "Correct Attrs::dev and Fingerprint::dev rustdoc: a device number is not a volume identity"
kind: task
status: open
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-15T00:15:54.757Z
updated_at: 2026-09-15T00:16:08.371Z
---
Found addressing PR #55 delta review 5204152578 (PR55-ID-1). At dda7e6a, crates/fdu-core/src/engine_contract.rs:128 documents Fingerprint::dev as 'Platform device or volume identity component', and :93 documents Attrs::dev as 'Device number. Zero where unavailable.' The value is st_dev on Unix (scan.rs:5075, attrs_from) and 0 on Windows (scan.rs:5097). st_dev is renumbered across reboots and remounts, so calling it a volume identity invites a durable consumer (the disk-usage checkpoint plan, the FSEvents plan's cursor gate) to key on it. Both plans now say volume identity is a volume UUID, never st_dev. Fix: reword both rustdoc lines to say it is the platform device number, valid for comparing entries within one observation, not stable across reboots or remounts, not a volume identity, and zero where unavailable. Code change, so it belongs in an engine PR, not the docs-only PR #55.
