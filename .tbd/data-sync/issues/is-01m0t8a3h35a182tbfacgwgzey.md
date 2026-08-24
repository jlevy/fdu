---
type: is
id: is-01m0t8a3h35a182tbfacgwgzey
title: Two experiment artifacts name a commit that does not contain their change
kind: bug
status: open
priority: 3
version: 1
labels:
  - performance
  - campaign-2
dependencies: []
created_at: 2026-08-24T16:05:30.274Z
updated_at: 2026-08-24T16:05:30.274Z
---
Found reviewing PR #46 (2026-08-24). experiment.py describes Decision.commit as 'Commit that landed it, or reverted it', and the ledger renders it as the place a reader goes to find the code. Two historical artifacts point somewhere else: exp-062 records 8286c7e ('docs: regenerate the experiment ledger through exp-057') and exp-063 records bd9779d ('perf(harness): add the cold-open-save job'), neither of which contains the H90 or H87 change those experiments accepted; exp-051 records null. The cause is recording before committing, so the field captures the control's hash -- the same slip hit exp-066..069 in PR #46 and was corrected there, and the runbook now states the two-commit shape that prevents it. Fix: correct the two artifacts to the commits that landed H90 and H87, or annotate them per the loop's 'the record is corrected, not rewritten' rule, then regenerate the views. Low priority: the reasoning and numbers in both artifacts are unaffected.
