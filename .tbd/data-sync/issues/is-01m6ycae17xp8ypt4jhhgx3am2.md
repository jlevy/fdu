---
type: is
id: is-01m6ycae17xp8ypt4jhhgx3am2
title: Document the written-recently recipe in the skill, --help, --docs, README, and usage guide
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
hold: null
hold_until: null
created_at: 2026-11-08T20:06:44.007Z
updated_at: 2026-11-08T20:16:10.568Z
started_at: 2026-11-08T20:07:44.333Z
closed_at: 2026-11-08T20:16:10.568Z
close_reason: Documented the written-recently recipe (--kind file --modified-since) and its limits in the skill, --help, --docs, README, crate README, and usage guide; goldens and parity artifact updated; plan-2026-11-08-fdu-recent-growth-and-volumes.md added. Branch claude/eloquent-galileo-dd771d.
resolution: null
duplicate_of: null
---
Phase 0. With current flags only: --kind file --modified-since WINDOW (tree, summary, largest), why --kind file is required (a time bound admits active directories at full size), and the limits: full-size counting of modified files, invisible deletions, arrivals with older mtimes (cp -p, rsync -a, archives, Cargo crate sources stamped 2006), space outside the walk. Point several-window use at Python fdu.scan. Update the cli-surface golden and the parity artifact.
