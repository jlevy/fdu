---
type: is
id: is-01m3btqb6dqhhxmg1ytdrvxhwv
title: Progress line counted apparent bytes and shifted as counts grew
kind: bug
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
labels: []
dependencies: []
parent_id: is-01m2nsj4zgw7r9j3d1rphnmwf2
created_at: 2026-09-25T08:25:47.465Z
updated_at: 2026-09-25T08:25:50.066Z
closed_at: 2026-09-25T08:25:50.065Z
close_reason: "Fixed in PR #125, merged to main as 963d4852 on 2026-09-25 after review and delta reviews; make check, cross-lint (darwin, windows), and CI green on tree ec8ae16f."
resolution: null
duplicate_of: null
---
User report 2026-09-24 on fdu ~/Library: the progress line jumped from a few GiB to 8.1 TiB on a smaller drive, and its numbers shifted the line as they grew. Cause: the walk counted apparent bytes; OrbStack's sparse VM disk (Group Containers/HUAQ24HBR6.dev.orbstack/data/data.img) is 8 TiB apparent, 39 MiB allocated, while the answer defaults to allocated. The performance footer had the same flaw. Fix in PR #125: allocated counting (ScanReport::allocated_walked, ProgressSnapshot::allocated, PerformanceSummary::walked_allocated), both lines follow --size; counts right-aligned in 9 columns, sizes in 8, analyzed count to its total; shrink order drops phase padding first.
