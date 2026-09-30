---
type: is
id: is-01m3rg104q0wkxr6qf22wdgsh0
title: "Atomic writes: state the rule in the design principles and enforce it with a check in make check and CI"
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m3rfz36eh8m7xp52aq5c0kmz
created_at: 2026-09-30T06:28:57.111Z
updated_at: 2026-09-30T06:28:57.111Z
---
The rule in fdu-design-principles.md, and a check (Rust, Python and Node) that fails on any write outside the atomic helpers and the staged-rename path, wired into make check and CI, with tests.
