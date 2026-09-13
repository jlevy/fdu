---
type: is
id: is-01m2eb1xqycp33s623fb12c3nr
title: "PR #54 review H86-3: pre-registered raw samples and max/min not recorded for the fdu arms"
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eb1cnhke030h15e79fyzvc
created_at: 2026-09-13T21:33:58.397Z
updated_at: 2026-09-13T22:07:28.809Z
closed_at: 2026-09-13T22:07:28.808Z
close_reason: "Fixed as far as the evidence allows in 2d36eab: the run JSON is unrecoverable (it lived only on the deleted VM), so the artifact and research note state which pre-registered records are missing and why, the candidate max/min is recorded as unverified, and the 'max/min <= 1.324' claim is withdrawn (also from the PR body and bead notes). The schema max_over_min field and durable run JSON are deferred to fdu-c4jr, which stays open."
resolution: null
duplicate_of: null
---
Medium. Campaign-2 plan :343-345 pre-registers all raw samples, p95/median, and max/min for every arm, and gates the candidate on max/min <= 2.0. Artifact :59 run_artifact points at a VM-local path (/home/user/perf/results/run-h86-linux-immediate.json) that no longer exists, so the fdu arms' per-trial samples and max/min are unrecoverable and the 'max/min at most 1.324' claim (:423-425) rests on nothing checkable; the paired intervals cannot be recomputed. The artifact schema has no max/min field. Fix here: record the loss honestly in the artifact and research note, withdraw the unverifiable tail claim; defer the schema max_over_min field to its own bead.
