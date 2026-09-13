---
type: is
id: is-01m2eb1xc9wtwxrn71ysbev4kk
title: "PR #54 review H86-2: measured commits orphaned; shipped binary differs; verdict.commit names an orphan"
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eb1cnhke030h15e79fyzvc
created_at: 2026-09-13T21:33:58.024Z
updated_at: 2026-09-13T22:07:26.481Z
closed_at: 2026-09-13T22:07:26.479Z
close_reason: "Fixed in 2d36eab: tags perf/h86-linux-candidate (5d7b86f) and perf/h86-linux-control (c6380f7) are pushed; the artifact names the restacked f972250 and a74ac2a, states the compiled-out gitignore feature, the controls-on default-tree probe and the ad52469 preflight with why none is a route to the gates, and sets verdict.commit to null."
resolution: null
duplicate_of: null
---
Medium. Artifact :42-43, :411. Candidate 5d7b86f and control c6380f7 are not ancestors of #52 or main after the restack (equivalents f972250 and a74ac2a). What ships differs: performance builds now enable --features gitignore (1a39be9) while the run's binaries record no feature set; the default-tree probe now matches the non-watch CLI's controls-off scope (64c6e61) while this run measured it controls-on; ad52469 adds the point-lookup preflight. Fix: tag the measured commits before any restack; add the qualifications and name the restacked equivalents in the artifact body; set verdict.commit: null (the convention for rejected runs, e.g. exp-043, exp-098, exp-100).
