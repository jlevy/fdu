---
type: is
id: is-01m2hsharbxs46p58q9n88mkvq
title: "PR #55 review PR55-FMT-1: literal snapshot 'format version 3' goes stale when PR A bumps FORMAT_VERSION in 0.1.0"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
parent_id: is-01m2hsgt9d35edzxt2rqvfc3pv
created_at: 2026-09-15T05:44:46.602Z
updated_at: 2026-09-15T05:50:39.087Z
closed_at: 2026-09-15T05:50:39.082Z
close_reason: "56fd808: plan What Exists Today, FSEvents plan snapshot-format section and Phase 1 test item, and research daily-comparison section now say the current format version (snapshot::FORMAT_VERSION) or the version current at integration time; comparability rule unchanged"
resolution: null
duplicate_of: null
---
PR #55, delta review 5205945198. Literal 'format version 3' at plan @55ce4a3 :45, fsevents plan :362 and :709, research-2026-08-10-performance-frontier.md :1355. fdu-1onj Q11 bumps snapshot FORMAT_VERSION in PR A (0.1.0) before slice 2. Fix: say 'the current format version' (snapshot::FORMAT_VERSION) or 'the next available version', keeping the story that the version never affects checkpoint comparability.
