---
type: is
id: is-01m0t5t2sa2rn3qm3m4dycv7hv
title: Fold Classification.flags into the tag model as Name-tier rules
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T15:21:48.073Z
updated_at: 2026-08-24T15:21:48.073Z
---
Classification.flags (generated, vendored, documentation) are per-entry booleans
recomputed from the name on every query and never maintained -- Name-tier tag rules
wearing a different hat, found during the 2026-08-24 genericity review. Fold them into
the tag model: each becomes an available Name-tier rule, classification keeps reporting
them unchanged for compatibility, and a consumer wanting "bytes excluding vendored"
gets it by tag instead of by a bespoke walk.

Not before the model settles: this moves goldens and is pure consolidation, so it waits
for fdu-mvt3 and rides behind the planes work. P3.
