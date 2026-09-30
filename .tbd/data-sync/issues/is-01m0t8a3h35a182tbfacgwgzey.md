---
type: is
id: is-01m0t8a3h35a182tbfacgwgzey
title: Two experiment artifacts name a commit that does not contain their change
kind: bug
status: closed
priority: 3
version: 3
labels:
  - performance
  - campaign-2
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-08-24T16:05:30.274Z
updated_at: 2026-09-30T03:25:11.730Z
closed_at: 2026-09-30T03:25:11.730Z
close_reason: "Corrected in 311ef6bb, with evidence: 87fd0bd ('perf: skip unread journal capture on the bootstrap apply path (exp-058, H90)') contains the index.rs journal-flag change and adds this record (then exp-058) with the recorded 8286c7e = its docs-only control; 575db66 ('perf: share the index with the snapshot writer (exp-059, H87)') changes open_with_pending_save to Arc<Index> and adds the record with the recorded bd9779d = the harness commit. exp-051's null -> 2475c82 (adds the parent memo and the record with identical -7.35% [-10.42%, -6.12%]). verdict.commit updated in all three; each body gains a dated correction note naming the original value; no measurement or verdict changed. Ledger, timeline.json and index.html regenerated (only these commits changed; prepared date kept). perf-evidence-check, perf-ledger-check, perf-report-check, docs-format-check pass. Note: these commits (like the originally recorded ones) are reachable from older campaign branches, not from the shallow main history in this clone."
resolution: null
duplicate_of: null
---
Found reviewing PR #46 (2026-08-24). experiment.py describes Decision.commit as 'Commit that landed it, or reverted it', and the ledger renders it as the place a reader goes to find the code. Two historical artifacts point somewhere else: exp-062 records 8286c7e ('docs: regenerate the experiment ledger through exp-057') and exp-063 records bd9779d ('perf(harness): add the cold-open-save job'), neither of which contains the H90 or H87 change those experiments accepted; exp-051 records null. The cause is recording before committing, so the field captures the control's hash -- the same slip hit exp-066..069 in PR #46 and was corrected there, and the runbook now states the two-commit shape that prevents it. Fix: correct the two artifacts to the commits that landed H90 and H87, or annotate them per the loop's 'the record is corrected, not rewritten' rule, then regenerate the views. Low priority: the reasoning and numbers in both artifacts are unaffected.
