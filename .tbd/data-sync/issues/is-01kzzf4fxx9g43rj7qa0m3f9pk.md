---
type: is
id: is-01kzzf4fxx9g43rj7qa0m3f9pk
title: "PR #21 R3: make uv version parsing fail closed"
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01kzzedh4wjer6vyq7b0yj782d
created_at: 2026-08-14T06:25:16.732Z
updated_at: 2026-08-14T06:34:52.801Z
closed_at: 2026-08-14T06:34:52.800Z
close_reason: "Fixed: the guard fails closed on missing/failing executables, malformed output, and prereleases, with stable numeric boundary tests. make check passed."
---
Review R3 from https://github.com/jlevy/fdu/pull/21#issuecomment-5290202229. Makefile:92-100 accepts malformed and prerelease second tokens through sort -V and loses uv --version failures. Validate the command/output and compare stable numeric X.Y.Z values with committed boundary/error tests.
