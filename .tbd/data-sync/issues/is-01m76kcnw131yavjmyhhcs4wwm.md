---
type: is
id: is-01m76kcnw131yavjmyhhcs4wwm
title: "Skill: point at --view ages when a time window selects everything"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-12T00:44:13.056Z
updated_at: 2026-11-12T00:44:13.056Z
---
Linux host gate manual check on #209 at 98784661: a fresh agent given only the shipped skill and asked 'where did my disk space go since yesterday?' on a home folder written entirely within the day ran 'fdu PATH --kind file --since 24h', got 'note: selected all 60,582 files', and concluded the window could not separate growth, without running --view ages, which the skill documents only after the --since recipes. Have 'Find What Was Written Recently' say that when the note reads 'selected all', a narrower window or --view ages (under 1h / 1h to 1d bands) shows when within the window the data arrived, and consider putting the ages line in the first recipe block.
