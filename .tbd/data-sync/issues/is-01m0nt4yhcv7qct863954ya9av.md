---
type: is
id: is-01m0nt4yhcv7qct863954ya9av
title: "PR #40 review R12: cause-chain dedup uses substring containment"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0nt3z2n7reqvg9sfy3j2ab1
created_at: 2026-08-22T22:41:03.531Z
updated_at: 2026-08-22T23:14:22.752Z
closed_at: 2026-08-22T23:14:22.752Z
close_reason: "Fixed in b8999e8; addressed the senior review on PR #40 with regression tests for each."
---
crates/fdu/src/cli.rs:1526. headline.contains(cause) can swallow a genuinely different deeper cause.
