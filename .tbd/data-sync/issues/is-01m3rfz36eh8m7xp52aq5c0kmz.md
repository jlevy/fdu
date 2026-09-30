---
type: is
id: is-01m3rfz36eh8m7xp52aq5c0kmz
title: Write every file atomically, everywhere, and enforce it
kind: epic
status: closed
priority: 1
version: 8
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
child_order_hints:
  - is-01m3rg104q0wkxr6qf22wdgsh0
  - is-01m3rg10nk2njtvckwk6a6n2dj
  - is-01m3rg116mr9nvstgsfjq7ma4w
  - is-01m3rg11pvht69maskknprscpw
  - is-01m3rg127f0yh58z7t75zt2mmd
created_at: 2026-09-30T06:27:54.701Z
updated_at: 2026-09-30T13:29:10.915Z
closed_at: 2026-09-30T13:29:10.915Z
close_reason: "Every child task is closed: the rule 'Write Every File Whole' is in the design principles, make atomic-writes enforces it in make check, and every writer in the release, gate, QA, benchmark and research tooling goes through scripts/atomic_write.py, scripts/atomic-write.mjs or snapshot::write_atomically (bb1404a2..acb2882b); the #164 review's R164-9 and S3 tightened the check and the exclusive mode (cfb6b042, 691640fa)."
resolution: null
duplicate_of: null
---
Maintainer request 2026-09-30: 'properly fix atomic file writes. That should be done everywhere.' Sized at about 65 plain write sites that matter plus about 95 in old research scripts, and none in shipped Rust, so it lands on the stability branch (worked on claude/atomic-writes, from claude/stability-tooling, merged into claude/stability-fixes). The engine and CLI already write outputs atomically (snapshot::write_atomically: sibling temp, write, fsync, rename; skill_install stages and renames); the engine journal is in memory. Scope: every file written for something else to read later; append-only files must instead detect and drop a torn tail. Test fixtures are scan inputs, covered by fdu-tq70.
