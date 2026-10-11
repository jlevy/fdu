---
type: is
id: is-01m4phmzw6t4vc86hpgmkspfef
title: "PR #197: honor configured scratch for stability correctness trees"
kind: bug
status: closed
priority: 1
version: 3
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
hold: null
hold_until: null
created_at: 2026-10-11T22:34:33.733Z
updated_at: 2026-10-11T22:38:51.291Z
started_at: 2026-10-11T22:35:13.961Z
closed_at: 2026-10-11T22:38:51.289Z
close_reason: "Fixed and pushed in d2c0ed011dcda7b1e2bd1d7bb292754107dbb621. Comment: https://github.com/jlevy/fdu/pull/197#issuecomment-6114685074. Two focused regressions failed before repair; all 88 stability-driver tests passed after repair. Real configured external scratch probe bound a 93-byte Unix socket, retained 0755/1777 modes, verified cleanup and clear failure for a missing TMPDIR. Scoped Ruff and pinned Flowmark formatting passed. Coordinator owns pending independent review, full release stability, integration gates, and final CI."
resolution: null
duplicate_of: null
---
Release execution preparation found scripts/release/stability_pass.py:1299 forces /tmp even with explicit TMPDIR, violating the mandatory external scratch policy. Honor the configured temporary parent without falling back when explicit TMPDIR is invalid, retain refusal-tree permissions and root/nobody reachability, document the short socket-path requirement, and add focused regression evidence. Coordinator owns full release-stability and integration validation.
