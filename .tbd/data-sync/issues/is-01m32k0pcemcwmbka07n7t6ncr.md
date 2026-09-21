---
type: is
id: is-01m32k0pcemcwmbka07n7t6ncr
title: "PR #97 review R4: exp-150 and exp-151 are one run recorded twice with opposite decisions"
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m32h6d4szbwm3n1sgxqyfn9x
created_at: 2026-09-21T18:17:55.342Z
updated_at: 2026-09-21T18:17:55.342Z
---
docs/project/experiments/exp-150 and exp-151 share run_artifact run-exp-150-h85-recycle-v612.json and identical numbers (-4.981% [-5.92%, -4.33%]); H147 was minted after H85 missed its 20% bar on the same 12 pairs; totals count two experiments from one measurement. Fix (a): keep both, put exp-150 in CLAIM_ONLY_EXPERIMENTS, add one sentence to Publishing the evidence that a bar-only re-decision carries the original run_artifact and is not a second measurement. PR #97 senior review, Medium.
