---
type: is
id: is-01m0nt4xbx91kvj3krx7zg048t
title: "PR #40 review R10: the Python one-shot path depends on a feature named cli"
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0nt3z2n7reqvg9sfy3j2ab1
created_at: 2026-08-22T22:41:02.333Z
updated_at: 2026-08-22T23:14:23.079Z
closed_at: 2026-08-22T23:14:23.079Z
close_reason: "Tracked as fdu-phdm rather than fixed: the feature-graph move has its own lib-only and MSRV consequences and is deliberately separate from the parity work that surfaced it."
---
crates/fdu/src/lib.rs:98 gates prepare_report behind feature=cli; fdu-py enables it. Feature-graph decision left untracked.
