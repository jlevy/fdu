---
type: is
id: is-01m0p06wekqeg0bf5qddqh22ts
title: "PR #42 R10: crates/fdu/build.rs comment says file-type rules belong to fdu"
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0p06qgww21s4fpdkj2bb6bf
created_at: 2026-08-23T00:26:58.387Z
updated_at: 2026-08-23T00:57:54.144Z
closed_at: 2026-08-23T00:57:54.144Z
close_reason: "Fixed in 4e34ce3; addressed the review on PR #42, verified through make check and make release-rehearse."
---
crates/fdu/build.rs:3-4. Written before the rename; the rules live in fdu-core.
