---
type: is
id: is-01m0ncyz0vnfen24v43n77wj6h
title: Define the run document schema and conformance fixtures
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md
labels: []
dependencies: []
parent_id: is-01m0ncyyjd1r8yh51evp1v4vcn
created_at: 2026-08-22T18:50:35.930Z
updated_at: 2026-08-22T19:00:59.415Z
---
Rung 2 of the adoption ladder, and optional: the artifact contract is the floor (metabrowser wrote artifacts nearly by hand), the run document removes retyping for loops with a machine-readable harness. Specify variants (content-identified), variant_order, jobs, conditions incl. tier, open subject, per-sample metric vector + validity flag with reasons. fdu-realtree-run-v1 is the first conformant profile. Also support the metabrowser mode: a pasted probe payload plus operator-supplied labels, with record-time provenance capture and validity guards that REFUSE invalid runs (viewport floor pattern) rather than annotate them.
