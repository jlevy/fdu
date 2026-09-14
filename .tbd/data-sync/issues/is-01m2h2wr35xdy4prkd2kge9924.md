---
type: is
id: is-01m2h2wr35xdy4prkd2kge9924
title: "PR #58 review PR58-REC-5: exp-104 prose mixes paired change and ratio of medians"
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m2h2t7k65srszyhv02tag8ba
created_at: 2026-09-14T23:09:03.458Z
updated_at: 2026-09-14T23:27:40.984Z
closed_at: 2026-09-14T23:27:40.983Z
close_reason: "bd8ed0b: component quoted as paired +0.07% [-0.77%, +1.00%] in the artifact and PR body, matching wall and the record."
resolution: null
duplicate_of: null
---
PR #58, P3. exp-104 "What happened" bullets 1-2 and the PR body table quote wall as the paired change (+0.05%) but component as a ratio of medians (+0.28%, 1,723.1 -> 1,727.9 ms). The record carries component `change_pct` +0.067 [-0.772, +0.996].

`performance-loop.md` "Publishing the evidence" requires a reported change to be the paired figure.

Fix: quote component as +0.07% [-0.77%, +1.00%] in the artifact and the PR body.
