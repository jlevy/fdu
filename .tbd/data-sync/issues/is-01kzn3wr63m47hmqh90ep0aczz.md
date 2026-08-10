---
type: is
id: is-01kzn3wr63m47hmqh90ep0aczz
title: Record declared environment when benchmark setup fails
kind: bug
status: closed
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
dependencies: []
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
created_at: 2026-08-10T05:56:24.386Z
updated_at: 2026-08-10T06:06:28.084Z
closed_at: 2026-08-10T06:06:28.084Z
close_reason: All local handoff gates and the complete Linux, macOS, Windows, and installed-wheel CI matrix pass with dedicated regressions and documented evidence.
---
A corpus setup failure currently leaves trial.environment.set empty, and result validation then raises a secondary schema error instead of preserving the original invalid-trial reason. Establish and tokenize placeholders/environment before fallible corpus creation, keep setup failures as schema-valid immutable evidence, and add a regression that exposes the primary setup error without executing the timed child.
