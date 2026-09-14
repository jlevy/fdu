---
type: is
id: is-01m2et6kbed4g3ahaapjkgadfz
title: "PR #48 review FIX48-2: a refused control orphans subdirectories earlier batches of the same listing committed"
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2et5s9yyv9e6tz783rp2m6c
created_at: 2026-09-14T01:58:40.237Z
updated_at: 2026-09-14T02:49:13.441Z
closed_at: 2026-09-14T02:49:13.440Z
close_reason: "Fixed in 9c29e6f (option 1): a refusal queues the subdirectories earlier batches of the listing committed; abandon_directory comment corrected; retry-without-control recorded on fdu-1onj. https://github.com/jlevy/fdu/pull/48#issuecomment-5658305981"
resolution: null
duplicate_of: null
---
Medium, introduced by c801d4e. At f917cb7: opened.rs:1084-1110 abandon_directory, 1239-1292 (every return abandon_directory exits before frontier.extend(discovered)). A listing longer than batch_size commits a subdirectory upsert in an earlier batch; a later batch carrying an oversized .gitignore is refused (discovery_rejection maps both control limits to Refused); the committed subdirectory stays incomplete and is never queued, so its subtree is silently missing and RollUp answers Present(files: 0). The abandon_directory comment ('either left with the stale parent or were never committed') is false for Refused. Chosen fix (option 1): on Refused, queue the subdirectories that already-committed batches carried (track the committed prefix of discovered), drop only the refused batch's entries, and correct the comment. Option 2 (retry the refused batch without its control op) belongs to fdu-1onj's degradation design. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5193206420
