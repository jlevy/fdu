---
type: is
id: is-01m2esgt594wns69rqrjzzx0ej
title: Replace the hand-maintained CLAIM_ONLY_EXPERIMENTS list with a decision value
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2eb1cnhke030h15e79fyzvc
created_at: 2026-09-14T01:46:46.312Z
updated_at: 2026-09-14T01:46:46.312Z
---
Follow-up on PR #54 review H86-5 (fdu-gq2r, fixed in 4dccad8 with the minimum option), recorded by the fixer.

**What landed.** `explorations/benchmarks/realtree/timeline.py` gained a hand-maintained `CLAIM_ONLY_EXPERIMENTS = {"exp-103"}`. For a listed experiment, `kept_variant` returns `None` and `report_html.figure_per_entry` skips the record, so the evidence page stops drawing the pre-H86 control (4.23 µs/entry) as Linux's current cost. `docs/project/guides/performance-loop.md` documents the list beside the kept-arm rule.

**Why it is a stopgap.** The page's kept-arm projection is decided by a list someone has to remember to edit, not by the record. The next experiment that passes its relative gates but fails a pre-registered claim (candidate retained, claim unmet) gets plotted wrong unless its id is added by hand. The fixer rejected the other two options:
- recording `accepted` would say the Linux stage passed;
- adding a decision value changes the `fdu.performance:Experiment/v1` contract, and the merged decision vocabulary is already tracked as fdu-02a0.

**To do.**
1. When the decision vocabulary is extended (fdu-02a0: accepted/rejected/unresolved/blocked/abandoned/superseded/baseline/in-progress, or an fdu-local extension if that extraction is not imminent), add a value for "candidate retained, pre-registered claim unmet", or a separate claim-outcome field.
2. Derive `kept_variant` and the per-entry figure from that value, record it on exp-103, and delete `CLAIM_ONLY_EXPERIMENTS` and its paragraph in `performance-loop.md`.
3. Recompile the schema (`make perf-schema`) and regenerate the ledger, `timeline.json`, and `index.html`.
4. Keep the two H86-5 tests, re-expressed against the new field.

Review: https://github.com/jlevy/fdu/pull/54#pullrequestreview-5192260482
