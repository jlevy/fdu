---
type: is
id: is-01m4phmzw6t4vc86hpgmkspfef
title: "PR #197: honor configured scratch for stability correctness trees"
kind: bug
status: in_progress
priority: 1
version: 2
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
hold: null
hold_until: null
created_at: 2026-10-11T22:34:33.733Z
updated_at: 2026-10-11T22:35:13.966Z
started_at: 2026-10-11T22:35:13.961Z
---
Release execution preparation found scripts/release/stability_pass.py:1299 forces /tmp even with explicit TMPDIR, violating the mandatory external scratch policy. Honor the configured temporary parent without falling back when explicit TMPDIR is invalid, retain refusal-tree permissions and root/nobody reachability, document the short socket-path requirement, and add focused regression evidence. Coordinator owns full release-stability and integration validation.
