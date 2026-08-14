---
type: is
id: is-01kzzf4g6xj5gx1fgy2ymbdhaw
title: "PR #21 R4: enforce one reviewed uv version"
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01kzzedh4wjer6vyq7b0yj782d
created_at: 2026-08-14T06:25:17.020Z
updated_at: 2026-08-14T06:34:53.088Z
closed_at: 2026-08-14T06:34:53.087Z
close_reason: "Fixed: supply-chain-policy.json now enforces the reviewed uv release across Makefile and both CI pins; invariant tests and supply-chain audit pass."
---
Review R4 from https://github.com/jlevy/fdu/pull/21#issuecomment-5290202229. UV_MIN_VERSION and both CI pins are synchronized only by comments even though supply-chain-policy.json already records the reviewed uv release. Add Makefile to the enforced policy files and cover the invariant.
