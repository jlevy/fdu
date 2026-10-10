---
type: is
id: is-01m4kwngg035sqgbbtk040qwvf
title: Refuse a walk-time root overlap at the first entered alias, not after the outer walk
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
created_at: 2026-10-10T21:49:21.792Z
updated_at: 2026-10-10T21:49:21.792Z
---
Review E7 on PR #192 (jlevy/fdu#192), and review D's what-next item 4. A walk-time overlap (an alias, bind mount, or firmlink no path showed) is refused only after the whole outer walk: 'fdu /System/Volumes/Data ~/src' walks the data volume for minutes before refusing. The fold already knows at the first entered alias (FoldAliases); the index tiers need a sink hook to stop early. Done when each tier stops its walk at the first alias it enters, with a test per tier that the refusal arrives before the walk completes.
