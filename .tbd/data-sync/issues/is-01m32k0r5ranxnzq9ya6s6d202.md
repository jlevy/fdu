---
type: is
id: is-01m32k0r5ranxnzq9ya6s6d202
title: "PR #97 review suggestions S1-S5"
kind: task
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m32h6d4szbwm3n1sgxqyfn9x
created_at: 2026-09-21T18:17:57.176Z
updated_at: 2026-09-21T18:38:12.454Z
closed_at: 2026-09-21T18:38:12.454Z
close_reason: S1 fixed (redundant inner cfg(unix) removed); S2 fixed (compact summary answer test uses threads Some(2)); S3 PR body updated; S4 fixed (campaign-2 H148 line says training and measurement share one tree, so -8.35% is a ceiling); S5 declined, screen wording stands.
resolution: null
duplicate_of: null
---
S1 redundant inner cfg(unix) in a cfg(unix) test (scan.rs ~5731); S2 compact_summary_matches_the_indexed_summary_exactly uses threads None so a one-vCPU runner takes the serial walker, use Some(2); S3 PR body test-plan boxes; S4 campaign-2 plan H148 checkbox should say training and measurement used the same tree; S5 exp-154 warm-revalidate spawn/size sentence is inferred, fine for a screen. PR #97 senior review, non-blocking.
