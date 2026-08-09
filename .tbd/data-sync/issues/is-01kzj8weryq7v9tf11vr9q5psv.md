---
type: is
id: is-01kzj8weryq7v9tf11vr9q5psv
title: "PR #1 review C2: Share traversal policy between scan and revalidation"
kind: bug
status: closed
priority: 2
version: 3
labels:
  - pr-review
dependencies:
  - type: blocks
    target: is-01kzj8wf8380p0rf53pzemj4w7
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:54.333Z
updated_at: 2026-08-09T03:54:45.513Z
closed_at: 2026-08-09T03:54:45.513Z
close_reason: Cold scan, revalidation, and reconciliation now share depth/filesystem descent policy and root-device context; regression tests pass.
---
PR #1 Cursor thread C2: https://github.com/jlevy/fdu/pull/1#discussion_r3742309825. File: crates/fdu/src/scan.rs. Revalidation must use the same max-depth, symlink, and one-filesystem descent predicate and root device context as cold scan.
