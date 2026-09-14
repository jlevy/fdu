---
type: is
id: is-01m2h2vcft3jzfxgbx4nwxnjmb
title: "PR #55 review PR55-STALE-1: merged stack described as unmerged, main as snapshot v2"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
parent_id: is-01m2h2v87crh401e90pfe52w8g
created_at: 2026-09-14T23:08:18.809Z
updated_at: 2026-09-14T23:54:30.134Z
closed_at: 2026-09-14T23:54:30.126Z
close_reason: "4727de0 (after merge 6599b2a): all five docs describe main at dda7e6a as snapshot format 3 with the merged opened-root lifecycle; stale stack/v2 claims removed"
resolution: null
duplicate_of: null
---
PR #55 at dc27c14: checkpoints plan :22-24, :32, :42-47; FSEvents plan :14-16, :354-356, :688; research :1237-1244; cache-design.md :152-153; campaign-2 :307-313 describe the #48-#54 stack as unmerged, main as snapshot v2, and opened-root work as not shipped. origin/main dda7e6a snapshot.rs:60 has FORMAT_VERSION = 3.
