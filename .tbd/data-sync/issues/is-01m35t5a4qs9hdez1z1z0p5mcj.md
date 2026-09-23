---
type: is
id: is-01m35t5a4qs9hdez1z1z0p5mcj
title: "PR #96 A96-1: scope machine-format depth exemption to flat lists"
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m35sm4jn2ytfmy91136g50b7
created_at: 2026-09-23T00:20:29.973Z
updated_at: 2026-09-23T00:20:29.973Z
---
At c3aeed8a, directory-query plan line 153 says depth has no effect under machine formats. Executed legacy tree JSON with depth=0 still folds children, correctly. Qualify exemption to flat List projections; legacy tree/full sections retain depth. Nonblocking review nit.
