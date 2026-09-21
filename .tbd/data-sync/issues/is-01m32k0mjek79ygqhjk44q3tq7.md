---
type: is
id: is-01m32k0mjek79ygqhjk44q3tq7
title: "PR #97 review R1: evidence page draws non-shipped arms as the product's current cost"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m32h6d4szbwm3n1sgxqyfn9x
created_at: 2026-09-21T18:17:53.486Z
updated_at: 2026-09-21T18:38:09.200Z
closed_at: 2026-09-21T18:38:09.199Z
close_reason: "Fixed: exp-146/148/149/150/152/154 added to CLAIM_ONLY_EXPERIMENTS with reasons; ledger, timeline.json and index.html regenerated; linux-v6.12 leaves the per-entry figure. exp-141 stays on #94."
resolution: null
duplicate_of: null
---
explorations/benchmarks/realtree/timeline.py:114 CLAIM_ONLY_EXPERIMENTS lacks exp-146/148/149/150/152/154. exp-146/148/149 (--threads 8 flag) and exp-154 (PGO build, [profile.release] unchanged) are accepted so kept=candidate; index.html reports linux-v6.12 latest 4.98 us/entry which is exp-154's PGO candidate while the shipped probe is 5.46. exp-150/152 rejected-but-shipped carry kept=control. Fix: add all six with one-line reasons, regenerate ledger and report. PR #97 senior review, High.
