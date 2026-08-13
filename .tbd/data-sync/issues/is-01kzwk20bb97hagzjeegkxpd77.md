---
type: is
id: is-01kzwk20bb97hagzjeegkxpd77
title: "H58: Prototype portable wide-directory stat chunks with work stealing"
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-08-13T03:36:06.250Z
updated_at: 2026-08-13T18:11:55.971Z
---
Inspired by dua v2.41.1: on the portable backend only, split metadata work from very wide directory reads into small stealable batches while retaining FDU region scheduling, bounded memory, ordering/progressive guarantees, exact observations, and no semantic changes. Profile first; preregister portable/Linux wall, system CPU, lock wait, and RSS; do not add crossbeam merely to run the experiment.
