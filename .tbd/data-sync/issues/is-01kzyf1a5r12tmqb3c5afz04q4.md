---
type: is
id: is-01kzyf1a5r12tmqb3c5afz04q4
title: "Docs: record the Linux scouting backlog in the hypothesis registry and correct the walker claim"
kind: chore
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-08-13T21:04:18.104Z
updated_at: 2026-08-13T21:04:18.104Z
---
Doc-accuracy follow-ups from the post-#8 audit, landing on PR #14: the performance-loop hypothesis registry said the next free number was H67 while H69/H70 were already in use; the Linux scouting results had no registry rows, so the next optimization round had no backlog entry for them; and the design principles still described the portable walker as scaffolding pending a getdents64/statx layer that Rust's standard library already emits on Linux. Adds H71 (Linux enumeration rearrangement, refuted on the scouting rig), H72 (d_type-gated stat elision), H73 (inode-ordered statting, unresolved), H74 (allocator), H75 (warm-open inversion), H76 (cold worker depth), and cross-links the research note from the architecture white paper and design principles.
