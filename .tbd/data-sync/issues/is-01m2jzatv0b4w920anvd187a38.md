---
type: is
id: is-01m2jzatv0b4w920anvd187a38
title: "PR #61 review PR61-SC-1: Rust toolchain pin check accepts a comment as the pin"
kind: bug
status: closed
priority: 3
version: 3
labels:
  - release
dependencies: []
parent_id: is-01m2jzadk7w8m1xcsewzwg5wj1
created_at: 2026-09-15T16:45:19.578Z
updated_at: 2026-09-15T17:10:15.547Z
closed_at: 2026-09-15T17:10:15.542Z
close_reason: "fc7d8fb: Rust pins read from channel/toolchain keys; each pin must be inventoried for its file and each inventoried version pinned; Node tests for comment-only, partial drift, commented channel"
resolution: null
duplicate_of: null
---
PR #61 at eb89150, scripts/check-supply-chain.mjs:748-749. The rustToolchains check is text.includes(version), so a workflow whose toolchain: values drifted but whose '# 1.97.1' action comments stayed passes. Match the actual toolchain: values in workflows and channel = "..." in rust-toolchain.toml, and add a Node test showing a comment-only pin fails. Pre-existing.
