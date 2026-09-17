---
type: is
id: is-01m0ncyze04cc4bebm2ax1z9h3
title: Extract the contract models with an open subject and metric roles
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md
labels: []
dependencies:
  - type: blocks
    target: is-01m2esgt594wns69rqrjzzx0ej
parent_id: is-01m0ncyyjd1r8yh51evp1v4vcn
created_at: 2026-08-22T18:50:36.351Z
updated_at: 2026-09-17T02:11:04.099Z
closed_at: 2026-09-17T02:11:04.098Z
close_reason: Delivered as experiment-loop skill assets (experiment.schema.yaml); fdu-local vocabulary via fdu-6xfq
resolution: canceled
duplicate_of: null
---
Contract base with the invariant spine (id/title/date/hypotheses/subject/method/results/complexity/verdict), subject as a project-supplied payload, method carrying re-run provenance (binary hashes, commit+entry-point code refs, or compute budget as the domain affords), and the FOUR result shapes: paired (fdu), conditions (metabrowser median+range+overlapping), record (score vs standing best, beat_record pass/fail — the packing-search shape), determination (categorical outcome from a declared enum — the proof-search shape). Merged decision vocabulary: accepted/rejected/unresolved/blocked/abandoned/superseded/baseline/in-progress, with abandoned requiring budget spent, best result reached, and what would justify reopening. Metric roles outcome/cost/guard/mechanism. Fixtures from all four domains.
